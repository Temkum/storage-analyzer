use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub id: String,
    pub name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub is_up: bool,
}

/// One application's cumulative network usage as observed at a point in time.
///
/// `app_id` is the canonical executable path (the OS-level identity); PID is
/// never used. `process_name` is display metadata best-effort.
/// `executable_path` preserves the resolved path (may be `None` on platforms
/// where it cannot be resolved, though on Linux it equals `app_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationNetworkUsage {
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Combined snapshot: one coherent observation of the system yielding BOTH
/// interface-level and application-level telemetry in a single IPC round-trip.
/// The `applications` array is always present and may be empty.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    pub timestamp: i64,
    pub interfaces: Vec<NetworkInterface>,
    pub applications: Vec<ApplicationNetworkUsage>,
}
