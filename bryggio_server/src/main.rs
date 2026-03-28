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
use axum::Router;
use axum::routing::{get, post};
use bryggio_core::model::BrewerySimulation;
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

    // Database
    let db_path = std::env::var("BRYGGIO_DB").unwrap_or_else(|_| "bryggio.db".into());
    let db = db::Db::connect(&db_path)
        .await
        .expect("failed to connect to database");
    let db_pool = db.pool.clone();
    let (db_tx, db_rx) = mpsc::channel::<BreweryState>(128);
    tokio::spawn(async move {
        db::run_db_writer(db, db_rx).await;
    });

    // Channels
    let (command_tx, command_rx) = mpsc::channel::<bryggio_core::command::Command>(64);
    let (state_tx, state_rx) = watch::channel(BreweryState::default());

    // HAL
    let time_scale: u32 = std::env::var("BRYGGIO_TIME_SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    if time_scale > 1 {
        tracing::info!(time_scale, "Running mock HAL with accelerated time");
    }
    let hal = Arc::new(MockHal::new(BrewerySimulation::new(), time_scale));

    // Spawn tick loop
    let tick_hal = hal.clone();
    tokio::spawn(async move {
        tick::run_tick_loop(&*tick_hal, command_rx, state_tx, db_tx).await;
    });

    // Axum router
    let app_state = AppState {
        command_tx,
        state_rx,
        db_pool,
    };

    let app = Router::new()
        .route("/events", get(api::sse::state_stream))
        .route("/command", post(api::commands::handle_command))
        .route("/readings", get(api::readings::get_readings))
        .with_state(app_state);

    let bind_addr = "0.0.0.0:8080";
    tracing::info!("Listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind");

    axum::serve(listener, app).await.expect("server error");
}
