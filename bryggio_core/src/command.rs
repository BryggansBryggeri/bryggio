//! Commands sent from the UI (or future recipe system) to the control loop.
use crate::control::{ControlConfig, ControlSource, ControllerType};
use crate::state::BrewPhase;
use crate::types::Temperature;
use serde::{Deserialize, Serialize};

/// A command that modifies the brewing system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Set a target temperature for the heater.
    SetTarget { temperature: Temperature },
    /// Change the current brew phase.
    SetPhase(BrewPhase),
    /// Turn the pump on or off.
    SetPump(bool),
    /// Switch the controller algorithm.
    SetController(ControllerType),
    /// Switch which sensor the controller reads from.
    SetControlSource(ControlSource),
    /// Set the full control configuration at once.
    SetControlConfig(ControlConfig),
}
