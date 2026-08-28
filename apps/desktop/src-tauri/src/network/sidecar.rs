use serde_json::Value;
use tauri::Runtime;
use tauri_plugin_shell::process::{Command, CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;

use super::types::NetworkSnapshot;

/// The sidecar program registered in tauri.conf.json `externalBin`.
const SIDECAR_PROGRAM: &str = "system-analyzer";

const SNAPSHOT_REQUEST: &str = r#"{"command":"network_snapshot"}"#;
const SHUTDOWN_REQUEST: &str = r#"{"command":"shutdown"}"#;

#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("failed to spawn network sidecar: {0}")]
    Spawn(String),

    #[error("failed to communicate with network sidecar: {0}")]
    Io(String),

    #[error("network sidecar returned an error: {0}")]
    Protocol(String),

    #[error("invalid network snapshot: {0}")]
    Serialization(String),

    #[error("network sidecar exited unexpectedly")]
    ProcessExited,
}

/// Manager for the long-lived C++ network sidecar.
///
/// Spawns `system-analyzer --network` once, keeps its stdin/stdout handles,
/// and drives the strictly sequential NDJSON request/response protocol
/// (snapshot, shutdown) over the same tauri-plugin-shell mechanism the disk
/// scanner uses — no second process-launching mechanism.
pub struct NetworkSidecar {
    child: Option<CommandChild>,
    receiver: tauri::async_runtime::Receiver<CommandEvent>,
    terminated: bool,
}

impl NetworkSidecar {
    /// Spawns the sidecar through the plugin's sidecar resolution (the
    /// `externalBin` entry in tauri.conf.json), exactly like the disk scan.
    pub async fn spawn<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<Self, NetworkError> {
        let command = app
            .shell()
            .sidecar(SIDECAR_PROGRAM)
            .map_err(|error| NetworkError::Spawn(error.to_string()))?
            .args(["--network"]);

        Self::spawn_with(command).await
    }

    async fn spawn_with(command: Command) -> Result<Self, NetworkError> {
        let (receiver, child) = command
            .spawn()
            .map_err(|error| NetworkError::Spawn(error.to_string()))?;

        Ok(Self {
            child: Some(child),
            receiver,
            terminated: false,
        })
    }

    /// Requests one snapshot. Requests are strictly sequential (no request
    /// IDs): send a line, then read exactly one response line.
    pub async fn snapshot(&mut self) -> Result<NetworkSnapshot, NetworkError> {
        let response = self.request(SNAPSHOT_REQUEST).await?;

        match response.get("type").and_then(Value::as_str) {
            Some("network_snapshot") => serde_json::from_value(response)
                .map_err(|error| NetworkError::Serialization(error.to_string())),
            Some("error") => Err(NetworkError::Protocol(error_message(&response))),
            _ => Err(NetworkError::Protocol(format!(
                "unexpected response: {response}"
            ))),
        }
    }

    /// Graceful shutdown: the C++ loop acknowledges, exits by itself and the
    /// handle waits for the process to terminate with a zero status.
    pub async fn shutdown(&mut self) -> Result<(), NetworkError> {
        if self.terminated {
            return Ok(());
        }

        let response = self.request(SHUTDOWN_REQUEST).await?;

        match response.get("type").and_then(Value::as_str) {
            Some("shutdown_ack") => {}
            Some("error") => return Err(NetworkError::Protocol(error_message(&response))),
            _ => {
                return Err(NetworkError::Protocol(format!(
                    "unexpected response: {response}"
                )))
            }
        }

        let code = self.wait_for_exit().await;

        match code {
            Some(0) | None => {
                // Process is gone; drop the dead handle so Drop won't try to
                // kill it and further requests fail with ProcessExited.
                self.child = None;
                Ok(())
            }
            Some(status) => Err(NetworkError::Io(format!(
                "network sidecar exited with status {status}"
            ))),
        }
    }

    async fn request(&mut self, request: &str) -> Result<Value, NetworkError> {
        self.send(request).await?;
        self.read_response().await
    }

    async fn send(&mut self, request: &str) -> Result<(), NetworkError> {
        let child = self.child.as_mut().ok_or(NetworkError::ProcessExited)?;

        child
            .write(format!("{request}\n").as_bytes())
            .map_err(|error| NetworkError::Io(error.to_string()))
    }

    /// Reads the next NDJSON response line. The shell plugin splits stdout
    /// into lines (raw_out defaults to false), so every Stdout event carries
    /// exactly one response.
    async fn read_response(&mut self) -> Result<Value, NetworkError> {
        loop {
            match self.receiver.recv().await {
                Some(CommandEvent::Stdout(bytes)) => {
                    let line = String::from_utf8(bytes)
                        .map_err(|error| NetworkError::Protocol(error.to_string()))?;

                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    return serde_json::from_str(line).map_err(|error| {
                        NetworkError::Protocol(format!("invalid JSON response: {error}"))
                    });
                }

                // Diagnostics from the C++ process are not part of the
                // request/response stream.
                Some(CommandEvent::Stderr(_)) => {}

                Some(CommandEvent::Error(error)) => return Err(NetworkError::Io(error)),

                Some(CommandEvent::Terminated(payload)) => {
                    self.terminated = true;
                    let _ = payload.code;
                    return Err(NetworkError::ProcessExited);
                }

                None => {
                    self.terminated = true;
                    return Err(NetworkError::ProcessExited);
                }

                // The event enum is #[non_exhaustive]; ignore future kinds.
                Some(_) => {}
            }
        }
    }

    /// Drains events until the process terminates; returns its exit code.
    async fn wait_for_exit(&mut self) -> Option<i32> {
        loop {
            match self.receiver.recv().await {
                Some(CommandEvent::Terminated(payload)) => {
                    self.terminated = true;
                    return payload.code;
                }
                Some(_) => {}
                None => {
                    self.terminated = true;
                    return None;
                }
            }
        }
    }
}

impl Drop for NetworkSidecar {
    fn drop(&mut self) {
        // Best-effort cleanup: a sidecar dropped without a graceful shutdown
        // must not leak as a stray process.
        if let Some(child) = self.child.take() {
            let _ = child.kill();
        }
    }
}

fn error_message(response: &Value) -> String {
    response
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unspecified sidecar error")
        .to_string()
}
