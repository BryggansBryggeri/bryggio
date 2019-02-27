use gpio_cdev;
use rand::distributions::{Distribution, Normal};

pub struct DummySensor {
    pub id: &'static str,
    pub prediction: f32,
    noise_level: f32,
}

impl Sensor for DummySensor {
    fn new(id: &'static str) -> DummySensor {
        DummySensor {
            id: id,
            prediction: 0.0,
            noise_level: 1.0,
        }
    }
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        let true_measurement = self.prediction;
        let normal = Normal::new(0.0, self.noise_level as f64);
        let measurement = true_measurement + normal.sample(&mut rand::thread_rng()) as f32;
        Ok(measurement)
    }
}

pub trait Sensor {
    fn new(id: &'static str) -> Self;
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error>;
}
