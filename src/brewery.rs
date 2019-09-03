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
    sensor_handle: sensor::SensorHandle,
}

impl Brewery {
    pub fn new(brew_config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let active_controllers: HashMap<String, control::ControllerHandle> = HashMap::new();
        // let control_box: Box<dyn Control> =
        //     Box::new(control::hysteresis::Controller::new(1.0, 0.0).expect("Invalid parameters."));
        // let controller = sync::Arc::new(sync::Mutex::new(control_box));

        // let actor: actor::ActorHandle = sync::Arc::new(sync::Mutex::new(Box::new(
        //     actor::dummy::Actor::new("dummy"),
        // )));

        // TODO: Fix ugly hack. Remove to handle if no sensor data is provided.
        let sensor_config = brew_config.sensors.clone().unwrap();
        let sensor: Box<dyn sensor::Sensor> =
            Box::new(sensor::dummy::Sensor::new(String::from(&sensor_config.id)));
        let sensor_handle = sync::Arc::new(sync::Mutex::new(sensor));
        Brewery {
            api_endpoint,
            active_controllers,
            sensor_handle,
        }
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
            Command::StartController => match self.start_controller(request.id.as_ref().unwrap()) {
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

            Command::GetMeasurement => match sensor::get_measurement(&self.sensor_handle) {
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

    fn start_controller(&mut self, id: &str) -> Result<(), Error> {
        if self.active_controllers.contains_key(id) {
            return Err(Error::AlreadyActive(id.into()));
        };

        let controller_handle: control::ControllerHandle = sync::Arc::new(sync::Mutex::new(
            Box::new(control::hysteresis::Controller::new(1.0, 0.0).expect("Invalid parameters.")),
        ));

        let mut controller = match controller_handle.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error: {}", err),
        };

        let actor: actor::ActorHandle = sync::Arc::new(sync::Mutex::new(Box::new(
            actor::dummy::Actor::new("dummy"),
        )));

        let controller_send = controller_handle.clone();
        let sensor = self.sensor_handle.clone();
        thread::spawn(move || control::run_controller(controller_send, actor, sensor));
        controller.set_state(control::State::Active);
        drop(controller);
        self.active_controllers.insert(id.into(), controller_handle);
        Ok(())
    }

    fn stop_controller(&mut self, id: &str) -> Result<(), Error> {
        let controller_handle = self.get_active_controller(id)?;
        let mut controller = match controller_handle.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        controller.set_state(control::State::Inactive);
        drop(controller);
        self.active_controllers.remove(id);
        Ok(())
    }

    fn get_active_controller(&mut self, id: &str) -> Result<&control::ControllerHandle, Error> {
        match self.active_controllers.get_mut(id) {
            Some(controller) => Ok(controller),
            None => Err(Error::Missing(String::from(id))),
        }
    }

    fn change_controller_target(&mut self, id: &str, new_target: Option<f32>) -> Result<(), Error> {
        let controller_handle = self.get_active_controller(id)?;
        let mut controller = match controller_handle.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        if let Some(new_target) = new_target {
            controller.set_target(new_target);
        };
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Missing(String),
    AlreadyActive(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Missing(id) => write!(f, "ID does not exist: {}", id),
            Error::AlreadyActive(id) => write!(f, "ID is already in use: {}", id),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Missing(_) => "Requested service does not exist",
            Error::AlreadyActive(_) => "ID is already in use",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
