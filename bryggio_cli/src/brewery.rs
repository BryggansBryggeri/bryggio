use crate::opts::PubSubOpt;
use bryggio_core::pub_sub::nats_client::{NatsClient, NatsClientConfig};
use bryggio_core::pub_sub::{PubSubError, PubSubMsg, Subject};
use bryggio_core::supervisor::config::SupervisorConfig;

fn get_client(opt: &PubSubOpt) -> Result<NatsClient, PubSubError> {
    let config = SupervisorConfig::try_new(&opt.config)
        .map_err(|err| PubSubError::Configuration(err.to_string()))?;
    NatsClient::try_new(&NatsClientConfig::from(config.nats.server))
}

pub fn request(opt: &PubSubOpt) -> Result<(), PubSubError> {
    let response =
        get_client(opt)?.request(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone()))?;
    println!("Response: {}", response.to_string());
    Ok(())
}

pub fn publish_command(opt: &PubSubOpt) -> Result<(), PubSubError> {
    println!("pub");
    get_client(opt)?.publish(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone()))
}
