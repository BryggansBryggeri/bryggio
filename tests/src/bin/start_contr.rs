#![forbid(unsafe_code)]
use bryggio_core::pub_sub::nats_client::decode_nats_data;
use bryggio_core::pub_sub::ClientId;
use bryggio_core::supervisor::ActiveClientsList;
use std::thread;
use tests::{setup, stop_supervisor};

use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use bryggio_core::{
    pub_sub::{
        nats_client::{run_nats_server, NatsClient, NatsClientConfig},
        PubSubClient, PubSubError,
    },
    supervisor::pub_sub::SupervisorSubMsg,
};

#[tokio::main]
async fn main() -> Result<(), SupervisorError> {
    let config = SupervisorConfig::dummy();
    let mut nats_server_child = run_nats_server(&config.nats.server, &config.nats.bin_path)?;
    let supervisor = Supervisor::init_from_config(config.clone())?;
    let sup_handle = thread::spawn(move || supervisor.client_loop());
    let nats_config = NatsClientConfig::from(config.nats.server);
    let client = setup(&nats_config)?;
    assert!(evaluate(&client)?);
    println!("Test passed.");
    println!("Stopping supervisor client.");
    stop_supervisor(&client)?;
    sup_handle.join().unwrap()?;
    println!("Shutting down NATS server.");
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}

fn evaluate(client: &NatsClient) -> Result<bool, SupervisorError> {
    let msg = SupervisorSubMsg::ListActiveClients;
    let active_clients = client.request(&msg.subject(), &msg.into())?;
    let active_clients: ActiveClientsList = decode_nats_data(&active_clients.data).unwrap();
    let res = active_clients.contains_id(&ClientId("controller".to_string()));
    Ok(res)
}
