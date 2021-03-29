use crate::pub_sub::{SensorBoxPubMsg, SensorBoxSubMsg};
use bryggio_lib::pub_sub::PubSubMsg;
use bryggio_lib::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError,
};
use bryggio_lib::sensor::ds18b20::Ds18b20Address;
use bryggio_lib::sensor::{SensorClient, SensorConfig, SensorError, SensorType};
use bryggio_lib::{
    logger::{debug, error, info},
    supervisor::config::{General, Hardware},
};
use nats::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::thread;
use thiserror::Error;
mod pub_sub;

pub struct SensorBox {
    client: NatsClient,
    active_clients: ActiveClients,
}

impl SensorBox {
    pub fn init_from_config(config: SensorBoxConfig) -> Result<SensorBox, SensorBoxError> {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut sensor_box = SensorBox {
            client,
            active_clients: ActiveClients::new(),
        };

        for sensor_config in config.hardware.sensors {
            sensor_box.add_sensor(sensor_config, &config.nats)?;
        }
        Ok(sensor_box)
    }

    fn process_command(
        &mut self,
        cmd: SensorBoxSubMsg,
        full_msg: &Message,
    ) -> Result<ClientState, SensorBoxError> {
        match cmd {
            SensorBoxSubMsg::Stop => Ok(ClientState::Active),
            _ => Ok(ClientState::Active),
        }
    }

    fn add_sensor(
        &mut self,
        sensor_config: SensorConfig,
        config: &NatsConfig,
    ) -> Result<(), SensorBoxError> {
        let id = &sensor_config.id;
        match self.active_clients.sensors.get(id) {
            Some(_) => Err(SensorBoxError::AlreadyActive(id.clone())),
            None => {
                let sensor = SensorClient::new(
                    sensor_config.id.clone(),
                    sensor_config.get_sensor()?,
                    config,
                );
                let handle = thread::spawn(|| sensor.client_loop().map_err(|err| err.into()));
                self.active_clients
                    .sensors
                    .insert(id.clone(), (handle, sensor_config));
                Ok(())
            }
        }
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), SensorBoxError> {
        if self.active_clients.contatins_id(id) {
            Ok(())
        } else {
            Err(SensorBoxError::Missing(id.clone()))
        }
    }

    fn handle_err(&self, err: SensorBoxError) -> ClientState {
        error(self, err.to_string(), "supervisor");
        ClientState::Active
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorBoxConfig {
    pub hardware: Hardware,
    pub nats: NatsConfig,
}

impl SensorBoxConfig {
    pub fn dummy() -> SensorBoxConfig {
        SensorBoxConfig {
            hardware: Hardware {
                sensors: vec![SensorConfig {
                    id: ClientId("dummy".into()),
                    type_: SensorType::Dsb(Ds18b20Address::dummy()),
                }],
                actors: Vec::new(),
            },
            nats: NatsConfig::dummy(),
        }
    }

    pub fn pprint(&self) -> String {
        //toml::ser::to_string_pretty(self).unwrap()
        serde_json::to_string_pretty(self).unwrap()
    }

    pub fn try_new(config_file: &Path) -> Result<SensorBoxConfig, SensorBoxError> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(SensorBoxError::Io(err)),
        };
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)?;
        let conf_presumptive = serde_json::from_str(&config_string)
            // TODO: Io --> more relevant error
            .map_err(|err| SensorBoxError::Config(err.to_string()))?;
        SensorBoxConfig::validate(conf_presumptive)
    }

    fn validate(pres: SensorBoxConfig) -> Result<SensorBoxConfig, SensorBoxError> {
        Ok(pres)
    }
}

#[derive(Debug)]
struct ActiveClients {
    sensors: HashMap<ClientId, (Handle, SensorConfig)>,
}

impl ActiveClients {
    fn new() -> Self {
        ActiveClients {
            sensors: HashMap::new(),
        }
    }

    fn contatins_id(&self, id: &ClientId) -> bool {
        self.sensors.contains_key(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActiveClientsList {
    sensors: HashMap<ClientId, SensorConfig>,
}

impl From<&ActiveClients> for ActiveClientsList {
    fn from(clients: &ActiveClients) -> Self {
        ActiveClientsList {
            sensors: clients
                .sensors
                .iter()
                .map(|(id, (_, config))| (id.clone(), config.clone()))
                .collect(),
        }
    }
}

type Handle = thread::JoinHandle<Result<(), SensorBoxError>>;

#[derive(Error, Debug)]
pub enum SensorBoxError {
    #[error("This should be its own error: {0}")]
    Cli(String),
    #[error("'{0}' is not an active client")]
    Io(#[from] std::io::Error),
    #[error("'{0}' is not an active client")]
    Missing(ClientId),
    #[error("'{0}' is already an active client")]
    AlreadyActive(ClientId),
    #[error("Sensor error")]
    Sensor(#[from] SensorError),
    #[error("Pubsub error: {0}")]
    PubSub(#[from] PubSubError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    #[error("Could not join thread with client id {0}")]
    ThreadJoin(ClientId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let _config: SensorBoxConfig = serde_json::from_str(
            r#"
            {
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
                    "type": {"dummy": 0}
                  },
                  {
                    "id": "boil",
                    "type": {"dsb": "28-dummy0000000"}
                  }
                ]
              },
              "nats": {
                "server": "localhost",
                "user": "ababa",
                "pass": "babab"
              }
            }"#,
        )
        .unwrap();
    }
}
