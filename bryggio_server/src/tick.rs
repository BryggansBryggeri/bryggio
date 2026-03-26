//! The async tick loop — bridges the HAL and the sync core.
use bryggio_core::command::Command;
use bryggio_core::control::{Controller, ControllerType};
use bryggio_core::hal::Hal;
use bryggio_core::state::BreweryState;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};

fn swap_controller(controller: &mut Controller, ct: &ControllerType) {
    match Controller::from_type(ct) {
        Ok(new_controller) => {
            tracing::info!(?ct, "Switching controller");
            *controller = new_controller;
        }
        Err(e) => {
            tracing::error!(?e, "Failed to create controller, keeping current");
        }
    }
}

/// Run the main control loop.
///
/// Reads sensors via the HAL, drains commands, calls the sync core tick,
/// applies outputs, and broadcasts state.
pub async fn run_tick_loop<H: Hal>(
    hal: &H,
    mut command_rx: mpsc::Receiver<Command>,
    state_tx: watch::Sender<BreweryState>,
    db_tx: mpsc::Sender<BreweryState>,
) {
    let mut state = BreweryState::default();
    let mut controller = Controller::from_type(&ControllerType::Pid {
        kp: 0.5,
        ki: 0.01,
        kd: 0.002,
    })
    .unwrap_or_else(|e| {
        tracing::error!(
            ?e,
            "Failed to create default controller, falling back to manual"
        );
        Controller::from_type(&ControllerType::Manual).expect("Manual controller cannot fail")
    });
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    tracing::info!("Tick loop started");

    loop {
        interval.tick().await;

        // Read sensors and stamp into state
        let readings = hal.read_sensors().await;
        state.vessel_temp_top = readings.vessel_temp_top;
        state.vessel_temp_bottom = readings.vessel_temp_bottom;

        // Drain pending commands, handle controller swaps here
        let mut commands = Vec::new();
        while let Ok(cmd) = command_rx.try_recv() {
            match &cmd {
                Command::SetController(ct) => {
                    swap_controller(&mut controller, ct);
                }
                Command::SetControlConfig(config) => {
                    swap_controller(&mut controller, &config.controller);
                }
                _ => {}
            }
            commands.push(cmd);
        }

        // Sync core tick
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (new_state, outputs) =
            bryggio_core::tick::tick(&state, &commands, now, &mut controller);
        state = new_state;

        // Apply actor outputs to hardware
        if let Err(e) = hal.apply_outputs(&outputs).await {
            tracing::error!(?e, "Failed to apply actor outputs");
        }

        // Broadcast state to SSE subscribers
        let _ = state_tx.send(state.clone());

        // Send to DB writer (non-blocking — drop if channel full)
        if let Err(e) = db_tx.try_send(state.clone()) {
            tracing::warn!("DB channel full, dropping reading: {e}");
        }
    }
}
