//! Raspberry Pi HAL
//!
//! GPIO character device and 1-wire sensors.
use bryggio_core::hal::{ActorOutputs, HalError};
use bryggio_core::sensor::SensorReadings;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::drivers::ds18b20::Ds18b20;
use crate::drivers::gpio::{BinaryGpio, PwmGpio};

/// Raspberry Pi hardware abstraction.
///
/// All fields use interior mutability (`Mutex` inside each GPIO driver),
/// so the `Hal` trait's `&self` methods work without outer locking.
pub struct RbpiHal {
    temp_top: Ds18b20,
    temp_bottom: Ds18b20,
    heater: PwmGpio,
    pump: BinaryGpio,
}

impl RbpiHal {
    pub fn new(
        top_address: &str,
        bottom_address: &str,
        heater_pin: u32,
        pump_pin: u32,
    ) -> Result<Self, HalError> {
        let temp_top =
            Ds18b20::try_new(top_address).map_err(|e| HalError::SensorRead(e.to_string()))?;
        let temp_bottom =
            Ds18b20::try_new(bottom_address).map_err(|e| HalError::SensorRead(e.to_string()))?;
        let heater = PwmGpio::new(heater_pin, "bryggio-heater")
            .map_err(|e| HalError::ActorWrite(format!("heater GPIO {heater_pin}: {e}")))?;
        let pump = BinaryGpio::new(pump_pin, "bryggio-pump")
            .map_err(|e| HalError::ActorWrite(format!("pump GPIO {pump_pin}: {e}")))?;
        Ok(Self {
            temp_top,
            temp_bottom,
            heater,
            pump,
        })
    }
}

impl bryggio_core::hal::Hal for RbpiHal {
    async fn read_sensors(&self) -> SensorReadings {
        let top = self.temp_top.get_measurement().ok();
        let bottom = self.temp_bottom.get_measurement().ok();
        if top.is_none() {
            tracing::warn!("Failed to read top temperature sensor");
        }
        if bottom.is_none() {
            tracing::warn!("Failed to read bottom temperature sensor");
        }
        SensorReadings {
            vessel_temp_top: top,
            vessel_temp_bottom: bottom,
        }
    }

    async fn apply_outputs(&self, outputs: &ActorOutputs) -> Result<(), HalError> {
        self.heater.set_power(outputs.heater_power);
        self.heater
            .tick()
            .map_err(|e| HalError::ActorWrite(format!("heater: {e}")))?;
        self.pump
            .set(outputs.pump_on)
            .map_err(|e| HalError::ActorWrite(format!("pump: {e}")))?;
        Ok(())
    }

    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
