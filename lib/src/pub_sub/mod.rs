use derive_more::{Display, From};
pub mod nats_client;
use nats::Subscription;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientState {
    Inactive,
    Active,
}

pub trait PubSubClient {
    fn client_loop(self) -> Result<(), PubSubError>;
    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError>;
    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError>;
}

#[derive(From, Serialize, Deserialize, Display, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId(pub String);

impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<ClientId> for String {
    fn from(x: ClientId) -> Self {
        x.0
    }
}

impl From<&str> for ClientId {
    fn from(x: &str) -> Self {
        String::from(x).into()
    }
}
#[derive(Display, Debug, Clone)]
pub struct Subject(pub String);

impl AsRef<str> for Subject {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Display)]
pub struct PubSubMsg(pub String);

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PubSubError {
    #[error("Could you be more specific? {0}")]
    Generic(String),
    #[error("Error subscribing to NATS server: {0}")]
    Subscription(String),
    #[error("Error replying to: {msg}. {err}")]
    Reply { msg: String, err: String },
    #[error("Error publishing to NATS server: {0}")]
    Publish(String),
    #[error("Config error: {0}")]
    Configuration(String),
    #[error("Client error: {0}")]
    Client(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Message parsing error: {0}")]
    MessageParse(String),
}
