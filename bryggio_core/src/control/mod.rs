//! Control algorithms for the brewing process.
pub mod hysteresis;
pub mod manual;
pub mod pid;

use hysteresis::HysteresisController;
use manual::ManualController;
use pid::PidController;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Controller type configuration — used to construct the right controller.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControllerType {
    Hysteresis { offset_on: f32, offset_off: f32 },
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
    Hysteresis(HysteresisController),
    Pid(PidController),
    Manual(ManualController),
}

impl Controller {
    pub fn from_type(ct: &ControllerType) -> Result<Self, ControllerError> {
        match ct {
            ControllerType::Hysteresis {
                offset_on,
                offset_off,
            } => Ok(Controller::Hysteresis(HysteresisController::try_new(
                0.0,
                *offset_on,
                *offset_off,
            )?)),
            ControllerType::Pid { kp, ki, kd } => {
                Ok(Controller::Pid(PidController::try_new(*kp, *ki, *kd)?))
            }
            ControllerType::Manual => Ok(Controller::Manual(ManualController::new(0.0))),
        }
    }

    pub fn calculate_signal(&mut self, measurement: Option<f32>, dt: f32) -> f32 {
        match self {
            Controller::Hysteresis(c) => c.calculate_signal(measurement, dt),
            Controller::Pid(c) => c.calculate_signal(measurement, dt),
            Controller::Manual(c) => c.calculate_signal(dt),
        }
    }

    pub fn set_target(&mut self, target: f32) {
        match self {
            Controller::Hysteresis(c) => c.set_target(target),
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
