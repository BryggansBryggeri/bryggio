use crate::actor;
use crate::error;
use crate::sensor;
use std::sync;
use std::{thread, time};

#[derive(Clone)]
pub enum State {
    Automatic,
    Manual,
    Boil,
    Inactive,
}

pub struct HysteresisControl {
    pub target: f32,
    pub current_signal: f32,
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
                state: State::Automatic,
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
            &self.update_mode();
            match &self.state {
                State::Inactive => {}
                _ => {
                    let measurement = match sensor::get_measurement(&sensor) {
                        Ok(measurement) => measurement,
                        Err(err) => panic!(
                            "Error getting measurment from sensor {}: {}",
                            "some_id", //sensor.get_id(),
                            err
                        ),
                    };
                    let signal = self.calculate_signal(&measurement);
                    match actor.set_signal(signal) {
                        Ok(()) => {}
                        Err(err) => println!("Error setting signal: {}", err),
                    };
                    println!(
                        "{}, {}, {}.",
                        start_time.elapsed().unwrap().as_secs(),
                        measurement,
                        signal
                    );
                }
            }
            thread::sleep(time::Duration::from_millis(sleep_time));
        }
    }

    fn update_mode(&mut self) {}

    fn calculate_signal(&self, value: &f32) -> f32 {
        let diff = self.target - value;
        if diff > self.offset_on {
            return 100.0;
        } else if diff <= self.offset_off {
            return 0.0;
        } else {
            self.current_signal
        }
    }

    fn get_state(&self) -> State {
        self.state.clone()
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
    fn calculate_signal(&self, measurement: &f32) -> f32;
    fn update_mode(&mut self);
    fn get_state(&self) -> State;
}
