use crate::actor;
use crate::error;
use crate::sensor;
use std::f32;
use std::sync;
use std::{thread, time};

#[derive(Clone)]
pub enum State {
    Inactive,
    Automatic,
    Manual,
}

pub fn run_controller<C, A, S>(
    controller_lock: sync::Arc<sync::Mutex<C>>,
    actor_lock: sync::Arc<sync::Mutex<A>>,
    sensor: sync::Arc<sync::Mutex<S>>,
) where
    C: Control,
    A: actor::Actor,
    S: sensor::Sensor,
{
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
            State::Automatic => {
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
            State::Manual => {
                let signal = controller.get_signal();
                drop(controller);
                match actor.set_signal(signal) {
                    Ok(()) => {}
                    Err(err) => println!("Error setting signal: {}", err),
                };
                println!("{}, {}.", start_time.elapsed().unwrap().as_secs(), signal);
            }
        }
        thread::sleep(time::Duration::from_millis(sleep_time));
    }
}

pub struct HysteresisControl {
    pub target: f32,
    pub current_signal: f32,
    previous_measurement: Option<f32>,
    pub state: State,
    offset_on: f32,
    offset_off: f32,
}

impl HysteresisControl {
    pub fn new(offset_on: f32, offset_off: f32) -> Result<HysteresisControl, error::ParamError> {
        if offset_off >= 0.0 && offset_on > offset_off {
            Ok(HysteresisControl {
                target: 20.0,
                current_signal: 0.0,
                previous_measurement: None,
                state: State::Inactive,
                offset_on: offset_on,
                offset_off: offset_off,
            })
        } else {
            Err(error::ParamError)
        }
    }
}

impl Control for HysteresisControl {
    fn run<A, S>(
        &mut self,
        sleep_time: u64,
        actor_mut: sync::Arc<sync::Mutex<A>>,
        sensor: sync::Arc<sync::Mutex<S>>,
    ) where
        A: actor::Actor,
        S: sensor::Sensor,
    {
        let start_time = time::SystemTime::now();
        let actor = match actor_mut.lock() {
            Ok(actor) => actor,
            Err(err) => panic!("Could not acquire actor lock"),
        };
        loop {
            &self.update_state();
            match &self.state {
                State::Inactive => {}
                _ => {
                    let measurement = match sensor::get_measurement(&sensor) {
                        Ok(measurement) => Some(measurement),
                        Err(err) => panic!(
                            "Error getting measurment from sensor {}: {}",
                            "some_id", //sensor.get_id(),
                            err
                        ),
                    };
                    let signal = self.calculate_signal(measurement);
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

    fn update_state(&self) {}

    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32 {
        let measurement = match measurement {
            Some(measurement) => Some(measurement),
            None => match self.previous_measurement {
                Some(previous_measurement) => Some(previous_measurement),
                None => None,
            },
        };
        match measurement {
            Some(measurement) => {
                let diff = self.target - measurement;
                if diff > self.offset_on {
                    self.current_signal = 100.0;
                } else if diff <= self.offset_off {
                    self.current_signal = 0.0;
                } else {
                }
                self.current_signal
            }
            None => self.current_signal,
        }
    }

    fn get_state(&self) -> State {
        // Tmp fix for the run_controller / controller.run mix
        self.state.clone()
    }

    fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    fn get_signal(&self) -> f32 {
        self.current_signal
    }
}

pub trait Control {
    fn run<A, S>(
        &mut self,
        sleep_time: u64,
        actor: sync::Arc<sync::Mutex<A>>,
        sensor: sync::Arc<sync::Mutex<S>>,
    ) where
        A: actor::Actor,
        S: sensor::Sensor;
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32;
    fn update_state(&self);
    fn get_state(&self) -> State;
    fn get_signal(&self) -> f32;
    fn set_target(&mut self, new_target: f32);
}
