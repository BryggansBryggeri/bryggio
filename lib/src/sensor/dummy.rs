use crate::sensor;
use rand_distr::{Distribution, Normal};
use std::thread::sleep;
use std::time::Duration;

pub struct Sensor {
    pub id: String,
    pub prediction: f32,
    // TODO: Make distribution parametrised.
    //noise_level: f32,
    delay: Duration,
    rng: Normal<f32>,
}

impl Sensor {
    pub fn new(id: &str, delay_in_ms: u64) -> Sensor {
        let normal_distr = match Normal::new(0.0, 10.0) {
            Ok(normal) => normal,
            // TODO: Hardcoded error string. Normal::Error cannot be converted to string
            Err(err) => panic!("Dummy sensor normal rng: {:?}", err), //return Err(sensor::SensorError::InvalidParam(String::from(""))),
        };
        Sensor {
            id: id.into(),
            prediction: 50.0,
            delay: Duration::from_millis(delay_in_ms),
            rng: normal_distr,
        }
    }
}

impl sensor::Sensor for Sensor {
    fn get_measurement(&self) -> Result<f32, sensor::SensorError> {
        let true_measurement = self.prediction;
        let measurement = true_measurement + self.rng.sample(&mut rand::thread_rng());
        sleep(self.delay);
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
