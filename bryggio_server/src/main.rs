#![forbid(unsafe_code)]
#![warn(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::as_conversions
)]

mod api;
mod db;
mod drivers;
mod tick;

use api::AppState;
use axum::routing::{get, post};
use axum::Router;
use bryggio_core::state::BreweryState;
use drivers::mock::MockHal;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bryggio_server=debug,bryggio_core=debug,sqlx=warn".into()),
        )
        .init();

    tracing::info!("Starting bryggio server");

    // Channels
    let (command_tx, command_rx) = mpsc::channel::<bryggio_core::command::Command>(64);
    let (state_tx, state_rx) = watch::channel(BreweryState::default());

    // HAL
    let hal = Arc::new(MockHal::new());

    // Spawn tick loop
    let tick_hal = hal.clone();
    tokio::spawn(async move {
        tick::run_tick_loop(&*tick_hal, command_rx, state_tx).await;
    });

    // Axum router
    let app_state = AppState {
        command_tx,
        state_rx,
    };

    let app = Router::new()
        .route("/events", get(api::sse::state_stream))
        .route("/command", post(api::commands::handle_command))
        .with_state(app_state);

    let bind_addr = "0.0.0.0:8080";
    tracing::info!("Listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
