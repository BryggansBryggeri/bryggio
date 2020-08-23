use crate::config;
use crate::logger::Log;
use crate::pub_sub::Message as PubSubMessage;
use crate::pub_sub::{nats_client::NatsClient, ClientId, PubSubClient, PubSubError, Subject};
use crate::sensor;
use nats::{Message, Subscription};
use std::collections::HashMap;
use std::error as std_error;
use std::thread;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

pub enum Command {
    StartClient { client_id: ClientId },
    KillClient { client_id: ClientId },
    Stop,
}

impl Command {
    pub fn try_from_msg(_msg: &Message) -> Result<Self, PubSubError> {
        todo!();
    }
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
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop());
        brewery
            .active_clients
            .insert(ClientId("log".into()), log_handle);

        let dummy_id = "dummy";
        let dummy_sensor =
            sensor::PubSubSensor::new(dummy_id, sensor::dummy::Sensor::new(dummy_id), &config.nats);
        let dummy_handle = thread::spawn(|| dummy_sensor.client_loop());
        brewery
            .active_clients
            .insert(ClientId(dummy_id.into()), dummy_handle);

        brewery
    }

    fn process_command(&self, cmd: &Command) -> bool {
        match cmd {
            Command::StartClient { client_id } => true,
            Command::KillClient { client_id } => true,
            Command::Stop => true,
        }
    }
}

impl PubSubClient for Brewery {
    fn client_loop(self) -> Result<(), PubSubError> {
        let subject = Subject("command".into());
        let sub = self.subscribe(&subject)?;
        let mut keep_running = true;
        while keep_running {
            for msg in sub.messages() {
                let cmd = match Command::try_from_msg(&msg) {
                    Ok(cmd) => cmd,
                    Err(_) => {
                        return Err(PubSubError::MessageParse(format!(
                            "Error parsing command: {}",
                            msg.to_string()
                        )))
                    }
                };
                keep_running = self.process_command(&cmd);
            }
        }
        Ok(())
    }
    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, _subject: &Subject, _msg: &PubSubMessage) -> Result<(), PubSubError> {
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
