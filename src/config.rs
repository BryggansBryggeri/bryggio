use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: Option<General>,
    pub control: Option<Control>,
    pub sensors: Option<Sensor>,
}

#[derive(Deserialize, Debug)]
pub struct General {
    pub brewery: String,
}

#[derive(Deserialize, Debug)]
pub struct Control {
    pub offset_on: f32,
    pub offset_off: f32,
}

#[derive(Deserialize, Debug, Clone)]
// TODO: Implement Deserialize for OneWireAddress
pub struct Sensor {
    pub id: String,
    pub address: String,
    pub offset: Option<f32>,
}

impl Config {
    pub fn new(config_file: &'static str) -> Config {
        let mut f = File::open(config_file).expect("Unable to open file");
        let mut toml_config = String::new();
        f.read_to_string(&mut toml_config)
            .expect("Unable to read string");
        let config: Config = Config::parse_toml(&toml_config);
        config
    }

    fn parse_toml(toml_string: &str) -> Config {
        let config: Config = toml::de::from_str(toml_string).unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let config: Config = Config::parse_toml(
            r#"
            [general]
            brewery = "BRYGGANS BRYGGERI BÄRS BB"
            [control]
            offset_on = 1.0
            offset_off = 0.0
            [sensors]
            id = "Mash tun"
            address = "random address"
        "#,
        );
        assert_eq!(config.control.unwrap().offset_on, 1.0);
    }
}
