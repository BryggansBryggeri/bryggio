//! Mock HAL — simulates thermal dynamics for integration testing.
//!
//! Physics-based model using real thermodynamic parameters:
//! - Heating: P·power / (m·cₚ)
//! - Cooling: Newton's law — h·A·(T - T_amb) / (m·cₚ)
//! - Basic stratification: bottom zone receives heater energy directly,
//!   top zone equilibrates via conduction and (when pump is on) convection.
use bryggio_core::hal::{ActorOutputs, HalError};
use bryggio_core::model::Brewery;
use bryggio_core::sensor::SensorReadings;

use crate::drivers::ds18b20::Ds18b20;
use crate::drivers::gpio::PwmGpio;

#[derive(Debug, Clone)]
pub struct RbpiHal {
    temp_top: Ds18b20,
    temp_bottom: Ds18b20,
    heater: PwmGpio,
}

impl RbpiHal {
    pub fn try_from_brewery_spec(spec: Brewery) -> Result<Self, HalError> {
        let _vessel = spec.vessel;
        let hal = Self {
            temp_top: Ds18b20::try_new("ababa1").expect("Will fail"),
            temp_bottom: Ds18b20::try_new("ababa2").expect("Will fail"),
            heater: PwmGpio::new(),
        };
        Ok(hal)
    }
}

impl bryggio_core::hal::Hal for RbpiHal {
    async fn read_sensors(&self) -> SensorReadings {
        let temp_top = self
            .temp_top
            .get_measurement()
            .map_err(|err| HalError::SensorRead(err.to_string()))
            .expect("Tmp expect before trait redesign");
        let temp_bottom = self
            .temp_bottom
            .get_measurement()
            .map_err(|err| HalError::SensorRead(err.to_string()))
            .expect("Tmp expect before trait redesign");
        SensorReadings {
            vessel_temp_top: Some(temp_top),
            vessel_temp_bottom: Some(temp_bottom),
        }
    }

    async fn apply_outputs(&self, _outputs: &ActorOutputs) -> Result<(), HalError> {
        // self.heater.set_power(outputs.heater_power);
        Ok(())
    }
}
