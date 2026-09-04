use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: i32 = 2;

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

    if current_version < 1 {
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

        let applied_at = now_secs();

        transaction.execute(
            "INSERT INTO schema_migrations (version, applied_at)
             VALUES (?1, ?2)",
            rusqlite::params![1, applied_at],
        )?;

        transaction.commit()?;
    }

    if current_version < 2 {
        let transaction = connection.transaction()?;

        transaction.execute_batch(
            "
            CREATE TABLE app_usage_rollups (
                ts INTEGER NOT NULL,
                app_id TEXT NOT NULL,
                process_name TEXT NOT NULL,
                executable_path TEXT,
                bytes_received INTEGER NOT NULL,
                bytes_sent INTEGER NOT NULL,
                PRIMARY KEY (ts, app_id)
            );

            CREATE INDEX idx_app_usage_rollups_ts
                ON app_usage_rollups(ts);

            CREATE INDEX idx_app_usage_rollups_app
                ON app_usage_rollups(app_id);
            ",
        )?;

        let applied_at = now_secs();

        transaction.execute(
            "INSERT INTO schema_migrations (version, applied_at)
             VALUES (?1, ?2)",
            rusqlite::params![CURRENT_SCHEMA_VERSION, applied_at],
        )?;

        transaction.commit()?;
    }

    Ok(())
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is before UNIX epoch")
        .as_secs() as i64
}
