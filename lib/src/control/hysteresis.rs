use crate::control;
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
    pub fn try_new(offset_on: f32, offset_off: f32) -> Result<Controller, control::Error> {
        if offset_off >= 0.0 {
            if offset_on > offset_off {
                Ok(Controller {
                    target: 50.0,
                    current_signal: 0.0,
                    previous_measurement: None,
                    state: control::State::Active,
                    offset_on,
                    offset_off,
                })
            } else {
                Err(control::Error::ParamError(format!(
                    "offset_on must be greater than the offset_off ({} !> {})",
                    offset_on, offset_off,
                )))
            }
        } else {
            Err(control::Error::ParamError(format!(
                "offset_off must be non-negative ({} !>= 0.0)",
                offset_off
            )))
        }
    }
}

impl control::Control for Controller {
    fn calculate_signal(&mut self, measurement: Option<f32>) -> f32 {
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
        self.state
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

    fn get_control_signal(&self) -> f32 {
        self.current_signal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::Control;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_constructor_valid_args() {
        let controller = Controller::try_new(2.0, 1.0);
        assert!(controller.is_ok())
    }

    #[test]
    fn test_constructor_neg_offset_off() {
        let controller = Controller::try_new(-1.5, 0.5);
        assert!(controller.is_err())
    }

    #[test]
    fn test_constructor_offset_off_lt_offset_on() {
        let controller = Controller::try_new(3.0, 4.0);
        assert!(controller.is_err())
    }

    #[test]
    fn test_constructor_active_on_init() {
        let controller = Controller::try_new(2.0, 1.0).unwrap();
        assert_eq!(controller.get_state(), control::State::Active);
    }

    #[test]
    fn test_control_under() {
        let mut controller = Controller::try_new(2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(90.0)), 100.0);
    }

    #[test]
    fn test_control_ower() {
        let mut controller = Controller::try_new(2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(110.0)), 0.0);
    }

    #[test]
    fn test_control_ower_offset_on() {
        let mut controller = Controller::try_new(2.0, 1.0).unwrap();
        controller.set_target(100.0);
        assert_approx_eq!(controller.calculate_signal(Some(98.5)), 0.0);
    }

    #[test]
    fn test_control_hysteresis_logic() {
        let mut controller = Controller::try_new(2.0, 1.0).unwrap();
        controller.set_target(100.0);

        // Make sure controller.current_signal is 100.0
        assert_approx_eq!(controller.calculate_signal(Some(30.0)), 100.0);
        // Make sure controller.current_signal remains
        assert_approx_eq!(controller.calculate_signal(Some(98.5)), 100.0);
        // Make sure controller.current_signal is switched to 0.0
        assert_approx_eq!(controller.calculate_signal(Some(99.5)), 0.0);
        // Make sure controller.current_signal remains
        assert_approx_eq!(controller.calculate_signal(Some(98.5)), 0.0);
    }
}
