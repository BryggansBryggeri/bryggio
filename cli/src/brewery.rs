use crate::opts::PubSubOpt;
use bryggio_lib::pub_sub::nats_client::NatsClient;
use bryggio_lib::pub_sub::{PubSubMsg, Subject};
use bryggio_lib::supervisor::config::SupervisorConfig;

fn get_client(opt: &PubSubOpt) -> NatsClient {
    let config = SupervisorConfig::try_new(&opt.config).unwrap_or_else(|err| {
        panic!(
            "Error parsing config '{}': {}",
            opt.config.to_string_lossy(),
            err
        )
    });
    NatsClient::try_new(&config.nats).unwrap_or_else(|err| {
        panic!(
            "Error connecting to NATS server:\n{:?}\n{}",
            &config.nats, err
        );
    })
}

pub fn request(opt: &PubSubOpt) {
    let response = get_client(opt)
        .request(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone()))
        .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
    println!("Response: {}", response.to_string());
}

pub fn publish_command(opt: &PubSubOpt) {
    get_client(opt)
        .publish(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone()))
        .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
}
