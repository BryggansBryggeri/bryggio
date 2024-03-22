//! Dummy sensor for debugging and prototyping.
use crate::sensor::{Sensor, SensorError};
use rand::prelude::*;
use rand_distr::{Distribution, Normal};
use tokio::time::Duration;

/// Generic dummy sensor
///
/// Provides mock measurements through a random walk.
pub struct DummySensor {
    /// Internal ID, must be unique.
    pub id: String,
    latest_value: f32,
    // TODO: Make distribution parametrised.
    //noise_level: f32,
    /// Fake delay to simulate a real measurement.
    _delay: Duration,
    rng: Normal<f32>,
}

impl DummySensor {
    pub fn new(id: &str, delay_in_ms: u64) -> DummySensor {
        let normal_distr = match Normal::new(0.0, 10.0) {
            Ok(normal) => normal,
            Err(err) => panic!("Dummy sensor normal rng: {:?}", err),
        };
        DummySensor {
            id: id.into(),
            latest_value: 50.0,
            _delay: Duration::from_millis(delay_in_ms),
            rng: normal_distr,
        }
    }
}

impl Sensor for DummySensor {
    fn get_measurement(&mut self) -> Result<f32, SensorError> {
        let measurement = self.latest_value + self.rng.sample(&mut thread_rng()) / 10.0;
        self.latest_value = measurement;
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
