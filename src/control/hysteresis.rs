use crate::control;
use crate::error;
use std::f32;

pub struct Controller {
    pub target: f32,
    pub current_signal: f32,
    previous_measurement: Option<f32>,
    pub state: control::State,
    offset_on: f32,
    offset_off: f32,
}

impl Controller {
    pub fn new(offset_on: f32, offset_off: f32) -> Result<Controller, error::ParamError> {
        if offset_off >= 0.0 && offset_on > offset_off {
            Ok(Controller {
                target: 20.0,
                current_signal: 0.0,
                previous_measurement: None,
                state: control::State::Inactive,
                offset_on,
                offset_off,
            })
        } else {
            Err(error::ParamError)
        }
    }
}

impl control::Control for Controller {
    fn update_state(&self) {}

    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32 {
        println!("Current target: {}", self.target);
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

    fn get_state(&self) -> control::State {
        // Tmp fix for the run_controller / controller.run mix
        self.state.clone()
    }

    fn set_state(&mut self, new_state: control::State) {
        self.state = new_state;
    }

    fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    fn get_signal(&self) -> f32 {
        self.current_signal
    }
}
