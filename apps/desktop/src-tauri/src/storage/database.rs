use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::migrations;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{NetworkRollup, NetworkRollupRepository};

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

        assert_eq!(version, 1);
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
}
