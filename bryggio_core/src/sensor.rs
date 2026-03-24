//! Sensor type definitions — no I/O.
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Concrete temperature sensor enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TempSensor {
    Ds18b20 { address: String },
    // Pt100 { adc_channel: u8 },
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
    #[error("Invalid sensor param: {0}")]
    InvalidParam(String),
}
