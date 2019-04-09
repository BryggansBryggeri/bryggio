use crate::actor;
use crate::actor::Actor;
use crate::sensor;
use crate::sensor::Sensor;
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
    pub current_power: f32,
    pub mode: Mode,
    actor: actor::DummyActor,
    sensor: sensor::DummySensor,
    offset_on: f32,
    offset_off: f32,
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

impl HysteresisControl {
    pub fn new(offset_on: f32, offset_off: f32) -> Result<HysteresisControl, ParamError> {
        if offset_off >= 0.0 && offset_on > offset_off {
            Ok(HysteresisControl {
                target: 20.0,
                current_power: 0.0,
                mode: Mode::Automatic,
                sensor: sensor::DummySensor::new("dummy"),
                actor: actor::DummyActor::new("dummy", None),
                offset_on: offset_on,
                offset_off: offset_off,
            })
        } else {
            Err(ParamError)
        }
    }
}

impl Control for HysteresisControl {
    fn run(&mut self, sleep_time: u64) {
        let start_time = time::SystemTime::now();
        loop {
            &self.update_mode();
            match &self.mode {
                Mode::Inactive => {}
                _ => {
                    let measurement = match self.sensor.get_measurement() {
                        Ok(measurement) => measurement,
                        Err(e) => panic!(
                            "Error getting measurment from sensor {}: {}",
                            self.sensor.id, e
                        ),
                    };
                    let power = self.calculate_signal(&measurement);
                    self.actor.set_signal(power).unwrap();
                    self.sensor.prediction += power * 0.05;
                    self.sensor.prediction *= 0.90;
                    println!(
                        "{}, {}, {}.",
                        start_time.elapsed().unwrap().as_secs(),
                        measurement,
                        power
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
            self.current_power
        }
    }
}

pub trait Control {
    fn run(&mut self, sleep_time: u64);
    fn calculate_signal(&self, measurement: &f32) -> f32;
    fn update_mode(&mut self);
}
