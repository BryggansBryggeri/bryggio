use crate::brewery;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::sync;
use std::sync::mpsc;

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
            Err(err) => {
                return Err(Error {
                    message: format!("Could not aquire web sender lock: {}", err),
                })
            }
        };

        match sender.send(request) {
            Ok(_) => {}
            Err(err) => {
                return Err(Error {
                    message: format!("Could not send request: {}", err),
                })
            }
        };

        let receiver = match self.receiver.lock() {
            Ok(receiver) => receiver,
            Err(err) => {
                return Err(Error {
                    message: format!("Could not aquire web receiver lock: {}", err),
                })
            }
        };

        match receiver.recv() {
            Ok(answer) => return Ok(answer),
            Err(err) => {
                return Err(Error {
                    message: format!("Could not aquire receiver response: {}", err),
                })
            }
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

pub fn generate_web_response(api_response: Result<Response, Error>) -> HashMap<String, String> {
    let mut response = HashMap::new();
    match api_response {
        Ok(api_response) => {
            response.insert("success".to_string(), api_response.success.to_string());
        }
        Err(err) => {
            response.insert("success".to_string(), "false".to_string());
            response.insert("response".to_string(), err.message);
        }
    };
    response
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "API error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
