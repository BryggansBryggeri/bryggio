//! POST command endpoints.
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use bryggio_core::command::Command;

use super::AppState;

/// POST /command — accept a command from the UI.
pub async fn handle_command(
    State(app): State<AppState>,
    Json(command): Json<Command>,
) -> StatusCode {
    tracing::info!(?command, "Received command");
    match app.command_tx.send(command).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
