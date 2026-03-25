//! Newtypes for domain values.
use serde::{Deserialize, Serialize};

/// Temperature in degrees Celsius.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Temperature(pub f32);

/// Heater power level in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Power(f32);

impl Power {
    pub fn new(value: f32) -> Self {
        Power(value.clamp(0.0, 1.0))
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn off() -> Self {
        Power(0.0)
    }
}
