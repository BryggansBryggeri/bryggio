//! # API
//!
//! ## Structure
//!
//! ### Communication
//! The brewery computer (currently a raspberry pi) runs a binary program called
//! `bryggio-server` which controls the brewery hardware and exposes an API in the
//! form of a webserver listening on a single port.
//!
//! The hardware control and webserver run in separate threads.
//!
//! The hardware control thread runs a never returning main loop
//! which waits and listens for commands from the webserver
//! and processes them as they come in.
//! Some commands spawns additional threads,
//! e.g. for starting an indefinite control of a sensor, actor pair.
//!
//! The commands are sent as http requests in the form of:
//!
//! `<ip_address_of_device>:<port>/<command_name>?<param_key_1>=<param_val_1>&<param_key_2>=<param_val_2>`
//!
//! ### Routing
//!
//! The http requests are routed in a very picky way.
//! Anything that cannot be properly matched to a *route*
//! (see `routes::backend`)
//! will default to a 404 not found error.
//!
//! That includes parsing of the query parameters into strict rust types, meaning that:
//!
//! `GET "/set_target_signal?controller_id=dummy&new_target_signal=10"`
//!
//! works, but
//!
//! `GET "/set_target_signal?controller_id=dummy&new_target_signal=aba"`
//!
//! will be routed to 404, since `aba` cannot be parsed to a float value.
//!
//! ### Processing
//!
//! In each route, a variant of the enum `brewery::Command` is created,
//! where each variant holds the necessary data for the command.
//! For commands that require some query parameters, they are parsed from the http request.
//!
//! The command is then sent to the hardware thread and processed accordingly.
//!
//! ### Response
//!
//! A response struct api::Response is created on the form:
//!
//! ```rust
//! pub struct Response {
//!     pub success: bool,
//!     pub result: Option<f32>,
//!     pub message: Option<String>,
//! }
//! ```
//!
//! which is serialised to a Json string and sent back to the caller.
//! The Json string will have keys corresponding to the properties in the struct.
//!
//! Requests which are not routed and are caught by the 404 route will
//! result in just the same `Response` struct.
//!
//! **Note:** This struct will likely change to look more like the brewery command enum,
//! but the only thing that will change for the frontend is the possible keys in the
//! Json string.
//!
//! ### Control process
//!
//! The only slightly complicated part in the program architecure are the *control processes*.
//! A control process is created with a pair of actor and sensor and runs in a separate thread.
//! The thread first locks the actor so that no other object can access it
//! and then enters its own main loop where in every iteration the controller:
//!
//! 1. Locks the controller, preventing access to it.
//! 1. Reads a measurement from the sensor.
//! 1. Updates the control signal using the new measurement and the target signal.
//! 1. Sends the new control signal to the actor.
//! 1. Unlocks the controller and sleeps for a fixed number of milliseconds (1000).
//! During this window the state of the controller can be changed by sending commands.
//!
//! When a command is sent to an active controller it waits until it sleeps and then communicates with it.
//! This means that in the worst case scenario, the response time will be as long as the sleep time.
//!
//! ## Current hardcoded status
//!
//! Since the dynamics are only partially implemented, the registration of sensors and actors
//! are hardcoded into the binary with the following members:
//!
//! ### Sensors
//!
//! | sensor_id | Sensor type |
//! | ----------- | ----------- |
//! | "dummy"     | `sensor::dummy::Sensor`     |
//! | "cpu"       | `sensor::cpu_temp::CpuTemp` |
//! | "dsb_test"  | `sensor::dsb1820::DSB1820`  |
//!
//! ### Actors
//!
//! | actor_id | Actor type |
//! | ----------- | ----------- |
//! | "dummy"     | `actor::dummy::Actor`       |
//!
use crate::supervisor;
use rocket_contrib::json;
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::sync;
use std::sync::mpsc;

pub mod routes;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T>
where
    T: Serialize,
{
    pub result: Option<T>,
    pub message: Option<String>,
    pub success: bool,
}

pub struct WebEndpoint<T>
where
    T: Serialize,
{
    sender: sync::Mutex<mpsc::Sender<supervisor::Command>>,
    receiver: sync::Mutex<mpsc::Receiver<Response<T>>>,
}

impl<T> WebEndpoint<T>
where
    T: Serialize,
{
    pub fn send_and_wait_for_response(
        &self,
        request: supervisor::Command,
    ) -> Result<Response<T>, Error> {
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

pub struct BreweryEndpoint<T>
where
    T: Serialize,
{
    pub sender: mpsc::Sender<Response<T>>,
    pub receiver: mpsc::Receiver<supervisor::Command>,
}

pub fn create_api_endpoints<T>() -> (WebEndpoint<T>, BreweryEndpoint<T>)
where
    T: Serialize,
{
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

pub fn generate_api_response<T>(api_response: Result<Response<T>, Error>) -> json::Json<Response<T>>
where
    T: Serialize,
{
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
