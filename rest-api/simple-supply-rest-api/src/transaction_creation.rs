use crypto::digest::Digest;
use crypto::sha2::Sha512;
use protobuf::Message;
use protos::payload::{CreateAgentAction, SimpleSupplyPayload, SimpleSupplyPayload_Action};
use sawtooth_sdk::messages::batch::{Batch, BatchHeader};
use sawtooth_sdk::messages::transaction::{Transaction, TransactionHeader};
use sawtooth_sdk::signing::Signer;
use simple_supply_tp::addresser;

pub fn make_create_agent_transaction(
    transaction_signer: Signer,
    batch_signer: Signer,
    name: &str,
    timestamp: i64,
) -> Batch {
    let public_key = transaction_signer.get_public_key().unwrap();
    let agent_address = addresser::get_agent_address(&public_key.as_hex());
    let create_agent = CreateAgentAction {
        name: name.to_string(),
        ..Default::default()
    };

    let inputs = vec![agent_address.clone()];

    let outputs = vec![agent_address.clone()];

    let payload = SimpleSupplyPayload {
        action: SimpleSupplyPayload_Action::CREATE_AGENT,
        create_agent: ::protobuf::SingularPtrField::some(create_agent),
        timestamp: timestamp as u64,
        ..Default::default()
    };
    let payload_bytes = payload.write_to_bytes().unwrap();

    make_batch(
        payload_bytes,
        inputs,
        outputs,
        transaction_signer,
        batch_signer,
    )
}

fn make_batch(
    payload_bytes: Vec<u8>,
    inputs: Vec<String>,
    outputs: Vec<String>,
    transaction_signer: Signer,
    batch_signer: Signer,
) -> Batch {
    let transaction_header = TransactionHeader {
        family_name: addresser::FAMILY_NAME.to_string(),
        family_version: addresser::FAMILY_VERSION.to_string(),
        inputs: ::protobuf::RepeatedField::from_vec(inputs),
        outputs: ::protobuf::RepeatedField::from_vec(outputs),
        signer_public_key: transaction_signer.get_public_key().unwrap().as_hex(),
        batcher_public_key: batch_signer.get_public_key().unwrap().as_hex(),
        dependencies: ::protobuf::RepeatedField::new(),
        payload_sha512: sha512(&payload_bytes),
        ..Default::default()
    };

    let transaction_header_bytes = transaction_header.write_to_bytes().unwrap();

    let transaction = Transaction {
        header: transaction_header_bytes.clone(),
        header_signature: transaction_signer.sign(&transaction_header_bytes).unwrap(),
        payload: payload_bytes,
        ..Default::default()
    };

    let batch_header = BatchHeader {
        signer_public_key: batch_signer.get_public_key().unwrap().as_hex(),
        transaction_ids: ::protobuf::RepeatedField::from_vec(vec![transaction
            .header_signature
            .clone()]),
        ..Default::default()
    };

    let batch_header_bytes = batch_header.write_to_bytes().unwrap();

    Batch {
        header: batch_header_bytes.clone(),
        header_signature: batch_signer.sign(&batch_header_bytes).unwrap(),
        transactions: ::protobuf::RepeatedField::from_vec(vec![transaction]),
        ..Default::default()
    }
}

pub fn sha512(src: &[u8]) -> String {
    let mut sha = Sha512::new();
    sha.input(src);
    sha.result_str().to_string()
}
