// pub mod cool_ds18b20;
pub mod cpu_temp;
pub mod ds18b20;
pub mod dummy;
mod pub_sub;
use crate::pub_sub::{ClientId, PubSubError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use crate::sensor::pub_sub::{SensorClient, SensorMsg};

pub trait Sensor: Send {
    // TODO: it's nice to have this return a common sensor error,
    // but this might snowball when more sensors are added.
    // This should be more generic
    fn get_measurement(&mut self) -> Result<f32, SensorError>;
    fn get_id(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum SensorType {
    #[serde(rename = "dummy")]
    Dummy(u64),
    #[serde(rename = "dsb")]
    Dsb(ds18b20::Ds18b20Address),
    #[serde(rename = "rbpi_cpu")]
    RbpiCPU,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorConfig {
    pub id: ClientId,
    #[serde(rename = "type")]
    pub type_: SensorType,
}

impl SensorConfig {
    pub fn get_sensor(&self) -> Result<Box<dyn Sensor>, SensorError> {
        match &self.type_ {
            SensorType::Dummy(delay_in_ms) => {
                let sensor = dummy::DummySensor::new(self.id.as_ref(), *delay_in_ms);
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Error)]
pub enum SensorError {
    #[error("Address must start with 28, got {0}")]
    InvalidAddressStart(String),
    #[error("Address length must be 15, got {0}")]
    InvalidAddressLength(usize),
    #[error("Unable to read from file: {0}")]
    Read(String),
    #[error("Could not parse value: {0}")]
    Parse(String),
    #[error("Unable to acquire sensor lock: {0}")]
    ThreadLock(String),
    #[error("Invalid sensor param: {0}")]
    InvalidParam(String),
    #[error("Unknown sensor: {0}")]
    UnknownSensor(String),
}

impl From<SensorError> for PubSubError {
    fn from(err: SensorError) -> PubSubError {
        PubSubError::Client(format!("Sensor error: '{}'", err))
    }
}
