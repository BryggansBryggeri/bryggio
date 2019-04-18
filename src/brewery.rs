use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::control::Control;
use crate::error;
use crate::sensor;
use rand::distributions::{Distribution, Normal};
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
    //controller: sync::Arc<control::HysteresisControl>,
    sensor: sync::Arc<sync::Mutex<sensor::DummySensor>>,
    actor: sync::Arc<sync::Mutex<actor::DummyActor>>,
}

impl Brewery {
    pub fn new(config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        //let controller = sync::Arc::new(control::HysteresisControl::new(1.0, 0.0).unwrap());
        let actor = sync::Arc::new(sync::Mutex::new(actor::DummyActor::new("mash_tun_dummy")));
        let sensor = sync::Arc::new(sync::Mutex::new(sensor::DummySensor::new("mash_tun_dummy")));
        Brewery {
            api_endpoint,
            //controller,
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
            let response = match request.command {
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
            };
            self.api_endpoint.sender.send(response).unwrap();
        }
    }

    fn start_controller(&self) -> Result<(), Box<std_error::Error>> {
        let tmp_state = control::State::Inactive;
        match tmp_state {
            control::State::Inactive => {}
            _ => return Err(Box::new(error::KeyError)), // TODO impl. warning
        };
        let mut controller = control::HysteresisControl::new(1.0, 0.0).unwrap();
        let actor = self.actor.clone();
        let sensor = self.sensor.clone();
        thread::spawn(move || controller.run(1000, actor, sensor));
        Ok(())
    }
}

pub struct MashTun {}

impl MashTun {
    pub fn new() -> MashTun {
        MashTun {}
    }
}
