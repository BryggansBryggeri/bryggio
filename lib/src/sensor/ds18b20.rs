use crate::sensor::{Sensor, SensorError};
use crate::utils;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path;
use std::thread::sleep;
use std::time::Duration;

const DS18_DIR: &str = "/sys/bus/w1/devices/";

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
lazy_static! {
    static ref TEMP_PATTERN: regex::Regex = regex::Regex::new(
        r"t=(-?[0-9]{4,6})"
    )
    .unwrap(); // This unwrap is fine since it is a constant valid regex.
}

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
        let device_path = format!("/sys/bus/w1/devices/{}/w1_slave", self.address.0);
        let meas = match utils::read_file_to_string(&device_path) {
            Ok(raw_read) => parse_temp_measurement(&raw_read),
            Err(err) => Err(SensorError::Read(err.to_string())),
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
    let mat = match TEMP_PATTERN.captures(raw_read) {
        Some(mat) => mat,
        None => {
            return Err(SensorError::Parse(format!(
                "No match in string '{}'",
                String::from(raw_read)
            )));
        }
    };
    let value_raw = match mat.get(1) {
        Some(mat) => mat.as_str(),
        None => {
            // Can this even happen? If match on whole regex above, then this must exist no?
            return Err(SensorError::Parse(format!(
                "No valid temp match in string '{}'",
                String::from(raw_read)
            )));
        }
    };
    let value: f32 = match value_raw.parse() {
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
    let device_path = path::Path::new(DS18_DIR);
    if !device_path.exists() {
        // TODO: Better error
        return Err(SensorError::Read(format!(
            "DSB path does not exist: '{}'",
            DS18_DIR
        )));
    } else {
    }
    let files = match fs::read_dir(device_path) {
        Ok(files) => files,
        Err(_error) => {
            return Err(SensorError::Read(format!(
                "Unable to list DSB files {}.",
                DS18_DIR
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
    match Ds18b20Address::try_new(&file_name) {
        Ok(address) => Some(address),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_address_correct() {
        let string = String::from("28-0416802230ff");
        let address = Ds18b20Address::verify(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_wrong_start() {
        let string = String::from("29-0416802230ff");
        let address = Ds18b20Address::verify(&string);
        address.unwrap();
    }

    #[test]
    #[should_panic]
    fn test_address_too_short() {
        let string = String::from("284E1F69140");
        let address = Ds18b20Address::verify(&string);
        address.unwrap();
    }

    #[test]
    fn test_parse_temp_measurement_single_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=8720",
        );
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 8.720);
    }

    #[test]
    fn test_parse_temp_measurement_two_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=28625",
        );
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 28.625);
    }

    #[test]
    fn test_parse_temp_measurement_three_digit() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=101625",
        );
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), 101.625);
    }

    #[test]
    fn test_parse_temp_measurement_negative() {
        let temp_string = String::from(
            "ca 01 4b 46 7f ff 06 10 65 : crc=65 YES\nca 01 4b 46 7f ff 06 10 65 t=-1724",
        );
        assert_approx_eq!(parse_temp_measurement(&temp_string).unwrap(), -1.724);
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        let temp_string = String::from("nonsense");
        assert!(parse_temp_measurement(&temp_string).is_err(),);
    }
}
