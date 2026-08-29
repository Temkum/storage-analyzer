use std::collections::{HashMap, VecDeque};
use std::future::Future;

use super::sidecar::NetworkError;
use super::types::NetworkSnapshot;
use crate::storage::network_rollup_repository::NetworkRollup;

/// Total number of 1-second raw samples retained per interface for the live
/// UI: 10 minutes of history at one sample/second. The buffer is strictly for
/// the live chart; SQLite owns persistent history.
pub const RING_BUFFER_CAPACITY: usize = 600;

/// Number of 1-second samples that make up one persisted rollup interval.
pub const ROLLUP_INTERVAL_SECONDS: i64 = 60;

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

/// A bounded ring buffer of raw samples ([`NetworkSample`]) for the live UI.
///
/// When at capacity, appending evicts the oldest sample. The evicted sample's
/// rollup is already persisted, so the raw value is safe to discard.
#[derive(Debug)]
pub struct SampleRingBuffer {
    buffer: VecDeque<NetworkSample>,
    capacity: usize,
}

impl SampleRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Appends a sample, evicting the oldest when at capacity.
    fn push(&mut self, sample: NetworkSample) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }

        self.buffer.push_back(sample);
    }

    /// Most recent sample, if any.
    fn latest(&self) -> Option<&NetworkSample> {
        self.buffer.back()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Samples whose timestamp falls strictly inside the trailing
    /// `duration_seconds` window measured from the latest sample.
    fn recent(&self, duration_seconds: i64) -> Vec<&NetworkSample> {
        let Some(latest_ts) = self.latest().map(|sample| sample.timestamp) else {
            return Vec::new();
        };

        let cutoff = latest_ts - duration_seconds;

        self.buffer
            .iter()
            .filter(move |sample| sample.timestamp > cutoff)
            .collect()
    }
}

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
/// Folds a sequence of C++ network snapshots into persisted 1-minute rollups,
/// retaining raw 1-second samples for the live UI. Kept deterministic and
/// timer-free: [`NetworkSampler::sample_once`] performs exactly one
/// snapshot → delta → buffer → rollup operation, decoupled from SQLite
/// through the `persist` closure. The scheduler (Step 5.9) becomes a thin
/// wrapper around `sample_once`.
///
/// The C++ sidecar reports cumulative OS counters; this Rust layer alone
/// converts them into per-interval deltas. Rate/delta math is never pushed
/// back into the C++ provider.
pub struct NetworkSampler {
    previous: Option<NetworkSnapshot>,
    ring_buffer: SampleRingBuffer,
    buckets: Buckets,
}

impl NetworkSampler {
    pub fn new() -> Self {
        Self {
            previous: None,
            ring_buffer: SampleRingBuffer::new(RING_BUFFER_CAPACITY),
            buckets: Buckets::new(),
        }
    }

    /// Exposes the raw-sample ring buffer for the live UI.
    pub fn samples(&self) -> &SampleRingBuffer {
        &self.ring_buffer
    }

    /// Performs exactly one snapshot → delta → buffer → rollup operation.
    ///
    /// Returns the raw samples produced by this single call (interfaces with
    /// a reset counter or no prior baseline yield no sample). When the fetch
    /// crosses a minute boundary, the completed rollup bucket is handed to
    /// `persist` as a single batch for one SQLite transaction.
    pub async fn sample_once<F, FutFetch, FutPersist>(
        &mut self,
        fetch: impl FnOnce() -> FutFetch,
        persist: &mut F,
    ) -> Result<Vec<NetworkSample>, NetworkError>
    where
        F: FnMut(Vec<NetworkRollup>) -> FutPersist + Send,
        FutFetch: Future<Output = Result<NetworkSnapshot, NetworkError>> + Send,
        FutPersist: Future<Output = Result<(), NetworkError>> + Send,
    {
        let snapshot = fetch().await?;

        let mut produced = Vec::new();

        let Some(previous) = self.previous.take() else {
            // First snapshot establishes the baseline; no traffic to report.
            self.previous = Some(snapshot);
            return Ok(produced);
        };

        let snapshot_ts = snapshot.timestamp;

        // A minute boundary completes the in-progress bucket first. Checked
        // once per snapshot, before the interface loop, so the completed
        // batch is persisted even when no interface produced a delta this
        // interval (e.g. every counter reset at the boundary).
        if self.buckets.is_complete_for(snapshot_ts) {
            let completed_ts = self.buckets.ts.expect("ts set when complete");

            let batch = self.buckets.drain(completed_ts);

            if !batch.is_empty() {
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
            produced.push(sample);

            self.buckets.accumulate(
                bucket_timestamp(snapshot_ts),
                &current.id,
                delta_received,
                delta_sent,
            );
        }

        self.previous = Some(snapshot);

        Ok(produced)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::types::NetworkInterface;

    fn interface(id: &str, bytes_received: u64, bytes_sent: u64) -> NetworkInterface {
        NetworkInterface {
            id: id.to_string(),
            name: id.to_string(),
            bytes_received,
            bytes_sent,
            is_up: true,
        }
    }

    fn snapshot(ts: i64, interfaces: Vec<NetworkInterface>) -> NetworkSnapshot {
        NetworkSnapshot {
            timestamp: ts,
            interfaces,
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
        batches: Vec<Vec<NetworkRollup>>,
    }

    impl PersistRecorder {
        fn new() -> Self {
            Self {
                batches: Vec::new(),
            }
        }

        fn call(
            &mut self,
            batch: Vec<NetworkRollup>,
        ) -> impl std::future::Future<Output = Result<(), NetworkError>> + Send {
            self.batches.push(batch);
            async { Ok(()) }
        }
    }

    /// Drives `sample_once` with the given snapshots in order, returning the
    /// produced samples across all calls plus the persisted batches.
    async fn drive(
        snapshots: Vec<NetworkSnapshot>,
    ) -> (Vec<NetworkSample>, Vec<Vec<NetworkRollup>>) {
        let mut sampler = NetworkSampler::new();
        let mut recorder = PersistRecorder::new();
        let mut all_samples = Vec::new();

        for snap in snapshots {
            let produced = sampler
                .sample_once(fetch(snap), &mut |batch| recorder.call(batch))
                .await
                .unwrap();
            all_samples.extend(produced);
        }

        (all_samples, recorder.batches)
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
}
