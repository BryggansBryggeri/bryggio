//! Mock HAL — simulates thermal dynamics for integration testing.
use bryggio_core::hal::{ActorOutputs, HalError, SensorReadings};
use bryggio_core::types::Temperature;
use std::sync::Mutex;
use std::time::Instant;

pub struct MockHal {
    sim: Mutex<BrewerySimulation>,
}

struct BrewerySimulation {
    vessel_temp: f32,
    heater_power: f32,
    ambient_temp: f32,
    heating_rate: f32, // °C/s at full power
    cooling_rate: f32, // °C/s passive loss coefficient
    last_update: Instant,
}

impl MockHal {
    pub fn new() -> Self {
        MockHal {
            sim: Mutex::new(BrewerySimulation {
                vessel_temp: 20.0,
                heater_power: 0.0,
                ambient_temp: 20.0,
                heating_rate: 0.5,
                cooling_rate: 0.01,
                last_update: Instant::now(),
            }),
        }
    }
}

impl bryggio_core::hal::Hal for MockHal {
    async fn read_sensors(&self) -> SensorReadings {
        let mut sim = self.sim.lock().unwrap_or_else(|e| e.into_inner());

        // Advance physics
        let dt = sim.last_update.elapsed().as_secs_f32();
        sim.last_update = Instant::now();
        sim.vessel_temp += (sim.heating_rate * sim.heater_power
            - sim.cooling_rate * (sim.vessel_temp - sim.ambient_temp))
            * dt;

        let temp = Temperature(sim.vessel_temp);
        SensorReadings {
            vessel_temp_top: Some(temp),
            vessel_temp_bottom: Some(temp),
        }
    }

    async fn apply_outputs(&self, outputs: &ActorOutputs) -> Result<(), HalError> {
        let mut sim = self.sim.lock().unwrap_or_else(|e| e.into_inner());
        sim.heater_power = outputs.heater_power.value();
        Ok(())
    }
}
