use crate::actor::{ActorClient, ActorConfig, ActorError};
use crate::config;
use crate::control::{ControllerClient, ControllerConfig, ControllerError};
#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;
use crate::logger::Log;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::sensor::{SensorClient, SensorConfig, SensorError};
use crate::supervisor::pub_sub::{SupervisorSubMsg, SUPERVISOR_SUBJECT};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::thread;
use thiserror::Error;

pub mod pub_sub;

type Handle = thread::JoinHandle<Result<(), SupervisorError>>;

pub struct Supervisor {
    client: NatsClient,
    config: config::Config,
    active_clients: HashMap<ClientId, Handle>,
}

impl Supervisor {
    pub fn init_from_config(config: config::Config) -> Result<Supervisor, SupervisorError> {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: HashMap::new(),
        };

        supervisor.add_logger(&config)?;

        for sensor_config in config.hardware.sensors {
            supervisor.add_sensor(sensor_config, &config.nats)?;
        }

        for actor_config in config.hardware.actors {
            supervisor.add_actor(actor_config, &config.nats)?;
        }

        Ok(supervisor)
    }

    fn process_command(&mut self, cmd: &SupervisorSubMsg) -> Result<ClientState, SupervisorError> {
        match cmd {
            SupervisorSubMsg::StartController { control_config } => {
                self.start_controller(&control_config, 0.0)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::SwitchController { control_config } => {
                self.switch_controller(control_config)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ListActiveClients => {
                self.list_active_clients();
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::KillClient { client_id: _ } => Ok(ClientState::Active),
            SupervisorSubMsg::Stop => Ok(ClientState::Active),
        }
    }

    fn start_controller(
        &mut self,
        config: &ControllerConfig,
        target: f32,
    ) -> Result<(), SupervisorError> {
        config
            .client_ids()
            .map(|id| self.client_is_active(id))
            .collect::<Result<Vec<_>, SupervisorError>>()?;

        let controller = config.get_controller(target)?;
        let controller_client = ControllerClient::new(
            config.controller_id.clone(),
            config.actor_id.clone(),
            config.sensor_id.clone(),
            controller,
            &self.config.nats,
        );
        let control_handle =
            thread::spawn(|| controller_client.client_loop().map_err(|err| err.into()));
        Ok(self.add_client(config.controller_id.clone(), control_handle)?)
    }

    fn switch_controller(&mut self, config: &ControllerConfig) -> Result<(), SupervisorError> {
        let contr_id = &config.controller_id;
        let target: f32 = self.kill_client(contr_id)?;
        println!("Keeping target: {}", target);
        self.start_controller(config, target)
    }

    fn list_active_clients(&self) {
        println!("Active clients:");
        for cl in self.active_clients.keys() {
            println!("{}", cl);
        }
    }

    fn kill_client<T: DeserializeOwned>(&mut self, id: &ClientId) -> Result<T, SupervisorError> {
        let handle = match self.active_clients.remove(id) {
            Some(contr) => Ok(contr),
            None => Err(SupervisorError::Missing(id.clone())),
        }?;
        let report = self.client.request(
            &SupervisorSubMsg::subject(id, "kill"),
            &PubSubMsg(String::new()),
        )?;
        match handle.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(SupervisorError::ThreadJoin(id.clone()));
            }
        };
        Ok(decode_nats_data::<T>(&report.data)?)
    }

    fn add_logger(&mut self, config: &config::Config) -> Result<(), SupervisorError> {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop().map_err(|err| err.into()));
        self.add_client(ClientId("log".into()), log_handle)
    }

    fn add_sensor(
        &mut self,
        sensor_config: SensorConfig,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let sensor = SensorClient::new(
            sensor_config.id.clone(),
            sensor_config.get_sensor()?,
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop().map_err(|err| err.into()));
        self.add_client(sensor_config.id, handle)
    }

    fn add_actor(
        &mut self,
        actor_config: ActorConfig,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let actor = actor_config.get_actor()?;
        let actor = ActorClient::new(actor_config.id.clone(), actor, config);
        let handle = thread::spawn(|| actor.client_loop().map_err(|err| err.into()));
        self.add_client(actor_config.id, handle)
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), SupervisorError> {
        if self.active_clients.contains_key(id) {
            Ok(())
        } else {
            Err(SupervisorError::Missing(id.clone()))
        }
    }

    fn add_client(&mut self, new_client: ClientId, handle: Handle) -> Result<(), SupervisorError> {
        match self.active_clients.get(&new_client) {
            Some(_) => Err(SupervisorError::AlreadyActive(new_client)),
            None => {
                self.active_clients.insert(new_client, handle);
                Ok(())
            }
        }
    }

    fn handle_err(err: SupervisorError) -> ClientState {
        println!("{}", err.to_string());
        ClientState::Active
    }
}

#[derive(Error, Debug, Clone)]
pub enum SupervisorError {
    #[error("This should be its own error: {0}")]
    Cli(String),
    #[error("'{0}' is not an active client")]
    Missing(ClientId),
    #[error("'{0}' is already an active client")]
    AlreadyActive(ClientId),
    #[error("Control error")]
    Controller(#[from] ControllerError),
    #[error("Sensor error")]
    Sensor(#[from] SensorError),
    #[error("Actor error")]
    Actor(#[from] ActorError),
    #[error("Pubsub error")]
    PubSub(#[from] PubSubError),
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    #[error("Could not join thread with client id {0}")]
    ThreadJoin(ClientId),
}
