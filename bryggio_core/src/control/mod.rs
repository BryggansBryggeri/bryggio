//! Control algorithms for the brewing process.
pub mod hysteresis;
pub mod manual;
pub mod pid;

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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ControllerError {
    #[error("Param. error: {0}")]
    ParamError(String),
    #[error("Invalid target '{0}': {1}")]
    InvalidTarget(f32, String),
}
