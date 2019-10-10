use crate::brewery;
use rocket_contrib::json;
use serde::Serialize;
use std::error;
use std::fmt;
use std::sync;
use std::sync::mpsc;

pub struct Request {
    pub command: brewery::Command,
    pub id: String,
    pub parameter: Option<f32>,
}

#[derive(Serialize)]
pub struct Response {
    pub result: Option<f32>,
    pub message: Option<String>,
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
                return Err(Error::APIError(format!(
                    "Could not aquire web sender lock: {}",
                    err
                )));
            }
        };

        let receiver = match self.receiver.lock() {
            Ok(receiver) => receiver,
            Err(err) => {
                return Err(Error::APIError(format!(
                    "Could not aquire web receiver lock: {}",
                    err
                )));
            }
        };

        match sender.send(request) {
            Ok(_) => {}
            Err(err) => {
                return Err(Error::APIError(format!("Could not send request: {}", err)));
            }
        };

        match receiver.recv() {
            Ok(answer) => Ok(answer),
            Err(err) => Err(Error::APIError(format!(
                "Could not aquire receiver response: {}",
                err
            ))),
        }
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

pub fn generate_web_response(api_response: Result<Response, Error>) -> json::Json<Response> {
    match api_response {
        Ok(response) => json::Json(response),
        Err(err) => {
            let error_response = Response {
                success: false,
                result: None,
                message: Some(err.to_string()),
            };
            json::Json(error_response)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    APIError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::APIError(message) => write!(f, "API error: {}", message),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::APIError(_) => "API error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
