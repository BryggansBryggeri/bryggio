use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::pub_sub::{nats::NatsClient, ClientId};
use crate::sensor;
use std::collections::HashMap;
use std::error as std_error;
use std::sync;
use std::thread;

#[cfg(target_arch = "x86_64")]
use crate::hardware::dummy as hardware_impl;
#[cfg(target_arch = "arm")]
use crate::hardware::rbpi as hardware_impl;

pub enum Command {
    KillClient { client_id: ClientId },
    StartClient { client_id: ClientId },
}

pub struct Brewery {
    client: NatsClient,
    active_clients: HashMap<ClientId, ()>,
}

impl Brewery {
    pub fn new(server: &str, user: &str, pass: &str) -> Brewery {
        let client = NatsClient::try_new(server, user, pass).unwrap();
        let active_clients: HashMap<ClientId, ()> = HashMap::new();

        Brewery {
            client,
            active_clients,
        }
    }

    pub fn init_from_config(&mut self, config: &config::Config) {
        let dummy_id = "dummy";
        let dummy_sensor = sensor::dummy::Sensor::new(dummy_id);
        self.add_sensor(dummy_id, sync::Arc::new(sync::Mutex::new(dummy_sensor)));

        let cpu_id = "cpu";
        let cpu_sensor = sensor::cpu_temp::CpuTemp::new(cpu_id);
        self.add_sensor(cpu_id, sync::Arc::new(sync::Mutex::new(cpu_sensor)));

        for sensor in &config.hardware.sensors {
            let ds18_sensor = match sensor::ds18b20::Ds18b20::try_new(&sensor.id, &sensor.address) {
                Ok(sensor) => sensor,
                Err(err) => {
                    println!("Error registering sensor, {}", err.to_string());
                    continue;
                }
            };
            let sensor_handle: sensor::SensorHandle = sync::Arc::new(sync::Mutex::new(ds18_sensor));
            self.add_sensor(&sensor.id, sensor_handle);
        }

        for actor in &config.hardware.actors {
            let gpio_pin = hardware_impl::get_gpio_pin(actor.gpio_pin, &actor.id).unwrap();
            match actor::simple_gpio::Actor::new(&actor.id, gpio_pin) {
                Ok(gpio_actor) => {
                    let actor_handle: actor::ActorHandle =
                        sync::Arc::new(sync::Mutex::new(gpio_actor));
                    self.add_actor(&actor.id, actor_handle);
                }
                Err(err) => println!("Error adding actor: {}", err),
            };
        }
    }

    fn add_sensor(&mut self, id: &str, sensor: sensor::SensorHandle) {
        self.sensors.insert(id.into(), sensor);
    }

    fn add_actor(&mut self, id: &str, actor: actor::ActorHandle) {
        self.actors.insert(id.into(), actor);
    }

    pub fn run(&mut self) {}

    fn process_request(&mut self, request: &Command) -> () {
        match request {
            Command::StartClient { client_id } => {}
            Command::KillClient { client_id } => {}
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
