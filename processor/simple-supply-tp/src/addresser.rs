use crypto::digest::Digest;
use crypto::sha2::Sha512;

pub const FAMILY_NAME: &str = "simple_supply";
pub const FAMILY_VERSION: &str = "0.1";
const AGENT_PREFIX: &str = "00";
const RECORD_PREFIX: &str = "01";

pub fn get_agent_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());
    get_namespace() + AGENT_PREFIX + &sha.result_str()[..62].to_string()
}

pub fn get_record_address(record_id: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(record_id.as_bytes());
    get_namespace() + RECORD_PREFIX + &sha.result_str()[..62].to_string()
}

pub fn get_namespace() -> String {
    let mut sha = Sha512::new();
    sha.input_str(FAMILY_NAME);
    sha.result_str()[..6].to_string()
}
