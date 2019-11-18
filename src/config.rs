use serde::{Deserialize, Serialize};
use std::error as std_error;
use std::fs;
use std::io::Read;
use toml;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub general: General,
    pub control: Option<Control>,
    pub hardware: Hardware,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General {
    pub brewery_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Control {
    pub offset_on: f32,
    pub offset_off: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hardware {
    pub sensors: Vec<Sensor>,
    pub actors: Vec<Actor>,
}

// TODO: Implement Deserialize for OneWireAddress
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sensor {
    pub id: String,
    pub address: String,
    pub offset: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Actor {
    pub id: String,
    pub gpio_pin: u32,
}

impl Config {
    pub fn new(config_file: &str) -> Result<Config, Error> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(Error::IO(format!("Error opening file, {}", err))),
        };
        let mut toml_config = String::new();
        match f.read_to_string(&mut toml_config) {
            Ok(_) => {}
            Err(err) => return Err(Error::IO(format!("Error reading file to string, {}", err))),
        };
        Config::parse_toml(&toml_config)
    }

    fn parse_toml(toml_string: &str) -> Result<Config, Error> {
        match toml::de::from_str::<Config>(toml_string) {
            Ok(config) => Ok(config),
            Err(err) => {
                return Err(Error::Parse(format!(
                    "could not parse config file, {}",
                    err
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_parse() {
        let config: Config = Config::parse_toml(
            r#"
            [general]
            brewery_name = "BRYGGANS BRYGGERI BÄRS BB"
            [control]
            offset_on = 1.0
            offset_off = 0.0
            [sensors]
            id = "Mash tun"
            address = "random address"
            [hardware]
            sensors = []
            actors = []
        "#,
        )
        .unwrap();
        assert_approx_eq!(config.control.unwrap().offset_on, 1.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    IO(String),
    Parse(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IO(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(_) => "Config error",
            Error::Parse(_) => "Parse error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
