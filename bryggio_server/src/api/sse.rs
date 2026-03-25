//! Server-Sent Events endpoint for broadcasting brewery state.
use axum::extract::State;
use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;

use super::AppState;

/// SSE handler — streams the full brewery state on every change.
pub async fn state_stream(
    State(app): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = WatchStream::new(app.state_rx).map(|state| {
        let json = serde_json::to_string(&state).unwrap_or_default();
        Ok(Event::default().data(json))
    });
    Sse::new(stream)
}
