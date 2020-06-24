use crate::config;
use crate::pub_sub::{nats_client::NatsClient, ClientId, PubSubClient, PubSubError};
use crate::sensor;
use std::collections::HashMap;
use std::error as std_error;
use std::thread;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

pub enum Command {
    KillClient { client_id: ClientId },
    StartClient { client_id: ClientId },
}

pub struct Brewery {
    client: NatsClient,
    active_clients: HashMap<ClientId, thread::JoinHandle<Result<(), PubSubError>>>,
}

impl Brewery {
    pub fn init_from_config(config: &config::Config) -> Brewery {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut brewery = Brewery {
            client,
            active_clients: HashMap::new(),
        };
        let dummy_id = "dummy";
        let dummy_sensor =
            sensor::PubSubSensor::new(dummy_id, sensor::dummy::Sensor::new(dummy_id), &config.nats);
        let dummy_handle = thread::spawn(move || dummy_sensor.start_loop());
        brewery
            .active_clients
            .insert(ClientId(dummy_id.into()), dummy_handle);

        brewery
    }

    fn start_client<C>(&mut self, id: ClientId, client: C)
    where
        C: PubSubClient,
    {
    }

    pub fn run(&mut self) {}

    fn process_request(&mut self, request: &Command) -> () {
        match request {
            Command::StartClient { client_id } => {}
            Command::KillClient { client_id } => {}
        }
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
