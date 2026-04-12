use super::ControllerError;

pub struct PidController {
    pub target: f32,
    pub current_signal: f32,
    kp: f32,
    ki: f32,
    kd: f32,
    integral: f32,
    prev_error: Option<f32>,
    integral_limit: f32,
}

impl PidController {
    pub fn try_new(kp: f32, ki: f32, kd: f32) -> Result<PidController, ControllerError> {
        if kp < 0.0 || ki < 0.0 || kd < 0.0 {
            return Err(ControllerError::ParamError(format!(
                "PID gains must be non-negative (kp={}, ki={}, kd={})",
                kp, ki, kd
            )));
        }
        Ok(PidController {
            target: 0.0,
            current_signal: 0.0,
            kp,
            ki,
            kd,
            integral: 0.0,
            prev_error: None,
            integral_limit: 1.0,
        })
    }

    /// Calculate control signal from measurement and time step.
    /// Returns a value in [0, 1] representing heater power.
    pub fn calculate_signal(&mut self, measurement: Option<f32>, dt: f32) -> f32 {
        let new_signal = if let Some(measurement) = measurement {
            let error = self.target - measurement;

            // Integrate with anti-windup clamp
            self.integral =
                (self.integral + error * dt).clamp(-self.integral_limit, self.integral_limit);

            // Derivative of error (zero on first call)
            let derivative = match self.prev_error {
                Some(prev) if dt > 0.0 => (error - prev) / dt,
                _ => 0.0,
            };
            self.prev_error = Some(error);

            (self.kp * error + self.ki * self.integral + self.kd * derivative).clamp(0.0, 1.0)
        } else {
            self.current_signal
        };
        self.current_signal = new_signal;
        new_signal
    }

    pub fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_proportional_only() {
        let mut pid = PidController::try_new(0.1, 0.0, 0.0).unwrap();
        pid.set_target(65.0);
        // 15 degrees below target => 0.1 * 15 = 1.5, clamped to 1.0
        assert_approx_eq!(pid.calculate_signal(Some(50.0), 1.0), 1.0);
        // 2 degrees below => 0.1 * 2 = 0.2
        assert_approx_eq!(pid.calculate_signal(Some(63.0), 1.0), 0.2);
    }

    #[test]
    fn test_above_target_gives_zero() {
        let mut pid = PidController::try_new(0.1, 0.0, 0.0).unwrap();
        pid.set_target(65.0);
        assert_approx_eq!(pid.calculate_signal(Some(70.0), 1.0), 0.0);
    }

    #[test]
    fn test_integral_accumulates() {
        let mut pid = PidController::try_new(0.0, 0.5, 0.0).unwrap();
        pid.set_target(65.0);
        // error = 1.0, integral = 1.0 * 1.0 = 1.0, signal = 0.5 * 1.0 = 0.5
        let s1 = pid.calculate_signal(Some(64.0), 1.0);
        assert_approx_eq!(s1, 0.5);
        // integral = 1.0 + 1.0 = 2.0, but clamped to 1.0, signal = 0.5 * 1.0 = 0.5
        let s2 = pid.calculate_signal(Some(64.0), 1.0);
        assert_approx_eq!(s2, 0.5);
    }

    #[test]
    fn test_derivative_responds_to_change() {
        let mut pid = PidController::try_new(0.0, 0.0, 0.1).unwrap();
        pid.set_target(65.0);
        // First call: derivative is 0 (no previous)
        assert_approx_eq!(pid.calculate_signal(Some(60.0), 1.0), 0.0);
        // Error went from 5 to 3, derivative = (3 - 5)/1 = -2, signal = 0.1 * -2 = -0.2, clamped to 0
        assert_approx_eq!(pid.calculate_signal(Some(62.0), 1.0), 0.0);
        // Error went from 3 to 6, derivative = (6 - 3)/1 = 3, signal = 0.1 * 3 = 0.3
        assert_approx_eq!(pid.calculate_signal(Some(59.0), 1.0), 0.3);
    }

    #[test]
    fn test_none_measurement_holds_signal() {
        let mut pid = PidController::try_new(0.1, 0.0, 0.0).unwrap();
        pid.set_target(65.0);
        let s = pid.calculate_signal(Some(63.0), 1.0);
        assert_approx_eq!(pid.calculate_signal(None, 1.0), s);
    }

    #[test]
    fn test_invalid_params() {
        assert!(PidController::try_new(-1.0, 0.0, 0.0).is_err());
    }
}
