use crate::sensor;
use crate::utils;

#[derive(Debug)]
pub struct CpuTemp {
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
    fn parse_temp_measurement(&self, raw_read: &str) -> Result<f32, sensor::Error> {
        let value: f32 = match raw_read.trim().parse() {
            Ok(value) => value,
            Err(err) => {
                return Err(sensor::Error::FileParseError(format!(
                    "Could not parse string '{}' to f32. Err: {}",
                    String::from(raw_read),
                    err.to_string()
                )));
            }
        };
        Ok(value / 1000.0)
    }
}

impl sensor::Sensor for CpuTemp {
    fn get_measurement(&self) -> Result<f32, sensor::Error> {
        let device_path = "/sys/class/thermal/thermal_zone0/temp";
        let raw_read = match utils::read_file_to_string(&device_path) {
            Ok(raw_read) => raw_read,
            Err(err) => {
                return Err(sensor::Error::FileReadError(err.to_string()));
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

    #[test]
    fn test_address_correct() {
        let temp_string = String::from("50000");
        let mock_sensor = CpuTemp::new("test");
        assert_eq!(
            mock_sensor.parse_temp_measurement(&temp_string).unwrap(),
            50.0
        );
    }
}
