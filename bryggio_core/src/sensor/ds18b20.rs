//! DS18B20 temperature sensor driver
//!
//! Reads measurements via the 1-wire sysfs interface provided by the OS.
use crate::sensor::SensorError;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// DS18B20 temperature sensor
#[derive(Debug, Clone)]
pub struct Ds18b20 {
    address: Ds18b20Address,
}

impl Ds18b20 {
    pub fn try_new(address: &str) -> Result<Ds18b20, SensorError> {
        let address = Ds18b20Address::try_new(address)?;
        Ok(Ds18b20 { address })
    }

    fn device_path(&self) -> PathBuf {
        Path::new(DS18B20_DIR)
            .join(&self.address.0)
            .join("temperature")
    }

    /// Take a temperature measurement. Returns degrees Celsius.
    pub fn get_measurement(&self) -> Result<f32, SensorError> {
        let raw_read = utils::read_file_to_string(self.device_path())
            .map_err(|err| SensorError::FileRead(err.to_string()))?;
        parse_temp_measurement(&raw_read)
    }
}

/// List available DS18B20 sensors on the 1-wire bus.
pub fn list_available() -> Result<Vec<Ds18b20Address>, SensorError> {
    let device_path = Path::new(DS18B20_DIR);
    if !device_path.exists() {
        return Err(SensorError::FileRead(format!(
            "DSB path does not exist: '{}'",
            DS18B20_DIR
        )));
    }
    let files = fs::read_dir(device_path).map_err(|_| {
        SensorError::FileRead(format!("Unable to list DSB files {}.", DS18B20_DIR))
    })?;
    Ok(files
        .filter_map(Result::ok)
        .flat_map(ds18b20_address_from_filename)
        .collect())
}

/// Temperature sensor address
///
/// The address is a string with 15 characters, beginning with "28".
#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Ds18b20Address(String);

impl Ds18b20Address {
    pub fn try_new(address: &str) -> Result<Ds18b20Address, SensorError> {
        Ds18b20Address::verify(address)?;
        Ok(Ds18b20Address(String::from(address)))
    }

    fn verify(address: &str) -> Result<(), SensorError> {
        if !address.starts_with("28") {
            return Err(SensorError::InvalidAddressStart(String::from(address)));
        }
        if address.len() != 15 {
            return Err(SensorError::InvalidAddressLength(address.len()));
        }
        Ok(())
    }

    pub fn dummy() -> Self {
        Ds18b20Address(String::from("28-dummy02230ff"))
    }
}

impl AsRef<str> for Ds18b20Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Ds18b20Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_temp_measurement(raw_read: &str) -> Result<f32, SensorError> {
    let value: f32 = raw_read.trim().parse().map_err(|err| {
        SensorError::Parse(format!(
            "Could not parse string '{}' to f32. Err: {}",
            raw_read, err
        ))
    })?;
    Ok(value / 1000.0)
}

fn ds18b20_address_from_filename(file: fs::DirEntry) -> Option<Ds18b20Address> {
    let tmp = file.path();
    let file_name = tmp.file_name()?.to_str()?;
    Ds18b20Address::try_new(file_name).ok()
}

/// Directory where the filesystem API registers DS18B20 sensors.
pub const DS18B20_DIR: &str = "/sys/bus/w1/devices/";

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
        assert!(parse_temp_measurement(&temp_string).is_err());
    }
}
