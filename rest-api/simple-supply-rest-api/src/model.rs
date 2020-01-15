use crate::schema::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Insertable)]
#[table_name = "agents"]
pub struct NewAgent {
    pub public_key: String,
    pub name: String,
    pub timestamp: i64,
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Queryable)]
pub struct Agent {
    pub id: i64,
    pub public_key: String,
    pub name: String,
    pub timestamp: i64,
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Insertable)]
#[table_name = "auths"]
pub struct NewAuth {
    pub public_key: String,
    pub hashed_password: String,
    pub encrypted_private_key: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Queryable)]
pub struct Auth {
    pub public_key: String,
    pub hashed_password: String,
    pub encrypted_private_key: String,
}
