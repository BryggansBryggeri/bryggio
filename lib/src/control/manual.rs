use crate::control;
use std::f32;

pub struct Controller {
    pub target: f32,
    pub current_signal: f32,
    pub state: control::State,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            target: 0.0,
            current_signal: 0.0,
            state: control::State::Active,
        }
    }
}

impl control::Control for Controller {
    fn calculate_signal(&mut self, _measurement: Option<f32>) -> f32 {
        self.current_signal = self.target;
        self.current_signal
    }

    fn get_state(&self) -> control::State {
        self.state
    }

    fn get_control_signal(&self) -> f32 {
        self.current_signal
    }

    fn set_state(&mut self, new_state: control::State) {
        self.state = new_state;
    }

    fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    fn get_target(&self) -> f32 {
        self.target
    }
}
