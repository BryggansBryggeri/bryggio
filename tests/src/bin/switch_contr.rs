//! This test is used to detect a bug (issue #??),
//! where the switching of controllers will generate a pub-sub error when replying to a "turn off
//! actor" command.
//! Note, everything works, the controller is switched, but the req-rep logic fails.
#![forbid(unsafe_code)]
use bryggio_core::control::pub_sub::ControllerSubMsg;
use bryggio_core::control::ControllerConfig;
use bryggio_core::supervisor::{config::SupervisorConfig, Supervisor, SupervisorError};
use bryggio_core::{
    pub_sub::{
        nats_client::{run_nats_server, NatsClient, NatsClientConfig},
        PubSubClient, PubSubError,
    },
    supervisor::pub_sub::{NewContrData, SupervisorSubMsg},
};
use std::thread;
use tests::{setup, stop_supervisor};

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
    reproduce_error(&client)?;
    stop_supervisor(&client)?;
    sup_handle.join().unwrap()?;
    nats_server_child
        .kill()
        .map_err(|err| PubSubError::Server(err.to_string()).into())
}

fn reproduce_error(client: &NatsClient) -> Result<(), SupervisorError> {
    // Setting a target to trigger the error.
    let contr_conf = ControllerConfig::dummy();
    let msg = ControllerSubMsg::SetTarget(0.5);
    client.request(
        &ControllerSubMsg::subject(&contr_conf.controller_id),
        &msg.into(),
    )?;

    let contr_data = NewContrData::new(contr_conf, 0.0);
    let msg = SupervisorSubMsg::SwitchController { contr_data };
    client.request(&msg.subject(), &msg.into())?;

    Ok(())
}
