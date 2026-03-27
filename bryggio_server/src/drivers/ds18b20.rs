//! DS18B20 temperature sensor driver.
//!
//! Reads measurements via the 1-wire sysfs interface provided by the OS.
use bryggio_core::sensor::SensorError;
use bryggio_core::types::Temperature;
use std::fs;
use std::path::{Path, PathBuf};

/// Directory where the filesystem API registers DS18B20 sensors.
const DS18B20_DIR: &str = "/sys/bus/w1/devices/";

/// DS18B20 temperature sensor handle.
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
    pub fn get_measurement(&self) -> Result<Temperature, SensorError> {
        let raw_read = fs::read_to_string(self.device_path())
            .map_err(|err| SensorError::FileRead(err.to_string()))?;
        parse_temp_measurement(&raw_read)
    }
}

/// List available DS18B20 sensors on the 1-wire bus.
pub fn list_available() -> Result<Vec<Ds18b20Address>, SensorError> {
    let device_path = Path::new(DS18B20_DIR);
    if !device_path.exists() {
        return Err(SensorError::FileRead(format!(
            "DS18B20 path does not exist: '{}'",
            DS18B20_DIR
        )));
    }
    let files = fs::read_dir(device_path).map_err(|_| {
        SensorError::FileRead(format!("Unable to list DS18B20 files {}.", DS18B20_DIR))
    })?;
    Ok(files
        .filter_map(Result::ok)
        .flat_map(ds18b20_address_from_filename)
        .collect())
}

/// Temperature sensor 1-wire address.
///
/// 15 characters, beginning with "28".
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Ds18b20Address(String);

impl Ds18b20Address {
    pub fn try_new(address: &str) -> Result<Ds18b20Address, SensorError> {
        Self::verify(address)?;
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
}

impl std::fmt::Display for Ds18b20Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_temp_measurement(raw_read: &str) -> Result<Temperature, SensorError> {
    let value: f32 = raw_read.trim().parse().map_err(|err| {
        SensorError::Parse(format!(
            "Could not parse string '{}' to f32. Err: {}",
            raw_read, err
        ))
    })?;
    Ok(Temperature(value / 1000.0))
}

fn ds18b20_address_from_filename(file: fs::DirEntry) -> Option<Ds18b20Address> {
    let tmp = file.path();
    let file_name = tmp.file_name()?.to_str()?;
    Ds18b20Address::try_new(file_name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_address_correct() {
        let string = "28-0416802230ff";
        assert!(Ds18b20Address::verify(string).is_ok());
    }

    #[test]
    fn test_address_wrong_start() {
        let string = "29-0416802230ff";
        assert!(matches!(
            Ds18b20Address::verify(string),
            Err(SensorError::InvalidAddressStart(..))
        ));
    }

    #[test]
    fn test_address_too_short() {
        let string = "28-4E1F69140";
        assert!(matches!(
            Ds18b20Address::verify(string),
            Err(SensorError::InvalidAddressLength(..))
        ));
    }

    #[test]
    fn test_parse_temp_measurement_single_digit() {
        assert_approx_eq!(parse_temp_measurement("8720").unwrap().0, 8.720);
    }

    #[test]
    fn test_parse_temp_measurement_two_digit() {
        assert_approx_eq!(parse_temp_measurement("28625").unwrap().0, 28.625);
    }

    #[test]
    fn test_parse_temp_measurement_three_digit() {
        assert_approx_eq!(parse_temp_measurement("101625").unwrap().0, 101.625);
    }

    #[test]
    fn test_parse_temp_measurement_negative() {
        assert_approx_eq!(parse_temp_measurement("-1724").unwrap().0, -1.724);
    }

    #[test]
    fn test_parse_temp_measurement_no_match() {
        assert!(parse_temp_measurement("nonsense").is_err());
    }
}
