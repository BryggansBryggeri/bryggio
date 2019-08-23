use crate::sensor;
use crate::utils;

#[derive(Debug)]
pub struct DS1820 {
    pub id: String,
    address: DS1820Address,
}

impl DS1820 {
    pub fn new(id: &str, address: &str) -> DS1820 {
        let address = DS1820Address::from_string(address).unwrap();
        let id = String::from(id);
        DS1820 { id, address }
    }
}

impl sensor::Sensor for DS1820 {
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
struct DS1820Address(String);

impl DS1820Address {
    pub fn from_string(s: &str) -> Result<DS1820Address, sensor::Error> {
        match &s[0..2] {
            "28" => {}
            _ => return Err(sensor::Error::InvalidAddressStart(String::from(s))),
        }
        match s.len() {
            13 => {}
            _ => return Err(sensor::Error::InvalidAddressLength(s.len())),
        }
        Ok(DS1820Address(String::from(s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_address_wrong_start() {
        let string = String::from("30FF4E1F69140");
        let address = DS1820Address::from_string(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_too_short() {
        let string = String::from("284E1F69140");
        let address = DS1820Address::from_string(&string);
        address.unwrap();
    }
}
