use crate::brewery;
use std::sync;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Error;

pub struct Request {
    pub command: brewery::Command,
    pub id: Option<String>,
    pub parameter: Option<f32>,
}

pub struct Response {
    pub result: Option<f32>,
    pub success: bool,
}

pub struct WebEndpoint {
    sender: sync::Mutex<mpsc::Sender<Request>>,
    receiver: sync::Mutex<mpsc::Receiver<Response>>,
}

impl WebEndpoint {
    pub fn send_and_wait_for_response(&self, request: Request) -> Result<Response, Error> {
        let sender = match self.sender.lock() {
            Ok(sender) => sender,
            Err(e) => return Err(Error),
        };

        match sender.send(request) {
            Ok(_) => {}
            Err(e) => return Err(Error),
        };

        let receiver = match self.receiver.lock() {
            Ok(receiver) => receiver,
            Err(e) => return Err(Error),
        };

        match receiver.recv() {
            Ok(answer) => return Ok(answer),
            Err(e) => return Err(Error),
        };
    }
}

pub struct BreweryEndpoint {
    pub sender: mpsc::Sender<Response>,
    pub receiver: mpsc::Receiver<Request>,
}

pub fn create_api_endpoints() -> (WebEndpoint, BreweryEndpoint) {
    let (tx_web, rx_brew) = mpsc::channel();
    let (tx_brew, rx_web) = mpsc::channel();
    let api_web = WebEndpoint {
        sender: sync::Mutex::new(tx_web),
        receiver: sync::Mutex::new(rx_web),
    };
    let api_brew = BreweryEndpoint {
        sender: tx_brew,
        receiver: rx_brew,
    };
    (api_web, api_brew)
}
