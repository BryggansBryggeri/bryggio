// pub mod cool_ds18b20;
pub mod cpu_temp;
pub mod ds18b20;
pub mod dummy;
mod pub_sub;
use crate::pub_sub::{ClientId, PubSubError};
use serde::{Deserialize, Serialize};
use std::error as std_error;

pub use crate::sensor::pub_sub::{SensorClient, SensorMsg};

pub trait Sensor: Send {
    // TODO: it's nice to have this return a common sensor error,
    // but this might snowball when more sensors are added.
    // This should be more generic
    fn get_measurement(&self) -> Result<f32, Error>;
    fn get_id(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum SensorType {
    #[serde(rename = "dummy")]
    Dummy,
    #[serde(rename = "dsb")]
    Dsb(ds18b20::Ds18b20Address),
    #[serde(rename = "rbpi_cpu")]
    RbpiCPU,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorConfig {
    pub id: ClientId,
    #[serde(rename = "type")]
    pub(crate) type_: SensorType,
}

impl SensorConfig {
    pub fn get_sensor(&self) -> Result<Box<dyn Sensor>, Error> {
        match &self.type_ {
            SensorType::Dummy => {
                let sensor = dummy::Sensor::new(self.id.as_ref());
                Ok(Box::new(sensor))
            }
            SensorType::Dsb(addr) => {
                let sensor = ds18b20::Ds18b20::try_new(self.id.as_ref(), addr.as_ref())?;
                Ok(Box::new(sensor))
            }
            SensorType::RbpiCPU => {
                let sensor = cpu_temp::CpuTemp::new(self.id.as_ref());
                Ok(Box::new(sensor))
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Error {
    InvalidAddressStart(String),
    InvalidAddressLength(usize),
    FileReadError(String),
    FileParseError(String),
    ThreadLockError(String),
    InvalidParam(String),
    UnknownSensor(String),
}

impl From<Error> for PubSubError {
    fn from(err: Error) -> PubSubError {
        PubSubError::Subscription(format!("Sensor error: '{}'", err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidAddressStart(address) => {
                write!(f, "Address must start with 28, got {}", address)
            }
            Error::InvalidAddressLength(address_length) => {
                write!(f, "Address length must be 15, got {}", address_length)
            }
            Error::FileReadError(io_message) => {
                write!(f, "Unable to read from file: {}", io_message)
            }
            Error::FileParseError(measurement) => {
                write!(f, "Could not parse value: {}", measurement)
            }
            Error::ThreadLockError(err) => write!(f, "Unable to acquire sensor lock: {}", err),
            Error::InvalidParam(err) => write!(f, "Invalid sensor param: {}", err),
            Error::UnknownSensor(err) => write!(f, "Unknown sensor: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidAddressStart(_) => "Address must start with 28",
            Error::InvalidAddressLength(_) => "Address length must be 13",
            Error::FileReadError(_) => "File read error",
            Error::FileParseError(_) => "File parse error",
            Error::ThreadLockError(_) => "Thread lock error",
            Error::InvalidParam(_) => "Invalid param error",
            Error::UnknownSensor(_) => "Unknown sensor",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_parse_dummy() {
        let config: SensorType = serde_json::from_str(
            r#"
            "dummy"
        "#,
        )
        .unwrap();
        assert!(config == SensorType::Dummy);
    }
}
