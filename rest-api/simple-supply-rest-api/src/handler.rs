use crate::database;
use crate::model::*;
use crate::Server;

use actix_web::error::ErrorInternalServerError;
use actix_web::{web, Error, HttpResponse, Result};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use openssl::aes::{aes_ige, AesKey};
use openssl::symm::Mode;
use rustc_hex::{FromHex, ToHex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    name: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentResponse {
    authorization: String,
}

pub async fn create_agent(
    server: web::Data<Server>,
    req: web::Json<CreateAgentRequest>,
) -> Result<HttpResponse, Error> {
    println!("{:?}", req);
    let (private_key, public_key) = server.messenger.get_new_key_pair();
    server
        .messenger
        .send_create_agent_transaction(&*private_key, &req.name, Utc::now().timestamp())
        .map_err(ErrorInternalServerError)?;

    let aes_key: Vec<u8> = FromHex::from_hex("ffffffffffffffffffffffffffffffff").unwrap();
    let encrypted_private_key =
        encrypt_private_key(&aes_key, public_key.as_slice(), private_key.as_slice())
            .map_err(ErrorInternalServerError)?;

    let hashed_password = hash_password(req.password.clone());

    let auth = NewAuth {
        public_key: public_key.as_hex(),
        hashed_password: hashed_password,
        encrypted_private_key: encrypted_private_key,
    };
    let conn = server.pool.get().map_err(ErrorInternalServerError)?;
    database::insert_auth(&conn, &auth).map_err(ErrorInternalServerError)?;

    let token = generate_auth_token("secret_key".to_string(), public_key.as_hex());

    Ok(HttpResponse::Created().json(CreateAgentResponse {
        authorization: token,
    }))
}

fn encrypt_private_key(aes_key: &[u8], public_key: &[u8], private_key: &[u8]) -> Result<String> {
    let key = AesKey::new_encrypt(aes_key).expect("invalid aes_key");

    let input = private_key;
    let mut output = [0u8; 32];
    let mut iv = public_key.to_vec();

    aes_ige(&input, &mut output, &key, &mut iv[..32], Mode::Encrypt);
    Ok(output.to_hex())
}

fn hash_password(password: String) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

fn generate_auth_token(_secret_key: String, public_key: String) -> String {
    public_key
}
