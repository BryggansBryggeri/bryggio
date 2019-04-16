use crate::actor;
use crate::sensor;
use std::error;
use std::fmt;
use std::{thread, time};

#[derive(Clone)]
pub enum Mode {
    Automatic,
    Manual,
    Boil,
    Inactive,
}

pub struct HysteresisControl {
    pub target: f32,
    pub current_signal: f32,
    pub mode: Mode,
    offset_on: f32,
    offset_off: f32,
}

impl HysteresisControl {
    pub fn new(offset_on: f32, offset_off: f32) -> Result<HysteresisControl, ParamError> {
        if offset_off >= 0.0 && offset_on > offset_off {
            Ok(HysteresisControl {
                target: 20.0,
                current_signal: 0.0,
                mode: Mode::Automatic,
                offset_on: offset_on,
                offset_off: offset_off,
            })
        } else {
            Err(ParamError)
        }
    }
}

impl Control for HysteresisControl {
    fn run<A, S>(&mut self, sleep_time: u64, actor: A, sensor: S)
    where
        A: actor::Actor,
        S: sensor::Sensor,
    {
        let start_time = time::SystemTime::now();
        loop {
            &self.update_mode();
            match &self.mode {
                Mode::Inactive => {}
                _ => {
                    let measurement = match sensor.get_measurement() {
                        Ok(measurement) => measurement,
                        Err(err) => panic!(
                            "Error getting measurment from sensor {}: {}",
                            sensor.get_id(),
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
}

pub trait Control {
    fn run<A, S>(&mut self, sleep_time: u64, actor: A, sensor: S)
    where
        A: actor::Actor,
        S: sensor::Sensor;
    fn calculate_signal(&self, measurement: &f32) -> f32;
    fn update_mode(&mut self);
}

#[derive(Debug, Clone)]
pub struct ParamError;

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid param values.")
    }
}
// This is important for other errors to wrap this one.
impl error::Error for ParamError {
    fn description(&self) -> &str {
        "Invalid param values."
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}
