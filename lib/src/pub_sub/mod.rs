use derive_more::{Display, From};
pub mod nats_client;
use crate::supervisor::SupervisorError;
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
#[derive(Debug)]
pub struct Subject(pub String);
#[derive(Debug)]
pub struct PubSubMsg(pub String);

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PubSubError {
    #[error("Could you be more specific?")]
    Generic(String),
    #[error("Error subscribing to NATS server")]
    Subscription(String),
    #[error("Error publishing to NATS server")]
    Publish(String),
    #[error("Config error")]
    Configuration(String),
    #[error("Client error")]
    Client(String),
    #[error("Server error")]
    Server(String),
    #[error("Message parsing error")]
    MessageParse(String),
    #[error("Supervisor error")]
    Supervisor(#[from] SupervisorError),
}

//impl std::fmt::Display for PubSubError {
//    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//        match self {
//            PubSubError::Generic(err) => write!(f, "Can you be more specfic?: {}", err),
//            PubSubError::Subscription(msg) => {
//                write!(f, "Error subscribing to NATS server: {}", msg)
//            }
//            PubSubError::Publish(msg) => write!(f, "Error publishing to NATS server: {}", msg),
//            PubSubError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
//            PubSubError::Server(msg) => write!(f, "Server error: {}", msg),
//            PubSubError::Client(msg) => write!(f, "Client error: {}", msg),
//            PubSubError::MessageParse(msg) => write!(f, "Message parse error. {}", msg),
//        }
//    }
//}
//impl std_error::Error for PubSubError {
//    fn description(&self) -> &str {
//        match *self {
//            PubSubError::Generic(_) => "Can you be more specfic?",
//            PubSubError::Subscription(_) => "Subscription error",
//            PubSubError::Publish(_) => "Publishing error",
//            PubSubError::Configuration(_) => "Configuration error",
//            PubSubError::Server(_) => "Server error",
//            PubSubError::Client(_) => "Client error",
//            PubSubError::MessageParse(_) => "Message parsing error",
//        }
//    }
//
//    fn cause(&self) -> Option<&dyn std_error::Error> {
//        None
//    }
//}
