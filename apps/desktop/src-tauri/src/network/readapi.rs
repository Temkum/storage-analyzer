//! Read-side API: DTOs returned to the Vue frontend plus the pure assembly
//! that turns live telemetry / SQLite query rows into those DTOs. No IPC and
//! no sidecar access here — commands read the monitor's published snapshots
//! and the persisted rollups only.

use std::collections::BTreeMap;

use serde::Serialize;

use super::monitor::LiveTelemetry;
use crate::storage::{AppUsageTotals, HistoryPoint, InterfaceHistory};

// Live DTOs

/// One 1-second live sample (byte counts transferred during that second).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSampleDto {
    pub timestamp: i64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Per-interface live series grouped out of the ring-buffer snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceLiveDto {
    pub interface_id: String,
    pub samples: Vec<LiveSampleDto>,
}

/// Per-application live series. Identity is `app_id` (the canonical
/// executable path); PID never appears here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationLiveDto {
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub samples: Vec<LiveSampleDto>,
}

/// The complete live view: a merged per-second totals series for the
/// throughput chart plus per-interface and per-application series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkLiveDto {
    /// Timestamp of the last successful tick; `None` until the first.
    pub updated_at: Option<i64>,
    /// Total bytes per second across all interfaces (chart series).
    pub totals: Vec<LiveSampleDto>,
    pub interfaces: Vec<InterfaceLiveDto>,
    pub applications: Vec<ApplicationLiveDto>,
    /// Last monitoring error, if any; the UI surfaces this as degraded.
    pub error: Option<String>,
}

/// Assembles the live DTO from a monitor snapshot. Grouping is keyed by
/// identity (`interface_id` / `app_id`); the totals series sums across all
/// interfaces per timestamp (ring-buffer timestamps are aligned per tick).
pub fn live_dto(telemetry: &LiveTelemetry) -> NetworkLiveDto {
    let mut interface_series: BTreeMap<String, Vec<LiveSampleDto>> = BTreeMap::new();

    for sample in &telemetry.interfaces {
        interface_series
            .entry(sample.interface_id.clone())
            .or_default()
            .push(LiveSampleDto {
                timestamp: sample.timestamp,
                bytes_received: sample.bytes_received,
                bytes_sent: sample.bytes_sent,
            });
    }

    let mut app_series: BTreeMap<String, (Option<String>, String, Vec<LiveSampleDto>)> =
        BTreeMap::new();

    for sample in &telemetry.applications {
        let entry = app_series.entry(sample.app_id.clone()).or_insert_with(|| {
            (
                sample.executable_path.clone(),
                sample.process_name.clone(),
                Vec::new(),
            )
        });

        entry.2.push(LiveSampleDto {
            timestamp: sample.timestamp,
            bytes_received: sample.bytes_received,
            bytes_sent: sample.bytes_sent,
        });
    }

    // Merged totals: ring-buffer samples for one tick share a timestamp, but
    // sum defensively with saturating adds rather than assuming alignment.
    let mut totals: BTreeMap<i64, (u64, u64)> = BTreeMap::new();

    for sample in &telemetry.interfaces {
        let entry = totals.entry(sample.timestamp).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(sample.bytes_received);
        entry.1 = entry.1.saturating_add(sample.bytes_sent);
    }

    NetworkLiveDto {
        updated_at: telemetry.updated_at,
        totals: totals
            .into_iter()
            .map(|(timestamp, (received, sent))| LiveSampleDto {
                timestamp,
                bytes_received: received,
                bytes_sent: sent,
            })
            .collect(),
        interfaces: interface_series
            .into_iter()
            .map(|(interface_id, samples)| InterfaceLiveDto {
                interface_id,
                samples,
            })
            .collect(),
        applications: app_series
            .into_iter()
            .map(
                |(app_id, (executable_path, process_name, samples))| ApplicationLiveDto {
                    app_id,
                    process_name,
                    executable_path,
                    samples,
                },
            )
            .collect(),
        error: telemetry.error.clone(),
    }
}

// ---- History DTOs --------------------------------------------------------

/// One aggregated history point over `bucketSeconds` starting at `ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPointDto {
    pub ts: i64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceHistoryDto {
    pub interface_id: String,
    pub points: Vec<HistoryPointDto>,
}

/// Per-application usage totals over a range, ranked by total bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationUsageDto {
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkHistoryDto {
    pub since: i64,
    pub until: i64,
    pub bucket_seconds: i64,
    pub totals: Vec<HistoryPointDto>,
    pub interfaces: Vec<InterfaceHistoryDto>,
}

pub fn history_point_dto(point: &HistoryPoint) -> HistoryPointDto {
    HistoryPointDto {
        ts: point.ts,
        bytes_received: point.bytes_received,
        bytes_sent: point.bytes_sent,
    }
}

pub fn network_history_dto(
    since: i64,
    until: i64,
    bucket_seconds: i64,
    totals: Vec<HistoryPoint>,
    interfaces: Vec<InterfaceHistory>,
) -> NetworkHistoryDto {
    NetworkHistoryDto {
        since,
        until,
        bucket_seconds,
        totals: totals.iter().map(history_point_dto).collect(),
        interfaces: interfaces
            .into_iter()
            .map(|series| InterfaceHistoryDto {
                interface_id: series.interface_id,
                points: series.points.iter().map(history_point_dto).collect(),
            })
            .collect(),
    }
}

pub fn application_usage_dto(usage: &AppUsageTotals) -> ApplicationUsageDto {
    ApplicationUsageDto {
        app_id: usage.app_id.clone(),
        process_name: usage.process_name.clone(),
        executable_path: usage.executable_path.clone(),
        bytes_received: usage.bytes_received,
        bytes_sent: usage.bytes_sent,
    }
}

pub fn application_usage_dtos(usage: &[AppUsageTotals]) -> Vec<ApplicationUsageDto> {
    usage.iter().map(application_usage_dto).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::telemetry::{AppSample, NetworkSample};

    fn interface_sample(ts: i64, id: &str, received: u64, sent: u64) -> NetworkSample {
        NetworkSample {
            timestamp: ts,
            interface_id: id.into(),
            bytes_received: received,
            bytes_sent: sent,
        }
    }

    fn app_sample(ts: i64, id: &str, received: u64, sent: u64) -> AppSample {
        AppSample {
            timestamp: ts,
            app_id: id.into(),
            process_name: id.into(),
            executable_path: Some(id.into()),
            bytes_received: received,
            bytes_sent: sent,
        }
    }

    #[test]
    fn live_dto_groups_series_and_builds_totals() {
        let telemetry = LiveTelemetry {
            interfaces: vec![
                interface_sample(1, "eth0", 10, 5),
                interface_sample(1, "wlan0", 7, 3),
                interface_sample(2, "eth0", 20, 10),
            ],
            applications: vec![
                app_sample(1, "/usr/bin/a", 5, 2),
                app_sample(1, "/usr/bin/b", 3, 1),
                app_sample(2, "/usr/bin/a", 6, 3),
            ],
            updated_at: Some(2),
            error: None,
        };

        let dto = live_dto(&telemetry);

        assert_eq!(dto.updated_at, Some(2));
        assert!(dto.error.is_none());

        // Totals merge interfaces per timestamp.
        assert_eq!(dto.totals.len(), 2);
        assert_eq!(dto.totals[0].timestamp, 1);
        assert_eq!(dto.totals[0].bytes_received, 17);
        assert_eq!(dto.totals[0].bytes_sent, 8);
        assert_eq!(dto.totals[1].bytes_received, 20);

        // Interfaces grouped, ordered by id (BTreeMap).
        assert_eq!(dto.interfaces.len(), 2);
        assert_eq!(dto.interfaces[0].interface_id, "eth0");
        assert_eq!(dto.interfaces[0].samples.len(), 2);
        assert_eq!(dto.interfaces[1].interface_id, "wlan0");

        // Applications grouped with metadata retained.
        assert_eq!(dto.applications.len(), 2);
        let a = dto
            .applications
            .iter()
            .find(|app| app.app_id == "/usr/bin/a")
            .unwrap();
        assert_eq!(a.samples.len(), 2);
        assert_eq!(a.process_name, "/usr/bin/a");
        assert_eq!(a.executable_path.as_deref(), Some("/usr/bin/a"));
    }

    #[test]
    fn live_dto_from_empty_telemetry_is_valid() {
        let dto = live_dto(&LiveTelemetry::default());

        assert_eq!(dto.updated_at, None);
        assert!(dto.totals.is_empty());
        assert!(dto.interfaces.is_empty());
        assert!(dto.applications.is_empty());
    }

    #[test]
    fn live_dto_carries_monitor_error() {
        let telemetry = LiveTelemetry {
            error: Some("sidecar died".into()),
            ..LiveTelemetry::default()
        };

        assert_eq!(live_dto(&telemetry).error.as_deref(), Some("sidecar died"));
    }

    #[test]
    fn history_dto_maps_rows() {
        let dto = network_history_dto(
            0,
            120,
            60,
            vec![HistoryPoint {
                ts: 0,
                bytes_received: 100,
                bytes_sent: 50,
            }],
            vec![InterfaceHistory {
                interface_id: "eth0".into(),
                points: vec![HistoryPoint {
                    ts: 0,
                    bytes_received: 100,
                    bytes_sent: 50,
                }],
            }],
        );

        assert_eq!(dto.since, 0);
        assert_eq!(dto.until, 120);
        assert_eq!(dto.bucket_seconds, 60);
        assert_eq!(dto.totals.len(), 1);
        assert_eq!(dto.interfaces.len(), 1);
        assert_eq!(dto.interfaces[0].interface_id, "eth0");
    }

    #[test]
    fn application_usage_dtos_map_metadata() {
        let dtos = application_usage_dtos(&[AppUsageTotals {
            app_id: "/usr/bin/app".into(),
            process_name: "app".into(),
            executable_path: Some("/usr/bin/app".into()),
            bytes_received: 10,
            bytes_sent: 5,
        }]);

        assert_eq!(dtos.len(), 1);
        assert_eq!(dtos[0].app_id, "/usr/bin/app");
        assert_eq!(dtos[0].process_name, "app");
        assert_eq!(dtos[0].bytes_received, 10);
        assert_eq!(dtos[0].bytes_sent, 5);
    }
}
