use bryggio_lib::pub_sub::ClientId;
use bryggio_lib::sensor::ds18b20::Ds18b20Address;
use bryggio_lib::sensor::{SensorConfig, SensorType};
use serde::{Deserialize, Serialize};
use std::error as std_error;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorBoxConfig {
    pub sensors: Vec<SensorConfig>,
}

impl SensorBoxConfig {
    pub fn dummy() -> SensorBoxConfig {
        SensorBoxConfig {
            sensors: vec![SensorConfig {
                id: ClientId("dummy".into()),
                type_: SensorType::Dsb(Ds18b20Address::dummy()),
            }],
        }
    }

    pub fn pprint(&self) -> String {
        //toml::ser::to_string_pretty(self).unwrap()
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn try_new(config_file: &Path) -> Result<SensorBoxConfig, Error> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(Error::IO(format!("Error opening file, {}", err))),
        };
        let mut config_string = String::new();
        match f.read_to_string(&mut config_string) {
            Ok(_) => {}
            Err(err) => return Err(Error::IO(format!("Error reading file to string, {}", err))),
        };
        let conf_presumptive =
            serde_json::from_str(&config_string).map_err(|err| Error::Parse(err.to_string()))?;
        SensorBoxConfig::validate(conf_presumptive)
    }

    fn validate(pres: SensorBoxConfig) -> Result<SensorBoxConfig, Error> {
        Ok(pres)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    IO(String),
    Parse(String),
    Config(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IO(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
            Error::Config(err) => write!(f, "Config error: {}", err),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IO(_) => "IO error",
            Error::Parse(_) => "Parse error",
            Error::Config(_) => "Config error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let _config: SensorBoxConfig = serde_json::from_str(
            r#"
            {
              "general": {
                "brewery_name": "BRYGGANS BRYGGERI BÄRS BB",
                "log_level": "debug"
              },
              "hardware": {
                "actors": [
                  {
                    "id": "mash",
                    "type": {"simple_gpio": 0}
                  },
                  {
                    "id": "boil",
                    "type": {"simple_gpio": 1}
                  }
                ]
              ,
                "sensors": [
                  {
                    "id": "mash",
                    "type": "dummy"
                  },
                  {
                    "id": "boil",
                    "type": {"dsb": "28-dummy0000000"}
                  }
                ]
              },
              "nats": {
                "bin_path": "target/nats-server",
                "config": "./nats-config.yaml",
                "server": "localhost",
                "user": "ababa",
                "pass": "babab"
              }
            }"#,
        )
        .unwrap();
    }
}
