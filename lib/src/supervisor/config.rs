use crate::actor::ActorConfig;
use crate::logger::LogLevel;
use crate::pub_sub::nats_client::NatsConfig;
use crate::pub_sub::ClientId;
use crate::sensor::ds18b20::Ds18b20Address;
use crate::sensor::{SensorConfig, SensorType};
use serde::{Deserialize, Serialize};
use std::error as std_error;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: NatsConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General {
    pub brewery_name: String,
    // TODO: Rename all
    pub log_level: LogLevel,
}

impl Default for General {
    fn default() -> Self {
        General {
            brewery_name: "No name".into(),
            log_level: LogLevel::Info,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hardware {
    pub actors: Vec<ActorConfig>,
    pub sensors: Vec<SensorConfig>,
}

impl SupervisorConfig {
    pub fn dummy() -> SupervisorConfig {
        SupervisorConfig {
            general: General::default(),
            nats: NatsConfig::dummy(),
            hardware: Hardware {
                sensors: vec![SensorConfig {
                    id: ClientId("dummy".into()),
                    type_: SensorType::Dsb(Ds18b20Address::dummy()),
                }],
                actors: Vec::new(),
            },
        }
    }

    pub fn pprint(&self) -> String {
        //toml::ser::to_string_pretty(self).unwrap()
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn try_new(config_file: &Path) -> Result<SupervisorConfig, Error> {
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
        SupervisorConfig::validate(conf_presumptive)
    }

    fn validate(pres: SupervisorConfig) -> Result<SupervisorConfig, Error> {
        if !pres.nats.bin_path.as_path().exists() {
            return Err(Error::Config(format!(
                "NATS server bin '{}' missing",
                pres.nats.bin_path.as_path().to_string_lossy()
            )));
        };
        if !pres.nats.config.as_path().exists() {
            return Err(Error::Config(format!(
                "NATS config '{}' missing",
                pres.nats.config.as_path().to_string_lossy()
            )));
        };
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
        let _config: SupervisorConfig = serde_json::from_str(
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
