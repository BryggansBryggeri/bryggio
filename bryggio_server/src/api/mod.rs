//! HTTP API — SSE broadcast, command, and readings endpoints.
pub mod commands;
pub mod readings;
pub mod sse;

use bryggio_core::command::Command;
use bryggio_core::state::BreweryState;
use sqlx::SqlitePool;
use tokio::sync::{mpsc, watch};

/// Shared application state for all axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub command_tx: mpsc::Sender<Command>,
    pub state_rx: watch::Receiver<BreweryState>,
    pub db_pool: SqlitePool,
}
