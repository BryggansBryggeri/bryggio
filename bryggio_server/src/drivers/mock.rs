//! Mock HAL — simulates thermal dynamics for development without hardware.
//!
//! Owns a virtual clock that advances by `time_scale` seconds per tick,
//! allowing the simulation to run faster than real time.
use bryggio_core::hal::{ActorOutputs, HalError};
use bryggio_core::model::BrewerySimulation;
use bryggio_core::sensor::SensorReadings;
use bryggio_core::types::Temperature;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MockHal {
    sim: Mutex<BrewerySimulation>,
    virtual_time: Mutex<u64>,
    time_scale: u32,
}

impl MockHal {
    pub fn new(sim: BrewerySimulation, time_scale: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            sim: Mutex::new(sim),
            virtual_time: Mutex::new(now),
            time_scale,
        }
    }
}

impl bryggio_core::hal::Hal for MockHal {
    async fn read_sensors(&self) -> SensorReadings {
        let mut sim = self.sim.lock().unwrap_or_else(|e| e.into_inner());
        let mut vt = self.virtual_time.lock().unwrap_or_else(|e| e.into_inner());

        // Advance virtual clock by 1 second (always)
        *vt += 1;

        // Run physics for 1 second
        *sim = sim.update_sensors(1.0);

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

    fn now(&self) -> u64 {
        *self.virtual_time.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn tick_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(1) / self.time_scale
    }
}
