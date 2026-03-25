//! HTTP API — SSE broadcast and command endpoints.
pub mod commands;
pub mod sse;

use bryggio_core::command::Command;
use bryggio_core::state::BreweryState;
use tokio::sync::{mpsc, watch};

/// Shared application state for all axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub command_tx: mpsc::Sender<Command>,
    pub state_rx: watch::Receiver<BreweryState>,
}
