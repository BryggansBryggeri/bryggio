use crate::sensor;
use crate::utils;
use lazy_static::lazy_static;
use regex;

#[derive(Debug)]
pub struct DSB1820 {
    pub id: String,
    address: DSB1820Address,
}

// TODO: Can this be done with a const fn?
// Or does that require the regex crate to provide const constructor?
// I tried it: it would work if Regex::new() was const fn.
lazy_static! {
    static ref TEMP_PATTERN: regex::Regex = regex::Regex::new(
        // More safe with the full regex, but can't get it to match with file read.
        //r"(?s:.*)(?:[a-z0-9]{2} ){9}: crc=[0-9]{2} YES(?s:.*)(?:[a-z0-9]{2} ){9}t=([0-9]{5})(?s:.*)"
        r"t=([0-9]{5})"
    )
    .unwrap();
}

impl DSB1820 {
    pub fn new(id: &str, address: &str) -> DSB1820 {
        let address = DSB1820Address::new(address).unwrap();
        let id = String::from(id);
        DSB1820 { id, address }
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

impl sensor::Sensor for DSB1820 {
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
struct DSB1820Address(String);

impl DSB1820Address {
    pub fn new(address: &str) -> Result<DSB1820Address, sensor::Error> {
        DSB1820Address::verify_address(address)?;
        Ok(DSB1820Address(String::from(address)))
    }

    pub fn verify_address(address: &str) -> Result<(), sensor::Error> {
        match &address[0..2] {
            "28" => {}
            _ => return Err(sensor::Error::InvalidAddressStart(String::from(address))),
        }
        match address.len() {
            15 => {}
            _ => return Err(sensor::Error::InvalidAddressLength(address.len())),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_correct() {
        let string = String::from("28-0416802230ff");
        let address = DSB1820Address::verify_address(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_wrong_start() {
        let string = String::from("29-0416802230ff");
        let address = DSB1820Address::verify_address(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_too_short() {
        let string = String::from("284E1F69140");
        let address = DSB1820Address::verify_address(&string);
        address.unwrap();
    }

    #[test]
    fn test_parse_temp_measurement_correct() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=28625",
        );
        let address = String::from("28-0416802230ff");
        let mock_sensor = DSB1820::new("test", &address);
        assert_eq!(
            mock_sensor.parse_temp_measurement(&temp_string).unwrap(),
            28.625
        );
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        let temp_string = String::from("nonsense");
        let address = String::from("28-0416802230ff");
        let mock_sensor = DSB1820::new("test", &address);
        assert!(mock_sensor.parse_temp_measurement(&temp_string).is_err(),);
    }
}
