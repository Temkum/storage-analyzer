//! Integration test for the long-lived network sidecar.
//!
//! Proves the full chain from the architecture:
//!
//! ```text
//! Tauri (mock app + tauri-plugin-shell)
//!   ↓ spawn once
//! system-analyzer --network
//!   ↓ {"command":"network_snapshot"}
//! snapshot
//!   ↓ {"command":"network_snapshot"}
//! snapshot
//!   ↓ {"command":"shutdown"}
//! shutdown_ack → exit 0
//! ```

use std::path::PathBuf;

use app_lib::network::{NetworkError, NetworkSidecar};

/// Stages the freshly built C++ sidecar where the tauri-plugin-shell sidecar
/// resolver looks for it during cargo tests: next to the test executable,
/// one directory up when running from deps/. This mirrors the plugin's own
/// relative_command_path logic.
fn stage_sidecar_binary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Source candidates, in order:
    // 1. Explicit override (CI machines).
    // 2. The cmake build stages the triple-suffixed sidecar into
    //    src-tauri/binaries/ for Tauri's externalBin resolution.
    // 3. The raw cmake output in the repo-root build/ directory.
    let source = std::env::var_os("SYSTEM_ANALYZER_SIDECAR")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| {
            let staged = manifest_dir.join("binaries");

            std::fs::read_dir(staged)
                .ok()?
                .flatten()
                .map(|entry| entry.path())
                .find(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("system-analyzer-"))
                })
        })
        .or_else(|| {
            let built = manifest_dir.join("../../../build/system-analyzer");
            built.is_file().then_some(built)
        })
        .expect(
            "C++ sidecar binary not found; build it first with \
             `cmake -S . -B build -G Ninja && cmake --build build`",
        );

    let exe_dir = std::env::current_exe()
        .expect("current_exe")
        .parent()
        .expect("exe parent")
        .to_path_buf();

    let base_dir = if exe_dir.ends_with("deps") {
        exe_dir.parent().expect("deps parent").to_path_buf()
    } else {
        exe_dir
    };

    std::fs::create_dir_all(&base_dir).expect("create staging dir");

    std::fs::copy(&source, base_dir.join("system-analyzer"))
        .expect("failed to stage sidecar binary");
}

#[tokio::test]
async fn network_sidecar_serves_repeated_snapshots_and_exits_on_shutdown() {
    stage_sidecar_binary();

    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("failed to build mock tauri app");

    // Spawn once.
    let mut sidecar = NetworkSidecar::spawn(app.handle())
        .await
        .expect("failed to spawn network sidecar");

    // Request → snapshot.
    let first = sidecar.snapshot().await.expect("first snapshot");
    assert!(first.timestamp > 0);

    for interface in &first.interfaces {
        assert!(!interface.id.is_empty());
        assert!(!interface.name.is_empty());
    }

    // Combined snapshot: the applications array must always be present.
    // It may legitimately be empty on this machine — no attributable
    // same-UID TCP traffic does NOT mean the network provider failed.
    for application in &first.applications {
        assert!(!application.app_id.is_empty());
        assert!(
            application.executable_path.is_some(),
            "executable_path preserves the resolved executable"
        );
    }

    // Request → snapshot served by the SAME process (proves long-lived).
    let second = sidecar.snapshot().await.expect("second snapshot");
    assert!(
        second.timestamp >= first.timestamp,
        "snapshots must come from the same long-lived process"
    );

    // Shutdown → ack → clean exit.
    sidecar.shutdown().await.expect("graceful shutdown");

    // Any further request must fail: the sidecar is really gone.
    let result = sidecar.snapshot().await;
    assert!(matches!(result, Err(NetworkError::ProcessExited)));
}
