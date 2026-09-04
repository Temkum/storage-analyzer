use rusqlite::{params, Connection};

/// One aggregated point on a historical time series: the total bytes
/// transferred during `bucket_seconds` starting at `ts` (a bucket start).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryPoint {
    pub ts: i64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Per-interface history series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceHistory {
    pub interface_id: String,
    pub points: Vec<HistoryPoint>,
}

/// Per-application usage totals over a time range, aggregated from the
/// persisted minute deltas. Identity is `app_id`; `process_name` and
/// `executable_path` are display metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppUsageTotals {
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Range is inclusive of `since`, exclusive of `until` — matching the
/// sampler's `ts >= bucket_start` bucket assignment so a query for
/// `[0, 60)` reads exactly the minutes 0..59 and never bleeds into the
/// next bucket.
pub fn bucket_seconds_for_range(since: i64, until: i64) -> i64 {
    let duration = until.saturating_sub(since).max(0);

    // Point-count targets: 1h → 60 points, 6h → 72, 24h → 96.
    if duration <= 60 * 60 {
        60
    } else if duration <= 6 * 60 * 60 {
        5 * 60
    } else {
        15 * 60
    }
}

/// Aggregates `network_rollups` into epoch-aligned buckets
/// (`(ts / bucket) * bucket`). `since` inclusive, `until` exclusive.
pub fn query_network_totals(
    connection: &Connection,
    since: i64,
    until: i64,
    bucket_seconds: i64,
) -> rusqlite::Result<Vec<HistoryPoint>> {
    let mut statement = connection.prepare(
        "
        SELECT
            (ts / ?1) * ?1 AS bucket_ts,
            SUM(bytes_received),
            SUM(bytes_sent)
        FROM network_rollups
        WHERE ts >= ?2 AND ts < ?3
        GROUP BY bucket_ts
        ORDER BY bucket_ts ASC
        ",
    )?;

    let rows = statement.query_map(params![bucket_seconds, since, until], |row| {
        Ok(HistoryPoint {
            ts: row.get(0)?,
            bytes_received: row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64,
            bytes_sent: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
        })
    })?;

    rows.collect()
}

/// Aggregates `network_rollups` per interface into epoch-aligned buckets.
pub fn query_network_by_interface(
    connection: &Connection,
    since: i64,
    until: i64,
    bucket_seconds: i64,
) -> rusqlite::Result<Vec<InterfaceHistory>> {
    let mut statement = connection.prepare(
        "
        SELECT
            interface_id,
            (ts / ?1) * ?1 AS bucket_ts,
            SUM(bytes_received),
            SUM(bytes_sent)
        FROM network_rollups
        WHERE ts >= ?2 AND ts < ?3
        GROUP BY interface_id, bucket_ts
        ORDER BY interface_id ASC, bucket_ts ASC
        ",
    )?;

    let rows = statement.query_map(params![bucket_seconds, since, until], |row| {
        Ok((
            row.get::<_, String>(0)?,
            HistoryPoint {
                ts: row.get(1)?,
                bytes_received: row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64,
                bytes_sent: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
            },
        ))
    })?;

    let mut series: Vec<InterfaceHistory> = Vec::new();

    for row in rows {
        let (interface_id, point) = row?;

        match series.last_mut() {
            Some(existing) if existing.interface_id == interface_id => {
                existing.points.push(point);
            }
            _ => {
                series.push(InterfaceHistory {
                    interface_id,
                    points: vec![point],
                });
            }
        }
    }

    Ok(series)
}

/// Aggregates `app_usage_rollups` per application over the range
/// (`since` inclusive, `until` exclusive), ordered by total bytes
/// (RX + TX) descending — the ranking order.
pub fn query_app_usage(
    connection: &Connection,
    since: i64,
    until: i64,
) -> rusqlite::Result<Vec<AppUsageTotals>> {
    let mut statement = connection.prepare(
        "
        SELECT
            app_id,
            process_name,
            MAX(executable_path),
            SUM(bytes_received),
            SUM(bytes_sent)
        FROM app_usage_rollups
        WHERE ts >= ?1 AND ts < ?2
        GROUP BY app_id
        ORDER BY SUM(bytes_received) + SUM(bytes_sent) DESC, app_id ASC
        ",
    )?;

    let rows = statement.query_map(params![since, until], |row| {
        Ok(AppUsageTotals {
            app_id: row.get(0)?,
            process_name: row.get(1)?,
            executable_path: row.get(2)?,
            bytes_received: row.get::<_, Option<i64>>(3)?.unwrap_or(0) as u64,
            bytes_sent: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u64,
        })
    })?;

    rows.collect()
}

/// Top-N application ranking over the range, from the persisted deltas.
pub fn top_applications(
    connection: &Connection,
    since: i64,
    until: i64,
    limit: usize,
) -> rusqlite::Result<Vec<AppUsageTotals>> {
    let mut usage = query_app_usage(connection, since, until)?;
    usage.truncate(limit);
    Ok(usage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        AppUsageRollup, AppUsageRollupRepository, Database, NetworkRollup, NetworkRollupRepository,
    };

    fn test_database() -> Database {
        let temp_dir = tempfile::tempdir().unwrap();
        Database::open(&temp_dir.keep()).unwrap()
    }

    fn rollup(ts: i64, interface_id: &str, received: u64, sent: u64) -> NetworkRollup {
        NetworkRollup {
            ts,
            interface_id: interface_id.into(),
            bytes_received: received,
            bytes_sent: sent,
        }
    }

    fn app_rollup(ts: i64, app_id: &str, received: u64, sent: u64) -> AppUsageRollup {
        AppUsageRollup {
            ts,
            app_id: app_id.into(),
            process_name: app_id.into(),
            executable_path: Some(app_id.into()),
            bytes_received: received,
            bytes_sent: sent,
        }
    }

    #[test]
    fn bucket_seconds_scale_with_range() {
        assert_eq!(bucket_seconds_for_range(0, 60 * 60), 60);
        assert_eq!(bucket_seconds_for_range(0, 6 * 60 * 60), 5 * 60);
        assert_eq!(bucket_seconds_for_range(0, 24 * 60 * 60), 15 * 60);
        // Degenerate/invalid range falls back to the finest bucket.
        assert_eq!(bucket_seconds_for_range(100, 100), 60);
        assert_eq!(bucket_seconds_for_range(200, 100), 60);
    }

    #[test]
    fn empty_history_returns_no_rows() {
        let mut database = test_database();

        let totals = query_network_totals(database.connection(), 0, 3_600, 60).unwrap();
        let interfaces = query_network_by_interface(database.connection(), 0, 3_600, 60).unwrap();
        let apps = query_app_usage(database.connection(), 0, 3_600).unwrap();

        assert!(totals.is_empty());
        assert!(interfaces.is_empty());
        assert!(apps.is_empty());
    }

    #[test]
    fn aggregation_merges_minute_buckets() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        // Minutes 0, 60, 120 on one interface; a 15-minute bucket range
        // collapses them into bucket 0.
        repository
            .insert_batch(&[
                rollup(0, "eth0", 100, 50),
                rollup(60, "eth0", 200, 100),
                rollup(120, "eth0", 300, 150),
            ])
            .unwrap();

        let totals = query_network_totals(database.connection(), 0, 24 * 60 * 60, 15 * 60).unwrap();

        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].ts, 0);
        assert_eq!(totals[0].bytes_received, 600);
        assert_eq!(totals[0].bytes_sent, 300);
    }

    #[test]
    fn timestamp_boundaries_are_inclusive_since_exclusive_until() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        repository
            .insert_batch(&[
                rollup(0, "eth0", 10, 5),    // at since → included
                rollup(60, "eth0", 20, 10),  // inside
                rollup(120, "eth0", 40, 20), // at until → excluded
            ])
            .unwrap();

        let totals = query_network_totals(database.connection(), 0, 120, 60).unwrap();

        assert_eq!(totals.len(), 2, "ts == until must be excluded");
        assert_eq!(totals[0].ts, 0);
        assert_eq!(totals[0].bytes_received, 10);
        assert_eq!(totals[1].ts, 60);
        assert_eq!(totals[1].bytes_received, 20);
    }

    #[test]
    fn per_interface_series_stay_separate() {
        let mut database = test_database();
        let mut repository = NetworkRollupRepository::new(database.connection());

        repository
            .insert_batch(&[
                rollup(0, "eth0", 10, 5),
                rollup(0, "wlan0", 7, 3),
                rollup(60, "eth0", 20, 10),
            ])
            .unwrap();

        let series = query_network_by_interface(database.connection(), 0, 120, 60).unwrap();

        assert_eq!(series.len(), 2);

        assert_eq!(series[0].interface_id, "eth0");
        assert_eq!(series[0].points.len(), 2);
        assert_eq!(series[0].points[1].bytes_received, 20);

        assert_eq!(series[1].interface_id, "wlan0");
        assert_eq!(series[1].points.len(), 1);
        assert_eq!(series[1].points[0].bytes_received, 7);
    }

    #[test]
    fn app_usage_totals_rank_by_total_bytes() {
        let mut database = test_database();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        // Totals in range: firefox 500, chrome 900, code 300.
        repository
            .insert_batch(&[
                app_rollup(0, "/usr/bin/firefox", 400, 100),
                app_rollup(60, "/usr/bin/firefox", 0, 0),
                app_rollup(0, "/usr/bin/chrome", 700, 200),
                app_rollup(0, "/usr/bin/code", 250, 50),
                app_rollup(120, "/usr/bin/code", 999, 999), // outside range
            ])
            .unwrap();

        let usage = query_app_usage(database.connection(), 0, 120).unwrap();

        assert_eq!(usage.len(), 3);
        assert_eq!(usage[0].app_id, "/usr/bin/chrome");
        assert_eq!(usage[0].bytes_received, 700);
        assert_eq!(usage[1].app_id, "/usr/bin/firefox");
        assert_eq!(usage[1].bytes_received, 400);
        assert_eq!(usage[2].app_id, "/usr/bin/code");
        assert_eq!(usage[2].bytes_received, 250);
    }

    #[test]
    fn top_applications_respects_limit() {
        let mut database = test_database();
        let mut repository = AppUsageRollupRepository::new(database.connection());

        repository
            .insert_batch(&[
                app_rollup(0, "/usr/bin/a", 100, 0),
                app_rollup(0, "/usr/bin/b", 200, 0),
                app_rollup(0, "/usr/bin/c", 300, 0),
            ])
            .unwrap();

        let top = top_applications(database.connection(), 0, 60, 2).unwrap();

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].app_id, "/usr/bin/c");
        assert_eq!(top[1].app_id, "/usr/bin/b");
    }
}
