//! Mock HAL — simulates thermal dynamics for integration testing.
//!
//! Physics-based model using real thermodynamic parameters:
//! - Heating: P·power / (m·cₚ)
//! - Cooling: Newton's law — h·A·(T - T_amb) / (m·cₚ)
//! - Basic stratification: bottom zone receives heater energy directly,
//!   top zone equilibrates via conduction and (when pump is on) convection.
use bryggio_core::hal::{ActorOutputs, HalError};
use bryggio_core::model::BrewerySimulation;
use bryggio_core::sensor::SensorReadings;
use bryggio_core::types::Temperature;
use std::sync::Mutex;

pub struct MockHal {
    sim: Mutex<BrewerySimulation>,
}

impl MockHal {
    pub fn with_params(sim: BrewerySimulation) -> Self {
        MockHal {
            sim: Mutex::new(sim),
        }
    }
}

impl bryggio_core::hal::Hal for MockHal {
    async fn read_sensors(&self) -> SensorReadings {
        let mut sim = self.sim.lock().unwrap_or_else(|e| e.into_inner());

        *sim = sim.update_sensors();
        let (temp_top, temp_bottom) = sim.temp();
        SensorReadings {
            vessel_temp_top: Some(Temperature(temp_top)),
            vessel_temp_bottom: Some(Temperature(temp_bottom)),
        }
    }

    async fn apply_outputs(&self, outputs: &ActorOutputs) -> Result<(), HalError> {
        let mut sim = self.sim.lock().unwrap_or_else(|e| e.into_inner());
        *sim = sim.update_actors(outputs);
        Ok(())
    }
}
