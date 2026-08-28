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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    pub timestamp: i64,
    pub interfaces: Vec<NetworkInterface>,
}
