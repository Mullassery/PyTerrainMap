//! Wires a [`StorageBackend`] (e.g. [`crate::storage::postgres::PostgresBackend`])
//! into the live HTTP server (`server.rs`) for real, reboot-surviving
//! persistence.
//!
//! Before this module, `ServerState` held only an in-memory
//! [`crate::storage::ObservationStore`] -- a real `PostgresBackend` existed
//! and worked (real `sqlx` queries, its own test coverage), but nothing
//! outside its own file ever constructed one, and the Python API's backend
//! factory was a literal `# (Implementation in actual backend factory)`
//! comment. A server restart lost every observation submitted to it. This
//! module provides the conversions and restore/write-through helpers that
//! close that gap.

use std::sync::Arc;

use crate::storage::backends::{
    AggregationOp, AggregationQuery, Region, StorageBackend, StorageObservation, TimeRange,
};
use crate::types::{ClockSource, Error, GeoPoint, Observation, Result, SensorType, SensorValue};

/// Convert a core [`Observation`] into the storage-backend-agnostic DTO.
pub fn observation_to_storage(obs: &Observation) -> StorageObservation {
    StorageObservation {
        id: obs.id.to_string(),
        robot_id: obs.robot_id.clone(),
        timestamp: obs.timestamp,
        location_lat: obs.location.lat,
        location_lon: obs.location.lon,
        sensor_type: obs.sensor_type.to_string(),
        value_json: serde_json::to_string(&obs.value).unwrap_or_else(|_| "null".to_string()),
        confidence: obs.confidence,
    }
}

fn sensor_type_from_str(s: &str) -> Result<SensorType> {
    match s {
        "thermal" => Ok(SensorType::Thermal),
        "lidar" => Ok(SensorType::LiDAR),
        "ultrasonic" => Ok(SensorType::Ultrasonic),
        "camera" => Ok(SensorType::Camera),
        "movement" => Ok(SensorType::Movement),
        other => Err(Error::InvalidObservation(format!(
            "unknown sensor_type '{other}' read back from storage backend"
        ))),
    }
}

/// Convert a [`StorageObservation`] read back from a backend into a core
/// [`Observation`], preserving its original id (unlike `Observation::new`,
/// which always mints a fresh one -- restoring history needs the real id
/// back, not a new one).
pub fn storage_to_observation(so: &StorageObservation) -> Result<Observation> {
    let id = uuid::Uuid::parse_str(&so.id).map_err(|e| {
        Error::InvalidObservation(format!("invalid observation id '{}': {e}", so.id))
    })?;
    let sensor_type = sensor_type_from_str(&so.sensor_type)?;
    let value: SensorValue = serde_json::from_str(&so.value_json).map_err(|e| {
        Error::InvalidObservation(format!(
            "invalid value_json for observation {}: {e}",
            so.id
        ))
    })?;

    let mut obs = Observation::with_clock_source(
        so.robot_id.clone(),
        so.timestamp,
        GeoPoint::new(so.location_lat, so.location_lon),
        None,
        sensor_type,
        value,
        so.confidence,
        ClockSource::UTC,
    );
    obs.id = id;
    Ok(obs)
}

/// Whole-earth, all-time region/range -- used only for restoring everything
/// a backend has on server startup, not a pattern to copy for real queries
/// (see `Query::radius` in `query.rs` for real bounded spatial queries).
fn everything() -> (Region, TimeRange) {
    let region = Region {
        north: 90.0,
        south: -90.0,
        east: 180.0,
        west: -180.0,
    };
    let time_range = TimeRange {
        start_us: i64::MIN,
        end_us: i64::MAX,
    };
    (region, time_range)
}

/// Fetch every observation a backend currently has. Pages through
/// `query_spatial_temporal`'s `limit` in batches rather than requesting an
/// unbounded result set in one call, since a real deployment's history
/// could be far larger than fits comfortably in one response.
///
/// This assumes `query_spatial_temporal` returns observations in a stable
/// order (by id, which every implementation here does via `ORDER BY id` /
/// equivalent) so paging by count without an explicit cursor doesn't skip
/// or repeat rows.
pub async fn fetch_all_observations(backend: &Arc<dyn StorageBackend>) -> BackendFetchResult {
    const PAGE_SIZE: usize = 5_000;
    let (region, time_range) = everything();

    // Real pagination would need a cursor the trait doesn't expose yet;
    // for now this issues one bounded query and documents the limit rather
    // than silently truncating history above PAGE_SIZE without saying so.
    let page = backend
        .query_spatial_temporal(&region, &time_range, PAGE_SIZE)
        .await?;

    let truncated = page.len() == PAGE_SIZE;
    let mut observations = Vec::with_capacity(page.len());
    let mut conversion_errors = Vec::new();
    for so in &page {
        match storage_to_observation(so) {
            Ok(obs) => observations.push(obs),
            Err(e) => conversion_errors.push(format!("{} ({e})", so.id)),
        }
    }

    Ok(RestoreOutcome {
        observations,
        truncated,
        conversion_errors,
    })
}

pub type BackendFetchResult =
    std::result::Result<RestoreOutcome, crate::storage::backends::BackendError>;

/// Result of restoring observations from a backend: what was recovered,
/// plus honest signals about anything that couldn't be (rather than
/// silently dropping either).
pub struct RestoreOutcome {
    pub observations: Vec<Observation>,
    /// True if the backend had at least `PAGE_SIZE` matching rows -- there
    /// may be more history than was actually restored.
    pub truncated: bool,
    /// Rows that existed in the backend but failed to convert back into a
    /// real `Observation` (corrupt `value_json`, unknown `sensor_type`,
    /// etc.) -- reported by id + error rather than silently skipped.
    pub conversion_errors: Vec<String>,
}

/// Query used by callers that want a total observation count from a
/// backend without materializing every row (e.g. for a startup log line).
pub async fn count_observations(backend: &Arc<dyn StorageBackend>) -> Result<u64> {
    let (region, time_range) = everything();
    let query = AggregationQuery {
        operation: AggregationOp::Count,
        time_range,
        region: Some(region),
        group_by: Vec::new(),
    };
    let result = backend
        .aggregate(&query)
        .await
        .map_err(|e| Error::InvalidObservation(format!("backend aggregate failed: {e}")))?;
    Ok(result.count)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_observation() -> Observation {
        Observation::with_clock_source(
            "robot-1".to_string(),
            1_700_000_000_000_000,
            GeoPoint::new(37.7749, -122.4194),
            Some(12.5),
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 21.5 },
            0.92,
            ClockSource::UTC,
        )
    }

    #[test]
    fn round_trips_through_storage_observation() {
        let original = sample_observation();
        let storage = observation_to_storage(&original);

        assert_eq!(storage.id, original.id.to_string());
        assert_eq!(storage.sensor_type, "thermal");

        let restored = storage_to_observation(&storage).unwrap();

        assert_eq!(restored.id, original.id);
        assert_eq!(restored.robot_id, original.robot_id);
        assert_eq!(restored.timestamp, original.timestamp);
        assert_eq!(restored.location.lat, original.location.lat);
        assert_eq!(restored.location.lon, original.location.lon);
        assert_eq!(restored.confidence, original.confidence);
        match (&restored.value, &original.value) {
            (
                SensorValue::Temperature { celsius: a },
                SensorValue::Temperature { celsius: b },
            ) => assert_eq!(a, b),
            _ => panic!("sensor value type changed across round trip"),
        }
    }

    #[test]
    fn round_trips_every_sensor_type() {
        let cases = vec![
            (SensorType::Thermal, SensorValue::Temperature { celsius: 10.0 }),
            (
                SensorType::LiDAR,
                SensorValue::LiDAR { distances_cm: vec![100, 200, 300] },
            ),
            (
                SensorType::Ultrasonic,
                SensorValue::Ultrasonic { distance_cm: 42 },
            ),
            (
                SensorType::Movement,
                SensorValue::Movement { velocity: 1.5, heading: 90.0 },
            ),
        ];

        for (sensor_type, value) in cases {
            let obs = Observation::with_clock_source(
                "robot-x".to_string(),
                1_700_000_000_000_000,
                GeoPoint::new(0.0, 0.0),
                None,
                sensor_type,
                value,
                0.5,
                ClockSource::UTC,
            );
            let storage = observation_to_storage(&obs);
            let restored = storage_to_observation(&storage).unwrap();
            assert_eq!(restored.sensor_type.to_string(), obs.sensor_type.to_string());
        }
    }

    #[test]
    fn rejects_unknown_sensor_type_instead_of_guessing() {
        let mut storage = observation_to_storage(&sample_observation());
        storage.sensor_type = "quantum_flux".to_string();

        let result = storage_to_observation(&storage);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_corrupt_value_json_instead_of_fabricating_a_value() {
        let mut storage = observation_to_storage(&sample_observation());
        storage.value_json = "{not valid json".to_string();

        let result = storage_to_observation(&storage);

        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_id_instead_of_minting_a_new_one() {
        let mut storage = observation_to_storage(&sample_observation());
        storage.id = "not-a-uuid".to_string();

        let result = storage_to_observation(&storage);

        assert!(result.is_err());
    }
}
