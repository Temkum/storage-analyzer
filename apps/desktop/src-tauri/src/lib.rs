mod platform;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{Emitter, State};
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

fn sidecar_command(app: &tauri::AppHandle, path: &str) -> Result<tauri_plugin_shell::process::Command, String> {
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
    state
        .cancel_requested
        .store(false, Ordering::SeqCst);

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
                was_cancelled =
                    state.cancel_requested.load(Ordering::SeqCst);

                if !was_cancelled && payload.code != Some(0) {
                    return Err(format!(
                        "C++ engine exited with status: {:?}",
                        payload.code
                    ));
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

    String::from_utf8(stdout)
        .map_err(|error| format!("C++ engine returned invalid UTF-8: {error}"))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(CurrentScan::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            cancel_scan,
            reveal_in_file_manager
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}