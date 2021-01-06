use crate::pub_sub::{PubSubError, PubSubMsg, Subject};
use nats::{Connection, Options, Subscription};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::any::type_name;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NatsConfig {
    pub(crate) bin_path: PathBuf,
    pub(crate) config: PathBuf,
    server: String,
    user: String,
    pass: String,
}

impl NatsConfig {
    pub(crate) fn dummy() -> Self {
        NatsConfig {
            bin_path: PathBuf::new(),
            config: PathBuf::new(),
            server: String::new(),
            user: String::new(),
            pass: String::new(),
        }
    }
}

pub(crate) fn decode_nats_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, PubSubError> {
    let json_string = from_utf8(&data).map_err(|err| {
        PubSubError::MessageParse(format!(
            "Invalid UTF-8: '{:?}', '{}'",
            data,
            err.to_string()
        ))
    })?;
    serde_json::from_str(json_string).map_err(|err| {
        PubSubError::MessageParse(format!(
            "Could not parse '{}' as '{}'. Err: '{}'",
            json_string,
            type_name::<T>(),
            err.to_string()
        ))
    })
}

#[derive(Clone)]
pub struct NatsClient(Connection);

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts = Options::with_user_pass(&config.user, &config.pass);
        match opts.connect(&config.server) {
            Ok(nc) => Ok(NatsClient(nc)),
            Err(err) => Err(PubSubError::Generic(err.to_string())),
        }
    }
    pub fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.0
            .subscribe(&subject.0)
            .map_err(|err| PubSubError::Subscription(err.to_string()))
    }

    pub fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.0
            .publish(&subject.0, &msg.0)
            .map_err(|err| PubSubError::Publish(err.to_string()))
    }

    pub fn request(
        &self,
        subject: &Subject,
        msg: &PubSubMsg,
    ) -> Result<nats::Message, PubSubError> {
        self.0
            .request(&subject.0, &msg.0)
            .map_err(|err| PubSubError::Publish(err.to_string()))
    }
}

pub fn run_nats_server(config: &NatsConfig) -> Result<Child, PubSubError> {
    let child = Command::new(&config.bin_path)
        .arg("-c")
        .arg(&config.config)
        .spawn();

    // Sleeps for a short while to ensure that the server is up and running before
    // the first connection comes.
    sleep(Duration::from_millis(10));
    child.map_err(|err| PubSubError::Server(err.to_string()))
}
