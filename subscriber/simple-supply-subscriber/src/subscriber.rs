use std::env;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use protobuf::Message;
use rest_api::messaging;
use sawtooth_sdk::messages::client_event;
use sawtooth_sdk::messages::events;
use sawtooth_sdk::messages::validator;
use sawtooth_sdk::messaging::stream::{MessageConnection, MessageReceiver, MessageSender};
use sawtooth_sdk::messaging::zmq_stream;
use simple_supply_tp::addresser;
use uuid::Uuid;

#[derive(Clone)]
pub struct Context {
    sender: Sender<()>,
}

impl Context {
    pub fn cancel(&self) {
        self.sender
            .send(())
            .unwrap_or_else(|err| error!("Failed to send cancel signal: {:?}", err));
    }
}

pub struct Subscriber {
    pool: Pool<ConnectionManager<PgConnection>>,
    cancel_receiver: Receiver<()>,
}

impl Subscriber {
    pub fn new() -> (Self, Context) {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let (cancel_sender, cancel_receiver) = channel();
        let context = Context {
            sender: cancel_sender,
        };

        let subscriber = Subscriber {
            pool,
            cancel_receiver,
        };
        (subscriber, context)
    }

    pub fn start(&self) {
        debug!("Subscribing to state delta events");

        let validator_url = env::var("VALIDATOR_URL").expect("VALIDATOR_URL is not set");
        let connection = zmq_stream::ZmqMessageConnection::new(&validator_url);
        let (mut validator_sender, validator_receiver) = connection.create();

        let block_sub = events::EventSubscription {
            event_type: "sawtooth/block-commit".to_string(),
            ..Default::default()
        };

        let delta_sub = events::EventSubscription {
            event_type: "sawtooth/state-delta".to_string(),
            filters: protobuf::RepeatedField::from_vec(vec![events::EventFilter {
                key: "address".to_string(),
                match_string: format!("^{}.*", addresser::get_namespace()),
                filter_type: events::EventFilter_FilterType::REGEX_ANY,
                ..Default::default()
            }]),
            ..Default::default()
        };

        let null_block_id = "0000000000000000".to_string();
        let request = client_event::ClientEventsSubscribeRequest {
            subscriptions: protobuf::RepeatedField::from_vec(vec![block_sub, delta_sub]),
            last_known_block_ids: ::protobuf::RepeatedField::from_vec(vec![null_block_id]),
            ..Default::default()
        };

        let request_bytes = request.write_to_bytes().unwrap();
        let mut future = match validator_sender.send(
            validator::Message_MessageType::CLIENT_EVENTS_SUBSCRIBE_REQUEST,
            &Uuid::new_v4().to_hyphenated().to_string(),
            &request_bytes,
        ) {
            Ok(fut) => fut,
            Err(e) => {
                error!("{}", e);
                return; // TODO error handling
            }
        };
        let response: client_event::ClientEventsSubscribeResponse;
        loop {
            match future.get_timeout(Duration::from_millis(10000)) {
                Ok(message) => {
                    response = protobuf::parse_from_bytes(&message.content[..]).unwrap();
                    break;
                }
                Err(err) => {
                    error!("{}", err);
                    return; // TODO error handling
                }
            };
        }

        match response.status {
            client_event::ClientEventsSubscribeResponse_Status::OK => {}
            client_event::ClientEventsSubscribeResponse_Status::INVALID_FILTER => {
                return error!("invalid filter");
            }
            client_event::ClientEventsSubscribeResponse_Status::UNKNOWN_BLOCK => {
                // Pytnonのコードは再スタートしている self.start()
                return error!("unknown block");
            }
            _ => unreachable!("status unset"),
        }

        debug!("Successfully subscribed to state delta events");

        // https://github.com/hyperledger/sawtooth-sdk-rust/blob/5e46e844d5c0615dfd05b3b885980c42e09c9a69/src/consensus/zmq_driver.rs
        loop {
            match validator_receiver.recv_timeout(Duration::from_millis(1000)) {
                Err(RecvTimeoutError::Timeout) => {
                    if self.cancel_receiver.try_recv().is_ok() {
                        break;
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    // break Err(Error::ReceiveError("Sender disconnected".into()));
                    break error!("error: sender disconnected");
                }
                Ok(Err(err)) => {
                    break error!("unexpected error while receiving: {}", err);
                    // break Err(Error::ReceiveError(format!(
                    //     "Unexpected error while receiving: {}",
                    //     err
                    // )));
                }
                Ok(Ok(msg)) => {
                    debug!("{:?}", msg);
                    // if let Err(err) =
                    //     handle_update(&msg, &mut validator_sender, &mut update_sender)
                    // {
                    //     break Err(err);
                    // }
                    if self.cancel_receiver.try_recv().is_ok() {
                        break;
                    }
                }
            }
        }
        validator_sender.close();
    }
}
