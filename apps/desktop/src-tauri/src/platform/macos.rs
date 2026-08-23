//! macOS Finder reveal support (`open -R <path>`).

use std::path::Path;
use std::process::Command;

pub fn reveal(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open Finder for: {} ({error})", path.display()))
}
