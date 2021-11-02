//! # NATS Pub-sub client
//!
//! BryggIO uses NATS for its pub-sub needs.
//! BryggIO only constructs NATS clients, and relies on a NATS server
//! (downloaded executable) to relay communication between clients.
//!
//! <https://docs.nats.io/>
use crate::logger::LogLevel;
use crate::pub_sub::{MessageParseError, PubSubError, PubSubMsg, Subject};
use crate::supervisor::config::SupervisorConfig;
use nats::{Connection, Options, Subscription};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::write;
use std::process::{Child, Command};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
use std::{any::type_name, path::Path};

/// Run a nats-server in a separate process
pub fn run_nats_server(config: &NatsServerConfig, bin_path: &Path) -> Result<Child, PubSubError> {
    // Some NATS settings (Web socket in particular) cannot be set with a command line flag.
    // Instead we generate a temporary config file.
    //
    // NATS config "language" is very forgiving. Serialising NatsConfig as pretty JSON parses fine.
    let config_str = serde_json::to_string_pretty(&config)
        .map_err(|err| PubSubError::Configuration(err.to_string()))?;

    println!("Starting NATS with config:\n{}", &config_str);
    let config_name = bin_path
        .parent()
        .ok_or_else(|| {
            PubSubError::Configuration(format!(
                "bin_path: {} has no parent.",
                &bin_path.to_string_lossy()
            ))
        })?
        .join("nats.conf");
    write(&config_name, config_str).map_err(|err| {
        PubSubError::Configuration(format!(
            "Unable to write NATS config to {}: '{}'",
            &config_name.to_string_lossy(),
            err.to_string()
        ))
    })?;

    let child = Command::new(bin_path).arg("-c").arg(config_name).spawn();
    // Sleeps for a short while to ensure that the server is up and running before
    // the first connection comes.
    sleep(Duration::from_millis(10));
    child.map_err(|err| PubSubError::Server(err.to_string()))
}

/// Generic deserialisation of NATS messages
pub fn decode_nats_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, MessageParseError> {
    let json_string =
        from_utf8(data).map_err(|err| MessageParseError::InvalidUtf8(data.to_vec(), err))?;
    serde_json::from_str(json_string).map_err(|err| {
        MessageParseError::Deserialization(String::from(json_string), type_name::<T>(), err)
    })
}

/// # NATS client wrapper
///
/// Wraps a [`Connection`] and exposes a minimal API.
#[derive(Clone)]
pub struct NatsClient(Connection);

impl NatsClient {
    pub fn try_new(config: &NatsClientConfig) -> Result<NatsClient, PubSubError> {
        let opts =
            Options::with_user_pass(&config.authorization.user, &config.authorization.password);
        match opts.connect(&config.nats_url()) {
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

// TODO: typedefs, e.g. Port
/// Serialisable NATS config understood by the nats-server executable.
///
/// See <https://docs.nats.io/nats-server/configuration>
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NatsServerConfig {
    /// Human-readable server name, displayed in logging.
    server_name: String,
    /// Host for client connections.
    host: String,
    /// Port for client connections.
    port: u32,
    /// http port for server monitoring.
    http_port: u32,
    /// Binary log level.
    debug: bool,
    /// Authorisation credentials, currently user and passwd.
    authorization: Authorization,
    /// Websocket config, allows web client communication.
    websocket: WebSocket,
}

impl NatsServerConfig {
    pub fn new(
        server_name: String,
        host: String,
        port: u32,
        http_port: u32,
        debug: bool,
        authorization: Authorization,
        websocket: WebSocket,
    ) -> Self {
        Self {
            server_name,
            host,
            port,
            http_port,
            debug,
            authorization,
            websocket,
        }
    }
    pub fn dummy() -> Self {
        Self::new(
            String::from("server-name"),
            String::from("localhost"),
            4222,
            8888,
            true,
            Authorization::dummy(),
            WebSocket::dummy(),
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NatsClientConfig {
    /// Host for client connections.
    host: String,
    /// Port for client connections.
    port: u32,
    /// Authorisation credentials, currently user and passwd.
    authorization: Authorization,
}

impl NatsClientConfig {
    pub fn new(host: String, port: u32, authorization: Authorization) -> Self {
        Self {
            host,
            port,
            authorization,
        }
    }

    pub fn nats_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn dummy() -> Self {
        Self::new(String::from("localhost"), 4222, Authorization::dummy())
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
