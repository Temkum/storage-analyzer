//! Linux desktop-environment reveal support.

use std::path::Path;
use std::process::Command;

/// Best-effort reveal for Linux: tries the desktop environment's file manager
/// in "select" mode so the item is highlighted, then falls back to opening
/// the containing folder.
pub fn reveal(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Path has no parent directory.".to_string())?;

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .to_lowercase();

    let launched = if desktop.contains("kde") {
        spawn_detached(Command::new("dolphin").arg("--select").arg(path))
    } else if desktop.contains("gnome") || desktop.contains("ubuntu") {
        spawn_detached(Command::new("nautilus").arg("--select").arg(path))
    } else if desktop.contains("xfce") {
        spawn_detached(Command::new("thunar").arg(parent))
    } else {
        spawn_detached(Command::new("xdg-open").arg(parent))
    };

    if launched {
        return Ok(());
    }

    if spawn_detached(Command::new("xdg-open").arg(parent)) {
        Ok(())
    } else {
        Err(format!(
            "Could not launch a file manager for: {}",
            path.display()
        ))
    }
}

/// Spawn a one-shot OS command detached from this process.
fn spawn_detached(command: &mut Command) -> bool {
    command.spawn().is_ok()
}
