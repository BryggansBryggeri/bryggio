use crate::pub_sub::nats_client::NatsConfig;
use serde::{Deserialize, Serialize};
use std::error as std_error;
use std::fs;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub general: General,
    pub hardware: Hardware,
    pub nats: NatsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General {
    pub brewery_name: String,
}

impl Default for General {
    fn default() -> Self {
        General {
            brewery_name: "No name".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

    #[test]
    fn test_parse() {
        let _config: Config = Config::parse_toml(
            r#"
            [general]
            brewery_name = "BRYGGANS BRYGGERI BÄRS BB"
            [sensors]
            id = "Mash tun"
            address = "random address"
            [hardware]
            sensors = []
            actors = []
            [nats]
            bin_path="/some/path/to/bin"
            server="localhost"
            user="jackonelli"
            pass="very_secret"
        "#,
        )
        .unwrap();
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
