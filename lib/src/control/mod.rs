pub mod hysteresis;
pub mod manual;
pub mod pid;

use crate::actor;
use crate::sensor;
use std::convert::TryFrom;
use std::error as std_error;
use std::f32;
use std::sync;
use std::{thread, time};

pub enum ControllerType {
    Hysteresis,
    Manual,
}

impl TryFrom<String> for ControllerType {
    type Error = Error;
    fn try_from(string: String) -> Result<Self, Error> {
        match string.to_ascii_lowercase().as_ref() {
            "hysteresis" => Ok(ControllerType::Hysteresis),
            "manual" => Ok(ControllerType::Manual),
            _ => Err(Error::ConversionError(string.into())),
        }
    }
}

pub struct ControllerHandle {
    pub lock: ControllerLock,
    pub thread: thread::JoinHandle<Result<(), Error>>,
}

pub type ControllerLock = sync::Arc<sync::Mutex<Box<dyn Control>>>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Inactive,
    Active,
}

pub fn run_controller(
    controller_lock: ControllerLock,
    sensor_handle: sensor::SensorHandle,
    actor_handle: actor::ActorHandle,
) -> Result<(), Error> {
    let _start_time = time::SystemTime::now();
    let sleep_time = 1000;
    let actor = match actor_handle.lock() {
        Ok(actor) => actor,
        Err(err) => {
            return Err(Error::ConcurrencyError(format!(
                "Could not acquire actor lock: {}",
                err
            )))
        }
    };
    loop {
        let mut controller = match controller_lock.lock() {
            Ok(controller) => controller,
            Err(err) => {
                return Err(Error::ConcurrencyError(format!(
                    "Could not acquire controller lock: {}",
                    err
                )))
            }
        };
        match controller.get_state() {
            State::Inactive => {
                println!("Inactivating controller, stopping");
                return Ok(());
            }
            State::Active => {
                let measurement = match sensor::get_measurement(&sensor_handle) {
                    Ok(measurement) => Some(measurement),
                    Err(err) => {
                        println!(
                            "Error getting measurment from sensor: '{}'. Error: {}",
                            sensor::get_id(&sensor_handle)
                                .unwrap_or_else(|_| String::from("Unknown id")),
                            err
                        );
                        None
                    }
                };
                let signal = controller.calculate_signal(measurement);
                // Need to drop controller so it is unlocked when the thread sleeps, otherwise it
                // will be unresponsive.
                drop(controller);
                match actor.set_signal(signal) {
                    Ok(()) => {}
                    Err(err) => println!("Error setting signal: {}", err),
                };
            }
        }
        thread::sleep(time::Duration::from_millis(sleep_time));
    }
}

pub trait Control: Send {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn get_state(&self) -> State;
    fn set_state(&mut self, new_state: State);
    fn get_control_signal(&self) -> f32;
    fn get_target(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ParamError(String),
    ConcurrencyError(String),
    ConversionError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParamError(param) => write!(f, "Invalid param: {}", param),
            Error::ConcurrencyError(err) => write!(f, "Concurrency error: {}", err),
            Error::ConversionError(type_string) => {
                write!(f, "Unable to parse '{}' to ControllerType", type_string)
            }
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParamError(_) => "Invalid param",
            Error::ConcurrencyError(_) => "Concurrency error",
            Error::ConversionError(_) => "Conversion error",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
