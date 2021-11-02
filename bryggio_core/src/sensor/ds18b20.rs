use crate::sensor::{Sensor, SensorError};
use crate::utils;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

impl Ds18b20 {
    pub fn try_new(id: &str, address: &str) -> Result<Ds18b20, SensorError> {
        let address = Ds18b20Address::try_new(address)?;
        let id = String::from(id);
        Ok(Ds18b20 { id, address })
    }
}

impl AsRef<str> for Ds18b20Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Sensor for Ds18b20 {
    fn get_measurement(&mut self) -> Result<f32, SensorError> {
        let device_path = Path::new(DS18B20_DIR)
            .join(&self.address.0)
            .join("temperature");
        let meas = match utils::read_file_to_string(&device_path) {
            Ok(raw_read) => parse_temp_measurement(&raw_read),
            Err(err) => Err(SensorError::FileRead(err.to_string())),
        };
        sleep(Duration::from_millis(1000));
        meas
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

// TODO: Fallible serde
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Ds18b20Address(String);

impl Ds18b20Address {
    pub fn try_new(address: &str) -> Result<Ds18b20Address, SensorError> {
        Ds18b20Address::verify(address)?;
        Ok(Ds18b20Address(String::from(address)))
    }

    pub fn verify(address: &str) -> Result<(), SensorError> {
        match &address[0..2] {
            "28" => {}
            _ => return Err(SensorError::InvalidAddressStart(String::from(address))),
        }
        match address.len() {
            15 => {}
            _ => return Err(SensorError::InvalidAddressLength(address.len())),
        }
        Ok(())
    }

    pub fn dummy() -> Self {
        Ds18b20Address(String::from("28-dummy02230ff"))
    }
}

impl fmt::Display for Ds18b20Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_temp_measurement(raw_read: &str) -> Result<f32, SensorError> {
    let value: f32 = match raw_read.trim().parse() {
        Ok(value) => value,
        Err(err) => {
            return Err(SensorError::Parse(format!(
                "Could not parse string '{}' to f32. Err: {}",
                String::from(raw_read),
                err.to_string()
            )));
        }
    };
    Ok(value / 1000.0)
}

pub fn list_available() -> Result<Vec<Ds18b20Address>, SensorError> {
    println!("Finding sensors");
    let device_path = Path::new(DS18B20_DIR);
    if !device_path.exists() {
        // TODO: Better error
        return Err(SensorError::FileRead(format!(
            "DSB path does not exist: '{}'",
            DS18B20_DIR
        )));
    } else {
    }
    let files = match fs::read_dir(device_path) {
        Ok(files) => files,
        Err(_error) => {
            return Err(SensorError::FileRead(format!(
                "Unable to list DSB files {}.",
                DS18B20_DIR
            )))
        }
    };
    Ok(files
        .filter_map(Result::ok)
        .flat_map(ds18b20_address_from_file)
        .collect())
}

fn ds18b20_address_from_file(file: fs::DirEntry) -> Option<Ds18b20Address> {
    let tmp = file.path();
    let file_name = tmp.file_name()?.to_str()?;
    match Ds18b20Address::try_new(file_name) {
        Ok(address) => Some(address),
        Err(_) => None,
    }
}

/// Directory where the filesystem API registers DS18B20 sensors.
/// This is true for RbPi and probably for most linux systems.
const DS18B20_DIR: &str = "/sys/bus/w1/devices/";
// const DS18B20_DIR: &str = "./dummy_ds18";

#[derive(Debug)]
pub struct Ds18b20 {
    pub id: String,
    address: Ds18b20Address,
}

// TODO: Can this be done with a const fn?
// Or does that require the regex crate to provide const constructor?
// I tried it: it would work if Regex::new() was const fn.
// const TEMP_PATTERN: regex::Regex = regex::Regex::new(r"t=(-?[0-9]{4,6})").unwrap();
// Instead, resort to lazy_static.
//
// The issue is addressed here: https://github.com/rust-lang/regex/issues/607
lazy_static! {
    static ref TEMP_PATTERN: regex::Regex = regex::Regex::new(
        r"t=(-?[0-9]{4,6})"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use std::matches;

    #[test]
    fn test_address_correct() {
        let string = String::from("28-0416802230ff");
        let address = Ds18b20Address::verify(&string);
        address.unwrap();
    }

    #[test]
    fn test_address_wrong_start() {
        let string = String::from("29-0416802230ff");
        let address = Ds18b20Address::verify(&string);
        assert!(matches!(address, Err(SensorError::InvalidAddressStart(..))));
    }

    #[test]
    fn test_address_too_short() {
        let string = String::from("28-4E1F69140");
        let address = Ds18b20Address::verify(&string);
        assert!(matches!(
            address,
            Err(SensorError::InvalidAddressLength(..))
        ));
    }

    #[test]
    fn test_parse_temp_measurement_single_digit() {
        let temp_string = String::from("8720");
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 8.720);
    }

    #[test]
    fn test_parse_temp_measurement_two_digit() {
        let temp_string = String::from("28625");
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 28.625);
    }

    #[test]
    fn test_parse_temp_measurement_three_digit() {
        let temp_string = String::from("101625");
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 101.625);
    }

    #[test]
    fn test_parse_temp_measurement_negative() {
        let temp_string = String::from("-1724");
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), -1.724);
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        let temp_string = String::from("nonsense");
        assert!(parse_temp_measurement(&temp_string).is_err(),);
    }
}