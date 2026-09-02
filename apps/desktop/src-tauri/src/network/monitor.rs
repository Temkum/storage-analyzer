use std::sync::{Arc, RwLock};
use std::time::Duration;

use tauri::Manager;

use super::sidecar::NetworkError;
use super::telemetry::{AppSample, NetworkSample, NetworkSampler, RollupBatch};
use crate::storage::DatabaseState;

/// How often the monitor asks the sidecar for one combined snapshot.
pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

/// Delay before respawning the sidecar after a session failure (crash,
/// spawn failure, protocol death). Keeps a broken environment from
/// busy-looping while still recovering automatically.
pub const RESPAWN_DELAY: Duration = Duration::from_secs(5);

/// An immutable snapshot of live telemetry published by the monitor after
/// every sampling tick. The read API clones the `Arc` — it never touches the
/// sampler or the sidecar while a tick is in flight.
#[derive(Debug, Clone, Default)]
pub struct LiveTelemetry {
    /// Raw 1-second interface samples from the live ring buffer.
    pub interfaces: Vec<NetworkSample>,
    /// Raw 1-second application samples from the live application ring
    /// buffer. Empty when no attributable TCP traffic was observed.
    pub applications: Vec<AppSample>,
    /// Timestamp of the last successful tick; `None` until the first
    /// snapshot lands.
    pub updated_at: Option<i64>,
    /// Last monitoring error, if any. Cleared on the next successful tick.
    pub error: Option<String>,
}

/// Cloneable handle to the monitor's published live state.
#[derive(Clone, Default)]
pub struct MonitorHandle {
    live: Arc<RwLock<Arc<LiveTelemetry>>>,
}

impl MonitorHandle {
    pub fn publish(&self, telemetry: LiveTelemetry) {
        if let Ok(mut guard) = self.live.write() {
            *guard = Arc::new(telemetry);
        }
    }

    /// Returns the most recently published telemetry snapshot.
    pub fn get(&self) -> Arc<LiveTelemetry> {
        self.live
            .read()
            .map(|guard| Arc::clone(&guard))
            .unwrap_or_default()
    }
}

/// Long-running monitor: one long-lived sidecar, one sampler, one 1-second
/// tick. Runs for the lifetime of the desktop app — independent of whether
/// any Network page is currently consuming the data (Phase 0 decision).
///
/// If the sidecar dies or fails to spawn, the session ends and is retried
/// after [`RESPAWN_DELAY`]; rollups already committed stay durably persisted
/// and monitoring resumes where the new sidecar's baseline starts.
pub async fn run_monitor(app: tauri::AppHandle, handle: MonitorHandle) {
    loop {
        if let Err(error) = run_session(&app, &handle).await {
            eprintln!("[network-monitor] session ended: {error}");
            handle.publish(LiveTelemetry {
                error: Some(error),
                ..LiveTelemetry::default()
            });
        }

        tokio::time::sleep(RESPAWN_DELAY).await;
    }
}

async fn run_session(app: &tauri::AppHandle, handle: &MonitorHandle) -> Result<(), String> {
    let mut sidecar = super::sidecar::NetworkSidecar::spawn(app)
        .await
        .map_err(|error| format!("failed to spawn network sidecar: {error}"))?;

    eprintln!("[network-monitor] sidecar session started");

    let mut sampler = NetworkSampler::new();
    let mut ticker = tokio::time::interval(SAMPLE_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        // The persist closure locks the single SQLite connection only when a
        // 60-second boundary completed; the guard is never held across an
        // await because persist_rollups is synchronous inside the block.
        let persist_app = app.clone();
        let mut persist = move |batch: RollupBatch| {
            let app = persist_app.clone();
            async move {
                let database_state = app.state::<DatabaseState>();
                let mut database = database_state
                    .0
                    .lock()
                    .map_err(|_| NetworkError::Io("database lock poisoned".into()))?;

                database
                    .persist_rollups(&batch.interfaces, &batch.applications)
                    .map_err(|error| NetworkError::Io(error.to_string()))
            }
        };

        match sampler
            .sample_once(|| sidecar.snapshot(), &mut persist)
            .await
        {
            Ok(_produced) => {
                handle.publish(LiveTelemetry {
                    interfaces: sampler.samples().snapshot(),
                    applications: sampler.app_samples().snapshot(),
                    updated_at: Some(now_secs()),
                    error: None,
                });
            }
            Err(NetworkError::ProcessExited) => {
                return Err("network sidecar exited unexpectedly".to_string());
            }
            Err(error) => {
                // Transient error (bad line, IO hiccup, persist failure):
                // publish it and keep the session alive; the sampler keeps
                // its baseline so no data is fabricated.
                eprintln!("[network-monitor] tick error: {error}");
                handle.publish(LiveTelemetry {
                    interfaces: sampler.samples().snapshot(),
                    applications: sampler.app_samples().snapshot(),
                    updated_at: Some(now_secs()),
                    error: Some(error.to_string()),
                });
            }
        }
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_secs() as i64
}
