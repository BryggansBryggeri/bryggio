use crate::sensor::{Sensor, SensorError};
use rand_distr::{Distribution, Normal};
use std::thread::sleep;
use std::time::Duration;

pub struct DummySensor {
    pub id: String,
    pub prediction: f32,
    // TODO: Make distribution parametrised.
    //noise_level: f32,
    delay: Duration,
    rng: Normal<f32>,
}

impl DummySensor {
    pub fn new(id: &str, delay_in_ms: u64) -> DummySensor {
        let normal_distr = match Normal::new(0.0, 10.0) {
            Ok(normal) => normal,
            // TODO: Hardcoded error string. Normal::Error cannot be converted to string
            Err(err) => panic!("Dummy sensor normal rng: {:?}", err), //return Err(sensor::SensorError::InvalidParam(String::from(""))),
        };
        DummySensor {
            id: id.into(),
            prediction: 50.0,
            delay: Duration::from_millis(delay_in_ms),
            rng: normal_distr,
        }
    }
}

impl Sensor for DummySensor {
    fn get_measurement(&self) -> Result<f32, SensorError> {
        let true_measurement = self.prediction;
        let measurement = true_measurement + self.rng.sample(&mut rand::thread_rng());
        sleep(self.delay);
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
