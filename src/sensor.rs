use crate::error;
use gpio_cdev;
use rand::distributions::{Distribution, Normal};
use std::error as std_error;
use std::sync;

pub fn get_measurement<S>(
    sensor_mut: &sync::Arc<sync::Mutex<S>>,
) -> Result<f32, Box<std_error::Error>>
where
    S: Sensor,
{
    let sensor = match sensor_mut.lock() {
        Ok(sensor) => sensor,
        Err(err) => {
            return Err(Box::new(error::KeyError)); // TODO: correct error
        }
    };

    match sensor.get_measurement() {
        Ok(measurement) => Ok(measurement),
        Err(e) => Err(Box::new(e)),
    }
}

pub struct DummySensor {
    pub id: &'static str,
    pub prediction: f32,
    noise_level: f32,
}

impl DummySensor {
    pub fn new(id: &'static str) -> DummySensor {
        DummySensor {
            id: id,
            prediction: 0.0,
            noise_level: 0.1,
        }
    }
}

impl Sensor for DummySensor {
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
    fn get_id(&self) -> &'static str {
        self.id
    }
}

pub struct OneWireSensor {
    pub id: &'static str,
}

impl Sensor for OneWireSensor {
    fn new(id: &'static str) -> OneWireSensor {
        OneWireSensor { id: id }
    }

    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        Ok(10.0)
    }
    fn get_id(&self) -> &'static str {
        self.id
    }
}

pub trait Sensor {
    fn new(id: &'static str) -> Self;
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error>;
    fn get_id(&self) -> &'static str;
}
