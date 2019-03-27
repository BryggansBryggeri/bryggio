use gpio_cdev;
use rand::distributions::{Distribution, Normal};

pub struct DummySensor {
    pub id: &'static str,
    pub prediction: f32,
    noise_level: f32,
}

impl Sensor for DummySensor {
    type SensorData = f32;
    fn new(id: &'static str) -> DummySensor {
        DummySensor {
            id: id,
            prediction: 0.0,
            noise_level: 0.1,
        }
    }
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        let true_measurement = self.prediction;
        let normal = Normal::new(0.0, self.noise_level as f64);
        let measurement = true_measurement + normal.sample(&mut rand::thread_rng()) as f32;
        Ok(measurement)
    }
}

pub struct OneWireSensor {
    pub id: &'static str,
}

impl Sensor for OneWireSensor {
    type SensorData = f32;

    fn new(id: &'static str) -> OneWireSensor {
        OneWireSensor { id: id }
    }

    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        Ok(10.0)
    }
}

pub trait Sensor {
    type SensorData;

    fn new(id: &'static str) -> Self;
    fn get_measurement(&self) -> Result<Self::SensorData, gpio_cdev::errors::Error>;
}
