use crate::error;
use crate::utils;
use gpio_cdev;
use rand::distributions::{Distribution, Normal};
use std::error as std_error;
use std::fs::File;
use std::io::prelude::*;
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

#[derive(Debug)]
pub struct OneWireSensor {
    pub id: &'static str,
    address: OneWireAddress,
}

#[derive(Debug)]
struct OneWireAddress(String);

impl OneWireAddress {
    pub fn from_string(s: String) -> Result<OneWireAddress, error::SensorError> {
        match &s[0..2] {
            "28" => {}
            _ => return Err(error::SensorError),
        }
        match s.len() {
            13 => {}
            _ => return Err(error::SensorError),
        }
        Ok(OneWireAddress(s))
    }
}

impl OneWireSensor {
    pub fn new(id: &'static str, address: &'static str) -> OneWireSensor {
        let address = OneWireAddress::from_string(String::from(address)).unwrap();
        OneWireSensor { id, address }
    }

    fn read_one_wire_file(&self) -> Result<f32, error::SensorError> {
        let device_path = format!("/sys/bus/w1/devices/{}/w1_slave", self.address.0);
        let raw_read = match utils::read_file_to_string(&device_path) {
            Ok(raw_read) => raw_read,
            Err(err) => panic!("File read error"),
        };
        let measurement: f32 = match raw_read.parse() {
            Ok(measurement) => measurement,
            Err(err) => panic!("Float convert error"),
        };
        Ok(measurement)
    }
}

impl Sensor for OneWireSensor {
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error> {
        Ok(10.0)
    }

    fn get_id(&self) -> &'static str {
        self.id
    }
}

pub trait Sensor {
    fn get_measurement(&self) -> Result<f32, gpio_cdev::errors::Error>;
    fn get_id(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_address_wrong_start() {
        let string = String::from("30FF4E1F69140");
        let address = OneWireAddress::from_string(string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_too_short() {
        let string = String::from("284E1F69140");
        let address = OneWireAddress::from_string(string);
        address.unwrap();
    }
}
