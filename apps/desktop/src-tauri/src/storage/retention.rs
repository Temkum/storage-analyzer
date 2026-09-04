use std::time::{SystemTime, UNIX_EPOCH};

use super::{AppUsageRollupRepository, NetworkRollupRepository};

pub const DEFAULT_RETENTION_DAYS: i64 = 30;

/// Deletes network rollups once they are older than the retention window.
pub struct RetentionManager {
    retention_days: i64,
}

impl RetentionManager {
    pub fn new(retention_days: i64) -> Self {
        assert!(retention_days > 0, "retention period must be positive");

        Self { retention_days }
    }

    pub fn cutoff_timestamp(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_secs() as i64;

        now - (self.retention_days * 24 * 60 * 60)
    }

    #[allow(dead_code)]
    pub fn cleanup(&self, repository: &NetworkRollupRepository<'_>) -> rusqlite::Result<usize> {
        repository.delete_before(self.cutoff_timestamp())
    }

    #[allow(dead_code)]
    pub fn cleanup_app(
        &self,
        repository: &AppUsageRollupRepository<'_>,
    ) -> rusqlite::Result<usize> {
        repository.delete_before(self.cutoff_timestamp())
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new(DEFAULT_RETENTION_DAYS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppUsageRollup, AppUsageRollupRepository, Database, NetworkRollup};

    fn now_seconds() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    #[test]
    fn default_retention_is_30_days() {
        let retention = RetentionManager::default();

        let expected = now_seconds() - (30 * 24 * 60 * 60);
        let cutoff = retention.cutoff_timestamp();

        assert!((cutoff - expected).abs() <= 1);
    }

    #[test]
    fn custom_retention_period_is_respected() {
        let retention = RetentionManager::new(7);

        let expected = now_seconds() - (7 * 24 * 60 * 60);

        assert!((retention.cutoff_timestamp() - expected).abs() <= 1);
    }

    #[test]
    fn cleanup_deletes_only_expired_rollups() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut database = Database::open(temp_dir.path()).unwrap();
        let mut repository = NetworkRollupRepository::new(database.connection());

        let now = now_seconds();

        repository
            .insert_batch(&[
                NetworkRollup {
                    ts: now - 31 * 24 * 60 * 60,
                    interface_id: "eth0".into(),
                    bytes_received: 1000,
                    bytes_sent: 500,
                },
                NetworkRollup {
                    ts: now - 60,
                    interface_id: "eth0".into(),
                    bytes_received: 2000,
                    bytes_sent: 900,
                },
            ])
            .unwrap();

        let deleted = RetentionManager::new(30).cleanup(&repository).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(repository.count().unwrap(), 1);
    }

    #[test]
    fn cleanup_app_deletes_only_expired_rollups() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut database = Database::open(temp_dir.path()).unwrap();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        let now = now_seconds();

        repository
            .insert_batch(&[
                AppUsageRollup {
                    ts: now - 31 * 24 * 60 * 60,
                    app_id: "/usr/bin/app1".into(),
                    process_name: "app1".into(),
                    executable_path: Some("/usr/bin/app1".into()),
                    bytes_received: 1000,
                    bytes_sent: 500,
                },
                AppUsageRollup {
                    ts: now - 60,
                    app_id: "/usr/bin/app1".into(),
                    process_name: "app1".into(),
                    executable_path: Some("/usr/bin/app1".into()),
                    bytes_received: 2000,
                    bytes_sent: 900,
                },
            ])
            .unwrap();

        let deleted = RetentionManager::new(30).cleanup_app(&repository).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(repository.count().unwrap(), 1);
    }
}
