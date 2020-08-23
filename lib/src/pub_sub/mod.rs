use std::error as std_error;
pub mod nats_client;
use nats::Subscription;

pub trait PubSubClient {
    fn client_loop(self) -> Result<(), PubSubError>;
    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError>;
    fn publish(&self, subject: &Subject, msg: &Message) -> Result<(), PubSubError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId(pub String);
#[derive(Debug)]
pub struct Subject(pub String);
#[derive(Debug)]
pub struct Message(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum PubSubError {
    Generic(String),
    Subscription(String),
    Publish(String),
    Server(String),
    MessageParse(String),
}

impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PubSubError::Generic(err) => write!(f, "Can you be more specfic?: {}", err),
            PubSubError::Subscription(msg) => {
                write!(f, "Error subscribing to NATS server: {}", msg)
            }
            PubSubError::Publish(msg) => write!(f, "Error publishing to NATS server: {}", msg),
            PubSubError::Server(msg) => write!(f, "Server error: {}", msg),
            PubSubError::MessageParse(msg) => write!(f, "Could not parse message {}", msg),
        }
    }
}
impl std_error::Error for PubSubError {
    fn description(&self) -> &str {
        match *self {
            PubSubError::Generic(_) => "Can you be more specfic?",
            PubSubError::Subscription(_) => "Subscription error",
            PubSubError::Publish(_) => "Publishing error",
            PubSubError::Server(_) => "Server error",
            PubSubError::MessageParse(_) => "Message parsing error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
