//! The sync core tick function.
//!
//! Pure function: takes state + inputs, returns new state + outputs.
use crate::command::Command;
use crate::control::Controller;
use crate::hal::ActorOutputs;
use crate::state::BreweryState;
use crate::types::Power;

/// Process one tick of the control loop.
///
/// This is the heart of bryggio_core: a pure function with no I/O.
/// `now` is the current unix epoch in seconds, provided by the caller.
///
/// The caller is responsible for stamping fresh sensor readings into
/// `state` before calling this function.
pub fn tick(
    state: &BreweryState,
    commands: &[Command],
    now: u64,
    controller: &mut Controller,
) -> (BreweryState, ActorOutputs) {
    let mut new_state = state.clone();
    let dt = if state.timestamp > 0 {
        (now.saturating_sub(state.timestamp)) as f32
    } else {
        0.0
    };
    new_state.timestamp = now;

    // Apply commands
    for cmd in commands {
        match cmd {
            Command::SetTarget { temperature } => {
                controller.set_target(temperature.0);
                new_state.target_temperature = Some(*temperature);
            }
            Command::SetPhase(phase) => {
                new_state.phase = *phase;
            }
            Command::SetPump(on) => {
                new_state.pump_on = *on;
            }
            Command::SetControlSource(source) => {
                new_state.control_source = *source;
            }
            Command::SetController(_) | Command::SetControlConfig(_) => {
                // Controller swap is handled by the async layer which owns the Controller.
            }
        }
    }

    // Run controller to compute heater power
    let measurement = new_state.control_temp();
    let signal = controller.calculate_signal(measurement.map(|t| t.0), dt);
    new_state.heater_power = Power::new(signal);

    let outputs = ActorOutputs {
        heater_power: new_state.heater_power,
        pump_on: new_state.pump_on,
    };

    (new_state, outputs)
}
