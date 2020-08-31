use crate::control;
use std::f32;
use std::time::Instant;

pub struct DutyCycleController {
    pub target: f32,
    pub current_signal: f32,
    pub state: control::State,
    clock: Instant,
    cycle_length: f32,
}

impl DutyCycleController {
    pub fn new(cycle_length: f32) -> DutyCycleController {
        DutyCycleController {
            target: 0.0,
            current_signal: 0.0,
            state: control::State::Active,
            clock: Instant::now(),
            cycle_length,
        }
    }
}

impl control::Control for DutyCycleController {
    fn calculate_signal(&mut self, _measurement: Option<f32>) -> f32 {
        let delta = Instant::now() - self.clock;
        if calculate_cycle_ratio(delta.as_secs(), self.cycle_length) > self.target {
            self.current_signal = 0.0
        } else {
            self.current_signal = 1.0
        }
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

fn calculate_cycle_ratio(delta: u64, cycle_length: f32) -> f32 {
    (delta as f32 % cycle_length) / cycle_length
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_duty_cycle() {
        assert_approx_eq!(calculate_cycle_ratio(17, 10.0), 0.7);
        assert_approx_eq!(calculate_cycle_ratio(27, 10.0), 0.7);
    }
}
