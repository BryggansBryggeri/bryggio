use crate::actor;
use crate::config;
use crate::control::pub_sub::ControllerClient;
use crate::control::ControllerConfig;
use crate::logger::Log;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::sensor;
use crate::supervisor::pub_sub::{SupervisorSubMsg, SUPERVISOR_SUBJECT};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::error as std_error;
use std::thread;
pub mod pub_sub;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

type Handle = thread::JoinHandle<Result<(), PubSubError>>;

pub struct Supervisor {
    client: NatsClient,
    config: config::Config,
    active_clients: HashMap<ClientId, Handle>,
}

impl Supervisor {
    fn handle_err(err: PubSubError) -> ClientState {
        println!("{}", err.to_string());
        ClientState::Active
    }

    fn list_active_clients(&self) {
        println!("Active clients:");
        for cl in self.active_clients.keys() {
            println!("{}", cl);
        }
    }
    fn process_command(&mut self, cmd: &SupervisorSubMsg) -> Result<ClientState, PubSubError> {
        match cmd {
            SupervisorSubMsg::StartController { control_config } => {
                self.start_controller(&control_config, 0.0)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::SwitchController { control_config } => {
                self.switch_controller(control_config)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::SetControllerTarget { id, new_target } => {
                self.set_controller_target(id, *new_target)?;
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

    fn kill_client<T: DeserializeOwned>(&mut self, id: &ClientId) -> Result<T, PubSubError> {
        let handle = match self.active_clients.remove(id) {
            Some(contr) => Ok(contr),
            None => Err(PubSubError::Client(format!(
                "'{}' not an active client",
                id
            ))),
        }?;
        let report = match self.client.request(
            &Subject(format!("{}.kill.{}", SUPERVISOR_SUBJECT, id)),
            &PubSubMsg(String::new()),
        ) {
            Ok(report) => Ok(report),
            Err(err) => {
                //self.active_clients.insert(*id, handle.clone());
                Err(err)
            }
        }?;
        match handle.join() {
            Ok(_) => {}
            Err(_) => {
                return Err(PubSubError::Client(format!(
                    "could not join client with id '{}'",
                    id,
                )));
            }
        };
        decode_nats_data::<T>(&report.data)
    }

    fn start_controller(
        &mut self,
        config: &ControllerConfig,
        target: f32,
    ) -> Result<(), PubSubError> {
        config
            .client_ids()
            .map(|id| self.client_is_active(id))
            .collect::<Result<Vec<_>, SupervisorError>>()?;

        let controller = config.get_controller(target).map_err(|err| {
            PubSubError::Client(format!("Could not start control: {}", err.to_string()))
        })?;
        let controller_client = ControllerClient::new(
            config.controller_id.clone(),
            config.actor_id.clone(),
            config.sensor_id.clone(),
            controller,
            &self.config.nats,
        );
        let control_handle = thread::spawn(|| controller_client.client_loop());
        Ok(self.add_client(config.controller_id.clone(), control_handle)?)
    }

    fn switch_controller(&mut self, config: &ControllerConfig) -> Result<(), PubSubError> {
        let contr_id = &config.controller_id;
        let target: f32 = self.kill_client(contr_id)?;
        println!("Keeping target: {}", target);
        self.start_controller(config, target)
    }

    fn set_controller_target(&self, id: &ClientId, new_target: f32) -> Result<(), PubSubError> {
        self.publish(
            &Subject(format!(
                "{}.set_controller_target.{}",
                SUPERVISOR_SUBJECT, id
            )),
            &PubSubMsg(format!("{}", new_target)),
        )
    }

    fn add_logger(&mut self, config: &config::Config) -> Result<(), SupervisorError> {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop());
        self.add_client(ClientId("log".into()), log_handle)
    }

    fn add_sensor(
        &mut self,
        sensor_id: ClientId,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let sensor = sensor::SensorClient::new(
            sensor_id.clone(),
            sensor::dummy::Sensor::new(&String::from(sensor_id.clone())),
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop());
        self.add_client(sensor_id, handle)
    }

    fn add_actor(
        &mut self,
        actor_id: ClientId,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let tmp_id = String::from(actor_id.clone());
        let gpio_pin = hardware_impl::get_gpio_pin(0, &tmp_id).unwrap();
        let actor = actor::ActorClient::new(
            actor_id.clone(),
            actor::simple_gpio::Actor::new(&tmp_id, gpio_pin).unwrap(),
            config,
        );
        let handle = thread::spawn(|| actor.client_loop());
        self.add_client(actor_id, handle)
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), SupervisorError> {
        if self.active_clients.contains_key(id) {
            Ok(())
        } else {
            Err(SupervisorError::Missing(id.clone()))
        }
    }

    pub fn init_from_config(config: config::Config) -> Result<Supervisor, SupervisorError> {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: HashMap::new(),
        };

        supervisor.add_logger(&config)?;

        let dummy_sensor = ClientId("dummy_sensor".into());
        supervisor.add_sensor(dummy_sensor, &config.nats)?;

        let dummy_actor = ClientId("dummy_actor".into());
        supervisor.add_actor(dummy_actor, &config.nats)?;

        Ok(supervisor)
    }

    fn add_client(&mut self, new_client: ClientId, handle: Handle) -> Result<(), SupervisorError> {
        if self.active_clients.contains_key(&new_client) {
            Err(SupervisorError::AlreadyActive(new_client))
        } else {
            self.active_clients.insert(new_client, handle);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SupervisorError {
    Missing(ClientId),
    AlreadyActive(ClientId),
    Sensor(String),
    ConcurrencyError(String),
    ThreadJoin,
}

impl std::fmt::Display for SupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SupervisorError::Missing(id) => write!(f, "'{}' is not an active client", id),
            SupervisorError::AlreadyActive(id) => write!(f, "'{}' is already an active client", id),
            SupervisorError::Sensor(err) => write!(f, "Measurement error: {}", err),
            SupervisorError::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            SupervisorError::ThreadJoin => write!(f, "Could not join thread"),
        }
    }
}
impl std_error::Error for SupervisorError {
    fn description(&self) -> &str {
        match *self {
            SupervisorError::Missing(_) => "Requested client does not exist.",
            SupervisorError::AlreadyActive(_) => "ID is already in use.",
            SupervisorError::Sensor(_) => "Measurement error.",
            SupervisorError::ConcurrencyError(_) => "Concurrency error",
            SupervisorError::ThreadJoin => "Error joining thread.",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
