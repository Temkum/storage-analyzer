use std::collections::{HashMap, VecDeque};
use std::future::Future;

use super::sidecar::NetworkError;
use super::types::NetworkSnapshot;
use crate::storage::app_usage_rollup_repository::AppUsageRollup;
use crate::storage::network_rollup_repository::NetworkRollup;

/// Total number of 1-second raw samples retained per interface for the live
/// UI: 10 minutes of history at one sample/second. The buffer is strictly for
/// the live chart; SQLite owns persistent history.
pub const RING_BUFFER_CAPACITY: usize = 600;

/// Number of 1-second samples that make up one persisted rollup interval.
pub const ROLLUP_INTERVAL_SECONDS: i64 = 60;

/// Upper bound on applications tracked simultaneously. Desktop machines
/// rarely exceed a few dozen concurrently network-active executables; the
/// cap keeps the application ring buffer bounded without blindly allocating
/// `600 × N` for an unbounded N.
pub const MAX_TRACKED_APPLICATIONS: usize = 100;

/// Bounded capacity of the application ring buffer, sized as
/// maximum samples × maximum tracked applications.
pub const APPLICATION_RING_BUFFER_CAPACITY: usize = RING_BUFFER_CAPACITY * MAX_TRACKED_APPLICATIONS;

/// One calculated telemetry sample for a single interface. These are byte
/// counts transferred during the sampling interval — NOT the cumulative
/// counters reported by the C++ provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkSample {
    pub timestamp: i64,
    pub interface_id: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// One calculated telemetry sample for a single application: the byte counts
/// that application transferred during the sampling interval. Identity is
/// `app_id` (the canonical executable path); a PID is never part of a sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSample {
    pub timestamp: i64,
    pub app_id: String,
    pub process_name: String,
    pub executable_path: Option<String>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Common timestamp accessor so one bounded ring-buffer implementation can
/// hold either interface or application samples.
pub trait Timestamped {
    fn timestamp(&self) -> i64;
}

impl Timestamped for NetworkSample {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl Timestamped for AppSample {
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// The minute (60-second bucket) a snapshot timestamp belongs to. A bucket is
/// identified by its *start* timestamp, so `12:00:59` persists under
/// `12:00:00` and an `ORDER BY ts ASC` yields a clean 1-minute time series.
fn bucket_timestamp(timestamp: i64) -> i64 {
    timestamp - timestamp.rem_euclid(ROLLUP_INTERVAL_SECONDS)
}

/// Pure counter delta. A decrease means the counter reset or wrapped
/// (interface reappeared, device reset, new namespace baseline, process
/// restart); it is never interpreted as negative traffic.
fn calculate_delta(previous: u64, current: u64) -> Option<u64> {
    current.checked_sub(previous)
}

/// RX/TX deltas for one interface, or `None` when either counter reset and no
/// traffic should be recorded for this interval.
fn interface_deltas(
    previous_received: u64,
    current_received: u64,
    previous_sent: u64,
    current_sent: u64,
) -> Option<(u64, u64)> {
    calculate_delta(previous_received, current_received)
        .zip(calculate_delta(previous_sent, current_sent))
}

/// A bounded ring buffer of raw samples for the live UI.
///
/// When at capacity, appending evicts the oldest sample. The evicted sample's
/// rollup is already persisted, so the raw value is safe to discard.
#[derive(Debug)]
pub struct SampleRingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> SampleRingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Appends a sample, evicting the oldest when at capacity.
    fn push(&mut self, sample: T) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }

        self.buffer.push_back(sample);
    }

    /// Most recent sample, if any.
    fn latest(&self) -> Option<&T> {
        self.buffer.back()
    }

    /// Copies the buffer contents, oldest first. The monitor publishes this
    /// as an immutable live snapshot so the read API never touches the
    /// sampler while a tick is in flight.
    pub fn snapshot(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.buffer.iter().cloned().collect()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl<T: Timestamped> SampleRingBuffer<T> {
    /// Samples whose timestamp falls strictly inside the trailing
    /// `duration_seconds` window measured from the latest sample.
    fn recent(&self, duration_seconds: i64) -> Vec<&T> {
        let Some(latest_ts) = self.latest().map(Timestamped::timestamp) else {
            return Vec::new();
        };

        let cutoff = latest_ts - duration_seconds;

        self.buffer
            .iter()
            .filter(move |sample| sample.timestamp() > cutoff)
            .collect()
    }
}

/// Interface samples for the live UI.
pub type NetworkRingBuffer = SampleRingBuffer<NetworkSample>;

/// Application samples for the live UI. Bounded by
/// [`APPLICATION_RING_BUFFER_CAPACITY`] (samples × tracked applications).
pub type ApplicationRingBuffer = SampleRingBuffer<AppSample>;

/// An in-progress 60-second aggregation bucket for one interface.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RollingBucket {
    bytes_received: u64,
    bytes_sent: u64,
}

impl RollingBucket {
    fn accumulate(&mut self, delta_received: u64, delta_sent: u64) {
        self.bytes_received = self
            .bytes_received
            .checked_add(delta_received)
            .expect("rollup byte counters overflowed");
        self.bytes_sent = self
            .bytes_sent
            .checked_add(delta_sent)
            .expect("rollup byte counters overflowed");
    }
}

/// An in-progress 60-second aggregation bucket for one application. Carries
/// the display metadata captured on the latest observation so the persisted
/// row always reflects the most recent `process_name` / `executable_path`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AppRollingBucket {
    process_name: String,
    executable_path: Option<String>,
    bytes_received: u64,
    bytes_sent: u64,
}

impl AppRollingBucket {
    fn accumulate(&mut self, delta_received: u64, delta_sent: u64) {
        self.bytes_received = self
            .bytes_received
            .checked_add(delta_received)
            .expect("rollup byte counters overflowed");
        self.bytes_sent = self
            .bytes_sent
            .checked_add(delta_sent)
            .expect("rollup byte counters overflowed");
    }
}

/// The in-progress minute's accumulator combined with the bucket timestamp the
/// samples belong to.
struct Buckets {
    ts: Option<i64>,
    by_interface: HashMap<String, RollingBucket>,
}

impl Buckets {
    fn new() -> Self {
        Self {
            ts: None,
            by_interface: HashMap::new(),
        }
    }

    /// True when a snapshot has crossed a minute boundary relative to the
    /// in-progress bucket, so the previous bucket is complete and must be
    /// persisted.
    fn is_complete_for(&self, snapshot_ts: i64) -> bool {
        self.ts.is_some() && self.ts != Some(bucket_timestamp(snapshot_ts))
    }

    fn accumulate(&mut self, bucket_ts: i64, interface_id: &str, received: u64, sent: u64) {
        self.by_interface
            .entry(interface_id.to_string())
            .or_insert_with(|| RollingBucket {
                bytes_received: 0,
                bytes_sent: 0,
            })
            .accumulate(received, sent);

        self.ts = Some(bucket_ts);
    }

    /// Consumes the completed buckets into rollup rows and clears the
    /// accumulator. No-op when there is no bucket in progress.
    fn drain(&mut self, ts: i64) -> Vec<NetworkRollup> {
        let rollups = self
            .by_interface
            .drain()
            .map(|(interface_id, bucket)| NetworkRollup {
                ts,
                interface_id,
                bytes_received: bucket.bytes_received,
                bytes_sent: bucket.bytes_sent,
            })
            .collect();

        self.ts = None;

        rollups
    }
}

/// The in-progress minute's application accumulator combined with the bucket
/// timestamp the samples belong to. Keyed by `app_id`.
struct AppBuckets {
    ts: Option<i64>,
    by_app: HashMap<String, AppRollingBucket>,
}

impl AppBuckets {
    fn new() -> Self {
        Self {
            ts: None,
            by_app: HashMap::new(),
        }
    }

    fn is_complete_for(&self, snapshot_ts: i64) -> bool {
        self.ts.is_some() && self.ts != Some(bucket_timestamp(snapshot_ts))
    }

    fn accumulate(
        &mut self,
        bucket_ts: i64,
        app_id: &str,
        process_name: &str,
        executable_path: Option<&str>,
        received: u64,
        sent: u64,
    ) {
        let bucket = self
            .by_app
            .entry(app_id.to_string())
            .or_insert_with(|| AppRollingBucket {
                process_name: process_name.to_string(),
                executable_path: executable_path.map(str::to_string),
                bytes_received: 0,
                bytes_sent: 0,
            });

        // Display metadata is refreshed on every observation so the persisted
        // row carries the latest known label; identity (`app_id`) never moves.
        bucket.process_name = process_name.to_string();
        if let Some(path) = executable_path {
            bucket.executable_path = Some(path.to_string());
        }

        bucket.accumulate(received, sent);

        self.ts = Some(bucket_ts);
    }

    fn drain(&mut self, ts: i64) -> Vec<AppUsageRollup> {
        let rollups = self
            .by_app
            .drain()
            .map(|(app_id, bucket)| AppUsageRollup {
                ts,
                app_id,
                process_name: bucket.process_name,
                executable_path: bucket.executable_path,
                bytes_received: bucket.bytes_received,
                bytes_sent: bucket.bytes_sent,
            })
            .collect();

        self.ts = None;

        rollups
    }
}

/// Interface + application rollups drained at one minute boundary. Both halves
/// must be committed in ONE SQLite transaction: if the transaction fails,
/// neither side claims that minute was durably persisted.
pub struct RollupBatch {
    pub interfaces: Vec<NetworkRollup>,
    pub applications: Vec<AppUsageRollup>,
}

/// Interface + application samples produced by one sampling cycle. One tick
/// represents one coherent telemetry point across both streams.
pub struct SampleResult {
    pub interfaces: Vec<NetworkSample>,
    pub applications: Vec<AppSample>,
}

/// Folds a sequence of C++ network snapshots into persisted 1-minute rollups,
/// retaining raw 1-second samples for the live UI. Kept deterministic and
/// timer-free: [`NetworkSampler::sample_once`] performs exactly one
/// snapshot → delta → buffer → rollup operation, decoupled from SQLite
/// through the `persist` closure. The scheduler (Step 5.9) becomes a thin
/// wrapper around `sample_once`.
///
/// A single tick obtains BOTH telemetry streams — interface snapshots and
/// application snapshots — from one combined C++ snapshot, so one tick is one
/// coherent observation of the system. There is deliberately no second
/// sampler: [`NetworkSampler`] owns two ring buffers and two accumulators
/// that are drained together at each 60-second boundary and persisted in a
/// single transaction.
///
/// The C++ sidecar reports cumulative OS counters; this Rust layer alone
/// converts them into per-interval deltas. Rate/delta math is never pushed
/// back into the C++ provider.
pub struct NetworkSampler {
    previous: Option<NetworkSnapshot>,
    ring_buffer: NetworkRingBuffer,
    app_ring_buffer: ApplicationRingBuffer,
    buckets: Buckets,
    app_buckets: AppBuckets,
}

impl NetworkSampler {
    pub fn new() -> Self {
        Self {
            previous: None,
            ring_buffer: SampleRingBuffer::new(RING_BUFFER_CAPACITY),
            app_ring_buffer: SampleRingBuffer::new(APPLICATION_RING_BUFFER_CAPACITY),
            buckets: Buckets::new(),
            app_buckets: AppBuckets::new(),
        }
    }

    /// Raw interface samples for the live UI.
    pub fn samples(&self) -> &NetworkRingBuffer {
        &self.ring_buffer
    }

    /// Raw application samples for the live UI.
    pub fn app_samples(&self) -> &ApplicationRingBuffer {
        &self.app_ring_buffer
    }

    /// Performs one snapshot → delta → buffer → rollup operation against the
    /// combined snapshot returned by one C++ request.
    ///
    /// Application counters follow exactly the same reset semantics as
    /// interface counters:
    ///
    /// * no previous counter (new application) → establish baseline, zero
    ///   attributed bytes for that first observation;
    /// * previous exists, current missing (process disappeared) → no sample;
    /// * current < previous (counter reset/process restart) → skip sample.
    pub async fn sample_once<F, FutFetch, FutPersist>(
        &mut self,
        fetch: impl FnOnce() -> FutFetch,
        persist: &mut F,
    ) -> Result<SampleResult, NetworkError>
    where
        F: FnMut(RollupBatch) -> FutPersist + Send,
        FutFetch: Future<Output = Result<NetworkSnapshot, NetworkError>> + Send,
        FutPersist: Future<Output = Result<(), NetworkError>> + Send,
    {
        let snapshot = fetch().await?;

        let mut produced = SampleResult {
            interfaces: Vec::new(),
            applications: Vec::new(),
        };

        let Some(previous) = self.previous.take() else {
            // First snapshot establishes the baseline; no traffic to report.
            self.previous = Some(snapshot);
            return Ok(produced);
        };

        let snapshot_ts = snapshot.timestamp;

        // A minute boundary completes the in-progress buckets first. Checked
        // once per snapshot, before the interface/application loops, so the
        // completed batch is persisted even when nothing produced a delta in
        // this interval (e.g. every counter reset at the boundary).
        if self.buckets.is_complete_for(snapshot_ts)
            || self.app_buckets.is_complete_for(snapshot_ts)
        {
            let completed_ts = self
                .buckets
                .ts
                .or(self.app_buckets.ts)
                .expect("ts set when a bucket is complete");

            let batch = RollupBatch {
                interfaces: self.buckets.drain(completed_ts),
                applications: self.app_buckets.drain(completed_ts),
            };

            if !batch.interfaces.is_empty() || !batch.applications.is_empty() {
                persist(batch).await?;
            }
        }

        for current in &snapshot.interfaces {
            let Some(previous_interface) = previous
                .interfaces
                .iter()
                .find(|interface| interface.id == current.id)
            else {
                // New interface with no prior baseline: record no traffic.
                continue;
            };

            let Some((delta_received, delta_sent)) = interface_deltas(
                previous_interface.bytes_received,
                current.bytes_received,
                previous_interface.bytes_sent,
                current.bytes_sent,
            ) else {
                // Counter reset/wrap: ignore this interval rather than
                // recording a bogus massive delta.
                continue;
            };

            let sample = NetworkSample {
                timestamp: snapshot_ts,
                interface_id: current.id.clone(),
                bytes_received: delta_received,
                bytes_sent: delta_sent,
            };

            self.ring_buffer.push(sample.clone());
            produced.interfaces.push(sample);

            self.buckets.accumulate(
                bucket_timestamp(snapshot_ts),
                &current.id,
                delta_received,
                delta_sent,
            );
        }

        for current in &snapshot.applications {
            let Some(previous_app) = previous
                .applications
                .iter()
                .find(|app| app.app_id == current.app_id)
            else {
                // New application: establish a baseline; produce zero
                // attributed bytes for the first observation. Process churn
                // must never fabricate traffic.
                continue;
            };

            let Some((delta_received, delta_sent)) = interface_deltas(
                previous_app.bytes_received,
                current.bytes_received,
                previous_app.bytes_sent,
                current.bytes_sent,
            ) else {
                // Application counter reset (process restart): skip rather
                // than recording a bogus massive delta.
                continue;
            };

            let sample = AppSample {
                timestamp: snapshot_ts,
                app_id: current.app_id.clone(),
                process_name: current.process_name.clone(),
                executable_path: current.executable_path.clone(),
                bytes_received: delta_received,
                bytes_sent: delta_sent,
            };

            self.app_ring_buffer.push(sample.clone());
            produced.applications.push(sample);

            self.app_buckets.accumulate(
                bucket_timestamp(snapshot_ts),
                &current.app_id,
                &current.process_name,
                current.executable_path.as_deref(),
                delta_received,
                delta_sent,
            );
        }

        // An application present in the previous snapshot but missing now has
        // disappeared: iterating only the current snapshot means no sample is
        // generated for it, so process churn cannot create fake traffic.

        self.previous = Some(snapshot);

        Ok(produced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::types::{ApplicationNetworkUsage, NetworkInterface};

    fn interface(id: &str, bytes_received: u64, bytes_sent: u64) -> NetworkInterface {
        NetworkInterface {
            id: id.to_string(),
            name: id.to_string(),
            bytes_received,
            bytes_sent,
            is_up: true,
        }
    }

    /// Application fixture: identity is the executable path; process_name is
    /// display metadata; executable_path preserves the resolved executable.
    fn app_usage(app_id: &str, bytes_received: u64, bytes_sent: u64) -> ApplicationNetworkUsage {
        ApplicationNetworkUsage {
            app_id: app_id.to_string(),
            process_name: app_id.to_string(),
            executable_path: Some(app_id.to_string()),
            bytes_received,
            bytes_sent,
        }
    }

    fn snapshot(ts: i64, interfaces: Vec<NetworkInterface>) -> NetworkSnapshot {
        NetworkSnapshot {
            timestamp: ts,
            interfaces,
            applications: vec![],
        }
    }

    fn snapshot_with_apps(
        ts: i64,
        interfaces: Vec<NetworkInterface>,
        applications: Vec<ApplicationNetworkUsage>,
    ) -> NetworkSnapshot {
        NetworkSnapshot {
            timestamp: ts,
            interfaces,
            applications,
        }
    }

    /// A fake fetch that yields the provided snapshot, returning a boxed future
    /// so the closure's return type is concrete and nameable.
    fn fetch(
        snapshot: NetworkSnapshot,
    ) -> impl FnOnce() -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<NetworkSnapshot, NetworkError>> + Send>,
    > + Send {
        move || {
            let snapshot = snapshot.clone();
            Box::pin(async move { Ok(snapshot) })
        }
    }

    /// A persist recorder capturing each completed batch.
    struct PersistRecorder {
        batches: Vec<RollupBatch>,
    }

    impl PersistRecorder {
        fn new() -> Self {
            Self {
                batches: Vec::new(),
            }
        }

        fn call(
            &mut self,
            batch: RollupBatch,
        ) -> impl std::future::Future<Output = Result<(), NetworkError>> + Send {
            self.batches.push(batch);
            async { Ok(()) }
        }
    }

    /// Drives `sample_once` with the given snapshots in order, returning the
    /// produced interface samples across all calls plus the interface half of
    /// each persisted batch.
    async fn drive(
        snapshots: Vec<NetworkSnapshot>,
    ) -> (Vec<NetworkSample>, Vec<Vec<NetworkRollup>>) {
        let (result, batches) = drive_full(snapshots).await;

        let interface_batches = batches
            .iter()
            .map(|batch| batch.interfaces.clone())
            .collect();

        (result.interfaces, interface_batches)
    }

    /// Drives `sample_once` returning everything: combined samples plus the
    /// full `RollupBatch` history (interface + application halves).
    async fn drive_full(snapshots: Vec<NetworkSnapshot>) -> (SampleResult, Vec<RollupBatch>) {
        let mut sampler = NetworkSampler::new();
        let mut recorder = PersistRecorder::new();
        let mut all = SampleResult {
            interfaces: Vec::new(),
            applications: Vec::new(),
        };

        for snap in snapshots {
            let produced = sampler
                .sample_once(fetch(snap), &mut |batch| recorder.call(batch))
                .await
                .unwrap();
            all.interfaces.extend(produced.interfaces);
            all.applications.extend(produced.applications);
        }

        (all, recorder.batches)
    }

    #[test]
    fn delta_calculation_matches_spec() {
        assert_eq!(calculate_delta(100, 150), Some(50));
        assert_eq!(calculate_delta(0, 100), Some(100));
        assert_eq!(calculate_delta(100, 100), Some(0));
        assert_eq!(calculate_delta(150, 100), None);
    }

    #[test]
    fn bucket_timestamp_is_minute_start() {
        assert_eq!(bucket_timestamp(0), 0);
        assert_eq!(bucket_timestamp(1), 0);
        assert_eq!(bucket_timestamp(59), 0);
        assert_eq!(bucket_timestamp(60), 60);
        assert_eq!(bucket_timestamp(61), 60);
        assert_eq!(bucket_timestamp(119), 60);
        assert_eq!(bucket_timestamp(120), 120);
    }

    #[test]
    fn ring_buffer_stores_and_bounds_samples() {
        let mut ring = SampleRingBuffer::new(RING_BUFFER_CAPACITY);

        for i in 0..RING_BUFFER_CAPACITY {
            ring.push(NetworkSample {
                timestamp: i as i64,
                interface_id: "eth0".to_string(),
                bytes_received: i as u64,
                bytes_sent: 0,
            });
        }

        assert_eq!(ring.len(), RING_BUFFER_CAPACITY);
        assert_eq!(
            ring.latest().unwrap().timestamp,
            (RING_BUFFER_CAPACITY - 1) as i64
        );

        ring.push(NetworkSample {
            timestamp: RING_BUFFER_CAPACITY as i64,
            interface_id: "eth0".to_string(),
            bytes_received: 0,
            bytes_sent: 0,
        });

        assert_eq!(ring.len(), RING_BUFFER_CAPACITY, "must stay bounded");
        assert_eq!(
            ring.latest().unwrap().timestamp,
            RING_BUFFER_CAPACITY as i64
        );

        let recent = ring.recent(5);
        assert_eq!(recent.len(), 5);
        assert!(recent
            .iter()
            .all(|s| s.timestamp > (RING_BUFFER_CAPACITY as i64 - 5)));
    }

    #[tokio::test]
    async fn first_snapshot_establishes_baseline_only() {
        let (samples, batches) = drive(vec![snapshot(
            0,
            vec![interface("eth0", 1_000_000, 500_000)],
        )])
        .await;

        assert!(
            samples.is_empty(),
            "first snapshot must not produce traffic"
        );
        assert!(batches.is_empty(), "no rollup before a minute boundary");
    }

    #[tokio::test]
    async fn second_snapshot_produces_rx_and_tx_deltas() {
        let (samples, _batches) = drive(vec![
            snapshot(0, vec![interface("eth0", 1_000_000, 500_000)]),
            snapshot(1, vec![interface("eth0", 1_250_000, 750_000)]),
        ])
        .await;

        assert_eq!(samples.len(), 1);
        let sample = &samples[0];
        assert_eq!(sample.interface_id, "eth0");
        assert_eq!(sample.bytes_received, 250_000, "RX delta");
        assert_eq!(sample.bytes_sent, 250_000, "TX delta");
        assert_eq!(sample.timestamp, 1);
    }

    #[tokio::test]
    async fn interfaces_are_handled_independently() {
        let (samples, _batches) = drive(vec![
            snapshot(
                0,
                vec![interface("eth0", 100, 50), interface("wlan0", 10, 5)],
            ),
            snapshot(
                1,
                vec![interface("eth0", 150, 70), interface("wlan0", 40, 20)],
            ),
        ])
        .await;

        assert_eq!(samples.len(), 2);

        let eth0 = samples.iter().find(|s| s.interface_id == "eth0").unwrap();
        let wlan0 = samples.iter().find(|s| s.interface_id == "wlan0").unwrap();

        assert_eq!(eth0.bytes_received, 50);
        assert_eq!(eth0.bytes_sent, 20);
        assert_eq!(wlan0.bytes_received, 30);
        assert_eq!(wlan0.bytes_sent, 15);
    }

    #[tokio::test]
    async fn counter_reset_does_not_create_bogus_traffic() {
        // eth0's RX counter reset (100 -> 50), so eth0 yields no sample.
        // wlan0 continues as normal.
        let (samples, _batches) = drive(vec![
            snapshot(
                0,
                vec![interface("eth0", 100, 50), interface("wlan0", 10, 5)],
            ),
            snapshot(
                1,
                vec![interface("eth0", 50, 60), interface("wlan0", 20, 10)],
            ),
        ])
        .await;

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].interface_id, "wlan0");
    }

    #[tokio::test]
    async fn zero_delta_is_valid_and_recorded() {
        let (samples, _batches) = drive(vec![
            snapshot(0, vec![interface("eth0", 100, 50)]),
            snapshot(1, vec![interface("eth0", 100, 50)]),
        ])
        .await;

        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].bytes_received, 0);
        assert_eq!(samples[0].bytes_sent, 0);
    }

    #[tokio::test]
    async fn samples_enter_ring_buffer_via_sampler() {
        let mut sampler = NetworkSampler::new();
        let mut recorder = PersistRecorder::new();

        sampler
            .sample_once(
                fetch(snapshot(0, vec![interface("eth0", 100, 50)])),
                &mut |b| recorder.call(b),
            )
            .await
            .unwrap();

        sampler
            .sample_once(
                fetch(snapshot(1, vec![interface("eth0", 110, 60)])),
                &mut |b| recorder.call(b),
            )
            .await
            .unwrap();

        assert_eq!(sampler.samples().len(), 1);
        assert_eq!(sampler.samples().latest().unwrap().bytes_received, 10);
    }

    #[tokio::test]
    async fn sixty_seconds_produce_one_rollup_bucket() {
        let mut snapshots = Vec::new();

        // Baseline at t=0, then samples t=1..=59 in the same minute (bucket 0).
        snapshots.push(snapshot(0, vec![interface("eth0", 0, 0)]));

        for ts in 1..=59 {
            snapshots.push(snapshot(
                ts,
                vec![interface("eth0", (ts * 10) as u64, (ts * 5) as u64)],
            ));
        }

        // First boundary crossing: t=60 lands in bucket 60, completing bucket 0.
        snapshots.push(snapshot(60, vec![interface("eth0", 600, 300)]));

        let (samples, batches) = drive(snapshots).await;

        // 60 samples: the 59 deltas from t=1..=59 (bucket 0) plus the t=60
        // delta which starts bucket 60.
        assert_eq!(samples.len(), 60);

        assert_eq!(batches.len(), 1, "one complete bucket persisted once");

        let batch = &batches[0];
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].ts, 0, "rollup uses the bucket start timestamp");
        assert_eq!(batch[0].interface_id, "eth0");

        // Each of the 59 sampled intervals produced an RX delta of exactly 10
        // (counters step ts*10) and a TX delta of exactly 5 (counters step
        // ts*5), accumulated across 59 intervals in bucket 0.
        assert_eq!(batch[0].bytes_received, 59 * 10);
        assert_eq!(batch[0].bytes_sent, 59 * 5);

        // The t=60 sample's own delta (600-590=10) is NOT in the bucket 0
        // rollup; it accumulates into the new bucket 60 (persisted at t=120).
        assert_eq!(samples[samples.len() - 1].bytes_received, 10);
        assert_eq!(samples[samples.len() - 1].bytes_sent, 5);
    }

    #[tokio::test]
    async fn multiple_interfaces_produce_separate_rollups_in_one_batch() {
        let (_samples, batches) = drive(vec![
            snapshot(0, vec![interface("eth0", 0, 0), interface("wlan0", 0, 0)]),
            snapshot(
                1,
                vec![interface("eth0", 10, 5), interface("wlan0", 20, 10)],
            ),
            snapshot(
                60,
                vec![interface("eth0", 70, 35), interface("wlan0", 80, 40)],
            ),
        ])
        .await;

        let batch = &batches[0];
        assert_eq!(batch.len(), 2, "two interfaces => two rows in ONE batch");
        assert_eq!(batch[0].ts, 0);
        assert_eq!(batch[1].ts, 0);

        let eth0 = batch.iter().find(|r| r.interface_id == "eth0").unwrap();
        let wlan0 = batch.iter().find(|r| r.interface_id == "wlan0").unwrap();

        assert_eq!(eth0.bytes_received, 10);
        assert_eq!(eth0.bytes_sent, 5);
        assert_eq!(wlan0.bytes_received, 20);
        assert_eq!(wlan0.bytes_sent, 10);
    }

    #[tokio::test]
    async fn partial_bucket_is_not_prematurely_persisted() {
        // 30 samples in the first minute; no boundary crossed yet.
        let mut snapshots = vec![snapshot(0, vec![interface("eth0", 0, 0)])];

        for ts in 1..=30 {
            snapshots.push(snapshot(ts, vec![interface("eth0", ts as u64, ts as u64)]));
        }

        let (_samples, batches) = drive(snapshots).await;

        assert!(batches.is_empty(), "partial bucket must not persist");
    }

    #[tokio::test]
    async fn counter_reset_at_boundary_still_persists_completed_bucket() {
        // eth0 receives traffic in bucket 0, then its counter resets at t=60
        // so that sample produces no delta. The boundary check runs before the
        // interface loop, so bucket 0 must still persist.
        let (_samples, batches) = drive(vec![
            snapshot(0, vec![interface("eth0", 0, 0)]),
            snapshot(1, vec![interface("eth0", 10, 5)]),
            snapshot(60, vec![interface("eth0", 2, 2)]), // reset: 10->2
        ])
        .await;

        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[0][0].ts, 0);
        assert_eq!(batches[0][0].bytes_received, 10);
        assert_eq!(batches[0][0].bytes_sent, 5);
    }

    // ---- Application telemetry (Phase 6.6) ----

    #[tokio::test]
    async fn application_first_observation_establishes_baseline() {
        let (result, batches) = drive_full(vec![snapshot_with_apps(
            0,
            vec![interface("eth0", 0, 0)],
            vec![app_usage("/usr/bin/firefox", 10_000_000, 5_000_000)],
        )])
        .await;

        assert!(
            result.applications.is_empty(),
            "first observation establishes the baseline; zero attributed bytes"
        );
        assert!(batches.is_empty());
    }

    #[tokio::test]
    async fn application_delta_calculated_correctly() {
        let (result, _batches) = drive_full(vec![
            snapshot_with_apps(
                0,
                vec![],
                vec![app_usage("/usr/bin/firefox", 10_000_000, 5_000_000)],
            ),
            snapshot_with_apps(
                1,
                vec![],
                vec![app_usage("/usr/bin/firefox", 10_500_000, 5_200_000)],
            ),
        ])
        .await;

        assert_eq!(result.applications.len(), 1);
        let sample = &result.applications[0];
        assert_eq!(sample.app_id, "/usr/bin/firefox");
        assert_eq!(sample.bytes_received, 500_000, "app RX delta");
        assert_eq!(sample.bytes_sent, 200_000, "app TX delta");
        assert_eq!(sample.timestamp, 1);
        assert_eq!(sample.process_name, "/usr/bin/firefox");
        assert_eq!(sample.executable_path.as_deref(), Some("/usr/bin/firefox"));
    }

    #[tokio::test]
    async fn multiple_applications_tracked_independently() {
        let (result, _batches) = drive_full(vec![
            snapshot_with_apps(
                0,
                vec![],
                vec![
                    app_usage("/usr/bin/a", 100, 50),
                    app_usage("/usr/bin/b", 10, 5),
                ],
            ),
            snapshot_with_apps(
                1,
                vec![],
                vec![
                    app_usage("/usr/bin/a", 200, 100),
                    app_usage("/usr/bin/b", 30, 15),
                ],
            ),
        ])
        .await;

        assert_eq!(result.applications.len(), 2);

        let a = result
            .applications
            .iter()
            .find(|s| s.app_id == "/usr/bin/a")
            .unwrap();
        let b = result
            .applications
            .iter()
            .find(|s| s.app_id == "/usr/bin/b")
            .unwrap();

        assert_eq!(a.bytes_received, 100);
        assert_eq!(a.bytes_sent, 50);
        assert_eq!(b.bytes_received, 20);
        assert_eq!(b.bytes_sent, 10);
    }

    #[tokio::test]
    async fn new_process_establishes_baseline_without_fake_traffic() {
        // Firefox observed from t=0; Chrome appears at t=1. Chrome's first
        // observation must produce zero attributed bytes while Firefox keeps
        // producing real deltas.
        let (result, _batches) = drive_full(vec![
            snapshot_with_apps(0, vec![], vec![app_usage("/usr/bin/firefox", 100, 50)]),
            snapshot_with_apps(
                1,
                vec![],
                vec![
                    app_usage("/usr/bin/firefox", 200, 100),
                    app_usage("/usr/bin/chrome", 5000, 2500),
                ],
            ),
        ])
        .await;

        assert_eq!(result.applications.len(), 1, "chrome baseline is silent");
        assert_eq!(result.applications[0].app_id, "/usr/bin/firefox");
        assert_eq!(result.applications[0].bytes_received, 100);
    }

    #[tokio::test]
    async fn disappeared_process_produces_no_bogus_delta() {
        // t=10: firefox only. t=11: chrome appears. t=12: chrome disappears.
        let (result, _batches) = drive_full(vec![
            snapshot_with_apps(10, vec![], vec![app_usage("/usr/bin/firefox", 100, 50)]),
            snapshot_with_apps(
                11,
                vec![],
                vec![
                    app_usage("/usr/bin/firefox", 110, 55),
                    app_usage("/usr/bin/chrome", 2000, 1000),
                ],
            ),
            snapshot_with_apps(12, vec![], vec![app_usage("/usr/bin/firefox", 120, 60)]),
        ])
        .await;

        let chrome_samples: Vec<&AppSample> = result
            .applications
            .iter()
            .filter(|s| s.app_id == "/usr/bin/chrome")
            .collect();

        assert!(
            chrome_samples.is_empty(),
            "a disappeared process must never produce a sample"
        );

        let firefox: Vec<&AppSample> = result
            .applications
            .iter()
            .filter(|s| s.app_id == "/usr/bin/firefox")
            .collect();
        assert_eq!(firefox.len(), 2, "firefox keeps producing deltas");
        assert_eq!(firefox[1].bytes_received, 10);
    }

    #[tokio::test]
    async fn application_counter_reset_is_ignored() {
        // The application's cumulative counter dropped (process restarted
        // with fresh TCP_INFO counters): no sample, no bogus traffic.
        let (result, _batches) = drive_full(vec![
            snapshot_with_apps(0, vec![], vec![app_usage("/usr/bin/app", 1000, 500)]),
            snapshot_with_apps(1, vec![], vec![app_usage("/usr/bin/app", 50, 20)]),
        ])
        .await;

        assert!(
            result.applications.is_empty(),
            "counter reset must not create bogus traffic"
        );
    }

    #[tokio::test]
    async fn application_samples_enter_bounded_ring_buffer() {
        let mut sampler = NetworkSampler::new();
        let mut recorder = PersistRecorder::new();

        for i in 0..=APPLICATION_RING_BUFFER_CAPACITY {
            let snap = snapshot_with_apps(
                i as i64,
                vec![],
                vec![app_usage("/usr/bin/app", i as u64, 0)],
            );

            sampler
                .sample_once(fetch(snap), &mut |b| recorder.call(b))
                .await
                .unwrap();
        }

        assert_eq!(
            sampler.app_samples().len(),
            APPLICATION_RING_BUFFER_CAPACITY,
            "application ring buffer must stay bounded"
        );
        assert_eq!(
            sampler.app_samples().latest().unwrap().timestamp,
            APPLICATION_RING_BUFFER_CAPACITY as i64
        );
    }

    #[tokio::test]
    async fn sixty_second_application_rollup_is_correct() {
        let mut snapshots = vec![snapshot_with_apps(
            0,
            vec![],
            vec![app_usage("/usr/bin/app", 0, 0)],
        )];

        for ts in 1..=60 {
            snapshots.push(snapshot_with_apps(
                ts,
                vec![],
                vec![app_usage("/usr/bin/app", (ts * 10) as u64, (ts * 5) as u64)],
            ));
        }

        let (result, batches) = drive_full(snapshots).await;

        assert_eq!(result.applications.len(), 60);
        assert_eq!(batches.len(), 1, "one complete bucket persisted once");

        let batch = &batches[0];
        assert_eq!(batch.applications.len(), 1);
        assert_eq!(batch.applications[0].ts, 0, "bucket start timestamp");
        assert_eq!(batch.applications[0].app_id, "/usr/bin/app");
        assert_eq!(batch.applications[0].bytes_received, 59 * 10);
        assert_eq!(batch.applications[0].bytes_sent, 59 * 5);
        assert_eq!(batch.applications[0].process_name, "/usr/bin/app");
        assert_eq!(
            batch.applications[0].executable_path.as_deref(),
            Some("/usr/bin/app")
        );
    }

    #[tokio::test]
    async fn interface_and_application_rollups_commit_atomically() {
        // One tick crossing the boundary with BOTH streams active must emit
        // one RollupBatch carrying both halves: the single persist call is
        // the atomic commit unit — minute 0 is either fully written or fully
        // absent.
        let (result, batches) = drive_full(vec![
            snapshot_with_apps(
                0,
                vec![interface("eth0", 0, 0)],
                vec![app_usage("/usr/bin/app", 0, 0)],
            ),
            snapshot_with_apps(
                1,
                vec![interface("eth0", 10, 5)],
                vec![app_usage("/usr/bin/app", 100, 50)],
            ),
            snapshot_with_apps(
                60,
                vec![interface("eth0", 20, 10)],
                vec![app_usage("/usr/bin/app", 200, 100)],
            ),
        ])
        .await;

        assert_eq!(result.interfaces.len(), 2);
        assert_eq!(result.applications.len(), 2);

        assert_eq!(batches.len(), 1, "ONE persist call at the boundary");
        let batch = &batches[0];
        assert_eq!(batch.interfaces.len(), 1, "interface half present");
        assert_eq!(batch.applications.len(), 1, "application half present");
        assert_eq!(batch.interfaces[0].ts, 0);
        assert_eq!(batch.applications[0].ts, 0);
    }

    #[tokio::test]
    async fn empty_applications_array_is_valid() {
        // No attributable TCP processes does NOT mean the network provider
        // failed: the snapshot is still valid and interface telemetry
        // continues unaffected.
        let (result, batches) = drive_full(vec![
            snapshot_with_apps(0, vec![interface("eth0", 0, 0)], vec![]),
            snapshot_with_apps(1, vec![interface("eth0", 10, 5)], vec![]),
        ])
        .await;

        assert!(result.applications.is_empty());
        assert_eq!(result.interfaces.len(), 1);
        assert_eq!(result.interfaces[0].bytes_received, 10);
        assert!(batches.is_empty());
    }
}
