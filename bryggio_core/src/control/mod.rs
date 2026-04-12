//! Control algorithms for the brewing process.
pub mod manual;
pub mod pid;

use manual::ManualController;
use pid::PidController;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Controller type configuration — used to construct the right controller.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControllerType {
    Pid { kp: f32, ki: f32, kd: f32 },
    Manual,
}

/// Which sensor reading feeds the controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSource {
    Top,
    Bottom,
    Average,
}

/// Full control configuration: what algorithm and what it reads from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlConfig {
    pub controller: ControllerType,
    pub source: ControlSource,
}

/// Active controller — dispatches to the concrete implementation.
pub enum Controller {
    Pid(PidController),
    Manual(ManualController),
}

impl Controller {
    pub fn from_type(ct: &ControllerType) -> Result<Self, ControllerError> {
        match ct {
            ControllerType::Pid { kp, ki, kd } => {
                Ok(Controller::Pid(PidController::try_new(*kp, *ki, *kd)?))
            }
            ControllerType::Manual => Ok(Controller::Manual(ManualController::new(0.0))),
        }
    }

    pub fn calculate_signal(&mut self, measurement: Option<f32>, dt: f32) -> f32 {
        match self {
            Controller::Pid(c) => c.calculate_signal(measurement, dt),
            Controller::Manual(c) => c.calculate_signal(dt),
        }
    }

    pub fn set_target(&mut self, target: f32) {
        match self {
            Controller::Pid(c) => c.set_target(target),
            Controller::Manual(c) => c.set_target(target),
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ControllerError {
    #[error("Param. error: {0}")]
    ParamError(String),
    #[error("Invalid target '{0}': {1}")]
    InvalidTarget(f32, String),
}
