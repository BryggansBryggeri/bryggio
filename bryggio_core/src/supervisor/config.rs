use crate::actor::{ActorConfig, ActorType};
use crate::logger::LogLevel;
use crate::pub_sub::nats_client::{NatsServerConfig, WebSocket};
use crate::pub_sub::ClientId;
use crate::sensor::{SensorConfig, SensorType};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

// TODO: Fix Parse struct.
/// Main supervisor configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: NatsConfig,
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
        let conf_presumptive: ParseSupervisorConfig = serde_json::from_str(&config_string)
            .map_err(|err| SupervisorConfigError::Parse(err.to_string()))?;
        let conf_presumptive = SupervisorConfig::from(conf_presumptive);
        SupervisorConfig::validate(conf_presumptive)
    }

    pub fn dummy() -> SupervisorConfig {
        SupervisorConfig {
            general: General::default(),
            nats: NatsConfig::dummy(),
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

impl From<ParseSupervisorConfig> for SupervisorConfig {
    fn from(parse: ParseSupervisorConfig) -> Self {
        Self {
            general: parse.general.clone(),
            hardware: parse.hardware.clone(),
            nats: NatsConfig::from_parsed(parse),
        }
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
                id: ClientId("dummy_sensor".into()),
                type_: SensorType::Dummy(1000),
            }],
            actors: vec![ActorConfig {
                id: ClientId("dummy_actor".into()),
                type_: ActorType::SimpleGpio {
                    pin_number: 0,
                    time_out: None,
                },
            }],
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NatsConfig {
    pub bin_path: PathBuf,
    pub server: NatsServerConfig,
}

impl NatsConfig {
    pub fn dummy() -> Self {
        NatsConfig {
            bin_path: PathBuf::from("target/nats-server"),
            server: NatsServerConfig::dummy(),
        }
    }
    fn from_parsed(parse: ParseSupervisorConfig) -> Self {
        Self {
            bin_path: parse.nats.bin_path.clone(),
            server: ParseNatsServerConfig::from_parsed(
                parse.nats,
                parse.general.log_level <= LogLevel::Debug,
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ParseSupervisorConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: ParseNatsServerConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ParseNatsServerConfig {
    /// Arbitrary server identifier
    pub server_name: String,
    /// NATS host
    pub host: String,
    /// NATS port
    pub port: u32,
    /// Username
    pub user: String,
    /// Password
    pub pass: String,
    /// Port for NATS web server monitor
    pub http_port: u32,
    /// Web socket config
    pub websocket: WebSocket,
    /// Path to NATS binary
    pub bin_path: PathBuf,
}

use crate::pub_sub::nats_client::Authorization;
impl ParseNatsServerConfig {
    fn from_parsed(parse: ParseNatsServerConfig, debug: bool) -> NatsServerConfig {
        NatsServerConfig::new(
            parse.server_name,
            parse.host,
            parse.port,
            parse.http_port,
            debug,
            Authorization::new(parse.user, parse.pass),
            parse.websocket,
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParseNatsClientConfig {
    pub host: String,
    pub port: u32,
    pub user: String,
    pub pass: String,
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

    const PARSE_STRING: &str = r#"
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
                    "http_port": 8888,
                    "user": "username",
                    "pass": "passwd",
                    "websocket": {
                      "port": 9222,
                      "no_tls": true
                    }
                  }
                }
            "#;

    #[test]
    fn test_parse() {
        let _config: ParseSupervisorConfig = serde_json::from_str(PARSE_STRING).unwrap();
    }

    #[test]
    fn test_nats_debug() {
        let config: ParseSupervisorConfig = serde_json::from_str(PARSE_STRING).unwrap();
        let config = SupervisorConfig::from(config);
        assert!(!config.nats.server.debug);
    }
}
