use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use super::app_usage_rollup_repository::AppUsageRollup;
use super::migrations;
use super::network_rollup_repository::NetworkRollup;

pub struct Database {
    connection: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(app_data_dir: &Path) -> rusqlite::Result<Self> {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;

        let path = app_data_dir.join("system-analyzer.sqlite3");
        let mut connection = Connection::open(&path)?;

        connection.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            ",
        )?;

        migrations::run(&mut connection)?;

        Ok(Self { connection, path })
    }

    /// Application-facing entry point used from the Tauri setup hook. Thin
    /// wrapper over [`Self::open`] so initialization has a single call site.
    pub fn initialize(app_data_dir: &Path) -> rusqlite::Result<Self> {
        Self::open(app_data_dir)
    }

    pub fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Persists interface and application rollups in a **single SQLite
    /// transaction**. If either half fails, neither side claims the minute
    /// was durably persisted — giving the consistency guarantee that
    /// minute N is either fully written or fully absent.
    pub fn persist_rollups(
        &mut self,
        interface_rollups: &[NetworkRollup],
        app_rollups: &[AppUsageRollup],
    ) -> rusqlite::Result<()> {
        let transaction = self.connection.transaction()?;

        // Interface rollups (ON CONFLICT DO UPDATE mirrors
        // NetworkRollupRepository::insert_batch semantics).
        {
            let mut statement = transaction.prepare(
                "
                INSERT INTO network_rollups (
                    ts,
                    interface_id,
                    bytes_received,
                    bytes_sent
                )
                VALUES (?1, ?2, ?3, ?4)
                ON CONFLICT(ts, interface_id) DO UPDATE SET
                    bytes_received = excluded.bytes_received,
                    bytes_sent = excluded.bytes_sent
                ",
            )?;

            for rollup in interface_rollups {
                statement.execute(params![
                    rollup.ts,
                    rollup.interface_id,
                    rollup.bytes_received,
                    rollup.bytes_sent,
                ])?;
            }
        }

        // Application rollups.
        {
            let mut statement = transaction.prepare(
                "
                INSERT INTO app_usage_rollups (
                    ts,
                    app_id,
                    process_name,
                    executable_path,
                    bytes_received,
                    bytes_sent
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(ts, app_id) DO UPDATE SET
                    process_name = excluded.process_name,
                    executable_path = excluded.executable_path,
                    bytes_received = excluded.bytes_received,
                    bytes_sent = excluded.bytes_sent
                ",
            )?;

            for rollup in app_rollups {
                statement.execute(params![
                    rollup.ts,
                    rollup.app_id,
                    rollup.process_name,
                    rollup.executable_path,
                    rollup.bytes_received,
                    rollup.bytes_sent,
                ])?;
            }
        }

        transaction.commit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        AppUsageRollup, AppUsageRollupRepository, NetworkRollup, NetworkRollupRepository,
    };

    fn test_database() -> Database {
        let temp_dir = tempfile::tempdir().unwrap();

        // Keep the temporary directory alive for the lifetime of the test.
        let path = temp_dir.keep();

        Database::open(&path).unwrap()
    }

    #[test]
    fn creates_network_rollups_table() {
        let mut database = test_database();

        let exists: bool = database
            .connection()
            .query_row(
                "
                SELECT EXISTS(
                    SELECT 1
                    FROM sqlite_master
                    WHERE type = 'table'
                    AND name = 'network_rollups'
                )
                ",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(exists);
    }

    #[test]
    fn migrations_are_idempotent() {
        let temp_dir = tempfile::tempdir().unwrap();

        let first = Database::open(temp_dir.path()).unwrap();
        drop(first);

        let mut second = Database::open(temp_dir.path()).unwrap();

        let version: i32 = second
            .connection()
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(version, 2);
    }

    #[test]
    fn inserts_and_reads_rollups() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        let rollups = vec![
            NetworkRollup {
                ts: 100,
                interface_id: "eth0".into(),
                bytes_received: 1000,
                bytes_sent: 500,
            },
            NetworkRollup {
                ts: 101,
                interface_id: "eth0".into(),
                bytes_received: 1200,
                bytes_sent: 600,
            },
        ];

        repository.insert_batch(&rollups).unwrap();

        assert_eq!(repository.count().unwrap(), 2);

        let result = repository.find_since(100).unwrap();

        assert_eq!(result, rollups);
    }

    #[test]
    fn duplicate_bucket_is_updated() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        repository
            .insert_batch(&[NetworkRollup {
                ts: 100,
                interface_id: "eth0".into(),
                bytes_received: 1000,
                bytes_sent: 500,
            }])
            .unwrap();

        repository
            .insert_batch(&[NetworkRollup {
                ts: 100,
                interface_id: "eth0".into(),
                bytes_received: 2000,
                bytes_sent: 900,
            }])
            .unwrap();

        assert_eq!(repository.count().unwrap(), 1);

        let result = repository.find_since(100).unwrap();

        assert_eq!(result[0].bytes_received, 2000);
        assert_eq!(result[0].bytes_sent, 900);
    }

    #[test]
    fn deletes_rows_before_cutoff() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        repository
            .insert_batch(&[
                NetworkRollup {
                    ts: 100,
                    interface_id: "eth0".into(),
                    bytes_received: 1000,
                    bytes_sent: 500,
                },
                NetworkRollup {
                    ts: 200,
                    interface_id: "eth0".into(),
                    bytes_received: 2000,
                    bytes_sent: 900,
                },
            ])
            .unwrap();

        let deleted = repository.delete_before(200).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(repository.count().unwrap(), 1);
    }

    // ---- Migration #2 / app_usage_rollups tests ----

    #[test]
    fn creates_app_usage_rollups_table() {
        let mut database = test_database();

        let exists: bool = database
            .connection()
            .query_row(
                "
                SELECT EXISTS(
                    SELECT 1
                    FROM sqlite_master
                    WHERE type = 'table'
                    AND name = 'app_usage_rollups'
                )
                ",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(exists);

        // Indexes must also exist.
        let ts_exists: bool = database
            .connection()
            .query_row(
                "
                SELECT EXISTS(
                    SELECT 1
                    FROM sqlite_master
                    WHERE type = 'index'
                    AND name = 'idx_app_usage_rollups_ts'
                )
                ",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(ts_exists);

        let app_exists: bool = database
            .connection()
            .query_row(
                "
                SELECT EXISTS(
                    SELECT 1
                    FROM sqlite_master
                    WHERE type = 'index'
                    AND name = 'idx_app_usage_rollups_app'
                )
                ",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(app_exists);
    }

    #[test]
    fn migrations_remain_idempotent_after_v2() {
        let temp_dir = tempfile::tempdir().unwrap();
        let first = Database::open(temp_dir.path()).unwrap();
        drop(first);

        let mut second = Database::open(temp_dir.path()).unwrap();

        let version: i32 = second
            .connection()
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(version, 2);
    }

    #[test]
    fn app_rollups_insert_and_read() {
        let mut database = test_database();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        let rollups = vec![
            AppUsageRollup {
                ts: 100,
                app_id: "/usr/bin/firefox".into(),
                process_name: "firefox".into(),
                executable_path: Some("/usr/bin/firefox".into()),
                bytes_received: 1000,
                bytes_sent: 500,
            },
            AppUsageRollup {
                ts: 101,
                app_id: "/usr/bin/firefox".into(),
                process_name: "firefox".into(),
                executable_path: Some("/usr/bin/firefox".into()),
                bytes_received: 1200,
                bytes_sent: 600,
            },
        ];

        repository.insert_batch(&rollups).unwrap();

        assert_eq!(repository.count().unwrap(), 2);

        let result = repository.find_since(100).unwrap();

        assert_eq!(result, rollups);
    }

    #[test]
    fn app_duplicate_bucket_is_updated() {
        let mut database = test_database();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        repository
            .insert_batch(&[AppUsageRollup {
                ts: 100,
                app_id: "/usr/bin/firefox".into(),
                process_name: "firefox".into(),
                executable_path: Some("/usr/bin/firefox".into()),
                bytes_received: 1000,
                bytes_sent: 500,
            }])
            .unwrap();

        repository
            .insert_batch(&[AppUsageRollup {
                ts: 100,
                app_id: "/usr/bin/firefox".into(),
                process_name: "firefox-beta".into(),
                executable_path: Some("/usr/bin/firefox-beta".into()),
                bytes_received: 2000,
                bytes_sent: 900,
            }])
            .unwrap();

        assert_eq!(repository.count().unwrap(), 1);

        let result = repository.find_since(100).unwrap();
        assert_eq!(result[0].bytes_received, 2000);
        assert_eq!(result[0].bytes_sent, 900);
        assert_eq!(result[0].process_name, "firefox-beta");
    }

    #[test]
    fn app_deletes_rows_before_cutoff() {
        let mut database = test_database();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        repository
            .insert_batch(&[
                AppUsageRollup {
                    ts: 100,
                    app_id: "/usr/bin/app1".into(),
                    process_name: "app1".into(),
                    executable_path: Some("/usr/bin/app1".into()),
                    bytes_received: 1000,
                    bytes_sent: 500,
                },
                AppUsageRollup {
                    ts: 200,
                    app_id: "/usr/bin/app1".into(),
                    process_name: "app1".into(),
                    executable_path: Some("/usr/bin/app1".into()),
                    bytes_received: 2000,
                    bytes_sent: 900,
                },
            ])
            .unwrap();

        let deleted = repository.delete_before(200).unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(repository.count().unwrap(), 1);
    }

    #[test]
    fn persist_rollups_is_atomic() {
        let mut database = test_database();

        let network_rollups = vec![NetworkRollup {
            ts: 0,
            interface_id: "eth0".into(),
            bytes_received: 100,
            bytes_sent: 50,
        }];

        let app_rollups = vec![AppUsageRollup {
            ts: 0,
            app_id: "/usr/bin/app".into(),
            process_name: "app".into(),
            executable_path: Some("/usr/bin/app".into()),
            bytes_received: 300,
            bytes_sent: 150,
        }];

        database
            .persist_rollups(&network_rollups, &app_rollups)
            .unwrap();

        {
            let net_repo = NetworkRollupRepository::new(database.connection());
            assert_eq!(net_repo.count().unwrap(), 1);
        }

        {
            let app_repo = AppUsageRollupRepository::new(database.connection());
            assert_eq!(app_repo.count().unwrap(), 1);
        }
    }
}
