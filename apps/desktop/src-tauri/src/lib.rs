use tauri::Emitter;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn scan_directory(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, String> {
    let sidecar = app
        .shell()
        .sidecar("system-analyzer")
        .map_err(|error| format!("Failed to locate C++ engine: {error}"))?
        .args([path]);

    let (mut receiver, _child) = sidecar
        .spawn()
        .map_err(|error| format!("Failed to start C++ engine: {error}"))?;

    let mut stdout = Vec::new();

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
                if payload.code != Some(0) {
                    return Err(format!(
                        "C++ engine exited with status: {:?}",
                        payload.code
                    ));
                }
            }

            _ => {}
        }
    }

    String::from_utf8(stdout)
        .map_err(|error| format!("C++ engine returned invalid UTF-8: {error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}