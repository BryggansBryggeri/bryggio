use crate::sensor;
use crate::utils;

#[derive(Debug)]
pub struct OneWireTemp {
    pub id: String,
    address: OneWireAddress,
}

impl OneWireTemp {
    pub fn new(id: &str, address: &str) -> OneWireTemp {
        let address = OneWireAddress::from_string(address).unwrap();
        let id = String::from(id);
        OneWireTemp { id, address }
    }
}

impl sensor::Sensor for OneWireTemp {
    fn get_measurement(&self) -> Result<f32, sensor::Error> {
        let device_path = format!("/sys/bus/w1/devices/{}/w1_slave", self.address.0);
        let raw_read = match utils::read_file_to_string(&device_path) {
            Ok(raw_read) => raw_read,
            Err(err) => {
                return Err(sensor::Error::FileReadError(err.to_string()));
            }
        };
        let measurement: f32 = match raw_read.parse() {
            Ok(measurement) => measurement,
            Err(_) => {
                return Err(sensor::Error::FileParseError(raw_read));
            }
        };
        Ok(measurement)
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Debug)]
struct OneWireAddress(String);

impl OneWireAddress {
    pub fn from_string(s: &str) -> Result<OneWireAddress, sensor::Error> {
        match &s[0..2] {
            "28" => {}
            _ => return Err(sensor::Error::InvalidAddressStart(String::from(s))),
        }
        match s.len() {
            13 => {}
            _ => return Err(sensor::Error::InvalidAddressLength(s.len())),
        }
        Ok(OneWireAddress(String::from(s)))
    }
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
