use crate::sensor;
use crate::utils;
use lazy_static::lazy_static;
use regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path;

const DSB_DIR: &str = "/sys/bus/w1/devices/";
//const DSB_DIR: &str = "dummy/";

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
        r"t=([0-9]{4,6})"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}

impl DSB1820 {
    pub fn try_new(id: &str, address: &str) -> Result<DSB1820, sensor::Error> {
        let address = DSB1820Address::try_new(address)?;
        let id = String::from(id);
        Ok(DSB1820 { id, address })
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
        parse_temp_measurement(&raw_read)
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}
#[derive(Deserialize, Serialize, Debug)]
pub struct DSB1820Address(String);

impl DSB1820Address {
    pub fn try_new(address: &str) -> Result<DSB1820Address, sensor::Error> {
        DSB1820Address::verify(address)?;
        Ok(DSB1820Address(String::from(address)))
    }

    pub fn verify(address: &str) -> Result<(), sensor::Error> {
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

impl fmt::Display for DSB1820Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_temp_measurement(raw_read: &str) -> Result<f32, sensor::Error> {
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

pub fn list_available() -> Result<Vec<DSB1820Address>, sensor::Error> {
    println!("Finding sensors");
    let device_path = path::Path::new(DSB_DIR);
    if !device_path.exists() {
        // TODO: Better error
        return Err(sensor::Error::FileReadError(format!(
            "DSB path does not exist: '{}'",
            DSB_DIR
        )));
    } else {
    }
    let files = match fs::read_dir(device_path) {
        Ok(files) => files,
        Err(_error) => {
            return Err(sensor::Error::FileReadError(format!(
                "Unable to list DSB files {}.",
                DSB_DIR
            )))
        }
    };
    Ok(files
        .filter_map(Result::ok)
        .flat_map(dsb1820_address_from_file)
        .collect())
}

fn dsb1820_address_from_file(file: fs::DirEntry) -> Option<DSB1820Address> {
    let tmp = file.path();
    let file_name = tmp.file_name()?.to_str()?;
    match DSB1820Address::try_new(&file_name) {
        Ok(address) => Some(address),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_correct() {
        let string = String::from("28-0416802230ff");
        let address = DSB1820Address::verify(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_wrong_start() {
        let string = String::from("29-0416802230ff");
        let address = DSB1820Address::verify(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_too_short() {
        let string = String::from("284E1F69140");
        let address = DSB1820Address::verify(&string);
        address.unwrap();
    }

    #[test]
    fn test_parse_temp_measurement_three_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=101625",
        );
        assert_eq!(parse_temp_measurement(&temp_string).unwrap(), 101.625);
    }

    #[test]
    fn test_parse_temp_measurement_two_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=28625",
        );
        assert_eq!(parse_temp_measurement(&temp_string).unwrap(), 28.625);
    }

    #[test]
    fn test_parse_temp_measurement_single_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=8720",
        );
        assert_eq!(parse_temp_measurement(&temp_string).unwrap(), 8.720);
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        let temp_string = String::from("nonsense");
        assert!(parse_temp_measurement(&temp_string).is_err(),);
    }
}
