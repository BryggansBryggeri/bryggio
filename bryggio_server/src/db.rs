//! SQLite persistence via sqlx.
//!
//! Buffers readings in memory and flushes in batched transactions.
//! Automatically creates/ends brews on phase transitions.

use bryggio_core::state::{BrewPhase, BreweryState};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;
use tokio::sync::mpsc;

const FLUSH_INTERVAL_SECS: u64 = 5;

struct Reading {
    brew_id: i64,
    timestamp: i64,
    source: &'static str,
    value: f64,
}

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn connect(path: &str) -> Result<Self, sqlx::Error> {
        let url = format!("sqlite:{path}?mode=rwc");
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS brews (
                id          INTEGER PRIMARY KEY,
                name        TEXT NOT NULL,
                recipe      TEXT,
                started_at  INTEGER NOT NULL,
                ended_at    INTEGER,
                notes       TEXT
            )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS readings (
                brew_id     INTEGER NOT NULL REFERENCES brews(id),
                timestamp   INTEGER NOT NULL,
                source      TEXT NOT NULL,
                value       REAL NOT NULL,
                PRIMARY KEY (brew_id, timestamp, source)
            )",
        )
        .execute(&pool)
        .await?;

        tracing::info!(%path, "Database initialized");
        Ok(Self { pool })
    }

    async fn create_brew(&self, started_at: i64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO brews (name, started_at) VALUES (?, ?)")
            .bind(format!("Brew {started_at}"))
            .bind(started_at)
            .execute(&self.pool)
            .await?;
        Ok(result.last_insert_rowid())
    }

    async fn end_brew(&self, brew_id: i64, ended_at: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE brews SET ended_at = ? WHERE id = ?")
            .bind(ended_at)
            .bind(brew_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn flush_readings(&self, readings: &[Reading]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        for r in readings {
            sqlx::query(
                "INSERT OR IGNORE INTO readings (brew_id, timestamp, source, value) VALUES (?, ?, ?, ?)",
            )
            .bind(r.brew_id)
            .bind(r.timestamp)
            .bind(r.source)
            .bind(r.value)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

fn is_brewing(phase: BrewPhase) -> bool {
    !matches!(phase, BrewPhase::Idle | BrewPhase::Done)
}

fn readings_from_state(brew_id: i64, state: &BreweryState) -> Vec<Reading> {
    let ts = i64::try_from(state.timestamp).unwrap_or(0);
    let mut readings = Vec::with_capacity(4);

    if let Some(temp) = state.vessel_temp_top {
        readings.push(Reading {
            brew_id,
            timestamp: ts,
            source: "vessel_temp_top",
            value: f64::from(temp.0),
        });
    }
    if let Some(temp) = state.vessel_temp_bottom {
        readings.push(Reading {
            brew_id,
            timestamp: ts,
            source: "vessel_temp_bottom",
            value: f64::from(temp.0),
        });
    }
    readings.push(Reading {
        brew_id,
        timestamp: ts,
        source: "heater_power",
        value: f64::from(state.heater_power.value()),
    });
    readings.push(Reading {
        brew_id,
        timestamp: ts,
        source: "pump_on",
        value: if state.pump_on { 1.0 } else { 0.0 },
    });

    readings
}

/// Receives brewery state snapshots and persists them to SQLite.
///
/// Creates a new brew row when phase transitions from Idle/Done to an active phase.
/// Ends the brew when phase returns to Idle or Done.
/// Buffers readings and flushes every [`FLUSH_INTERVAL_SECS`] seconds.
pub async fn run_db_writer(db: Db, mut rx: mpsc::Receiver<BreweryState>) {
    let mut buffer: Vec<Reading> = Vec::new();
    let mut current_brew_id: Option<i64> = None;
    let mut last_brewing = false;
    let mut flush_interval = tokio::time::interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
    flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            msg = rx.recv() => {
                let Some(state) = msg else { break };
                let brewing = is_brewing(state.phase);

                // Start of a new brew
                if brewing && !last_brewing {
                    let ts = i64::try_from(state.timestamp).unwrap_or(0);
                    match db.create_brew(ts).await {
                        Ok(id) => {
                            tracing::info!(brew_id = id, "New brew started");
                            current_brew_id = Some(id);
                        }
                        Err(e) => tracing::error!(?e, "Failed to create brew"),
                    }
                }

                // End of a brew
                if !brewing && last_brewing {
                    flush(&db, &mut buffer).await;
                    if let Some(id) = current_brew_id.take() {
                        let ts = i64::try_from(state.timestamp).unwrap_or(0);
                        if let Err(e) = db.end_brew(id, ts).await {
                            tracing::error!(?e, "Failed to end brew");
                        }
                        tracing::info!(brew_id = id, "Brew ended");
                    }
                }

                last_brewing = brewing;

                if let Some(brew_id) = current_brew_id {
                    buffer.extend(readings_from_state(brew_id, &state));
                }
            }
            _ = flush_interval.tick() => {
                flush(&db, &mut buffer).await;
            }
        }
    }

    // Channel closed — flush remaining
    flush(&db, &mut buffer).await;
}

async fn flush(db: &Db, buffer: &mut Vec<Reading>) {
    if buffer.is_empty() {
        return;
    }
    match db.flush_readings(buffer).await {
        Ok(()) => {
            tracing::debug!(count = buffer.len(), "Flushed readings");
            buffer.clear();
        }
        Err(e) => tracing::error!(?e, "Failed to flush readings"),
    }
}
