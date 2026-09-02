mod platform;
mod storage;

// Network monitoring: Rust-side wire types and the long-lived sidecar
// manager. Public so the integration test can exercise the full
// spawn → snapshot → snapshot → shutdown → exit round-trip.
pub mod network;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager, State};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

/// Shared state for the currently running scan so it can be cancelled.
#[derive(Default)]
struct CurrentScan {
    /// Handle to the running C++ sidecar, if any.
    child: Mutex<Option<CommandChild>>,
    /// Set when the user asks to cancel the in-flight scan.
    cancel_requested: Arc<AtomicBool>,
}

/// Tauri-managed database state lives in `storage::DatabaseState` so the
/// network monitor task and the commands share the same single writer.
use storage::DatabaseState;

fn sidecar_command(
    app: &tauri::AppHandle,
    path: &str,
) -> Result<tauri_plugin_shell::process::Command, String> {
    Ok(app
        .shell()
        .sidecar("system-analyzer")
        .map_err(|error| format!("Failed to locate C++ engine: {error}"))?
        .args([path]))
}

#[tauri::command]
async fn scan_directory(
    app: tauri::AppHandle,
    state: State<'_, CurrentScan>,
    path: String,
) -> Result<String, String> {
    state.cancel_requested.store(false, Ordering::SeqCst);

    let mut receiver = {
        let sidecar = sidecar_command(&app, &path)?;
        let (receiver, child) = sidecar
            .spawn()
            .map_err(|error| format!("Failed to start C++ engine: {error}"))?;

        // If a previous scan is somehow still attached, terminate it first.
        {
            let mut guard = state
                .child
                .lock()
                .map_err(|_| "Scan state lock poisoned".to_string())?;

            if let Some(previous) = guard.take() {
                let _ = previous.kill();
            }

            *guard = Some(child);
        }

        receiver
    };

    let mut stdout = Vec::new();
    let mut was_cancelled = false;

    while let Some(event) = receiver.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                stdout.extend_from_slice(&bytes);
            }

            CommandEvent::Stderr(bytes) => {
                let message = String::from_utf8_lossy(&bytes);

                for line in message.lines() {
                    if let Some(value) = line.strip_prefix("PROGRESS:") {
                        if let Ok(scanned) = value.trim().parse::<u64>() {
                            let _ = app.emit("scan-progress", scanned);
                        }
                    }
                }
            }

            CommandEvent::Error(error) => {
                return Err(format!("C++ engine error: {error}"));
            }

            CommandEvent::Terminated(payload) => {
                was_cancelled = state.cancel_requested.load(Ordering::SeqCst);

                if !was_cancelled && payload.code != Some(0) {
                    return Err(format!("C++ engine exited with status: {:?}", payload.code));
                }
            }

            _ => {}
        }
    }

    {
        let mut guard = state
            .child
            .lock()
            .map_err(|_| "Scan state lock poisoned".to_string())?;
        *guard = None;
    }

    if was_cancelled {
        return Err("SCAN_CANCELLED".to_string());
    }

    String::from_utf8(stdout).map_err(|error| format!("C++ engine returned invalid UTF-8: {error}"))
}

/* Requests cancellation of the currently running scan. The running sidecar
process is killed; the `scan_directory` invocation reports the distinct
`SCAN_CANCELLED` outcome so the UI can treat it as a cancellation rather
than an error. */
#[tauri::command]
fn cancel_scan(state: State<'_, CurrentScan>) -> Result<(), String> {
    state.cancel_requested.store(true, Ordering::SeqCst);

    let mut guard = state
        .child
        .lock()
        .map_err(|_| "Scan state lock poisoned".to_string())?;

    // `CommandChild::kill` consumes the handle, so take it out of the state.
    if let Some(child) = guard.take() {
        child
            .kill()
            .map_err(|error| format!("Failed to terminate scan: {error}"))?;
    }

    Ok(())
}

/// Best-effort "reveal in file manager" for a scanned file/directory.
///
/// Delegates to the platform integration layer; returns an error when the
/// path no longer exists on disk (the filesystem may have changed since the
/// scan) or when no file manager could be launched.
#[tauri::command]
fn reveal_in_file_manager(path: String) -> Result<(), String> {
    let target = std::path::PathBuf::from(&path);

    if !target.exists() {
        return Err(format!("File no longer exists on disk: {path}"));
    }

    platform::reveal_in_file_manager(&target)
}

// ---- Network read API (Phase 6.7) ----------------------------------------

/// Live telemetry from the monitor's published ring-buffer snapshots.
/// Never touches the sidecar: the sampler is already running in the
/// background, so the UI can poll this freely.
#[tauri::command]
fn get_network_live(
    monitor: State<'_, network::MonitorHandle>,
) -> network::readapi::NetworkLiveDto {
    network::readapi::live_dto(&monitor.get())
}

/// Aggregated interface history over `[since, until)` from `network_rollups`.
/// Minute buckets are merged into range-appropriate buckets server-side.
#[tauri::command]
fn get_network_history(
    state: State<'_, DatabaseState>,
    since: i64,
    until: i64,
) -> Result<network::readapi::NetworkHistoryDto, String> {
    if since >= until {
        return Err(format!(
            "invalid range: since ({since}) must be before until ({until})"
        ));
    }

    let mut database_state = state
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    let connection = database_state.connection();

    let bucket_seconds = storage::bucket_seconds_for_range(since, until);

    let totals = storage::query_network_totals(connection, since, until, bucket_seconds)
        .map_err(|error| error.to_string())?;
    let interfaces = storage::query_network_by_interface(connection, since, until, bucket_seconds)
        .map_err(|error| error.to_string())?;

    Ok(network::readapi::network_history_dto(
        since,
        until,
        bucket_seconds,
        totals,
        interfaces,
    ))
}

/// Per-application usage totals over `[since, until)`, ranked by total bytes.
#[tauri::command]
fn get_application_history(
    state: State<'_, DatabaseState>,
    since: i64,
    until: i64,
) -> Result<Vec<network::readapi::ApplicationUsageDto>, String> {
    if since >= until {
        return Err(format!(
            "invalid range: since ({since}) must be before until ({until})"
        ));
    }

    let mut database_state = state
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;

    let usage = storage::query_app_usage(database_state.connection(), since, until)
        .map_err(|error| error.to_string())?;

    Ok(network::readapi::application_usage_dtos(&usage))
}

/// Top-N application ranking over `[since, until)` from the persisted
/// minute deltas — never from PIDs.
#[tauri::command]
fn get_top_applications(
    state: State<'_, DatabaseState>,
    since: i64,
    until: i64,
    limit: Option<usize>,
) -> Result<Vec<network::readapi::ApplicationUsageDto>, String> {
    if since >= until {
        return Err(format!(
            "invalid range: since ({since}) must be before until ({until})"
        ));
    }

    let mut database_state = state
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;

    let top = storage::top_applications(
        database_state.connection(),
        since,
        until,
        limit.unwrap_or(DEFAULT_TOP_APPLICATIONS),
    )
    .map_err(|error| error.to_string())?;

    Ok(network::readapi::application_usage_dtos(&top))
}

const DEFAULT_TOP_APPLICATIONS: usize = 10;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(CurrentScan::default())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            // TEMP diagnostic: trace setup hook execution.
            eprintln!("[setup] app_data_dir: {app_data_dir:?}");

            let mut database = storage::Database::initialize(&app_data_dir)
                .map_err(|error| format!("failed to initialize database: {error}"))?;

            eprintln!("[setup] database initialized");

            let retention = storage::RetentionManager::default();
            let cutoff = retention.cutoff_timestamp();

            {
                let repository = storage::NetworkRollupRepository::new(database.connection());

                let deleted = repository
                    .delete_before(cutoff)
                    .map_err(|error| format!("failed to clean network history: {error}"))?;

                eprintln!("[setup] retention cleanup deleted {deleted} network rollups");
            }

            {
                let repository = storage::AppUsageRollupRepository::new(database.connection());

                let deleted = repository
                    .delete_before(cutoff)
                    .map_err(|error| format!("failed to clean application history: {error}"))?;

                eprintln!("[setup] retention cleanup deleted {deleted} application rollups");
            }

            app.manage(DatabaseState(Mutex::new(database)));

            eprintln!("[setup] database state managed");

            // Network monitor: one long-lived sidecar + sampler ticking every
            // second for the lifetime of the app, independent of whether any
            // Network page is currently open (Phase 0 decision).
            let monitor = network::MonitorHandle::default();
            app.manage(monitor.clone());

            tauri::async_runtime::spawn(network::monitor::run_monitor(
                app.handle().clone(),
                monitor,
            ));

            eprintln!("[setup] network monitor started");

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            cancel_scan,
            reveal_in_file_manager,
            get_network_live,
            get_network_history,
            get_application_history,
            get_top_applications
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
