pub mod hysteresis;
pub mod manual;

use crate::actor;
use crate::sensor;
use std::error as std_error;
use std::f32;
use std::sync;
use std::{thread, time};

pub struct ControllerHandle {
    pub lock: ControllerLock,
    pub thread: thread::JoinHandle<()>,
}

pub type ControllerLock = sync::Arc<sync::Mutex<dyn Control>>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum State {
    Inactive,
    Active,
}

pub fn run_controller(
    controller_lock: ControllerLock,
    actor_lock: actor::ActorHandle,
    sensor: sensor::SensorHandle,
) {
    let start_time = time::SystemTime::now();
    let sleep_time = 1000;
    let actor = match actor_lock.lock() {
        Ok(actor) => actor,
        Err(err) => panic!("Could not acquire actor lock: {}", err),
    };
    loop {
        let mut controller = match controller_lock.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock {}", err),
        };
        match controller.get_state() {
            State::Inactive => {
                println!("Inactivating controller, stopping");
                return;
            }
            State::Active => {
                let measurement = match sensor::get_measurement(&sensor) {
                    Ok(measurement) => Some(measurement),
                    Err(err) => {
                        println!(
                            "Error getting measurment from sensor: {}. Error: {}",
                            "some_id", //sensor.get_id(),
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
                println!(
                    "{}, {}, {}.",
                    start_time.elapsed().unwrap().as_secs(),
                    measurement.unwrap_or(f32::NAN),
                    signal
                );
            }
        }
        thread::sleep(time::Duration::from_millis(sleep_time));
    }
}

pub trait Control: Send {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn get_state(&self) -> State;
    fn set_state(&mut self, new_state: State);
    fn get_signal(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ParamError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParamError(param) => write!(f, "Invalid param: {}", param),
        }
    }
}
impl std_error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParamError(_) => "Invalid param",
        }
    }

    fn cause(&self) -> Option<&dyn std_error::Error> {
        None
    }
}
