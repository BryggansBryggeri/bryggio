use super::{Control, State};
use std::f32;

pub struct ManualController {
    pub target: f32,
    pub current_signal: f32,
    pub state: State,
}

impl ManualController {
    pub fn new(target: f32) -> ManualController {
        ManualController {
            target,
            current_signal: 0.0,
            state: State::Active,
        }
    }
}

impl Control for ManualController {
    fn calculate_signal(&mut self, _measurement: Option<f32>) -> f32 {
        self.current_signal = self.target;
        self.current_signal
    }

    fn get_state(&self) -> State {
        self.state
    }

    fn get_control_signal(&self) -> f32 {
        self.current_signal
    }

    fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    fn get_target(&self) -> f32 {
        self.target
    }
}
