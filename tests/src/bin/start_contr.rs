#![forbid(unsafe_code)]
use std::{thread, time::Duration};

use bryggio_core::{pub_sub::{
    nats_client::{run_nats_server, NatsClientConfig, NatsClient},
    PubSubClient, PubSubError,
}, supervisor::pub_sub::{NewContrData, SupervisorSubMsg}, control::ControllerConfig};
use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};

#[tokio::main]
async fn main() -> Result<(), SupervisorError>{
    let config = SupervisorConfig::dummy();
    let mut nats_server_child = run_nats_server(
        &config.nats.server.clone(),
        &config.nats.bin_path,
    )?;
    let supervisor = Supervisor::init_from_config(config.clone())?;
    let sup_handle = thread::spawn(move || {supervisor.client_loop()});
    let nats_config = NatsClientConfig::from(config.nats.server);
    setup(&nats_config)?;
    sup_handle.join().unwrap()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}

pub fn setup(nats_config: &NatsClientConfig) -> Result<(), PubSubError> {
    println!("Sleeping");
    thread::sleep(Duration::from_millis(5000));
    println!("Starting controller");
    let client = NatsClient::try_new(nats_config)?;
    let contr_data = NewContrData::new(ControllerConfig::dummy(), 0.7);
    let msg = SupervisorSubMsg::StartController{contr_data};
    client.request(&msg.subject(), &msg.into())?;
    Ok(())
}

pub fn evaluate(client: &NatsClient) -> bool {
    todo!()
}
