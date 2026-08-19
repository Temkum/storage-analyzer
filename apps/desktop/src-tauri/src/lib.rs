use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn scan_directory(
    app: tauri::AppHandle,
    path: String,
) -> Result<String, String> {
    let sidecar = app
        .shell()
        .sidecar("system-analyzer")
        .map_err(|error| format!("Failed to locate C++ engine: {error}"))?;

    let output = sidecar
        .args([path])
        .output()
        .await
        .map_err(|error| format!("Failed to execute C++ engine: {error}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        return Err(if error.is_empty() {
            format!(
                "C++ engine exited with status: {:?}",
                output.status
            )
        } else {
            error.to_string()
        });
    }

    String::from_utf8(output.stdout)
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