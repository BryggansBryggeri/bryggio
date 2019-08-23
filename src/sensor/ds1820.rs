use crate::sensor;
use crate::utils;
use lazy_static::lazy_static;
use regex;

#[derive(Debug)]
pub struct DS1820 {
    pub id: String,
    address: DS1820Address,
}

// TODO: Can this be done with a const fn?
// Or does that require the regex crate to provide const constructor?
// I tried it: it would work if Regex::new() was const fn.
lazy_static! {
    static ref TEMP_PATTERN: regex::Regex = regex::Regex::new(
        r"^(?:[a-z0-9]{2} ){9}: crc=[0-9]{2} YES\n(?:[a-z0-9]{2} ){9}t=([0-9]{5})$"
    )
    .unwrap();
}

impl DS1820 {
    pub fn new(id: &str, address: &str) -> DS1820 {
        let address = DS1820Address::from_string(address).unwrap();
        let id = String::from(id);
        DS1820 { id, address }
    }

    fn parse_temp_measurement(&self, raw_read: &str) -> Result<f32, sensor::Error> {
        let mat = match TEMP_PATTERN.captures(raw_read) {
            Some(mat) => mat,
            None => {
                return Err(sensor::Error::FileParseError(format!(
                    "No match in string '{}'",
                    String::from(raw_read)
                )));
            }
        };
        let value_raw = match mat.get(1) {
            Some(mat) => mat.as_str(),
            None => {
                // Can this even happen? If match on whole regex above, then this must exist no?
                return Err(sensor::Error::FileParseError(format!(
                    "No valid temp match in string '{}'",
                    String::from(raw_read)
                )));
            }
        };
        let value: f32 = match value_raw.parse() {
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

impl sensor::Sensor for DS1820 {
    fn get_measurement(&self) -> Result<f32, sensor::Error> {
        let device_path = format!("/sys/bus/w1/devices/{}/w1_slave", self.address.0);
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

    #[test]
    fn test_parse_temp_measurement_correct() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=28625",
        );
        let address = String::from("28FF4E1F69140");
        let mock_sensor = DS1820::new("test", &address);
        assert_eq!(
            mock_sensor.parse_temp_measurement(&temp_string).unwrap(),
            28.625
        );
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        let temp_string = String::from("nonsense");
        let address = String::from("28FF4E1F69140");
        let mock_sensor = DS1820::new("test", &address);
        assert!(mock_sensor.parse_temp_measurement(&temp_string).is_err(),);
    }
}
