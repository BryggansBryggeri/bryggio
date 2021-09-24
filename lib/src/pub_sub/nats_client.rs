use crate::logger::LogLevel;
use crate::pub_sub::{PubSubError, PubSubMsg, Subject};
use crate::supervisor::config::SupervisorConfig;
use nats::{Connection, Options, Subscription};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::Write;
use std::process::{Child, Command};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
use std::{any::type_name, path::Path};
use tempfile::NamedTempFile;

use super::MessageParseError;

pub fn run_nats_server(config: &NatsConfig, bin_path: &Path) -> Result<Child, PubSubError> {
    // Some NATS settings (Web socket in particular) cannot be set with a command line flag.
    // Instead we generate a temporary config file.
    //
    // NATS config "language" is very forgiving. Serialising NatsConfig as prett JSON parses fine.
    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|err| PubSubError::Configuration(err.to_string()))?;

    println!("Starting NATS with config:\n{}", &config_str);
    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file, "{}", &config_str)?;

    let child = Command::new(bin_path)
        .arg("-c")
        .arg(temp_file.path())
        .spawn();
    // Sleeps for a short while to ensure that the server is up and running before
    // the first connection comes.
    sleep(Duration::from_millis(10));
    child.map_err(|err| PubSubError::Server(err.to_string()))
}

// TODO: typedefs, e.g. Port
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NatsConfig {
    server_name: String,
    listen: String,
    http_port: u32,
    debug: bool,
    authorization: Authorization,
    websocket: WebSocket,
}

impl From<SupervisorConfig> for NatsConfig {
    fn from(config: SupervisorConfig) -> Self {
        let debug = config.general.log_level <= LogLevel::Debug;
        let nats = config.nats;
        Self {
            server_name: nats.server_name,
            listen: nats.listen,
            http_port: nats.http_port,
            debug,
            authorization: Authorization::new(nats.user, nats.pass),
            websocket: nats.websocket,
        }
    }
}

impl NatsConfig {
    pub fn new(
        server_name: String,
        http_port: u32,
        debug: bool,
        listen: String,
        authorization: Authorization,
        websocket: WebSocket,
    ) -> Self {
        NatsConfig {
            server_name,
            http_port,
            debug,
            listen,
            authorization,
            websocket,
        }
    }
    pub fn dummy() -> Self {
        Self::new(
            String::from("server-name"),
            8888,
            true,
            String::from("localhost:4222"),
            Authorization::dummy(),
            WebSocket::dummy(),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Authorization {
    user: String,
    password: String,
}

impl Authorization {
    pub fn new(user: String, password: String) -> Self {
        Self { user, password }
    }

    pub(crate) fn dummy() -> Self {
        Self::new(String::from("user"), String::from("passwd"))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WebSocket {
    port: u32,
    no_tls: bool,
}

impl WebSocket {
    pub fn dummy() -> Self {
        Self {
            port: 9222,
            no_tls: true,
        }
    }
}

pub fn decode_nats_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, MessageParseError> {
    let json_string =
        from_utf8(data).map_err(|err| MessageParseError::InvalidUtf8(data.to_vec(), err))?;
    serde_json::from_str(json_string).map_err(|err| {
        MessageParseError::Deserialization(String::from(json_string), type_name::<T>(), err)
    })
}

#[derive(Clone)]
pub struct NatsClient(Connection);

impl NatsClient {
    pub fn try_new(config: &NatsConfig) -> Result<NatsClient, PubSubError> {
        let opts =
            Options::with_user_pass(&config.authorization.user, &config.authorization.password);
        match opts.connect(&config.listen) {
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
