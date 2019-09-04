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
}

impl Brewery {
    pub fn new(_config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let active_controllers: HashMap<String, control::ControllerHandle> = HashMap::new();
        let sensors: HashMap<String, sensor::SensorHandle> = HashMap::new();

        Brewery {
            api_endpoint,
            active_controllers,
            sensors,
        }
    }

    pub fn init_from_config(&mut self, _config: &config::Config) {
        let id = "dummy";
        let dummy_sensor = Box::new(sensor::dummy::Sensor::new("dummy".into()));
        self.add_sensor(id, dummy_sensor);
    }

    pub fn add_sensor(&mut self, id: &str, sensor: Box<dyn sensor::Sensor>) {
        self.sensors
            .insert(id.into(), sync::Arc::new(sync::Mutex::new(sensor)));
    }

    pub fn run(&mut self) {
        loop {
            let request = match self.api_endpoint.receiver.recv() {
                Ok(request) => request,
                Err(_) => api::Request {
                    command: Command::Error,
                    id: None,
                    parameter: None,
                },
            };
            let response = self.process_request(&request);
            self.api_endpoint.sender.send(response).unwrap();
        }
    }

    fn process_request(&mut self, request: &api::Request) -> api::Response {
        match request.command {
            Command::StartController => {
                match self.start_controller(request.id.as_ref().unwrap(), "dummy") {
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

            Command::StopController => match self.stop_controller(request.id.as_ref().unwrap()) {
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
            },

            Command::GetMeasurement => match self.get_measurement(request.id.as_ref().unwrap()) {
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
            },

            Command::SetTarget => {
                match self.change_controller_target(request.id.as_ref().unwrap(), request.parameter)
                {
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

    fn start_controller(&mut self, controller_id: &str, sensor_id: &str) -> Result<(), Error> {
        self.validate_controller_id(controller_id)?;

        let controller_lock: control::ControllerLock = sync::Arc::new(sync::Mutex::new(
            //Box::new(control::hysteresis::Controller::new(1.0, 0.0).expect("Invalid parameters.")),
            control::manual::Controller::new(),
        ));

        let actor: actor::ActorHandle = sync::Arc::new(sync::Mutex::new(Box::new(
            actor::dummy::Actor::new("dummy"),
        )));

        let controller_send = controller_lock.clone();
        let sensor_handle = self.get_sensor(sensor_id)?.clone();
        let thread_handle =
            thread::spawn(move || control::run_controller(controller_send, actor, sensor_handle));

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
            Ok(()) => Ok(()),
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
