use crate::control;
use pid as ext_pid;
use std::f32;

pub struct Controller {
    pub target: f32,
    pub current_signal: f32,
    state: control::State,
    pid: ext_pid::Pid<f32>,
}

impl Controller {
    pub fn new(
        kp: f32,
        ki: f32,
        kd: f32,
        p_limit: Option<f32>,
        i_limit: Option<f32>,
        d_limit: Option<f32>,
    ) -> Controller {
        let p_limit = p_limit.unwrap_or(100.0);
        let i_limit = i_limit.unwrap_or(100.0);
        let d_limit = d_limit.unwrap_or(100.0);
        let pid = ext_pid::Pid::new(kp, ki, kd, p_limit, i_limit, d_limit, 0.0);
        Controller {
            target: 0.0,
            current_signal: 0.0,
            state: control::State::Active,
            pid,
        }
    }
}

impl control::Control for Controller {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32 {
        if let Some(measurement) = measurement {
            self.pid.next_control_output(measurement).output
        } else {
            self.current_signal
        }
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
