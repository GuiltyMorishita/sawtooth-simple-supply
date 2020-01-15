use crate::transaction_creation::make_create_agent_transaction;

use std::boxed::Box;
use std::time::Duration;

use protobuf::Message;
use sawtooth_sdk::messages::batch::Batch;
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatusRequest, ClientBatchStatusResponse, ClientBatchStatusResponse_Status,
    ClientBatchSubmitRequest,
};
use sawtooth_sdk::messages::validator;
use sawtooth_sdk::messaging::stream::{MessageConnection, MessageFuture, MessageSender, SendError};
use sawtooth_sdk::messaging::zmq_stream::{ZmqMessageConnection, ZmqMessageSender};
use sawtooth_sdk::signing::{create_context, CryptoFactory, PrivateKey, PublicKey};
use uuid::Uuid;

#[derive(Clone)]
pub struct Connection {
    sender: ZmqMessageSender,
}

impl Connection {
    pub fn new(validator_url: String) -> Connection {
        let connection = ZmqMessageConnection::new(&validator_url);
        let (sender, _) = connection.create();
        Connection { sender }
    }

    pub fn send(
        &self,
        message_type: validator::Message_MessageType,
        correlation_id: &str,
        message_content: &[u8],
    ) -> Result<MessageFuture, SendError> {
        self.sender
            .send(message_type, correlation_id, message_content)
    }
}

#[derive(Clone)]
pub struct Messenger {
    connection: Connection,
}

impl Messenger {
    pub fn new(validator_url: String) -> Messenger {
        Messenger {
            connection: Connection::new(validator_url),
        }
    }
    pub fn get_new_key_pair(&self) -> (Box<dyn PrivateKey>, Box<dyn PublicKey>) {
        let context = create_context("secp256k1").unwrap();
        let private_key = context.new_random_private_key().unwrap();
        let public_key = context.get_public_key(&*private_key).unwrap();
        (private_key, public_key)
    }

    pub fn send_create_agent_transaction(
        &self,
        private_key: &dyn PrivateKey,
        name: &str,
        timestamp: i64,
    ) -> Result<(), SendError> {
        let context = create_context("secp256k1").unwrap();
        let crypto_factory = CryptoFactory::new(&*context);
        let transaction_signer = crypto_factory.new_signer(&*private_key);
        let random_private_key = context.new_random_private_key().unwrap();
        let batch_signer = crypto_factory.new_signer(&*random_private_key); // TODO ここはMessengerの中に入れる?
        let batch =
            make_create_agent_transaction(transaction_signer, batch_signer, name, timestamp);
        self.send_and_wait_for_commit(batch) // await
    }

    pub fn send_and_wait_for_commit(&self, batch: Batch) -> Result<(), SendError> {
        let submit_request = ClientBatchSubmitRequest {
            batches: ::protobuf::RepeatedField::from_vec(vec![batch.clone()]),
            ..Default::default()
        };
        let submit_request_bytes = submit_request.write_to_bytes().unwrap();

        let mut future = match self.connection.send(
            validator::Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
            &Uuid::new_v4().to_hyphenated().to_string(),
            &submit_request_bytes,
        ) {
            Ok(fut) => fut,
            Err(_) => return Err(SendError::UnknownError),
        };
        loop {
            match future.get_timeout(Duration::from_millis(10000)) {
                Ok(_) => break,
                Err(_) => {
                    return Err(SendError::UnknownError);
                }
            };
        }

        let batch_id = batch.header_signature;
        let status_request = ClientBatchStatusRequest {
            batch_ids: ::protobuf::RepeatedField::from_vec(vec![batch_id]),
            wait: true,
            ..Default::default()
        };
        let status_request_bytes = status_request.write_to_bytes().unwrap();

        let mut future = match self.connection.send(
            validator::Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
            &Uuid::new_v4().to_hyphenated().to_string(),
            &status_request_bytes,
        ) {
            Ok(fut) => fut,
            Err(_) => return Err(SendError::UnknownError),
        };

        let status_response: ClientBatchStatusResponse;
        loop {
            match future.get_timeout(Duration::from_millis(10000)) {
                Ok(validator_response) => {
                    status_response =
                        protobuf::parse_from_bytes(&validator_response.content[..]).unwrap();
                    break;
                }
                Err(_) => {
                    return Err(SendError::UnknownError);
                }
            };
        }

        debug!("{:?}", status_response);

        match status_response.get_status() {
            ClientBatchStatusResponse_Status::OK => Ok(()),
            _ => Err(SendError::UnknownError),
        }
    }
}
