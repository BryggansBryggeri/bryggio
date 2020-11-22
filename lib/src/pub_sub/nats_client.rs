use crate::pub_sub::{PubSubError, PubSubMsg, Subject};
use nats::{Connection, Options, Subscription};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, Command};
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
