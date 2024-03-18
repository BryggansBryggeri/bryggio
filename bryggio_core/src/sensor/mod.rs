//! General sensor logic
pub mod cpu_temp;
pub mod ds18b20;
pub mod dummy;
mod pub_sub;
use std::fmt;
use std::path::Path;

use crate::pub_sub::{ClientId, PubSubError};
pub use crate::sensor::pub_sub::{SensorClient, SensorMsg};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use ds18b20::Ds18b20Address;

pub struct SensorList {
    cpu: Vec<u32>,
    ds18b20: Vec<Ds18b20Address>,
}

impl SensorList {
    pub fn list() -> Self {
        let cpu_path = Path::new("/sys/class/thermal");
        let mut cpu: Vec<u32> = if let Ok(dir_iter) = cpu_path.read_dir() {
            dir_iter
                .filter_map(|x| x.ok())
                .map(|x| x.path().to_string_lossy().to_string())
                .filter(|x| x.contains("thermal_zone"))
                .map(|x| {
                    x.split("thermal_zone")
                        .take(2)
                        .last()
                        .expect("separator exists")
                        .parse::<u32>()
                        .expect("failed parsing cpu file id")
                })
                .collect()
        } else {
            Vec::new()
        };
        cpu.sort();

        let ds18b20: Vec<Ds18b20Address> = ds18b20::list_available().unwrap_or(Vec::new());

        SensorList { cpu, ds18b20 }
    }
}

impl fmt::Display for SensorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sensors:\n\nCPU:\n{}\n\nDS18B20:\n {}",
            self.cpu
                .iter()
                .map(|x| format!("\t - thermal_zone{}", x))
                .collect::<Vec<_>>()
                .join("\n"),
            self.ds18b20
                .iter()
                .map(|x| format!("\t - {}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Common sensor interface
pub trait Sensor: Send {
    /// Make a reading from a sensor
    fn get_measurement(&mut self) -> Result<f32, SensorError>;
    /// Return unique internal ID
    fn get_id(&self) -> String;
}

/// Sensor type list
///
/// Helper type for creating sensors at runtime using [`SensorConfig::create_sensor`]
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum SensorType {
    #[serde(rename = "dummy")]
    Dummy(u64),
    #[serde(rename = "dsb")]
    Dsb(Ds18b20Address),
    #[serde(rename = "rbpi_cpu")]
    RbpiCpu(u64),
}

/// Sensor config
///
/// Helper type for creating sensors at runtime using [`SensorConfig::create_sensor`]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorConfig {
    /// Pub-sub ID, must be unique.
    pub id: ClientId,
    #[serde(rename = "type")]
    pub type_: SensorType,
}

impl SensorConfig {
    /// Create sensor at runtime with dynamic dispatch.
    ///
    /// The type of the sensors specified in e.g., config files cannot be known at runtime,
    /// therefore are we forced to use dynamic dispatch.
    pub fn create_sensor(&self) -> Result<Box<dyn Sensor>, SensorError> {
        match &self.type_ {
            SensorType::Dummy(delay_in_ms) => {
                let sensor = dummy::DummySensor::new(self.id.as_ref(), *delay_in_ms);
                Ok(Box::new(sensor))
            }
            SensorType::Dsb(addr) => {
                let sensor = ds18b20::Ds18b20::try_new(self.id.as_ref(), addr.as_ref())?;
                Ok(Box::new(sensor))
            }
            SensorType::RbpiCpu(delay_in_ms) => {
                let sensor = cpu_temp::CpuTemp::new(self.id.as_ref(), *delay_in_ms);
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
    FileRead(String),
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
