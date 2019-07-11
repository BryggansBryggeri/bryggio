pub mod dummy;
pub mod simple_gpio;
pub mod xor;

use std::error as std_error;

pub trait Actor {
    fn validate_signal(&self, signal: &f32) -> Result<(), Error>;
    fn set_signal(&self, signal: &f32) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidSignal(f32),
    ActorError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            &Error::InvalidSignal(signal) => write!(f, "Invalid signal: {}", signal),
            &Error::ActorError(error) => write!(f, "Actor error: {}", error),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidSignal(_) => "Invalid signal",
            &Error::ActorError(_) => "Actor error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
