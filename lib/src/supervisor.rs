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

pub(crate) const SUPERVISOR_TOPIC: &str = "supervisor";

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

type Handle = thread::JoinHandle<Result<(), PubSubError>>;

#[derive(Deserialize)]
pub enum SupervisorSubMsg {
    #[serde(rename = "start_controller")]
    StartController { control_config: ControllerConfig },
    #[serde(rename = "list_active_clients")]
    ListActiveClients,
    #[serde(rename = "toggle_controller")]
    ToggleController { control_config: ControllerConfig },
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
                let control_config: ControllerConfig = decode_nats_data(&msg.data)?;
                Ok(SupervisorSubMsg::ToggleController { control_config })
            }
            "command.list_active_clients" => Ok(SupervisorSubMsg::ListActiveClients),
            _ => Err(PubSubError::MessageParse(String::new())),
        }
    }
}

pub struct Supervisor {
    client: NatsClient,
    config: config::Config,
    active_clients: HashMap<ClientId, Handle>,
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

    fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg)
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
                Ok(ClientState::Active)
            }
            SupervisorSubMsg::ToggleController { control_config } => {
                self.toggle_controller(control_config)?;
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

    fn toggle_controller(&mut self, config: &ControllerConfig) -> Result<(), PubSubError> {
        let contr_id = &config.controller_id;
        self.kill_client(contr_id)
    }

    fn kill_client(&mut self, id: &ClientId) -> Result<(), PubSubError> {
        let handle = match self.active_clients.remove(id) {
            Some(contr) => Ok(contr),
            None => Err(PubSubError::Client(format!(
                "'{}' not an active client",
                id
            ))),
        }?;
        self.publish(
            &Subject(format!("{}.kill.{}", SUPERVISOR_TOPIC, id)),
            &PubSubMsg(String::new()),
        )?;
        match handle.join() {
            Ok(res) => res,
            Err(_err) => Err(PubSubError::Client(format!(
                "could not join client with id '{}'",
                id,
            ))),
        }
    }

    fn start_controller(&mut self, config: &ControllerConfig) -> Result<(), PubSubError> {
        config
            .client_ids()
            .map(|id| self.client_is_active(id))
            .collect::<Result<Vec<_>, SupervisorError>>()?;

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
        Ok(self.add_client(config.controller_id.clone(), control_handle)?)
    }

    fn add_logger(&mut self, config: &config::Config) -> Result<(), SupervisorError> {
        let log = Log::new(&config.nats, config.general.log_level);
        let log_handle = thread::spawn(|| log.client_loop());
        self.add_client(ClientId("log".into()), log_handle)
    }

    fn add_sensor(
        &mut self,
        sensor_id: ClientId,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let sensor = sensor::SensorClient::new(
            sensor_id.clone(),
            sensor::dummy::Sensor::new(&String::from(sensor_id.clone())),
            config,
        );
        let handle = thread::spawn(|| sensor.client_loop());
        self.add_client(sensor_id, handle)
    }

    fn add_actor(
        &mut self,
        actor_id: ClientId,
        config: &NatsConfig,
    ) -> Result<(), SupervisorError> {
        let tmp_id = String::from(actor_id.clone());
        let gpio_pin = hardware_impl::get_gpio_pin(0, &tmp_id).unwrap();
        let actor = actor::ActorClient::new(
            actor_id.clone(),
            actor::simple_gpio::Actor::new(&tmp_id, gpio_pin).unwrap(),
            config,
        );
        let handle = thread::spawn(|| actor.client_loop());
        self.add_client(actor_id, handle)
    }

    fn client_is_active(&self, id: &ClientId) -> Result<(), SupervisorError> {
        if self.active_clients.contains_key(id) {
            Ok(())
        } else {
            Err(SupervisorError::Missing(id.clone()))
        }
    }

    pub fn init_from_config(config: config::Config) -> Result<Supervisor, SupervisorError> {
        let client = NatsClient::try_new(&config.nats).unwrap();
        let mut supervisor = Supervisor {
            client,
            config: config.clone(),
            active_clients: HashMap::new(),
        };

        supervisor.add_logger(&config)?;

        let dummy_sensor = ClientId("dummy_sensor".into());
        supervisor.add_sensor(dummy_sensor, &config.nats)?;

        let dummy_actor = ClientId("dummy_actor".into());
        supervisor.add_actor(dummy_actor, &config.nats)?;

        Ok(supervisor)
    }

    fn add_client(&mut self, new_client: ClientId, handle: Handle) -> Result<(), SupervisorError> {
        if self.active_clients.contains_key(&new_client) {
            Err(SupervisorError::AlreadyActive(new_client))
        } else {
            self.active_clients.insert(new_client, handle);
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SupervisorError {
    Missing(ClientId),
    AlreadyActive(ClientId),
    Sensor(String),
    ConcurrencyError(String),
    ThreadJoin,
}

impl std::fmt::Display for SupervisorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SupervisorError::Missing(id) => write!(f, "Id '{}' is not an active client", id),
            SupervisorError::AlreadyActive(id) => write!(f, "Id '{}' is already id", id),
            SupervisorError::Sensor(err) => write!(f, "Measurement error: {}", err),
            SupervisorError::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            SupervisorError::ThreadJoin => write!(f, "Could not join thread"),
        }
    }
}
impl std_error::Error for SupervisorError {
    fn description(&self) -> &str {
        match *self {
            SupervisorError::Missing(_) => "Requested client does not exist.",
            SupervisorError::AlreadyActive(_) => "ID is already in use.",
            SupervisorError::Sensor(_) => "Measurement error.",
            SupervisorError::ConcurrencyError(_) => "Concurrency error",
            SupervisorError::ThreadJoin => "Error joining thread.",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
