//! Full system state — the snapshot broadcast via SSE.
use crate::control::ControlSource;
use crate::types::{Power, Temperature};
use serde::{Deserialize, Serialize};

/// The full state of the brewery, sent to the UI on every tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreweryState {
    pub phase: BrewPhase,
    pub vessel_temp_top: Option<Temperature>,
    pub vessel_temp_bottom: Option<Temperature>,
    pub heater_power: Power,
    pub pump_on: bool,
    pub target_temperature: Option<Temperature>,
    pub control_source: ControlSource,
    pub timestamp: u64,
}

impl BreweryState {
    /// Resolve the control measurement based on the configured source.
    pub fn control_temp(&self) -> Option<Temperature> {
        match self.control_source {
            ControlSource::Top => self.vessel_temp_top,
            ControlSource::Bottom => self.vessel_temp_bottom,
            ControlSource::Average => match (self.vessel_temp_top, self.vessel_temp_bottom) {
                (Some(top), Some(bottom)) => Some((top + bottom) / 2.0),
                (Some(t), None) | (None, Some(t)) => Some(t),
                (None, None) => None,
            },
        }
    }
}

impl Default for BreweryState {
    fn default() -> Self {
        BreweryState {
            phase: BrewPhase::Idle,
            vessel_temp_top: None,
            vessel_temp_bottom: None,
            heater_power: Power::off(),
            pump_on: false,
            target_temperature: None,
            control_source: ControlSource::Average,
            timestamp: 0,
        }
    }
}

/// Brewing process phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrewPhase {
    Idle,
    Prep,
    Mashing,
    Lautering,
    Boiling,
    Cooling,
    Done,
}
