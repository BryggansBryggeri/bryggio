use crate::actor::ActorConfig;
use crate::logger::LogLevel;
use crate::pub_sub::nats_client::NatsConfig;
use crate::pub_sub::ClientId;
use crate::sensor::ds18b20::Ds18b20Address;
use crate::sensor::{SensorConfig, SensorType};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: NatsConfig,
    pub nats_bin_path: PathBuf,
    pub nats_config: PathBuf,
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

impl Hardware {
    pub fn validate(&self) -> bool {
        let unique_count = self
            .sensors
            .iter()
            .map(|x| &x.id)
            .chain(self.actors.iter().map(|x| &x.id))
            .unique()
            .count();
        unique_count == self.sensors.len() + self.actors.len()
    }
}
impl SupervisorConfig {
    pub fn dummy() -> SupervisorConfig {
        SupervisorConfig {
            general: General::default(),
            nats: NatsConfig::dummy(),
            nats_bin_path: PathBuf::new(),
            nats_config: PathBuf::new(),
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

    pub fn try_new(config_file: &Path) -> Result<SupervisorConfig, SupervisorConfigError> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(SupervisorConfigError::Io(err)),
        };
        let mut config_string = String::new();
        match f.read_to_string(&mut config_string) {
            Ok(_) => {}
            Err(err) => return Err(SupervisorConfigError::Io(err)),
        };
        let conf_presumptive = serde_json::from_str(&config_string)
            .map_err(|err| SupervisorConfigError::Parse(err.to_string()))?;
        SupervisorConfig::validate(conf_presumptive)
    }

    fn validate(pres: SupervisorConfig) -> Result<SupervisorConfig, SupervisorConfigError> {
        if !pres.hardware.validate() {
            return Err(SupervisorConfigError::Config(String::from(
                "Non-unique client IDs",
            )));
        }
        if !pres.nats_bin_path.as_path().exists() {
            return Err(SupervisorConfigError::Config(format!(
                "NATS server bin '{}' missing",
                pres.nats_bin_path.as_path().to_string_lossy()
            )));
        };
        if !pres.nats_config.as_path().exists() {
            return Err(SupervisorConfigError::Config(format!(
                "NATS config '{}' missing",
                pres.nats_config.as_path().to_string_lossy()
            )));
        };
        Ok(pres)
    }
}

#[derive(Error, Debug)]
pub enum SupervisorConfigError {
    #[error("Error opening config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Parse error: {0}")]
    Parse(String),
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
                    "log_level": "info"
                  },
                  "hardware": {
                    "actors": [
                      {
                        "id": "mash_heater",
                        "type": {"simple_gpio": 0}
                      },
                      {
                        "id": "boil_heater",
                        "type": {"simple_gpio": 1}
                      }
                    ]
                  ,
                    "sensors": [
                      {
                        "id": "mash",
                        "type": {"dummy": 1000}
                      },
                      {
                        "id": "boil",
                        "type": {"dsb": "28-dummy0000000"}
                      }
                    ]
                  },
                  "nats_bin_path": "target/nats-server",
                  "nats_config": "./nats-config.yaml",
                  "nats": {
                    "server": "localhost",
                    "user": "username",
                    "pass": "passwd"
                  }
                }
            "#,
        )
        .unwrap();
    }
}
