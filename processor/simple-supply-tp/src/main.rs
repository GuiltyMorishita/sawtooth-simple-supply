use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use sawtooth_sdk::processor::TransactionProcessor;
use simple_supply_tp::handler::SimpleSupplyTransactionHandler;
use std::process;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

fn main() {
    let matches = clap_app!(intkey =>
        (version: crate_version!())
        (about: "Intkey Transaction Processor (Rust)")
        (@arg connect: -C --connect +takes_value "connection endpoint for validator")
        (@arg verbose: -v --verbose +multiple "increase output verbosity"))
    .get_matches();

    let endpoint = matches
        .value_of("connect")
        .unwrap_or("tcp://localhost:4004");

    let console_log_level = match matches.occurrences_of("verbose") {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3 | _ => LevelFilter::Trace,
    };

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({l:5.5})} | {({M}:{L}):20.20} | {m}{n}",
        )))
        .build();

    let config = match Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(console_log_level))
    {
        Ok(x) => x,
        Err(e) => {
            for err in e.errors().iter() {
                info!("Configuration error: {}", err.to_string());
            }
            process::exit(1);
        }
    };

    match log4rs::init_config(config) {
        Ok(_) => (),
        Err(e) => {
            info!("Configuration error: {}", e.to_string());
            process::exit(1);
        }
    }

    let handler = SimpleSupplyTransactionHandler::new();
    let mut processor = TransactionProcessor::new(endpoint);

    info!("Console logging level: {}", console_log_level);

    processor.add_handler(&handler);
    processor.start();
}
