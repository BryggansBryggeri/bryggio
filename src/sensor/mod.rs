pub mod dummy;
pub mod one_wire;

use std::error as std_error;
use std::sync;

pub fn get_measurement<S>(sensor_mut: &sync::Arc<sync::Mutex<S>>) -> Result<f32, Error>
where
    S: Sensor,
{
    let sensor = match sensor_mut.lock() {
        Ok(sensor) => sensor,
        Err(err) => {
            return Err(Error::ThreadLockError(err.to_string()));
        }
    };

    match sensor.get_measurement() {
        Ok(measurement) => Ok(measurement),
        Err(err) => Err(err),
    }
}

pub trait Sensor {
    fn get_measurement(&self) -> Result<f32, Error>;
    fn get_id(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidAddressStart(String),
    InvalidAddressLength(usize),
    FileReadError(String),
    FileParseError(String),
    ThreadLockError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self {
            &Error::InvalidAddressStart(address) => {
                write!(f, "Address must start with 28, got {}", address)
            }
            &Error::InvalidAddressLength(address_length) => {
                write!(f, "Address length must be 13, got {}", address_length)
            }
            &Error::FileReadError(io_message) => {
                write!(f, "Unable to read from file: {}", io_message)
            }
            &Error::FileParseError(measurement) => {
                write!(f, "Could not parse value: {}", measurement)
            }
            &Error::ThreadLockError(err) => write!(f, "Unable to acquire sensor lock: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidAddressStart(_) => "Address must start with 28",
            &Error::InvalidAddressLength(_) => "Address length must be 13",
            &Error::FileReadError(_) => "File read error",
            &Error::FileParseError(_) => "File parse error",
            &Error::ThreadLockError(_) => "Thread lock error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
