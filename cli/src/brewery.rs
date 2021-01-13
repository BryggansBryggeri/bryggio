use crate::opts::BreweryOpt;
use bryggio_lib::config::Config;
use bryggio_lib::pub_sub::nats_client::NatsClient;
use bryggio_lib::pub_sub::{PubSubMsg, Subject};

pub fn process_command(command: &BreweryOpt) {
    let config = Config::try_new(&command.config).unwrap_or_else(|err| {
        panic!(
            "Error parsing config '{}': {}",
            command.config.to_string_lossy(),
            err
        )
    });
    let client = NatsClient::try_new(&config.nats).unwrap_or_else(|err| {
        panic!(
            "Error connecting to NATS server:\n{:?}\n{}",
            &config.nats, err
        );
    });

    client
        .publish(
            &Subject(command.topic.clone()),
            &PubSubMsg(command.msg.clone()),
        )
        .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
    println!("published!");
}
