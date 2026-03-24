use pid as ext_pid;

use super::ControllerError;

pub struct PidController {
    pub target: f32,
    pub current_signal: f32,
    pid: ext_pid::Pid<f32>,
}

impl PidController {
    pub fn new(
        target: f32,
        kp: f32,
        ki: f32,
        kd: f32,
        p_limit: Option<f32>,
        i_limit: Option<f32>,
        d_limit: Option<f32>,
    ) -> PidController {
        let p_limit = p_limit.unwrap_or(100.0);
        let i_limit = i_limit.unwrap_or(100.0);
        let d_limit = d_limit.unwrap_or(100.0);
        let output_limit = 100.0;
        let mut pid = ext_pid::Pid::new(target, output_limit);
        pid.p(kp, p_limit).i(ki, i_limit).d(kd, d_limit);
        PidController {
            target,
            current_signal: 0.0,
            pid,
        }
    }

    /// Calculate control signal from measurement.
    /// Returns a value in [0, 1] representing heater power.
    pub fn calculate_signal(&mut self, measurement: Option<f32>) -> f32 {
        let new_signal = if let Some(measurement) = measurement {
            let pid_output = self.pid.next_control_output(measurement).output;
            // Map PID output \in [-output_limit, output_limit] --> [0, 1]
            (pid_output + self.pid.output_limit) / (2.0 * self.pid.output_limit)
        } else {
            self.current_signal
        };
        self.current_signal = new_signal;
        new_signal
    }

    pub fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
        self.pid.setpoint = new_target;
    }

    pub fn validate_target(&self, new_target: f32) -> Result<f32, ControllerError> {
        if (0.0..=100.0).contains(&new_target) {
            Ok(new_target)
        } else {
            Err(ControllerError::InvalidTarget(
                new_target,
                String::from("You likely want a temp in [0, 100]C"),
            ))
        }
    }
}
