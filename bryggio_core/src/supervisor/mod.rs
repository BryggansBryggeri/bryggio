use self::config::SupervisorConfigError;
use crate::actor::{ActorClient, ActorConfig, ActorError};
use crate::control::{
    pub_sub::ControllerPubMsg, ControllerClient, ControllerConfig, ControllerError,
};
use crate::data_logger::DataLogger;
use crate::logger::{debug, error, info, Log};
use crate::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsClientConfig},
    ClientId, ClientState, PubSubClient, PubSubError, PubSubMsg,
};
use crate::sensor::{SensorClient, SensorConfig, SensorError};
use crate::supervisor::pub_sub::{SupervisorPubMsg, SupervisorSubMsg};
use crate::time::TimeStamp;
use nats::Message;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use thiserror::Error;

pub mod config;
pub mod pub_sub;

type Handle = thread::JoinHandle<Result<(), SupervisorError>>;

/// BryggIO supervisor client
///
/// Responsible for monitoring, starting, and stopping other clients.
/// The pub-sub topics that the supervisor subscribes to is the closest thing resembling a public
/// API for BryggIO.
pub struct Supervisor {
    client: NatsClient,
    config: config::SupervisorConfig,
    active_clients: ActiveClients,
}

impl Supervisor {
    pub fn init_from_config(
        config: config::SupervisorConfig,
    ) -> Result<Supervisor, SupervisorError> {
        println!("Starting supervisor");
        let nats_config = NatsClientConfig::from(config.nats.server.clone());
        let client = NatsClient::try_new(&nats_config)?;
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: ActiveClients::new(),
        };

        supervisor.add_logger(&config)?;
        supervisor.add_data_logger(&config)?;

        for sensor_config in config.hardware.sensors {
            supervisor.add_sensor(sensor_config, &nats_config)?;
        }

        for actor_config in config.hardware.actors {
            supervisor.add_actor(actor_config, &nats_config)?;
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
                self.start_controller(contr_data.config, contr_data.new_target, full_msg)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::StopController { contr_id } => {
                self.stop_controller(&contr_id, full_msg)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::SwitchController { contr_data } => {
                self.switch_controller(contr_data.config, contr_data.new_target, full_msg)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ListActiveClients => {
                if let Err(err) = self.reply_active_clients(full_msg) {
                    full_msg
                        .respond(format!("Error replying with active clients. {}", err))
                        .map_err(|err| PubSubError::Reply {
                            task: "list active clients",
                            msg: full_msg.clone(),
                            source: err,
                        })?;
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
        msg: &Message,
    ) -> Result<(), SupervisorError> {
        let id = contr_config.controller_id.clone();
        let start_res = self.common_start_controller(contr_config, target);
        match start_res {
            Ok(()) => msg.respond(format!("Controller '{}' started", id,)),
            Err(err) => msg.respond(format!(
                "Failed starting controller '{}': {}",
                id,
                err.to_string()
            )),
        }
        .map_err(|err| PubSubError::Reply {
            task: "start controller",
            msg: msg.clone(),
            source: err,
        })
        .map_err(SupervisorError::from)
    }

    fn common_start_controller(
        &mut self,
        contr_config: ControllerConfig,
        target: f32,
    ) -> Result<(), SupervisorError> {
        // TODO: Disabled checks pga SensorBox
        // contr_config
        //     .client_ids()
        //     .map(|id| self.client_is_active(id))
        //     .collect::<Result<Vec<_>, SupervisorError>>()?;

        let id = contr_config.controller_id.clone();
        match self.active_clients.controllers.get(&id) {
            Some(_) => Err(SupervisorError::AlreadyActive(id.clone())),
            None => {
                let controller_client = ControllerClient::new(
                    id.clone(),
                    contr_config.actor_id.clone(),
                    contr_config.sensor_id.clone(),
                    contr_config.get_controller(target)?,
                    &NatsClientConfig::from(self.config.nats.server.clone()),
                    contr_config.type_.clone(),
                );
                let control_handle =
                    thread::spawn(|| controller_client.client_loop().map_err(|err| err.into()));
                self.active_clients
                    .controllers
                    .insert(id.clone(), (control_handle, contr_config));
                Ok(())
            }
        }
    }

    fn stop_controller(
        &mut self,
        contr_id: &ClientId,
        msg: &Message,
    ) -> Result<(), SupervisorError> {
        match self.common_stop_controller(contr_id) {
            Ok(()) => msg.respond(format!("Controller '{}' stopped", contr_id,)),
            Err(err) => msg.respond(format!(
                "Failed stopping controller '{}': {}",
                contr_id,
                err.to_string()
            )),
        }
        .map_err(|err| PubSubError::Reply {
            task: "stop controller",
            msg: msg.clone(),
            source: err,
        })
        .map_err(SupervisorError::from)
    }

    fn common_stop_controller(&mut self, contr_id: &ClientId) -> Result<(), SupervisorError> {
        match self.active_clients.controllers.get(contr_id) {
            Some(_) => match self.kill_client::<f32>(contr_id) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            },
            None => Err(SupervisorError::Missing(contr_id.clone())),
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
        if let Err(err) = self.common_stop_controller(contr_id) {
            msg.respond(format!(
                "Failed stopping controller '{}': {}",
                contr_id,
                err.to_string()
            ))
            .map_err(|err| PubSubError::Reply {
                task: "stop contr. when switching contr.",
                msg: msg.clone(),
                source: err,
            })?;
            return Err(SupervisorError::Missing(contr_id.clone()));
        };
        self.common_start_controller(config.clone(), new_target)?;
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
                task: "start contr. when switching contr.",
                msg: msg.clone(),
                source: err,
            })?)
    }

    fn reply_active_clients(&self, msg: &Message) -> Result<(), PubSubError> {
        debug(self, String::from("Listing active clients"), "supervisor");
        let clients = PubSubMsg::from(SupervisorPubMsg::ActiveClients(ActiveClientsList::from(
            &self.active_clients,
        )));
        msg.respond(clients.to_string())
            .map_err(|err| PubSubError::Reply {
                task: "list active clients",
                msg: msg.clone(),
                source: err,
            })
    }

    fn kill_client<T: DeserializeOwned>(&mut self, id: &ClientId) -> Result<T, SupervisorError> {
        // TODO: Non-generic hack.
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
        Ok(decode_nats_data::<T>(&report.data).map_err(PubSubError::from)?)
    }

    fn add_logger(&mut self, config: &config::SupervisorConfig) -> Result<(), SupervisorError> {
        let log = Log::new(
            &NatsClientConfig::from(config.nats.server.clone()),
            config.general.log_level,
        );
        let log_handle = thread::spawn(|| log.client_loop().map_err(|err| err.into()));
        self.add_misc_client(ClientId("log".into()), log_handle)
    }

    fn add_data_logger(
        &mut self,
        config: &config::SupervisorConfig,
    ) -> Result<(), SupervisorError> {
        let id = ClientId(String::from("data_logger"));
        let log = DataLogger::new(
            id.clone(),
            &NatsClientConfig::from(config.nats.server.clone()),
            PathBuf::from("log_file.csv"),
        );
        let log_handle = thread::spawn(|| log.client_loop().map_err(|err| err.into()));
        self.add_misc_client(id, log_handle)
    }

    fn add_sensor(
        &mut self,
        sensor_config: SensorConfig,
        config: &NatsClientConfig,
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
        config: &NatsClientConfig,
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

    // fn client_is_active(&self, id: &ClientId) -> bool {
    //     self.active_clients.contains_id(id)
    // }

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
        // match err {
        //     SupervisorError::PubSub(pub_err) => match pub_err {
        //         PubSubError::Io(err) => {
        //             panic!("{}", err);
        //         }
        //         _ => {}
        //     },
        //     _ => {}
        // }
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

    fn contains_id(&self, id: &ClientId) -> bool {
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
    #[error("Config error: {0}")]
    Config(#[from] SupervisorConfigError),
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
    #[error("Could not join thread with client id {0}")]
    ThreadJoin(ClientId),
}
