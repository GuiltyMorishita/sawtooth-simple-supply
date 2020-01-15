use protobuf::{CodedInputStream, Message};
use protos::agent::{Agent, AgentContainer};
use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;

use crate::addresser::{get_agent_address, get_record_address};

pub struct SimpleSupplyState<'a> {
    context: &'a mut dyn TransactionContext,
}

impl<'a> SimpleSupplyState<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> SimpleSupplyState {
        SimpleSupplyState { context }
    }

    pub fn get_agent(&mut self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = get_agent_address(public_key);
        let state_entry = self.context.get_state_entry(&address)?;
        match state_entry {
            Some(data) => {
                let agent_container: AgentContainer = protobuf::parse_from_bytes(&data[..])
                    .map_err(|_| {
                        ApplyError::InternalError(String::from("failed to deserialize"))
                    })?;

                for agent in agent_container.get_entries() {
                    if agent.public_key == public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_agent(
        &mut self,
        public_key: &str,
        name: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let agent = Agent {
            public_key: String::from(public_key),
            name: String::from(name),
            timestamp: timestamp,
            ..Default::default()
        };
        let mut agent_container = AgentContainer::new();
        let address = get_agent_address(public_key);
        let state_entry = self.context.get_state_entry(&address)?;
        match state_entry {
            Some(data) => {
                agent_container
                    .merge_from(&mut CodedInputStream::from_bytes(&data[..]))
                    .map_err(|_| {
                        ApplyError::InternalError(String::from("failed to deserialize"))
                    })?;
            }
            None => (),
        }

        agent_container.entries.push(agent);

        let data = agent_container
            .write_to_bytes()
            .map_err(|_| ApplyError::InternalError(String::from("failed to serialize")))?;

        self.context
            .set_state_entry(address, data)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}
