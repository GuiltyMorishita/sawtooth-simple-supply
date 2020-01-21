#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use ctrlc;
use dotenv::dotenv;

mod subscriber;

fn main() {
    dotenv().ok();
    env_logger::init();

    let (subscriber, context) = subscriber::Subscriber::new();
    let subscriber_thread = thread::spawn(move || {
        subscriber.start();
    });

    ctrlc::set_handler(move || {
        context.cancel();
    })
    .expect("Error setting Ctrl-C handler");

    subscriber_thread.join().expect("subscriber panicked");
}
