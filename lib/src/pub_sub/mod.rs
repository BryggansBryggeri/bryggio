use std::error as std_error;
pub mod nats_client;
use nats::Subscription;

pub trait PubSubClient {
    fn client_loop(self) -> Result<(), PubSubError>;
    fn subscribe(&self, subject: &Subject) -> Subscription;
    fn publish(&self, subject: &Subject, msg: &Message);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId(pub String);
pub struct Subject(pub String);
pub struct Message(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum PubSubError {
    Generic(String),
    MessageParse(String),
}

impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PubSubError::Generic(err) => write!(f, "Can you be more specfic?: {}", err),
            PubSubError::MessageParse(msg) => write!(f, "Could not parse message {}", msg),
        }
    }
}
impl std_error::Error for PubSubError {
    fn description(&self) -> &str {
        match *self {
            PubSubError::Generic(_) => "Can you be more specfic?",
            PubSubError::MessageParse(_) => "Message parsing error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
