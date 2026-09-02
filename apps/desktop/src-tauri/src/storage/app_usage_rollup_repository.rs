use rusqlite::{params, Connection};

/// One persisted application rollup row. `ts` is the start of the 60-second
/// bucket (matching `network_rollups` semantics). `app_id` is the canonical
/// executable path. `process_name` and `executable_path` are display metadata
/// captured at rollup time. RX/TX are the delta bytes accumulated during that
/// minute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppUsageRollup {
    pub ts: i64,
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

pub struct AppUsageRollupRepository<'a> {
    connection: &'a mut Connection,
}

impl<'a> AppUsageRollupRepository<'a> {
    pub fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }

    /// Inserts all rollups in one rusqlite transaction. Duplicate
    /// `(ts, app_id)` rows are updated in place (ON CONFLICT DO UPDATE),
    /// preserving the latest `process_name` / `executable_path` / counters.
    pub fn insert_batch(&mut self, rollups: &[AppUsageRollup]) -> rusqlite::Result<()> {
        let transaction = self.connection.transaction()?;

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

            for rollup in rollups {
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

    pub fn find_since(&self, since: i64) -> rusqlite::Result<Vec<AppUsageRollup>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                ts,
                app_id,
                process_name,
                executable_path,
                bytes_received,
                bytes_sent
            FROM app_usage_rollups
            WHERE ts >= ?1
            ORDER BY ts ASC, app_id ASC
            ",
        )?;

        let rows = statement.query_map([since], |row| {
            Ok(AppUsageRollup {
                ts: row.get(0)?,
                app_id: row.get(1)?,
                process_name: row.get(2)?,
                executable_path: row.get(3)?,
                bytes_received: row.get(4)?,
                bytes_sent: row.get(5)?,
            })
        })?;

        rows.collect()
    }

    pub fn delete_before(&self, cutoff: i64) -> rusqlite::Result<usize> {
        self.connection
            .execute("DELETE FROM app_usage_rollups WHERE ts < ?1", [cutoff])
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM app_usage_rollups", [], |row| {
                row.get(0)
            })
    }
}
