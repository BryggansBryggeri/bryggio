use crate::actor::ActorConfig;
use crate::logger::LogLevel;
use crate::pub_sub::nats_client::{Authorization, NatsClientConfig, NatsServerConfig, WebSocket};
use crate::pub_sub::ClientId;
use crate::sensor::ds18b20::Ds18b20Address;
use crate::sensor::{SensorConfig, SensorType};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Main supervisor configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: ParseNatsConfig,
}

impl SupervisorConfig {
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

    pub fn dummy() -> SupervisorConfig {
        SupervisorConfig {
            general: General::default(),
            nats: ParseNatsConfig::dummy(),
            hardware: Hardware::dummy(),
        }
    }

    fn validate(pres: SupervisorConfig) -> Result<SupervisorConfig, SupervisorConfigError> {
        if !pres.hardware.validate() {
            return Err(SupervisorConfigError::Config(String::from(
                "Non-unique client IDs",
            )));
        }
        if !pres.nats.bin_path.as_path().exists() {
            return Err(SupervisorConfigError::Config(format!(
                "NATS server bin '{}' missing",
                pres.nats.bin_path.as_path().to_string_lossy()
            )));
        };
        Ok(pres)
    }
}

impl From<SupervisorConfig> for NatsServerConfig {
    fn from(config: SupervisorConfig) -> Self {
        let debug = config.general.log_level <= LogLevel::Debug;
        let nats = config.nats;
        Self::new(
            nats.server_name,
            nats.host,
            nats.port,
            nats.http_port,
            debug,
            Authorization::new(nats.user, nats.pass),
            nats.websocket,
        )
    }
}

impl From<SupervisorConfig> for NatsClientConfig {
    fn from(config: SupervisorConfig) -> Self {
        let nats = config.nats;
        Self::new(
            nats.host,
            nats.port,
            Authorization::new(nats.user, nats.pass),
        )
    }
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
    pub fn dummy() -> Self {
        Hardware {
            sensors: vec![SensorConfig {
                id: ClientId("dummy".into()),
                type_: SensorType::Dsb(Ds18b20Address::dummy()),
            }],
            actors: Vec::new(),
        }
    }

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParseNatsConfig {
    pub server_name: String,
    pub host: String,
    pub port: u32,
    pub user: String,
    pub pass: String,
    pub http_port: u32,
    pub websocket: WebSocket,
    pub bin_path: PathBuf,
}

impl ParseNatsConfig {
    pub fn dummy() -> Self {
        Self {
            bin_path: PathBuf::new(),
            server_name: String::from("server-name"),
            user: String::from("user"),
            pass: String::from("passwd"),
            host: String::from("localhost"),
            port: 8888,
            http_port: 8888,
            websocket: WebSocket::dummy(),
        }
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
mod supervisor_config_tests {
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
                        "type": {"simple_gpio": {"pin_number": 0}}
                      },
                      {
                        "id": "boil_heater",
                        "type": {"simple_gpio": {"pin_number": 1}}
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
                  "nats": {
                    "bin_path": "target/nats-server",
                    "server_name": "bryggio-nats-server",
                    "host": "localhost",
                    "port": 4222,
                    "user": "username",
                    "pass": "passwd",
                    "http_port": 8888,
                    "websocket": {
                      "port": 9222,
                      "no_tls": true
                    }
                  }
                }
            "#,
        )
        .unwrap();
    }
}
