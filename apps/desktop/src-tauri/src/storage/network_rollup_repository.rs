use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkRollup {
    pub ts: i64,
    pub interface_id: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

pub struct NetworkRollupRepository<'a> {
    connection: &'a mut Connection,
}

impl<'a> NetworkRollupRepository<'a> {
    pub fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }

    pub fn insert_batch(&mut self, rollups: &[NetworkRollup]) -> rusqlite::Result<()> {
        let transaction = self.connection.transaction()?;

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

            for rollup in rollups {
                statement.execute(params![
                    rollup.ts,
                    rollup.interface_id,
                    rollup.bytes_received,
                    rollup.bytes_sent,
                ])?;
            }
        }

        transaction.commit()
    }

    pub fn find_since(&self, since: i64) -> rusqlite::Result<Vec<NetworkRollup>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                ts,
                interface_id,
                bytes_received,
                bytes_sent
            FROM network_rollups
            WHERE ts >= ?1
            ORDER BY ts ASC, interface_id ASC
            ",
        )?;

        let rows = statement.query_map([since], |row| {
            Ok(NetworkRollup {
                ts: row.get(0)?,
                interface_id: row.get(1)?,
                bytes_received: row.get(2)?,
                bytes_sent: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    pub fn delete_before(&self, cutoff: i64) -> rusqlite::Result<usize> {
        self.connection
            .execute("DELETE FROM network_rollups WHERE ts < ?1", [cutoff])
    }

    pub fn count(&self) -> rusqlite::Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM network_rollups", [], |row| row.get(0))
    }
}
