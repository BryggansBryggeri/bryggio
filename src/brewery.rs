use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::sensor;
use std::collections::HashMap;
use std::error as std_error;
use std::sync;
use std::thread;

pub enum Command {
    GetMeasurement,
    SetTarget,
    StartController,
    StopController,
    GetFullState,
    Error,
}

pub struct Brewery {
    api_endpoint: api::BreweryEndpoint,
    active_controllers: HashMap<String, control::ControllerHandle>,
    sensors: HashMap<String, sensor::SensorHandle>,
    actors: HashMap<String, actor::ActorHandle>,
}

impl Brewery {
    pub fn new(_config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let active_controllers: HashMap<String, control::ControllerHandle> = HashMap::new();
        let sensors: HashMap<String, sensor::SensorHandle> = HashMap::new();
        let actors: HashMap<String, actor::ActorHandle> = HashMap::new();

        Brewery {
            api_endpoint,
            active_controllers,
            sensors,
            actors,
        }
    }

    pub fn init_from_config(&mut self, _config: &config::Config) {
        // Not implemented yet, emulate config here.
        let dummy_id = "dummy";
        let dummy_sensor = sensor::dummy::Sensor::new("dummy");
        self.add_sensor(dummy_id, sync::Arc::new(sync::Mutex::new(dummy_sensor)));
        let cpu_id = "cpu";
        let cpu_sensor = sensor::rbpi_cpu_temp::RbpiCpuTemp::new("cpu");
        self.add_sensor(cpu_id, sync::Arc::new(sync::Mutex::new(cpu_sensor)));

        let actor: actor::ActorHandle =
            sync::Arc::new(sync::Mutex::new(actor::dummy::Actor::new("dummy")));
        self.add_actor(dummy_id, actor);
    }

    pub fn add_sensor(&mut self, id: &str, sensor: sensor::SensorHandle) {
        self.sensors.insert(id.into(), sensor);
    }

    pub fn add_actor(&mut self, id: &str, actor: actor::ActorHandle) {
        self.actors.insert(id.into(), actor);
    }

    pub fn run(&mut self) {
        loop {
            let request = match self.api_endpoint.receiver.recv() {
                Ok(request) => request,
                Err(_) => {
                    let mut id = HashMap::new();
                    id.insert(String::from("none"), String::from("none"));
                    api::Request {
                        command: Command::Error,
                        id,
                        parameter: None,
                    }
                }
            };
            let response = self.process_request(&request);
            self.api_endpoint.sender.send(response).unwrap();
        }
    }

    fn process_request(&mut self, request: &api::Request) -> api::Response {
        match request.command {
            Command::StartController => {
                match self.start_controller(
                    &request.id.get("controller").unwrap(),
                    &request.id.get("sensor").unwrap(),
                    &request.id.get("actor").unwrap(),
                ) {
                    Ok(_) => api::Response {
                        result: None,
                        message: None,
                        success: true,
                    },
                    Err(err) => api::Response {
                        result: None,
                        message: Some(err.to_string()),
                        success: false,
                    },
                }
            }

            Command::StopController => {
                match self.stop_controller(&request.id.get("controller").unwrap()) {
                    Ok(_) => api::Response {
                        result: None,
                        message: None,
                        success: true,
                    },
                    Err(err) => api::Response {
                        result: None,
                        message: Some(err.to_string()),
                        success: false,
                    },
                }
            }

            Command::GetMeasurement => {
                match self.get_measurement(&request.id.get("sensor").unwrap()) {
                    Ok(measurement) => api::Response {
                        result: Some(measurement),
                        message: None,
                        success: true,
                    },
                    Err(err) => api::Response {
                        result: None,
                        message: Some(err.to_string()),
                        success: false,
                    },
                }
            }

            Command::SetTarget => {
                match self.change_controller_target(
                    &request.id.get("controller").unwrap(),
                    request.parameter,
                ) {
                    Ok(()) => api::Response {
                        result: None,
                        message: None,
                        success: true,
                    },
                    Err(err) => api::Response {
                        result: None,
                        message: Some(err.to_string()),
                        success: false,
                    },
                }
            }

            _ => api::Response {
                result: None,
                message: Some(String::from("Not implemented yet")),
                success: false,
            },
        }
    }

    fn start_controller(
        &mut self,
        controller_id: &str,
        sensor_id: &str,
        actor_id: &str,
    ) -> Result<(), Error> {
        self.validate_controller_id(controller_id)?;

        let controller_lock: control::ControllerLock = sync::Arc::new(sync::Mutex::new(
            //control::hysteresis::Controller::new(1.0, 0.0).expect("Invalid parameters."),
            control::manual::Controller::new(),
        ));

        let controller_send = controller_lock.clone();
        let sensor_handle = self.get_sensor(sensor_id)?.clone();
        let actor_handle = self.get_actor(actor_id)?.clone();
        let thread_handle = thread::spawn(move || {
            control::run_controller(controller_send, sensor_handle, actor_handle)
        });

        let controller_handle = control::ControllerHandle {
            lock: controller_lock,
            thread: thread_handle,
        };
        self.active_controllers
            .insert(controller_id.into(), controller_handle);
        Ok(())
    }

    fn stop_controller(&mut self, id: &str) -> Result<(), Error> {
        let controller_handle = self.active_controllers.remove(id).unwrap();
        let mut controller = match controller_handle.lock.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        controller.set_state(control::State::Inactive);
        drop(controller);
        match controller_handle.thread.join() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::ThreadJoin),
        }
    }

    fn change_controller_target(&mut self, id: &str, new_target: Option<f32>) -> Result<(), Error> {
        let controller_handle = self.get_active_controller(id)?;
        let mut controller = match controller_handle.lock.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        if let Some(new_target) = new_target {
            controller.set_target(new_target);
        };
        Ok(())
    }

    fn get_measurement(&mut self, sensor_id: &str) -> Result<f32, Error> {
        let sensor = self.get_sensor(sensor_id)?;
        match sensor::get_measurement(sensor) {
            Ok(measurement) => Ok(measurement),
            Err(err) => Err(Error::Sensor(err.to_string())),
        }
    }

    fn validate_controller_id(&self, id: &str) -> Result<(), Error> {
        if self.active_controllers.contains_key(id) {
            Err(Error::AlreadyActive(id.into()))
        } else {
            Ok(())
        }
    }

    fn get_active_controller(&mut self, id: &str) -> Result<&control::ControllerHandle, Error> {
        match self.active_controllers.get_mut(id) {
            Some(controller) => Ok(controller),
            None => Err(Error::Missing(String::from(id))),
        }
    }

    fn get_sensor(&mut self, id: &str) -> Result<&sensor::SensorHandle, Error> {
        match self.sensors.get_mut(id) {
            Some(sensor) => Ok(sensor),
            None => Err(Error::Missing(String::from(id))),
        }
    }

    fn get_actor(&mut self, id: &str) -> Result<&actor::ActorHandle, Error> {
        match self.actors.get_mut(id) {
            Some(actor) => Ok(actor),
            None => Err(Error::Missing(String::from(id))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Missing(String),
    AlreadyActive(String),
    Sensor(String),
    ThreadJoin,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Missing(id) => write!(f, "ID does not exist: {}", id),
            Error::AlreadyActive(id) => write!(f, "ID is already in use: {}", id),
            Error::Sensor(err) => write!(f, "Measurement error: {}", err),
            Error::ThreadJoin => write!(f, "Could not join thread"),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Missing(_) => "Requested service does not exist.",
            Error::AlreadyActive(_) => "ID is already in use.",
            Error::Sensor(_) => "Measurement error.",
            Error::ThreadJoin => "Error joining thread.",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
