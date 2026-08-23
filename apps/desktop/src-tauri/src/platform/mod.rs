//! Platform-specific integrations for the desktop shell.
//!
//! Each supported OS gets a module exposing OS-specific operations (file
//! manager reveal, etc.). Only the module matching the build target is
//! compiled; the dispatcher below routes to it.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// Reveals a path in the operating system's file manager, highlighting it
/// where the desktop environment supports selection.
pub fn reveal_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        return linux::reveal(path);
    }

    #[cfg(target_os = "windows")]
    {
        return windows::reveal(path);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::reveal(path);
    }
}
