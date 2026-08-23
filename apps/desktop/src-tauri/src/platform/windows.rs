//! Windows Explorer reveal support (`explorer /select,<path>`).

use std::path::Path;
use std::process::Command;

pub fn reveal(path: &Path) -> Result<(), String> {
    // Explorer highlights the item via /select but reports exit code 1 even
    // on success, so treat a successful launch as success (fire-and-forget).
    Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open Explorer for: {} ({error})", path.display()))
}
