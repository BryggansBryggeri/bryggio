use crate::pub_sub::{PubSubError, PubSubMsg, Subject};
use nats::{Connection, Options, Subscription};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::process::{Child, Command};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
use std::{any::type_name, path::Path};
use std::path::PathBuf;

use super::MessageParseError;


// TODO: typedefs, e.g. Port
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NatsConfig {
    pub nats_bin_path: PathBuf,
    pub nats_config: PathBuf,
    server: String,
    server_name: String,
    user: String,
    pass: String,
    listen: String,
    http_port: u32,
    websocket: WebSocket,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSocket {
    port: u32,
    no_tls: bool
}

impl WebSocket {
    pub fn dummy() -> Self {
        Self {
            port: 9222,
            no_tls: true,
        }
    }
}

impl NatsConfig {
    pub fn dummy() -> Self {
        NatsConfig {
            server: String::from("nats-server"),
            server_name: String::from("server-name"),
            user: String::from("user"),
            pass: String::from("pass"),
            nats_bin_path: PathBuf::new(),
            nats_config: PathBuf::new(),
            http_port: 8888,
            listen: String::from("localhost:4222"),
            websocket: WebSocket::dummy(),

        }
    }
}

pub fn decode_nats_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, MessageParseError> {
    let json_string =
        from_utf8(&data).map_err(|err| MessageParseError::InvalidUtf8(data.to_vec(), err))?;
    serde_json::from_str(json_string).map_err(|err| {
        MessageParseError::Deserialization(String::from(json_string), type_name::<T>(), err)
    })
}

#[derive(Clone)]
pub struct NatsClient(Connection);

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts = Options::with_user_pass(&config.user, &config.pass);
        match opts.connect(&config.server) {
            Ok(nc) => Ok(NatsClient(nc)),
            Err(err) => Err(PubSubError::Server(err.to_string())),
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

pub fn run_nats_server(bin_path: &Path, config: &Path) -> Result<Child, PubSubError> {
    let child = Command::new(bin_path).arg("-c").arg(config).spawn();

    // Sleeps for a short while to ensure that the server is up and running before
    // the first connection comes.
    sleep(Duration::from_millis(10));
    child.map_err(|err| PubSubError::Server(err.to_string()))
}
