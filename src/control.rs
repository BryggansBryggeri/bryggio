use crate::actor;
use crate::actor::Actor;
use crate::sensor;
use crate::sensor::Sensor;
use std::error;
use std::fmt;
use std::{thread, time};

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
                target: 0.0,
                current_power: 0.0,
                mode: Mode::Inactive,
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
    fn run(&self) {
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
                let power = self.calculate_power(&measurement);
                self.actor.set_power(power);
                thread::sleep(self.get_sleep_time());
            }
        }
    }

    fn get_measurement(&self) -> f32 {
        65.0
    }

    fn calculate_power(&self, value: &f32) -> f32 {
        let diff = self.target - value;
        if diff > self.offset_on {
            return 100.0;
        } else if diff <= self.offset_off {
            return 0.0;
        } else {
            self.current_power
        }
    }

    fn get_sleep_time(&self) -> time::Duration {
        time::Duration::from_secs(3)
    }

    fn get_period(&self) -> time::Duration {
        std::time::Duration::from_millis(10000)
    }

    fn is_active(&self) -> bool {
        true
    }
}

pub trait Control {
    fn run(&self);
    fn get_measurement(&self) -> f32;
    fn calculate_power(&self, measurement: &f32) -> f32;
    fn get_sleep_time(&self) -> time::Duration;
    fn get_period(&self) -> time::Duration;
    fn is_active(&self) -> bool;
}
