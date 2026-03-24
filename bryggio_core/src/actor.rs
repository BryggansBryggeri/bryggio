//! Actor type definitions — no I/O.
use crate::types::Power;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Signal commanding an actor to a specific power level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActorSignal {
    pub power: Power,
}

impl ActorSignal {
    pub fn new(power: Power) -> Self {
        ActorSignal { power }
    }

    pub fn off() -> Self {
        ActorSignal {
            power: Power::off(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ActorError {
    #[error("Invalid signal: {0}, must be in [0, 1]")]
    InvalidSignal(f32),
    #[error("No state change in signal")]
    ChangingToAlreadyActiveState,
    #[error("Hardware: {0}")]
    Hardware(String),
    #[error("Failed turning off actor")]
    TurnOff,
}
