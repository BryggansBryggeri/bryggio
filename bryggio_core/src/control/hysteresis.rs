use super::ControllerError;

pub struct HysteresisController {
    pub target: f32,
    pub current_signal: f32,
    previous_measurement: Option<f32>,
    offset_on: f32,
    offset_off: f32,
}

impl HysteresisController {
    pub fn try_new(
        target: f32,
        offset_on: f32,
        offset_off: f32,
    ) -> Result<HysteresisController, ControllerError> {
        if offset_off < 0.0 {
            return Err(ControllerError::ParamError(format!(
                "offset_off must be non-negative ({} !>= 0.0)",
                offset_off
            )));
        }
        if offset_on <= offset_off {
            return Err(ControllerError::ParamError(format!(
                "offset_on must be greater than the offset_off ({} !> {})",
                offset_on, offset_off,
            )));
        }
        Ok(HysteresisController {
            target,
            current_signal: 0.0,
            previous_measurement: None,
            offset_on,
            offset_off,
        })
    }

    /// Calculate control signal from measurement.
    /// Returns 0.0 or 1.0.
    pub fn calculate_signal(&mut self, measurement: Option<f32>, _dt: f32) -> f32 {
        let measurement = measurement.or(self.previous_measurement);
        if let Some(measurement) = measurement {
            self.previous_measurement = Some(measurement);
            let diff = self.target - measurement;
            if diff > self.offset_on {
                self.current_signal = 1.0;
            } else if diff <= self.offset_off {
                self.current_signal = 0.0;
            }
        }
        self.current_signal
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
    fn test_constructor_valid_args() {
        let controller = HysteresisController::try_new(0.0, 2.0, 1.0);
        assert!(controller.is_ok())
    }

    #[test]
    fn test_constructor_neg_offset_off() {
        let controller = HysteresisController::try_new(0.0, -1.5, 0.5);
        assert!(controller.is_err())
    }

    #[test]
    fn test_constructor_offset_off_lt_offset_on() {
        let controller = HysteresisController::try_new(0.0, 3.0, 4.0);
        assert!(controller.is_err())
    }

    #[test]
    fn test_control_under() {
        let mut controller = HysteresisController::try_new(0.0, 2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(90.0), 1.0), 1.0);
    }

    #[test]
    fn test_control_over() {
        let mut controller = HysteresisController::try_new(0.0, 2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(110.0), 1.0), 0.0);
    }

    #[test]
    fn test_control_over_offset_on() {
        let mut controller = HysteresisController::try_new(0.0, 2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(98.5), 1.0), 0.0);
    }

    #[test]
    fn test_control_hysteresis_logic() {
        let mut controller = HysteresisController::try_new(0.0, 2.0, 1.0).unwrap();
        controller.set_target(100.0);

        assert_approx_eq!(controller.calculate_signal(Some(30.0), 1.0), 1.0);
        assert_approx_eq!(controller.calculate_signal(Some(98.5), 1.0), 1.0);
        assert_approx_eq!(controller.calculate_signal(Some(99.5), 1.0), 0.0);
        assert_approx_eq!(controller.calculate_signal(Some(98.5), 1.0), 0.0);
    }
}
