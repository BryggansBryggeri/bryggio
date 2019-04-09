use crate::api;
use crate::config;
use crate::control;
use crate::control::Control;

pub enum Command {
    GetTemperature,
    SetTemperature,
    StartController,
    StopController,
    GetFullState,
}

pub struct Brewery {
    mash_tun: MashTun,
    api_endpoint: api::BreweryEndpoint,
}

impl Brewery {
    pub fn new(config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        Brewery {
            mash_tun: MashTun::new(),
            api_endpoint: api_endpoint,
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
                    println!("Received from web: Start controller");
                    api::Response {
                        result: None,
                        success: true,
                    }
                }
                _ => api::Response {
                    result: None,
                    success: false,
                },
            };
            self.api_endpoint.sender.send(response).unwrap();
        }
    }
}

pub struct MashTun {
    pub controller: control::HysteresisControl,
}

impl MashTun {
    pub fn new() -> MashTun {
        MashTun {
            controller: control::HysteresisControl::new(2.0, 1.0).unwrap(),
        }
    }

    pub fn run(&mut self) {
        self.controller.run(1000);
    }
}
