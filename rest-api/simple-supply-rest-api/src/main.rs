#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;

use actix_service::ServiceFactory;
use actix_web::body::Body;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::Error;
use actix_web::{web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;

pub mod database;
pub mod handler;
pub mod messaging;
pub mod model;
pub mod schema;
pub mod transaction_creation;

#[derive(Clone)]
pub struct Server {
    messenger: messaging::Messenger,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Server {
    pub fn new() -> Self {
        let validator_url = env::var("VALIDATOR_URL").expect("VALIDATOR_URL is not set");
        let messenger = messaging::Messenger::new(validator_url);

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        Server { messenger, pool }
    }
}

// ルーティングなどを書く。
// ちょっと返り値の型がゴツい。
pub fn app(
    server: Server,
) -> App<
    impl ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<Body>,
        Error = Error,
        InitError = (),
    >,
    Body,
> {
    use crate::handler::*;

    App::new()
        .data(server)
        .service(web::resource("/agents").route(web::post().to(create_agent)))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let server = Server::new();
    HttpServer::new(move || app(server.clone()))
        .bind("localhost:3000")?
        .run()
        .await
}
