#![forbid(unsafe_code)]
use bryggio_core::supervisor::SupervisorError;
use bryggio_core::{
    control::ControllerConfig,
    pub_sub::{
        nats_client::{NatsClient, NatsClientConfig},
        PubSubError,
    },
    supervisor::pub_sub::{NewContrData, SupervisorSubMsg},
};
use std::{thread, time::Duration};

pub fn stop_supervisor(client: &NatsClient) -> Result<(), SupervisorError> {
    let msg = SupervisorSubMsg::Stop;
    Ok(client.publish(&msg.subject(), &msg.into())?)
}

pub fn setup(nats_config: &NatsClientConfig) -> Result<NatsClient, PubSubError> {
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
