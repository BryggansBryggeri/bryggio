//! GET /readings — return historical snapshots for the latest brew.
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use bryggio_core::control::ControlSource;
use bryggio_core::state::{BrewPhase, BreweryState};
use bryggio_core::types::{Power, Temperature};
use sqlx::Row;

use super::AppState;

fn f64_to_f32(v: f64) -> f32 {
    #[allow(clippy::as_conversions)]
    let r = v as f32;
    r
}

fn row_to_state(row: &sqlx::sqlite::SqliteRow) -> Option<BreweryState> {
    let ts: i64 = row.try_get("timestamp").ok()?;
    let phase_str: &str = row.try_get("phase").ok()?;
    let phase: BrewPhase =
        serde_json::from_value(serde_json::Value::String(phase_str.to_owned())).ok()?;
    let top: Option<f64> = row.try_get("vessel_temp_top").ok()?;
    let bottom: Option<f64> = row.try_get("vessel_temp_bottom").ok()?;
    let heater: f64 = row.try_get("heater_power").ok()?;
    let pump: bool = row.try_get("pump_on").ok()?;
    let target: Option<f64> = row.try_get("target_temperature").ok()?;
    let source_str: &str = row.try_get("control_source").ok()?;
    let control_source: ControlSource =
        serde_json::from_value(serde_json::Value::String(source_str.to_owned())).ok()?;

    #[allow(clippy::as_conversions)]
    let timestamp = ts as u64;

    Some(BreweryState {
        phase,
        vessel_temp_top: top.map(|v| Temperature(f64_to_f32(v))),
        vessel_temp_bottom: bottom.map(|v| Temperature(f64_to_f32(v))),
        heater_power: Power::new(f64_to_f32(heater)),
        pump_on: pump,
        target_temperature: target.map(|v| Temperature(f64_to_f32(v))),
        control_source,
        timestamp,
    })
}

/// Subsample resolution tiers: (age threshold in seconds, sample interval in seconds).
/// Recent data is kept at full resolution, older data is progressively thinned.
const TIERS: &[(u64, u64)] = &[
    (600, 1),       // last 10 min: every second
    (3600, 10),     // 10 min – 1 hr: every 10s
    (u64::MAX, 60), // older than 1 hr: every 60s
];

/// Keep a point if its timestamp aligns with the sample interval for its age tier.
fn should_keep(timestamp: u64, latest: u64) -> bool {
    let age = latest.saturating_sub(timestamp);
    for &(threshold, interval) in TIERS {
        if age < threshold {
            return timestamp % interval == 0;
        }
    }
    false
}

/// Returns subsampled `Vec<BreweryState>` for the most recent brew.
///
/// Dense for recent data, progressively sparser for older data.
/// Returns empty if the server is not currently brewing.
pub async fn get_readings(
    State(app): State<AppState>,
) -> Result<Json<Vec<BreweryState>>, StatusCode> {
    let current_phase = app.state_rx.borrow().phase;
    if matches!(current_phase, BrewPhase::Idle | BrewPhase::Done) {
        return Ok(Json(Vec::new()));
    }

    let rows = sqlx::query(
        "SELECT timestamp, phase, vessel_temp_top, vessel_temp_bottom,
                heater_power, pump_on, target_temperature, control_source
         FROM snapshots
         WHERE brew_id = (SELECT id FROM brews ORDER BY started_at DESC LIMIT 1)
         ORDER BY timestamp",
    )
    .fetch_all(&app.db_pool)
    .await
    .map_err(|e| {
        tracing::error!(?e, "Failed to query readings");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let all_states: Vec<BreweryState> = rows.iter().filter_map(row_to_state).collect();

    let latest = all_states.last().map_or(0, |s| s.timestamp);
    let states = all_states
        .into_iter()
        .filter(|s| should_keep(s.timestamp, latest))
        .collect();

    Ok(Json(states))
}
