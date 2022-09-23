use crate::pub_sub::SensorBoxSubMsg;
use bryggio_core::actor::{ActorClient, ActorConfig, ActorError};
use bryggio_core::pub_sub::nats_client::Authorization;
use bryggio_core::pub_sub::{
    nats_client::{NatsClient, NatsClientConfig},
    ClientId, ClientState, PubSubClient, PubSubError,
};
use bryggio_core::sensor::{SensorClient, SensorConfig, SensorError};
use bryggio_core::{
    logger::{error, LogLevel},
    supervisor::config::Hardware,
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
        let nats_config = NatsClientConfig::from(config.nats.clone());
        println!("{:?}", nats_config);
        let client = NatsClient::try_new(&nats_config)?;
        let mut sensor_box = SensorBox {
            client,
            active_clients: ActiveClients::new(),
        };

        for sensor_config in config.hardware.sensors {
            sensor_box.add_sensor(sensor_config, &nats_config)?;
        }
        for actor_config in config.hardware.actors {
            sensor_box.add_actor(actor_config, &nats_config)?;
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
        config: &NatsClientConfig,
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

    fn add_actor(
        &mut self,
        actor_config: ActorConfig,
        config: &NatsClientConfig,
    ) -> Result<(), SensorBoxError> {
        let id = &actor_config.id;
        match self.active_clients.actors.get(id) {
            Some(_) => Err(SensorBoxError::AlreadyActive(id.clone())),
            None => {
                let actor = ActorClient::new(id.clone(), actor_config.get_actor()?, config);
                let handle = thread::spawn(|| actor.client_loop().map_err(|err| err.into()));
                self.active_clients
                    .actors
                    .insert(id.clone(), (handle, actor_config));
                Ok(())
            }
        }
    }

    fn handle_err(&self, err: SensorBoxError) -> ClientState {
        error(self, err.to_string(), "sensor_box");
        ClientState::Active
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SensorBoxConfig {
    pub general: General,
    pub hardware: Hardware,
    pub nats: NatsClientConfig,
}

#[derive(Deserialize, Debug, Clone)]
struct ParseSensorBoxConfig {
    general: General,
    hardware: Hardware,
    nats: ParseNatsClientConfig,
}

impl From<ParseSensorBoxConfig> for SensorBoxConfig {
    fn from(parse: ParseSensorBoxConfig) -> Self {
        Self {
            general: parse.general,
            hardware: parse.hardware,
            nats: NatsClientConfig::from(parse.nats),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParseNatsClientConfig {
    host: String,
    port: u32,
    user: String,
    pass: String,
}

impl From<ParseNatsClientConfig> for NatsClientConfig {
    fn from(parse: ParseNatsClientConfig) -> Self {
        NatsClientConfig::new(
            parse.host,
            parse.port,
            Authorization::new(parse.user, parse.pass),
        )
    }
}

impl SensorBoxConfig {
    pub fn try_new(config_file: &Path) -> Result<SensorBoxConfig, SensorBoxError> {
        let mut f = match fs::File::open(config_file) {
            Ok(f) => f,
            Err(err) => return Err(SensorBoxError::Io(err)),
        };
        let mut config_string = String::new();
        f.read_to_string(&mut config_string)?;
        let conf_presumptive: ParseSensorBoxConfig = serde_json::from_str(&config_string)
            // TODO: Io --> more relevant error
            .map_err(|err| SensorBoxError::Config(err.to_string()))?;
        let conf_presumptive = SensorBoxConfig::from(conf_presumptive);
        SensorBoxConfig::validate(conf_presumptive)
    }

    pub fn dummy() -> SensorBoxConfig {
        SensorBoxConfig {
            general: General::default(),
            hardware: Hardware::dummy(),
            nats: NatsClientConfig::dummy(),
        }
    }

    fn validate(pres: SensorBoxConfig) -> Result<SensorBoxConfig, SensorBoxError> {
        if pres.hardware.validate() {
            Ok(pres)
        } else {
            Err(SensorBoxError::Config(String::from(
                "Non-unique client IDs",
            )))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General {
    pub client_name: String,
    // TODO: Rename all
    pub log_level: LogLevel,
}

impl Default for General {
    fn default() -> Self {
        General {
            client_name: "No name".into(),
            log_level: LogLevel::Info,
        }
    }
}

#[derive(Debug)]
struct ActiveClients {
    sensors: HashMap<ClientId, (Handle, SensorConfig)>,
    actors: HashMap<ClientId, (Handle, ActorConfig)>,
}

impl ActiveClients {
    fn new() -> Self {
        ActiveClients {
            sensors: HashMap::new(),
            actors: HashMap::new(),
        }
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
    #[error("Io: '{0}'")]
    Io(#[from] std::io::Error),
    #[error("'{0}' is not an active client")]
    Missing(ClientId),
    #[error("'{0}' is already an active client")]
    AlreadyActive(ClientId),
    #[error("Actor error")]
    Actor(#[from] ActorError),
    #[error("Sensor error: {0}")]
    Sensor(#[from] SensorError),
    #[error("Pub-sub error: {0}")]
    PubSub(#[from] PubSubError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Could not join thread with client id {0}")]
    ThreadJoin(ClientId),
}

#[cfg(test)]
mod sensor_box_config_tests {
    use super::*;

    #[test]
    fn parse() {
        let _config: ParseSensorBoxConfig = serde_json::from_str(
            r#"
            {
              "general": {
              "client_name": "Sensor box",
              "log_level": "info"
              },
              "hardware": {
                "actors": [
                  {
                    "id": "mash",
                    "type": {"simple_gpio": {"pin_number": 0}}
                  },
                  {
                    "id": "boil",
                    "type": {"simple_gpio": {"pin_number": 1}}
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
                "user": "username",
                "pass": "passwd",
                "host": "localhost",
                "port": 4222
              }
            }"#,
        )
        .unwrap();
    }
}
