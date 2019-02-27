use crate::hardware;
use gpio_cdev;

pub struct DummySensor {
    pub id: &'static str,
    current_power: f32,
}

impl Sensor for DummySensor {
    fn new(id: &'static str) -> DummySensor {
        DummySensor {
            id: id,
            current_power: 0.0,
        }
    }
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        1.0
    }
}

pub trait Sensor {
    fn new(id: &'static str) -> Self;
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error>;
}
