use crate::sensor;
use rand_distr::{Distribution, Normal};
use std::thread::sleep;
use std::time::Duration;

pub struct Sensor {
    pub id: String,
    pub prediction: f32,
    noise_level: f32,
    delay: Duration,
}

impl Sensor {
    pub fn new(id: &str, delay_in_ms: u64) -> Sensor {
        Sensor {
            id: id.into(),
            prediction: 50.0,
            noise_level: 10.0,
            delay: Duration::from_millis(delay_in_ms),
        }
    }
}

impl sensor::Sensor for Sensor {
    fn get_measurement(&self) -> Result<f32, sensor::SensorError> {
        let true_measurement = self.prediction;
        let normal = match Normal::new(0.0, self.noise_level) {
            Ok(normal) => normal,
            // TODO: Hardcoded error string. Normal::Error cannot be converted to string
            Err(_) => return Err(sensor::SensorError::InvalidParam(String::from(""))),
        };
        let measurement = true_measurement + normal.sample(&mut rand::thread_rng());
        sleep(self.delay);
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
