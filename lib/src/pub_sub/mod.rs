use std::error as std_error;
pub(crate) mod nats;

pub(crate) trait PubSubClient {
    fn subscribe(subject: &Subject);
    fn publish(subject: &Subject, msg: &Message);
}

pub(crate) struct ClientId(String);
pub(crate) struct Subject(String);
pub(crate) struct Message(String);

#[derive(Debug, Clone, PartialEq)]
pub enum PubSubError {
    Generic(String),
}

impl std::fmt::Display for PubSubError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PubSubError::Generic(err) => write!(f, "Can you be more specfic?: {}", err),
        }
    }
}
impl std_error::Error for PubSubError {
    fn description(&self) -> &str {
        match *self {
            PubSubError::Generic(_) => "Can you be more specfic?",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
