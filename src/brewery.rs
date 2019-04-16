use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::sensor;
use std::collections::HashMap;

pub enum Command {
    GetTemperature,
    SetTemperature,
    StartController,
    StopController,
    GetFullState,
}

pub struct Brewery {
    api_endpoint: api::BreweryEndpoint,
    sensors: HashMap<String, sensor::DummySensor>,
    actors: HashMap<String, actor::DummyActor>,
}

impl Brewery {
    pub fn new(config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let mut sensors = HashMap::new();
        sensors.insert(
            String::from("mash_tun"),
            sensor::DummySensor::new("mash_tun_dummy"),
        );
        let mut actors = HashMap::new();
        actors.insert(
            String::from("mash_tun"),
            actor::DummyActor::new("mash_tun_dummy"),
        );
        Brewery {
            api_endpoint,
            sensors,
            actors,
        }
    }

    pub fn run(&mut self) {
        loop {
            let request = match self.api_endpoint.receiver.recv() {
                Ok(request) => request,
                Err(e) => panic!("Command error: {}", e),
            };
            let response = match request.command {
                Command::StartController => {
                    println!(
                        "Received from web: Start controller {}",
                        request.id.unwrap_or("".to_string())
                    );
                    api::Response {
                        result: None,
                        message: Some(String::from("Not implemented yet")),
                        success: false,
                    }
                }
                _ => api::Response {
                    result: None,
                    message: Some(String::from("Not implemented yet")),
                    success: false,
                },
            };
            self.api_endpoint.sender.send(response).unwrap();
        }
    }
}

pub struct MashTun {}

impl MashTun {
    pub fn new() -> MashTun {
        MashTun {}
    }
}
