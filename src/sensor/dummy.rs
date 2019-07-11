use crate::sensor;
use rand::distributions::{Distribution, Normal};

pub struct Sensor {
    pub id: String,
    pub prediction: f32,
    noise_level: f32,
}

impl Sensor {
    pub fn new(id: String) -> Sensor {
        Sensor {
            id: id,
            prediction: 0.0,
            noise_level: 0.1,
        }
    }
}

impl sensor::Sensor for Sensor {
    fn get_measurement(&self) -> Result<f32, sensor::Error> {
        let true_measurement = self.prediction;
        let normal = Normal::new(0.0, self.noise_level as f64);
        let measurement = true_measurement + normal.sample(&mut rand::thread_rng()) as f32;
        Ok(measurement)
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
}
