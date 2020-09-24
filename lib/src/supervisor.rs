use crate::actor;
use crate::config;
use crate::control::manual;
use crate::control::pub_sub::ControllerClient;
use crate::logger::Log;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::{NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::sensor;
use nats::{Message, Subscription};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error as std_error;
use std::thread;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

pub enum SupervisorSubMsg {
    StartClient { client_id: ClientId },
    KillClient { client_id: ClientId },
    Stop,
}

impl TryFrom<Message> for SupervisorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, PubSubError> {
        todo!();
    }
}

pub struct Supervisor {
    client: NatsClient,
    active_clients: HashMap<ClientId, thread::JoinHandle<Result<(), PubSubError>>>,
}

impl Supervisor {
    fn add_logger(&mut self, config: &config::Config) {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop());
        self.active_clients
            .insert(ClientId("log".into()), log_handle);
    }

    fn add_sensor(&mut self, id: ClientId, config: &NatsConfig) {
        let sensor = sensor::SensorClient::new(
            id.clone(),
            sensor::dummy::Sensor::new(&String::from(id.clone())),
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop());
        self.active_clients.insert(id, handle);
    }

    fn add_actor(&mut self, actor_id: ClientId, controller_id: ClientId, config: &NatsConfig) {
        let tmp_id = String::from(actor_id.clone());
        let gpio_pin = hardware_impl::get_gpio_pin(0, &tmp_id).unwrap();
        let sensor = actor::ActorClient::new(
            actor_id.clone(),
            controller_id,
            actor::simple_gpio::Actor::new(&tmp_id, gpio_pin).unwrap(),
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop());
        self.active_clients.insert(actor_id, handle);
    }

    pub fn init_from_config(config: &config::Config) -> Supervisor {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            active_clients: HashMap::new(),
        };

        supervisor.add_logger(config);

        let dummy_sensor = ClientId("dummy".into());
        supervisor.add_sensor(dummy_sensor.clone(), &config.nats);

        let controller_id = ClientId::from("test");
        let dummy_actor = ClientId("dummy".into());
        supervisor.add_actor(dummy_sensor.clone(), controller_id.clone(), &config.nats);

        let controller = manual::Controller::new();
        let controller_client = ControllerClient::new(
            controller_id.clone(),
            dummy_actor,
            dummy_sensor,
            controller,
            &config.nats,
        );
        let control_handle = thread::spawn(|| controller_client.client_loop());
        supervisor
            .active_clients
            .insert(controller_id, control_handle);

        supervisor
    }

    fn process_command(&self, cmd: &SupervisorSubMsg) -> ClientState {
        match cmd {
            SupervisorSubMsg::StartClient { client_id } => ClientState::Active,
            SupervisorSubMsg::KillClient { client_id } => ClientState::Active,
            SupervisorSubMsg::Stop => ClientState::Active,
        }
    }
}

impl PubSubClient for Supervisor {
    fn client_loop(self) -> Result<(), PubSubError> {
        let subject = Subject("command".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SupervisorSubMsg::try_from(msg) {
                    Ok(cmd) => self.process_command(&cmd),
                    Err(err) => {
                        return Err(PubSubError::MessageParse(format!(
                            "Error parsing command: {}",
                            err.to_string()
                        )))
                    }
                };
            }
        }
        Ok(())
    }
    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, _subject: &Subject, _msg: &PubSubMsg) -> Result<(), PubSubError> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Missing(String, String),
    AlreadyActive(String),
    Sensor(String),
    ConcurrencyError(String),
    ThreadJoin,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Missing(type_, id) => write!(f, "ID '{}' does not exist for {}", id, type_),
            Error::AlreadyActive(id) => write!(f, "ID is already in use: {}", id),
            Error::Sensor(err) => write!(f, "Measurement error: {}", err),
            Error::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            Error::ThreadJoin => write!(f, "Could not join thread"),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Missing(_, _) => "Requested service does not exist.",
            Error::AlreadyActive(_) => "ID is already in use.",
            Error::Sensor(_) => "Measurement error.",
            Error::ConcurrencyError(_) => "Concurrency error",
            Error::ThreadJoin => "Error joining thread.",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
