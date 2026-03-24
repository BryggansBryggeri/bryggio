//! The async tick loop — bridges the HAL and the sync core.
use bryggio_core::command::Command;
use bryggio_core::hal::Hal;
use bryggio_core::state::BreweryState;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, watch};

/// Run the main control loop.
///
/// Reads sensors via the HAL, drains commands, calls the sync core tick,
/// applies outputs, and broadcasts state.
pub async fn run_tick_loop<H: Hal>(
    hal: &H,
    mut command_rx: mpsc::Receiver<Command>,
    state_tx: watch::Sender<BreweryState>,
) {
    let mut state = BreweryState::default();
    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    tracing::info!("Tick loop started");

    loop {
        interval.tick().await;

        // Read sensors
        let readings = hal.read_sensors().await;

        // Drain pending commands
        let mut commands = Vec::new();
        while let Ok(cmd) = command_rx.try_recv() {
            commands.push(cmd);
        }

        // Sync core tick
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let (new_state, outputs) = bryggio_core::tick::tick(&state, &readings, &commands, now);
        state = new_state;

        // Apply actor outputs to hardware
        if let Err(e) = hal.apply_outputs(&outputs).await {
            tracing::error!(?e, "Failed to apply actor outputs");
        }

        // Broadcast state to SSE subscribers
        let _ = state_tx.send(state.clone());
    }
}
