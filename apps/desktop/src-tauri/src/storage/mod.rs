pub mod app_usage_rollup_repository;
pub mod database;
pub mod history;
pub mod migrations;
pub mod network_rollup_repository;
pub mod retention;

pub use app_usage_rollup_repository::{AppUsageRollup, AppUsageRollupRepository};
pub use database::Database;
pub use history::{
    bucket_seconds_for_range, query_app_usage, query_network_by_interface, query_network_totals,
    top_applications, AppUsageTotals, HistoryPoint, InterfaceHistory,
};
pub use network_rollup_repository::{NetworkRollup, NetworkRollupRepository};
pub use retention::RetentionManager;

use std::sync::Mutex;

/// Tauri-managed database state. Owns the single SQLite connection so the
/// application has exactly one writer, shared behind a mutex. Lives here so
/// both the Tauri commands (lib.rs) and the network monitor can reach the
/// same writer.
pub struct DatabaseState(pub Mutex<Database>);
