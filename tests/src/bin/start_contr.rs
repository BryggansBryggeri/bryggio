#![forbid(unsafe_code)]
use bryggio_core::pub_sub::nats_client::decode_nats_data;
use bryggio_core::pub_sub::ClientId;
use bryggio_core::supervisor::ActiveClientsList;
use tests::{setup, stop_supervisor};

use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use bryggio_core::{
    pub_sub::{
        nats_client::{run_nats_server, NatsClient, NatsClientConfig},
        PubSubClient, PubSubError,
    },
    supervisor::pub_sub::SupervisorSubMsg,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), SupervisorError> {
    let config = SupervisorConfig::dummy();
    let mut nats_server_child = run_nats_server(&config.nats.server, &config.nats.bin_path)?;
    let supervisor = Supervisor::init_from_config(config.clone()).await?;
    let sup_handle = tokio::task::spawn(async move { supervisor.client_loop() });
    sleep(Duration::from_secs(3)).await;
    let nats_config = NatsClientConfig::from(config.nats.server);
    let client = setup(&nats_config).await?;
    assert!(evaluate(&client).await?);
    println!("Test passed.");
    println!("Stopping supervisor client.");
    stop_supervisor(&client).await?;
    let _ = sup_handle.await;
    println!("Shutting down NATS server.");
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}

async fn evaluate(client: &NatsClient) -> Result<bool, SupervisorError> {
    let msg = SupervisorSubMsg::ListActiveClients;
    println!("msg, {:?}", msg);
    let active_clients = client.request(&msg.subject(), &msg.into()).await?;
    let active_clients: ActiveClientsList = decode_nats_data(&active_clients.payload).unwrap();
    let res = active_clients.contains_id(&ClientId("controller".to_string()));
    Ok(res)
}
