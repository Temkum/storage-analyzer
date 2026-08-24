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

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
