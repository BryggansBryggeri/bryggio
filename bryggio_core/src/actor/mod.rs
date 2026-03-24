//! Actor (heater/pump) driver logic.
pub mod bin_gpio;
pub mod simple_gpio;

use crate::hardware::HardwareError;
use crate::time::TimeStamp;
use crate::hardware::GpioState;
use thiserror::Error;

/// Signal commanding an actor to a specific power level.
#[derive(Debug, Clone, PartialEq)]
pub struct ActorSignal {
    pub signal: f32, // [0, 1] normalized
}

impl ActorSignal {
    pub fn new(signal: f32) -> Self {
        ActorSignal { signal }
    }

    pub fn gpio_state(&self) -> GpioState {
        if self.signal > 0.0 {
            GpioState::High
        } else {
            GpioState::Low
        }
    }
}

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Invalid signal: {signal}, must be in ({lower_bound}, {upper_bound})")]
    InvalidSignal {
        signal: f32,
        lower_bound: f32,
        upper_bound: f32,
    },
    #[error("No state change in signal")]
    ChangingToAlreadyActiveState,
    #[error("Remaining cool down time: {0}")]
    TimeOut(TimeStamp),
    #[error("Generic: {0}")]
    Generic(String),
    #[error("Hardware: {0}")]
    Hardware(#[from] HardwareError),
    #[error("Failed turning off actor")]
    TurnOff,
}
