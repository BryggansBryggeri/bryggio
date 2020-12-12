use crate::actor;
use crate::config;
use crate::control::pub_sub::ControllerClient;
use crate::control::ControllerConfig;
use crate::logger::Log;
use crate::pub_sub::PubSubMsg;
use crate::pub_sub::{
    nats_client::{decode_nats_data, NatsClient, NatsConfig},
    ClientId, ClientState, PubSubClient, PubSubError, Subject,
};
use crate::sensor;
use nats::{Message, Subscription};
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error as std_error;
use std::thread;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

#[derive(Deserialize)]
pub enum SupervisorSubMsg {
    #[serde(rename = "start_controller")]
    StartController { control_config: ControllerConfig },
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "toggle_controller")]
    ToggleController(ClientId),
    #[serde(rename = "kill_client")]
    KillClient { client_id: ClientId },
    #[serde(rename = "stop")]
    Stop,
}

impl TryFrom<Message> for SupervisorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, PubSubError> {
        match msg.subject.as_ref() {
            "command.start_controller" => {
                let control_config: ControllerConfig = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::StartController { control_config })
            }
            "command.toggle_controller" => {
                let id: ClientId = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::ToggleController(id))
            }
            "command.list_active_clients" => Ok(SupervisorSubMsg::ListActiveClients),
            _ => Err(PubSubError::MessageParse(String::new())),
        }
    }
}

pub struct Supervisor {
    client: NatsClient,
    config: config::Config,
    active_clients: HashMap<ClientId, thread::JoinHandle<Result<(), PubSubError>>>,
}

impl PubSubClient for Supervisor {
    fn client_loop(mut self) -> Result<(), PubSubError> {
        let subject = Subject("command.>".into());
        let sub = self.subscribe(&subject)?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(msg) = sub.next() {
                state = match SupervisorSubMsg::try_from(msg) {
                    Ok(cmd) => match self.process_command(&cmd) {
                        Ok(state) => state,
                        Err(err) => Supervisor::handle_err(err),
                    },
                    Err(err) => Supervisor::handle_err(err),
                };
            }
        }
        Ok(())
    }

    fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject)
    }

    fn publish(&self, _subject: &Subject, _msg: &PubSubMsg) -> Result<(), PubSubError> {
        todo!()
    }
}

impl Supervisor {
    fn handle_err(err: PubSubError) -> ClientState {
        println!("{}", err.to_string());
        ClientState::Active
    }

    fn list_active_clients(&self) {
        println!("Active clients:");
        for cl in self.active_clients.keys() {
            println!("{}", cl);
        }
    }
    fn process_command(&mut self, cmd: &SupervisorSubMsg) -> Result<ClientState, PubSubError> {
        match cmd {
            SupervisorSubMsg::StartController { control_config } => {
                self.start_controller(&control_config)?;
                println!("starting controller");
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ToggleController(contr_id) => {
                self.toggle_controller(contr_id)?;
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ListActiveClients => {
                self.list_active_clients();
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::KillClient { client_id: _ } => Ok(ClientState::Active),
            SupervisorSubMsg::Stop => Ok(ClientState::Active),
        }
    }

    fn toggle_controller(&mut self, contr_id: &ClientId) -> Result<(), PubSubError> {
        println!("togg contr method");
        let controller = match self.active_clients.get(contr_id) {
            Some(contr) => Ok(contr),
            None => Err(PubSubError::Client(format!(
                "'{}' not an active client",
                contr_id
            ))),
        }?;
        Ok(())
    }

    fn start_controller(&mut self, config: &ControllerConfig) -> Result<(), PubSubError> {
        config
            .client_ids()
            .map(|id| self.client_is_active(id))
            .collect::<Result<Vec<_>, PubSubError>>()?;

        let controller = config.get_controller().map_err(|err| {
            PubSubError::Client(format!("Could not start control: {}", err.to_string()))
        })?;
        let controller_client = ControllerClient::new(
            config.controller_id.clone(),
            config.actor_id.clone(),
            config.sensor_id.clone(),
            controller,
            &self.config.nats,
        );
        let control_handle = thread::spawn(|| controller_client.client_loop());
        match self.add_client(config.controller_id.clone(), control_handle) {
            Ok(_) => {}
            Err(err) => {
                Supervisor::handle_err(err);
            }
        };
        Ok(())
    }

    fn add_logger(&mut self, config: &config::Config) {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop());
        match self.add_client(ClientId("log".into()), log_handle) {
            Ok(_) => {}
            Err(err) => {
                Supervisor::handle_err(err);
            }
        };
    }

    fn add_sensor(&mut self, sensor_id: ClientId, config: &NatsConfig) {
        let sensor = sensor::SensorClient::new(
            sensor_id.clone(),
            sensor::dummy::Sensor::new(&String::from(sensor_id.clone())),
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop());
        match self.add_client(sensor_id, handle) {
            Ok(_) => {}
            Err(err) => {
                Supervisor::handle_err(err);
            }
        };
    }

    fn add_actor(&mut self, actor_id: ClientId, config: &NatsConfig) {
        let tmp_id = String::from(actor_id.clone());
        let gpio_pin = hardware_impl::get_gpio_pin(0, &tmp_id).unwrap();
        let actor = actor::ActorClient::new(
            actor_id.clone(),
            actor::simple_gpio::Actor::new(&tmp_id, gpio_pin).unwrap(),
            config,
        );
        let handle = thread::spawn(|| actor.client_loop());
        match self.add_client(actor_id, handle) {
            Ok(_) => {}
            Err(err) => {
                Supervisor::handle_err(err);
            }
        };
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), PubSubError> {
        if self.active_clients.contains_key(id) {
            Ok(())
        } else {
            Err(PubSubError::Client(format!(
                "No active client with id '{}'",
                id
            )))
        }
    }

    pub fn init_from_config(config: config::Config) -> Supervisor {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: HashMap::new(),
        };

        supervisor.add_logger(&config);

        let dummy_sensor = ClientId("dummy_sensor".into());
        supervisor.add_sensor(dummy_sensor, &config.nats);

        let dummy_actor = ClientId("dummy_actor".into());
        supervisor.add_actor(dummy_actor, &config.nats);

        supervisor
    }

    fn add_client(
        &mut self,
        new_client: ClientId,
        handle: thread::JoinHandle<Result<(), PubSubError>>,
    ) -> Result<(), PubSubError> {
        if self.active_clients.contains_key(&new_client) {
            Err(PubSubError::Client(format!(
                "'{}' already in active clients",
                &new_client
            )))
        } else {
            self.active_clients.insert(new_client, handle);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Missing(String, String),
    AlreadyActive(String),
    Sensor(String),
    ConcurrencyError(String),
    ThreadJoin,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Missing(type_, id) => write!(f, "ID '{}' does not exist for {}", id, type_),
            Error::AlreadyActive(id) => write!(f, "ID is already in use: {}", id),
            Error::Sensor(err) => write!(f, "Measurement error: {}", err),
            Error::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            Error::ThreadJoin => write!(f, "Could not join thread"),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Missing(_, _) => "Requested service does not exist.",
            Error::AlreadyActive(_) => "ID is already in use.",
            Error::Sensor(_) => "Measurement error.",
            Error::ConcurrencyError(_) => "Concurrency error",
            Error::ThreadJoin => "Error joining thread.",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
