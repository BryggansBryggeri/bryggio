pub mod config;
use crate::actor::{ActorClient, ActorConfig, ActorError};
use crate::control::{
    pub_sub::ControllerPubMsg, ControllerClient, ControllerConfig, ControllerError,
};
use crate::logger::Log;
use crate::logger::{debug, error, info};
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError,
};
use crate::sensor::{SensorClient, SensorConfig, SensorError};
use crate::supervisor::pub_sub::{SupervisorPubMsg, SupervisorSubMsg};
use crate::time::TimeStamp;
use nats::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;
use thiserror::Error;

pub mod pub_sub;

type Handle = thread::JoinHandle<Result<(), SupervisorError>>;

pub struct Supervisor {
    client: NatsClient,
    config: config::SupervisorConfig,
    active_clients: ActiveClients,
}

impl Supervisor {
    pub fn init_from_config(
        config: config::SupervisorConfig,
    ) -> Result<Supervisor, SupervisorError> {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: ActiveClients::new(),
        };

        supervisor.add_logger(&config)?;

        for sensor_config in config.hardware.sensors {
            supervisor.add_sensor(sensor_config, &config.nats)?;
        }

        for actor_config in config.hardware.actors {
            supervisor.add_actor(actor_config, &config.nats)?;
        }

        info(&supervisor, String::from("Supervisor ready"), "supervisor");
        Ok(supervisor)
    }

    fn process_command(
        &mut self,
        cmd: SupervisorSubMsg,
        full_msg: &Message,
    ) -> Result<ClientState, SupervisorError> {
        match cmd {
            SupervisorSubMsg::StartController { contr_data } => {
                self.start_controller(contr_data.config, contr_data.new_target)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::SwitchController { contr_data } => {
                self.switch_controller(contr_data.config, contr_data.new_target, full_msg)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ListActiveClients => {
                if let Err(err) = self.reply_active_clients(&full_msg) {
                    full_msg.respond(format!("Error replying with active clients. {}", err))?;
                    error(
                        self,
                        format!("Failed replying with active clients. {}", err),
                        "supervisor",
                    );
                };
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::Stop => Ok(ClientState::Active),
        }
    }

    fn start_controller(
        &mut self,
        contr_config: ControllerConfig,
        target: f32,
    ) -> Result<(), SupervisorError> {
        contr_config
            .client_ids()
            .map(|id| self.client_is_active(id))
            .collect::<Result<Vec<_>, SupervisorError>>()?;

        let id = &contr_config.controller_id;
        match self.active_clients.controllers.get(id) {
            Some(_) => Err(SupervisorError::AlreadyActive(id.clone())),
            None => {
                let controller_client = ControllerClient::new(
                    id.clone(),
                    contr_config.actor_id.clone(),
                    contr_config.sensor_id.clone(),
                    contr_config.get_controller(target)?,
                    &self.config.nats,
                    contr_config.type_.clone(),
                );
                let control_handle =
                    thread::spawn(|| controller_client.client_loop().map_err(|err| err.into()));
                self.active_clients.controllers.insert(
                    contr_config.controller_id.clone(),
                    (control_handle, contr_config),
                );
                Ok(())
            }
        }
    }

    fn switch_controller(
        &mut self,
        config: ControllerConfig,
        new_target: f32,
        msg: &Message,
    ) -> Result<(), SupervisorError> {
        info(
            self,
            format!(
                "Switching controller to type: {:?} with target {}",
                config.type_, new_target,
            ),
            "supervisor",
        );
        let contr_id = &config.controller_id;
        let _: f32 = self.kill_client(contr_id)?;
        self.start_controller(config.clone(), new_target)?;
        let status: PubSubMsg = ControllerPubMsg::Status {
            id: contr_id.clone(),
            timestamp: TimeStamp::now(),
            target: new_target,
            type_: config.type_,
        }
        .into();
        Ok(msg
            .respond(status.to_string())
            .map_err(|err| PubSubError::Reply {
                msg: msg.to_string(),
                err: err.to_string(),
            })?)
    }

    fn reply_active_clients(&self, msg: &Message) -> Result<(), PubSubError> {
        debug(self, String::from("Listing active clients"), "supervisor");
        let clients: PubSubMsg =
            SupervisorPubMsg::ActiveClients((&self.active_clients).into()).into();
        msg.respond(clients.to_string())
            .map_err(|err| PubSubError::Reply {
                msg: msg.to_string(),
                err: err.to_string(),
            })
    }

    fn kill_client<T: DeserializeOwned>(&mut self, id: &ClientId) -> Result<T, SupervisorError> {
        // TODO: NOn-generic hack.
        let (handle, _config) = match self.active_clients.controllers.remove(id) {
            Some(contr) => Ok(contr),
            None => Err(SupervisorError::Missing(id.clone())),
        }?;
        let msg = SupervisorPubMsg::KillClient {
            client_id: id.clone(),
        };
        let report = self.client.request(&msg.subject(), &msg.into())?;
        match handle.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(SupervisorError::ThreadJoin(id.clone()));
            }
        };
        Ok(decode_nats_data::<T>(&report.data)?)
    }

    fn add_logger(&mut self, config: &config::SupervisorConfig) -> Result<(), SupervisorError> {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop().map_err(|err| err.into()));
        self.add_misc_client(ClientId("log".into()), log_handle)
    }

    fn add_sensor(
        &mut self,
        sensor_config: SensorConfig,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let id = &sensor_config.id;
        match self.active_clients.sensors.get(id) {
            Some(_) => Err(SupervisorError::AlreadyActive(id.clone())),
            None => {
                let sensor = SensorClient::new(
                    sensor_config.id.clone(),
                    sensor_config.get_sensor()?,
                    config,
                );
                let handle = thread::spawn(|| sensor.client_loop().map_err(|err| err.into()));
                self.active_clients
                    .sensors
                    .insert(id.clone(), (handle, sensor_config));
                Ok(())
            }
        }
    }

    fn add_actor(
        &mut self,
        actor_config: ActorConfig,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let id = &actor_config.id;
        match self.active_clients.actors.get(id) {
            Some(_) => Err(SupervisorError::AlreadyActive(id.clone())),
            None => {
                let actor = ActorClient::new(id.clone(), actor_config.get_actor()?, config);
                let handle = thread::spawn(|| actor.client_loop().map_err(|err| err.into()));
                self.active_clients
                    .actors
                    .insert(id.clone(), (handle, actor_config));
                Ok(())
            }
        }
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), SupervisorError> {
        if self.active_clients.contatins_id(id) {
            Ok(())
        } else {
            Err(SupervisorError::Missing(id.clone()))
        }
    }

    fn add_misc_client(
        &mut self,
        new_client: ClientId,
        handle: Handle,
    ) -> Result<(), SupervisorError> {
        match self.active_clients.misc.get(&new_client) {
            Some(_) => Err(SupervisorError::AlreadyActive(new_client)),
            None => {
                self.active_clients.misc.insert(new_client, handle);
                Ok(())
            }
        }
    }

    fn handle_err(&self, err: SupervisorError) -> ClientState {
        error(self, err.to_string(), "supervisor");
        ClientState::Active
    }
}

#[derive(Debug)]
struct ActiveClients {
    sensors: HashMap<ClientId, (Handle, SensorConfig)>,
    actors: HashMap<ClientId, (Handle, ActorConfig)>,
    controllers: HashMap<ClientId, (Handle, ControllerConfig)>,
    misc: HashMap<ClientId, Handle>,
}

impl ActiveClients {
    fn new() -> Self {
        ActiveClients {
            sensors: HashMap::new(),
            actors: HashMap::new(),
            controllers: HashMap::new(),
            misc: HashMap::new(),
        }
    }

    fn contatins_id(&self, id: &ClientId) -> bool {
        self.sensors.contains_key(id)
            || self.actors.contains_key(id)
            || self.controllers.contains_key(id)
            || self.misc.contains_key(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActiveClientsList {
    sensors: HashMap<ClientId, SensorConfig>,
    actors: HashMap<ClientId, ActorConfig>,
    controllers: HashMap<ClientId, ControllerConfig>,
    misc: Vec<ClientId>,
}

impl From<&ActiveClients> for ActiveClientsList {
    fn from(clients: &ActiveClients) -> Self {
        ActiveClientsList {
            sensors: clients
                .sensors
                .iter()
                .map(|(id, (_, config))| (id.clone(), config.clone()))
                .collect(),
            actors: clients
                .actors
                .iter()
                .map(|(id, (_, config))| (id.clone(), config.clone()))
                .collect(),
            controllers: clients
                .controllers
                .iter()
                .map(|(id, (_, config))| (id.clone(), config.clone()))
                .collect(),
            misc: clients.misc.iter().map(|(id, _)| id).cloned().collect(),
        }
    }
}

#[derive(Error, Debug)]
pub enum SupervisorError {
    #[error("This should be its own error: {0}")]
    Cli(String),
    #[error("'{0}' is not an active client")]
    Io(#[from] std::io::Error),
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
    #[error("Pubsub error: {0}")]
    PubSub(#[from] PubSubError),
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    #[error("Could not join thread with client id {0}")]
    ThreadJoin(ClientId),
}
