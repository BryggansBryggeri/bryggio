use super::ControllerError;

pub struct ManualController {
    pub target: f32,
    pub current_signal: f32,
}

impl ManualController {
    pub fn new(target: f32) -> ManualController {
        ManualController {
            target,
            current_signal: 0.0,
        }
    }

    /// Signal equals target directly — manual power control.
    pub fn calculate_signal(&mut self) -> f32 {
        self.current_signal = self.target;
        self.current_signal
    }

    pub fn set_target(&mut self, new_target: f32) {
        self.target = new_target;
    }

    pub fn validate_target(&self, new_target: f32) -> Result<f32, ControllerError> {
        if (0.0..=1.0).contains(&new_target) {
            Ok(new_target)
        } else {
            Err(ControllerError::InvalidTarget(
                new_target,
                String::from("Target must be in [0, 1]."),
            ))
        }
    }
}
