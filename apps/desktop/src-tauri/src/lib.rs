use std::process::Command;

#[tauri::command]
fn scan_directory(path: String) -> Result<String, String> {
    let output = Command::new("../../../build/system-analyzer")
        .arg(&path)
        .output()
        .map_err(|error| format!("Failed to start C++ engine: {error}"))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);

        return Err(if error.is_empty() {
            format!("C++ engine exited with status: {}", output.status)
        } else {
            error.to_string()
        });
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("Invalid UTF-8 from C++ engine: {error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan_directory])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
