//! Sensor types and driver logic.
pub mod ds18b20;
pub mod dummy;

use ds18b20::Ds18b20Address;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Concrete sensor enum — no trait objects.
#[derive(Debug, Clone)]
pub enum TempSensor {
    Ds18b20 { address: Ds18b20Address },
    // Pt100 { adc_channel: u8 },
}

pub struct SensorList {
    ds18b20: Vec<Ds18b20Address>,
}

impl SensorList {
    pub fn list() -> Self {
        let ds18b20: Vec<Ds18b20Address> = ds18b20::list_available().unwrap_or_default();
        SensorList { ds18b20 }
    }
}

impl fmt::Display for SensorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sensors:\n\nDS18B20:\n {}",
            self.ds18b20
                .iter()
                .map(|x| format!("\t - {}", x))
                .collect::<Vec<_>>()
                .join("\n")
        )
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
    #[error("Invalid sensor param: {0}")]
    InvalidParam(String),
}
