//! Linux CPU temperature sensors
//!
//! A quasi-generic driver for getting the internal CPU temp of the hardware running BryggIO.
//! This driver is not embedded; it relies on the presence of an OS,
//! wich provides a file descriptor from which the temperature is read.
use crate::sensor::{Sensor, SensorError};
use crate::utils;

#[derive(Debug)]
pub struct CpuTemp {
    /// Internal ID, must be unique.
    pub id: String,
}

impl CpuTemp {
    pub fn new(id: &str) -> CpuTemp {
        let id = String::from(id);
        CpuTemp { id }
    }

    /// Parse CPU temperature from file
    ///
    /// The value is given in millidegrees C, but parsed to C.
    fn parse_temp_measurement(&self, raw_read: &str) -> Result<f32, SensorError> {
        let value: f32 = match raw_read.trim().parse() {
            Ok(value) => value,
            Err(err) => {
                return Err(SensorError::Parse(format!(
                    "Could not parse string '{}' to f32. Err: {}",
                    String::from(raw_read),
                    err
                )));
            }
        };
        Ok(value / 1000.0)
    }
}

impl Sensor for CpuTemp {
    /// Get DS18B20 temperature measurement
    ///
    /// The sensor is available through a file.
    /// By simply reading the contents of the file a measurement is taken,
    /// from which a float value is parsed.
    ///
    /// The returned value is the temperature in Celsius.
    fn get_measurement(&mut self) -> Result<f32, SensorError> {
        let device_path = "/sys/class/thermal/thermal_zone0/temp";
        let raw_read = match utils::read_file_to_string(device_path) {
            Ok(raw_read) => raw_read,
            Err(err) => {
                return Err(SensorError::FileRead(format!("'{}'. {}", device_path, err)));
            }
        };
        self.parse_temp_measurement(&raw_read)
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_address_correct() {
        let temp_string = String::from("50000");
        let mock_sensor = CpuTemp::new("test");
        assert_approx_eq!(
            mock_sensor.parse_temp_measurement(&temp_string).unwrap(),
            50.0
        );
    }
}
