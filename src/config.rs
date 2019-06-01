use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: Option<General>,
    pub control: Option<Control>,
}

#[derive(Deserialize, Debug)]
pub struct General {
    pub brewery: String,
    pub sw_version: String,
}

#[derive(Deserialize, Debug)]
pub struct Control {
    pub offset_on: f32,
    pub offset_off: f32,
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
            sw_version = "1.6"
            [control]
            offset_on = 1.0
            offset_off = 0.0
        "#,
        );
        assert_eq!(config.general.unwrap().sw_version, "1.6");
        assert_eq!(config.control.unwrap().offset_on, 1.0);
    }
}
