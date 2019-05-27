use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::control::Control;
use crate::sensor;
use std::error as std_error;
use std::sync;
use std::thread;

pub enum Command {
    GetMeasurement,
    SetMeasurement,
    StartController,
    StopController,
    GetFullState,
    Error,
}

pub struct Brewery {
    api_endpoint: api::BreweryEndpoint,
    controller: sync::Arc<sync::Mutex<control::HysteresisControl>>,
    sensor: sync::Arc<sync::Mutex<sensor::DummySensor>>,
    actor: sync::Arc<sync::Mutex<actor::DummyActor>>,
}

impl Brewery {
    pub fn new(_config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let controller = sync::Arc::new(sync::Mutex::new(
            control::HysteresisControl::new(1.0, 0.0).unwrap(),
        ));
        let actor = sync::Arc::new(sync::Mutex::new(actor::DummyActor::new("mash_tun")));
        let sensor = sync::Arc::new(sync::Mutex::new(sensor::DummySensor::new("mash_tun")));
        Brewery {
            api_endpoint,
            controller,
            sensor,
            actor,
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
            let response = self.process_command(&request.command);
            self.api_endpoint.sender.send(response).unwrap();
        }
    }

    fn process_command(&mut self, command: &Command) -> api::Response {
        match command {
            Command::StartController => match self.start_controller() {
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
            Command::GetMeasurement => match sensor::get_measurement(&self.sensor) {
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
            _ => api::Response {
                result: None,
                message: Some(String::from("Not implemented yet")),
                success: false,
            },
        }
    }

    fn start_controller(&mut self) -> Result<(), Box<std_error::Error>> {
        let mut controller = match self.controller.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock"),
        };
        match controller.get_state() {
            control::State::Inactive => {
                let controller_send = self.controller.clone();
                let actor = self.actor.clone();
                let sensor = self.sensor.clone();
                thread::spawn(move || control::run_controller(controller_send, actor, sensor));
                controller.state = control::State::Automatic;
            }
            control::State::Automatic => println!("Already running"),
            control::State::Manual => {}
            control::State::Boil => {} //_ => return Err(Box::new(error::KeyError)), // TODO impl. warning
        };
        Ok(())
    }
}
