//! The sync core tick function.
//!
//! Pure function: takes state + inputs, returns new state + outputs.
use crate::command::Command;
use crate::hal::ActorOutputs;
use crate::sensor::SensorReadings;
use crate::state::BreweryState;

/// Process one tick of the control loop.
///
/// This is the heart of bryggio_core: a pure function with no I/O.
/// `now` is the current unix epoch in seconds, provided by the caller.
pub fn tick(
    state: &BreweryState,
    readings: &SensorReadings,
    commands: &[Command],
    now: u64,
) -> (BreweryState, ActorOutputs) {
    let mut new_state = state.clone();
    new_state.timestamp = now;

    // Apply commands
    for cmd in commands {
        match cmd {
            Command::SetTarget { temperature } => {
                // TODO: feed into active controller
                let _ = temperature;
            }
            Command::SetPhase(phase) => {
                new_state.phase = *phase;
            }
            Command::SetPump(on) => {
                new_state.pump_on = *on;
            }
        }
    }

    // Update state with latest sensor readings
    new_state.vessel_temp_top = readings.vessel_temp_top;
    new_state.vessel_temp_bottom = readings.vessel_temp_bottom;

    // TODO: run active controller to compute heater power
    let outputs = ActorOutputs {
        heater_power: new_state.heater_power,
        pump_on: new_state.pump_on,
    };

    (new_state, outputs)
}
