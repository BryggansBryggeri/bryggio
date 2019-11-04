use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
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
    pub fn new(config_file: &str) -> Result<Config, io::Error> {
        let mut f = fs::File::open(config_file)?;
        let mut toml_config = String::new();
        f.read_to_string(&mut toml_config)?;
        let config: Config = Config::parse_toml(&toml_config);
        Ok(config)
    }

    fn parse_toml(toml_string: &str) -> Config {
        let config: Config = toml::de::from_str(toml_string).unwrap();
        config
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
        );
        assert_approx_eq!(config.control.unwrap().offset_on, 1.0);
    }
}
