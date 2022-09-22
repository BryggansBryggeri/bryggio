#![forbid(unsafe_code)]
use bryggio_core::pub_sub::nats_client::decode_nats_data;
use bryggio_core::pub_sub::ClientId;
use bryggio_core::supervisor::ActiveClientsList;
use std::{thread, time::Duration};

use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use bryggio_core::{
    control::ControllerConfig,
    pub_sub::{
        nats_client::{run_nats_server, NatsClient, NatsClientConfig},
        PubSubClient, PubSubError,
    },
    supervisor::pub_sub::{NewContrData, SupervisorSubMsg},
};

#[tokio::main]
async fn main() -> Result<(), SupervisorError> {
    let config = SupervisorConfig::dummy();
    // println!("{:?}", config);
    // return Ok(());
    let mut nats_server_child = run_nats_server(&config.nats.server, &config.nats.bin_path)?;
    let supervisor = Supervisor::init_from_config(config.clone())?;
    let sup_handle = thread::spawn(move || supervisor.client_loop());
    let nats_config = NatsClientConfig::from(config.nats.server);
    let client = setup(&nats_config)?;
    assert!(evaluate(&client)?);
    stop_supervisor(&client)?;
    sup_handle.join().unwrap()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}

fn stop_supervisor(client: &NatsClient) -> Result<(), SupervisorError> {
    let msg = SupervisorSubMsg::Stop;
    Ok(client.publish(&msg.subject(), &msg.into())?)
}

fn setup(nats_config: &NatsClientConfig) -> Result<NatsClient, PubSubError> {
    println!("Sleeping");
    thread::sleep(Duration::from_millis(5000));
    println!("Starting controller");
    let client = NatsClient::try_new(nats_config)?;
    let contr_data = NewContrData::new(ControllerConfig::dummy(), 0.7);
    let msg = SupervisorSubMsg::StartController { contr_data };
    client.request(&msg.subject(), &msg.into())?;
    println!("Controller started");
    Ok(client)
}

fn evaluate(client: &NatsClient) -> Result<bool, SupervisorError> {
    let msg = SupervisorSubMsg::ListActiveClients;
    let active_clients = client.request(&msg.subject(), &msg.into())?;
    let active_clients: ActiveClientsList = decode_nats_data(&active_clients.data).unwrap();
    let res = active_clients.contains_id(&ClientId("controller".to_string()));
    Ok(res)
}
