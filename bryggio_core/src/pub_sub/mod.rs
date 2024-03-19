use std::str::Utf8Error;

use derive_more::{Display, From};
pub mod nats_client;
use async_nats::{Message, Subscriber};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClientState {
    Inactive,
    Active,
}

/// Common Publish-Subscribe functionality
///
/// Every BryggIO client implement this trait.
pub trait PubSubClient {
    async fn client_loop(self) -> Result<(), PubSubError>;
    async fn subscribe(&self, subject: &Subject) -> Result<Subscriber, PubSubError>;
    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError>;
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

#[derive(Display, Debug, Clone, PartialEq)]
pub struct Subject(pub String);

impl AsRef<str> for Subject {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Subject {
    fn from(x: &str) -> Self {
        Subject(String::from(x))
    }
}

#[derive(Debug, Display)]
pub struct PubSubMsg(pub String);

impl PubSubMsg {
    pub fn empty() -> Self {
        PubSubMsg(String::new())
    }
}

#[derive(Error, Debug)]
pub enum PubSubError {
    #[error("Failed subscribing to NATS server: {0}")]
    Subscription(String),
    #[error("Error replying during '{task}'. Err: '{source}', msg: {msg}.")]
    Reply {
        task: &'static str,
        msg: Message,
        source: std::io::Error,
    },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error publishing to NATS server: {0}")]
    Publish(String),
    #[error("Config error: {0}")]
    Configuration(String),
    #[error("Client error: {0}")]
    Client(String),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Message parsing failed: {0}")]
    MessageParse(#[from] MessageParseError),
}

#[derive(Error, Debug)]
pub enum MessageParseError {
    #[error("Invalid UTF-8: {1}")]
    InvalidUtf8(Vec<u8>, Utf8Error),
    #[error("Faild parsing {0} as {1}")]
    Deserialization(String, &'static str, serde_json::Error),
    #[error("Invalid subject {0}")]
    InvalidSubject(Subject),
}
