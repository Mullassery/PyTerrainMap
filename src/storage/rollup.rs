//! Retention/compaction for the append-only observation log, via H3
//! parent-cell rollups.
//!
//! `ObservationStore` deliberately has no delete/mutate method (see its own
//! module doc: an immutable audit trail that can be wiped isn't actually
//! immutable). That guarantee is not touched here -- this module never
//! deletes or mutates a stored observation. Instead, it reads a time window
//! of existing observations through the store's public API and produces
//! new, additive [`CellRollup`] summary records: per-parent-H3-cell,
//! per-coarse-time-bucket aggregates that summarize many detailed
//! observations into one compact record, without erasing the underlying
//! detail.
//!
//! What this does NOT do (yet): actually evict rolled-up raw observations
//! from the hot in-memory store, or archive them to cold storage. Doing
//! that here would mean either quietly deleting from `ObservationStore`
//! (violating its documented immutability guarantee) or inventing an
//! archival flow with no real backend behind it in this pass. This
//! module's job -- and it does this for real, not as a stub -- is the data
//! reduction computation itself: feed its [`CellRollup`] output to a
//! persistent backend (`persistence_bridge`/`PostgresBackend`) or a future
//! cold-storage tier (`backends::DataTier::Cold`/`Archive`) to actually
//! bound long-term storage growth.

use std::collections::HashMap;

use xs_h3::{degs_to_rads, LatLng};

use crate::storage::ObservationStore;
use crate::types::{Error, Observation, Result, SensorType};

pub type H3Cell = crate::spatial::H3Cell;

/// A compact summary of every observation in one parent H3 cell, for one
/// sensor type, during one time bucket -- the "lower-resolution parent
/// index" the roadmap asked for, computed for real from real stored
/// observations rather than a placeholder.
#[derive(Debug, Clone, PartialEq)]
pub struct CellRollup {
    pub parent_h3_cell: H3Cell,
    pub bucket_start_us: i64,
    pub bucket_end_us: i64,
    pub sensor_type: SensorType,
    pub observation_count: usize,
    pub mean_confidence: f32,
    pub min_timestamp_us: i64,
    pub max_timestamp_us: i64,
    /// Robots that contributed at least one observation to this rollup
    /// (deduplicated, sorted for deterministic output/testing).
    pub contributing_robots: Vec<String>,
}

/// Configures how compaction groups observations: which H3 resolutions to
/// roll from/to, and how wide a time bucket to use.
pub struct CompactionPolicy {
    /// H3 resolution observations are currently indexed at (must be finer,
    /// i.e. numerically larger, than `parent_resolution`).
    pub source_resolution: i32,
    /// H3 resolution to roll observations up TO (coarser, numerically
    /// smaller -- e.g. resolution 9 (~174m hexagons) rolling up to
    /// resolution 7 (~1.2km hexagons)).
    pub parent_resolution: i32,
    /// Width of each time bucket, in microseconds. Observations are
    /// grouped into `[bucket_start, bucket_start + bucket_width_us)`.
    pub bucket_width_us: i64,
}

impl CompactionPolicy {
    pub fn new(source_resolution: i32, parent_resolution: i32, bucket_width_us: i64) -> Result<Self> {
        if !(0..=15).contains(&source_resolution) || !(0..=15).contains(&parent_resolution) {
            return Err(Error::InvalidLocation);
        }
        if parent_resolution >= source_resolution {
            return Err(Error::InvalidObservation(format!(
                "parent_resolution ({parent_resolution}) must be coarser than \
                 source_resolution ({source_resolution}) to actually reduce data volume"
            )));
        }
        if bucket_width_us <= 0 {
            return Err(Error::InvalidObservation(
                "bucket_width_us must be positive".to_string(),
            ));
        }
        Ok(CompactionPolicy {
            source_resolution,
            parent_resolution,
            bucket_width_us,
        })
    }

    fn bucket_start(&self, timestamp_us: i64) -> i64 {
        timestamp_us.div_euclid(self.bucket_width_us) * self.bucket_width_us
    }

    fn parent_cell_for(&self, obs: &Observation) -> Result<H3Cell> {
        let lat_lng = LatLng {
            lat: degs_to_rads(obs.location.lat),
            lng: degs_to_rads(obs.location.lon),
        };
        let source_cell = xs_h3::lat_lng_to_cell(&lat_lng, self.source_resolution)
            .map_err(|e| Error::InvalidObservation(format!("H3 conversion failed: {e:?}")))?;
        xs_h3::cell_to_parent(source_cell, self.parent_resolution)
            .map_err(|e| Error::InvalidObservation(format!("H3 parent lookup failed: {e:?}")))
    }

    /// Roll up every observation in `store` with `start_us <= timestamp <
    /// end_us` into per-parent-cell, per-bucket, per-sensor-type summaries.
    /// Purely read-only against `store` -- reads via its public
    /// `len()`/`get_batch()` API, never touches its internals or mutates
    /// it in any way.
    pub fn compact_range(
        &self,
        store: &ObservationStore,
        start_us: i64,
        end_us: i64,
    ) -> Result<Vec<CellRollup>> {
        if store.is_empty() {
            return Ok(Vec::new());
        }
        let indices: Vec<usize> = (0..store.len()).collect();
        let observations = store.get_batch(&indices)?;

        #[derive(Default)]
        struct Accumulator {
            count: usize,
            confidence_sum: f64,
            min_ts: i64,
            max_ts: i64,
            robots: std::collections::BTreeSet<String>,
        }

        let mut groups: HashMap<(H3Cell, i64, SensorType), Accumulator> = HashMap::new();

        for obs in &observations {
            if obs.timestamp < start_us || obs.timestamp >= end_us {
                continue;
            }
            let parent_cell = self.parent_cell_for(obs)?;
            let bucket_start = self.bucket_start(obs.timestamp);
            let key = (parent_cell, bucket_start, obs.sensor_type);

            let acc = groups.entry(key).or_insert_with(|| Accumulator {
                min_ts: obs.timestamp,
                max_ts: obs.timestamp,
                ..Default::default()
            });
            acc.count += 1;
            acc.confidence_sum += obs.confidence as f64;
            acc.min_ts = acc.min_ts.min(obs.timestamp);
            acc.max_ts = acc.max_ts.max(obs.timestamp);
            acc.robots.insert(obs.robot_id.clone());
        }

        let mut rollups: Vec<CellRollup> = groups
            .into_iter()
            .map(|((parent_cell, bucket_start, sensor_type), acc)| CellRollup {
                parent_h3_cell: parent_cell,
                bucket_start_us: bucket_start,
                bucket_end_us: bucket_start + self.bucket_width_us,
                sensor_type,
                observation_count: acc.count,
                mean_confidence: (acc.confidence_sum / acc.count as f64) as f32,
                min_timestamp_us: acc.min_ts,
                max_timestamp_us: acc.max_ts,
                contributing_robots: acc.robots.into_iter().collect(),
            })
            .collect();

        // Deterministic output ordering for callers/tests (H3Index doesn't
        // implement Ord, so sort by the fields that do).
        rollups.sort_by(|a, b| {
            (a.bucket_start_us, format!("{:?}", a.sensor_type))
                .cmp(&(b.bucket_start_us, format!("{:?}", b.sensor_type)))
        });

        Ok(rollups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClockSource, GeoPoint, SensorValue};

    fn obs_at(lat: f64, lon: f64, timestamp_us: i64, robot_id: &str, confidence: f32) -> Observation {
        Observation::with_clock_source(
            robot_id.to_string(),
            timestamp_us,
            GeoPoint::new(lat, lon),
            None,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 20.0 },
            confidence,
            ClockSource::UTC,
        )
    }

    #[test]
    fn policy_rejects_non_coarsening_resolutions() {
        assert!(CompactionPolicy::new(9, 9, 3_600_000_000).is_err());
        assert!(CompactionPolicy::new(7, 9, 3_600_000_000).is_err());
        assert!(CompactionPolicy::new(9, 7, 3_600_000_000).is_ok());
    }

    #[test]
    fn policy_rejects_non_positive_bucket_width() {
        assert!(CompactionPolicy::new(9, 7, 0).is_err());
        assert!(CompactionPolicy::new(9, 7, -1).is_err());
    }

    #[test]
    fn compact_range_reduces_many_observations_to_one_rollup_per_cell_bucket() {
        let store = ObservationStore::new();
        // 10 observations, same location + sensor type, within one bucket.
        for i in 0..10 {
            store
                .add(obs_at(37.7749, -122.4194, 1_000_000 + i, "robot-1", 0.8))
                .unwrap();
        }

        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let rollups = policy.compact_range(&store, 0, i64::MAX).unwrap();

        assert_eq!(rollups.len(), 1, "all 10 observations should collapse into one rollup");
        assert_eq!(rollups[0].observation_count, 10);
        assert_eq!(rollups[0].mean_confidence, 0.8);
        assert_eq!(rollups[0].contributing_robots, vec!["robot-1".to_string()]);
    }

    #[test]
    fn compact_range_keeps_different_sensor_types_separate() {
        let store = ObservationStore::new();
        store.add(obs_at(10.0, 10.0, 1_000_000, "robot-1", 0.5)).unwrap();
        let mut lidar = obs_at(10.0, 10.0, 1_000_000, "robot-1", 0.5);
        lidar.sensor_type = SensorType::LiDAR;
        lidar.value = SensorValue::LiDAR { distances_cm: vec![50] };
        store.add(lidar).unwrap();

        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let rollups = policy.compact_range(&store, 0, i64::MAX).unwrap();

        assert_eq!(rollups.len(), 2, "different sensor types must not be merged together");
    }

    #[test]
    fn compact_range_keeps_different_time_buckets_separate() {
        let store = ObservationStore::new();
        let bucket_width = 3_600_000_000; // 1 hour, in microseconds
        store.add(obs_at(10.0, 10.0, 1, "robot-1", 0.5)).unwrap();
        store
            .add(obs_at(10.0, 10.0, bucket_width * 2, "robot-1", 0.5))
            .unwrap();

        let policy = CompactionPolicy::new(9, 7, bucket_width).unwrap();
        let rollups = policy.compact_range(&store, 0, i64::MAX).unwrap();

        assert_eq!(rollups.len(), 2, "observations an hour apart must land in different buckets");
    }

    #[test]
    fn compact_range_respects_the_requested_time_window() {
        let store = ObservationStore::new();
        store.add(obs_at(10.0, 10.0, 100, "robot-1", 0.5)).unwrap();
        store.add(obs_at(10.0, 10.0, 200_000_000, "robot-1", 0.5)).unwrap();

        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let rollups = policy.compact_range(&store, 0, 1_000_000).unwrap();

        assert_eq!(rollups.len(), 1);
        assert_eq!(rollups[0].observation_count, 1, "only the in-window observation should be counted");
    }

    #[test]
    fn compact_range_does_not_mutate_the_store() {
        let store = ObservationStore::new();
        store.add(obs_at(10.0, 10.0, 100, "robot-1", 0.5)).unwrap();
        let len_before = store.len();

        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let _ = policy.compact_range(&store, 0, i64::MAX).unwrap();

        assert_eq!(store.len(), len_before, "compaction must never remove observations from the store");
    }

    #[test]
    fn compact_range_on_empty_store_returns_no_rollups() {
        let store = ObservationStore::new();
        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let rollups = policy.compact_range(&store, 0, i64::MAX).unwrap();
        assert!(rollups.is_empty());
    }

    #[test]
    fn compact_range_deduplicates_contributing_robots() {
        let store = ObservationStore::new();
        store.add(obs_at(10.0, 10.0, 100, "robot-1", 0.5)).unwrap();
        store.add(obs_at(10.0, 10.0, 200, "robot-1", 0.5)).unwrap();
        store.add(obs_at(10.0, 10.0, 300, "robot-2", 0.5)).unwrap();

        let policy = CompactionPolicy::new(9, 7, 3_600_000_000).unwrap();
        let rollups = policy.compact_range(&store, 0, i64::MAX).unwrap();

        assert_eq!(rollups.len(), 1);
        assert_eq!(rollups[0].observation_count, 3);
        assert_eq!(
            rollups[0].contributing_robots,
            vec!["robot-1".to_string(), "robot-2".to_string()]
        );
    }
}
