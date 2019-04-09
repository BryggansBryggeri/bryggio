use crate::config;
use crate::control;
use crate::control::Control;
use std::sync;
use std::sync::mpsc;

pub struct APIWebEndpoint {
    pub sender: sync::Mutex<mpsc::Sender<String>>,
    pub receiver: sync::Mutex<mpsc::Receiver<String>>,
}

pub struct APIBreweryEndpoint {
    pub sender: mpsc::Sender<String>,
    pub receiver: mpsc::Receiver<String>,
}

pub fn create_api_endpoints() -> (APIWebEndpoint, APIBreweryEndpoint) {
    let (tx_web, rx_brew) = mpsc::channel();
    let (tx_brew, rx_web) = mpsc::channel();
    let api_web = APIWebEndpoint {
        sender: sync::Mutex::new(tx_web),
        receiver: sync::Mutex::new(rx_web),
    };
    let api_brew = APIBreweryEndpoint {
        sender: tx_brew,
        receiver: rx_brew,
    };
    (api_web, api_brew)
}

pub struct Brewery {
    mash_tun: MashTun,
    api_endpoint: APIBreweryEndpoint,
}

impl Brewery {
    pub fn new(config: &config::Config, api_endpoint: APIBreweryEndpoint) -> Brewery {
        Brewery {
            mash_tun: MashTun::new(),
            api_endpoint: api_endpoint,
        }
    }

    pub fn run(&mut self) {
        loop {
            let command = match self.api_endpoint.receiver.recv() {
                Ok(command) => command,
                Err(e) => panic!("Command error: {}", e),
            };
            println!("Received from web: {}", command);
            self.api_endpoint.sender.send("Got it".to_string()).unwrap();
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
