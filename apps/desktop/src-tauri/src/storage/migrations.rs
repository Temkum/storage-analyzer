use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub fn run(connection: &mut Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );
        ",
    )?;

    let current_version: i32 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if current_version < CURRENT_SCHEMA_VERSION {
        let transaction = connection.transaction()?;

        transaction.execute_batch(
            "
            CREATE TABLE network_rollups (
                ts INTEGER NOT NULL,
                interface_id TEXT NOT NULL,
                bytes_received INTEGER NOT NULL,
                bytes_sent INTEGER NOT NULL,
                PRIMARY KEY (ts, interface_id)
            );

            CREATE INDEX idx_network_rollups_ts
                ON network_rollups(ts);
            ",
        )?;

        let applied_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_secs() as i64;

        transaction.execute(
            "INSERT INTO schema_migrations (version, applied_at)
             VALUES (?1, ?2)",
            rusqlite::params![CURRENT_SCHEMA_VERSION, applied_at],
        )?;

        transaction.commit()?;
    }

    Ok(())
}
