use protobuf::{CodedInputStream, Message};
use protos::payload::{SimpleSupplyPayload, SimpleSupplyPayload_Action};
use sawtooth_sdk::messages::processor::TpProcessRequest;
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::TransactionHandler;

use crate::addresser::{get_namespace, FAMILY_NAME, FAMILY_VERSION};
use crate::state::SimpleSupplyState;

pub struct SimpleSupplyTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl SimpleSupplyTransactionHandler {
    pub fn new() -> SimpleSupplyTransactionHandler {
        SimpleSupplyTransactionHandler {
            family_name: FAMILY_NAME.to_string(),
            family_versions: vec![FAMILY_VERSION.to_string()],
            namespaces: vec![get_namespace().to_string()],
        }
    }
}

impl TransactionHandler for SimpleSupplyTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let mut payload = SimpleSupplyPayload::new();
        payload
            .merge_from(&mut CodedInputStream::from_bytes(request.get_payload()))
            .map_err(|_| ApplyError::InternalError(String::from("failed to parse")))?;

        let mut state = SimpleSupplyState::new(context);

        match payload.get_action() {
            SimpleSupplyPayload_Action::CREATE_AGENT => {
                return create_agent(
                    &mut state,
                    request.get_header().get_signer_public_key(),
                    payload,
                );
            }
            SimpleSupplyPayload_Action::CREATE_RECORD => {}
            SimpleSupplyPayload_Action::UPDATE_RECORD => {}
            SimpleSupplyPayload_Action::TRANSFER_RECORD => {}
        }
        Ok(())
    }
}

fn create_agent(
    state: &mut SimpleSupplyState,
    public_key: &str,
    payload: SimpleSupplyPayload,
) -> Result<(), ApplyError> {
    match state.get_agent(public_key) {
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Agent with the public key {} already exists",
                public_key,
            )));
        }
        Ok(None) => (),
        Err(e) => return Err(e),
    }

    println!("{}", public_key);
    println!("{:?}", payload);

    state.set_agent(
        public_key,
        payload.get_create_agent().get_name(),
        payload.get_timestamp(),
    )
}
