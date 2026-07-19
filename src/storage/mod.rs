//! Storage architecture with pluggable backends
//!
//! Immutable append-only observation log with multi-backend support.
//! Observations can be persisted to PostgreSQL (hot), BigQuery (warm), S3 (cold), or memory.

pub mod backends;
pub mod postgres;
pub mod federation;

use crate::types::{Observation, Result, Error};
use std::sync::Arc;
use parking_lot::RwLock;

/// Immutable in-memory observation storage
pub struct ObservationStore {
    /// All observations, append-only
    observations: RwLock<Vec<Arc<Observation>>>,
}

impl ObservationStore {
    /// Create new empty observation store
    pub fn new() -> Self {
        ObservationStore {
            observations: RwLock::new(Vec::new()),
        }
    }

    /// Add observation (immutable append-only)
    pub fn add(&self, observation: Observation) -> Result<usize> {
        if !observation.is_valid() {
            return Err(Error::InvalidObservation("Observation failed validation".to_string()));
        }

        let mut obs = self.observations.write();
        let index = obs.len();
        obs.push(Arc::new(observation));
        Ok(index)
    }

    /// Get observation by index
    pub fn get(&self, index: usize) -> Result<Arc<Observation>> {
        let obs = self.observations.read();
        obs.get(index)
            .cloned()
            .ok_or_else(|| Error::QueryError(format!("Observation index {} not found", index)))
    }

    /// Get multiple observations by indices
    pub fn get_batch(&self, indices: &[usize]) -> Result<Vec<Arc<Observation>>> {
        let obs = self.observations.read();
        let mut results = Vec::with_capacity(indices.len());

        for &idx in indices {
            if let Some(observation) = obs.get(idx) {
                results.push(observation.clone());
            } else {
                return Err(Error::QueryError(format!("Observation index {} not found", idx)));
            }
        }

        Ok(results)
    }

    /// Get total number of observations
    pub fn len(&self) -> usize {
        self.observations.read().len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.observations.read().is_empty()
    }

    /// Get observations for a robot
    pub fn observations_by_robot(&self, robot_id: &str) -> Vec<usize> {
        let obs = self.observations.read();
        obs.iter()
            .enumerate()
            .filter(|(_, o)| o.robot_id == robot_id)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Count observations by sensor type
    pub fn count_by_sensor_type(&self) -> std::collections::HashMap<String, usize> {
        let obs = self.observations.read();
        let mut counts = std::collections::HashMap::new();

        for observation in obs.iter() {
            let sensor_name = observation.sensor_type.to_string();
            *counts.entry(sensor_name).or_insert(0) += 1;
        }

        counts
    }

    /// Get minimum and maximum timestamps
    pub fn timestamp_range(&self) -> Option<(i64, i64)> {
        let obs = self.observations.read();
        if obs.is_empty() {
            return None;
        }

        let min = obs.iter().map(|o| o.timestamp).min()?;
        let max = obs.iter().map(|o| o.timestamp).max()?;
        Some((min, max))
    }

    /// Get observations added after a certain index
    pub fn since_index(&self, start_index: usize) -> Vec<usize> {
        let obs = self.observations.read();
        let total = obs.len();

        if start_index >= total {
            return vec![];
        }

        (start_index..total).collect()
    }

    /// Clear all observations (use with caution - breaks immutability guarantee)
    pub fn clear(&self) {
        self.observations.write().clear();
    }
}

impl Default for ObservationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType, SensorValue};

    fn create_test_observation(robot_id: &str, timestamp: i64) -> Observation {
        Observation::new(
            robot_id.to_string(),
            timestamp,
            GeoPoint::new(40.7128, -74.0060),
            Some(100.0),
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        )
    }

    #[test]
    fn test_observation_store_creation() {
        let store = ObservationStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_add_and_get() {
        let store = ObservationStore::new();
        let obs = create_test_observation("bot_1", 1000);

        let idx = store.add(obs.clone()).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(store.len(), 1);

        let retrieved = store.get(idx).unwrap();
        assert_eq!(retrieved.robot_id, "bot_1");
        assert_eq!(retrieved.timestamp, 1000);
    }

    #[test]
    fn test_multiple_observations() {
        let store = ObservationStore::new();

        for i in 0..5 {
            let obs = create_test_observation("bot_1", 1000 + (i * 100) as i64);
            let idx = store.add(obs).unwrap();
            assert_eq!(idx, i);
        }

        assert_eq!(store.len(), 5);

        // Retrieve middle one
        let obs = store.get(2).unwrap();
        assert_eq!(obs.timestamp, 1200);
    }

    #[test]
    fn test_get_batch() {
        let store = ObservationStore::new();

        for i in 0..5 {
            let obs = create_test_observation("bot_1", 1000 + (i * 100) as i64);
            store.add(obs).unwrap();
        }

        let batch = store.get_batch(&[1, 3, 4]).unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].timestamp, 1100);
        assert_eq!(batch[1].timestamp, 1300);
        assert_eq!(batch[2].timestamp, 1400);
    }

    #[test]
    fn test_invalid_index() {
        let store = ObservationStore::new();
        let obs = create_test_observation("bot_1", 1000);
        store.add(obs).unwrap();

        // Try to get non-existent index
        assert!(store.get(10).is_err());
        assert!(store.get_batch(&[0, 10]).is_err());
    }

    #[test]
    fn test_observations_by_robot() {
        let store = ObservationStore::new();

        for i in 0..3 {
            let obs = create_test_observation("bot_1", 1000 + (i * 100) as i64);
            store.add(obs).unwrap();
        }

        for i in 0..2 {
            let obs = create_test_observation("bot_2", 2000 + (i * 100) as i64);
            store.add(obs).unwrap();
        }

        let bot1_indices = store.observations_by_robot("bot_1");
        assert_eq!(bot1_indices, vec![0, 1, 2]);

        let bot2_indices = store.observations_by_robot("bot_2");
        assert_eq!(bot2_indices, vec![3, 4]);

        let bot3_indices = store.observations_by_robot("bot_3");
        assert!(bot3_indices.is_empty());
    }

    #[test]
    fn test_count_by_sensor_type() {
        let store = ObservationStore::new();

        // Add thermal observations
        for _ in 0..3 {
            let obs = create_test_observation("bot_1", 1000);
            store.add(obs).unwrap();
        }

        // Add LiDAR observations
        let mut lidar_obs = create_test_observation("bot_1", 2000);
        lidar_obs.sensor_type = SensorType::LiDAR;
        lidar_obs.value = SensorValue::LiDAR { distances_cm: vec![100, 200] };
        store.add(lidar_obs).unwrap();
        store.add({
            let mut o = create_test_observation("bot_1", 3000);
            o.sensor_type = SensorType::LiDAR;
            o.value = SensorValue::LiDAR { distances_cm: vec![150] };
            o
        }).unwrap();

        let counts = store.count_by_sensor_type();
        assert_eq!(counts.get("thermal"), Some(&3));
        assert_eq!(counts.get("lidar"), Some(&2));
    }

    #[test]
    fn test_timestamp_range() {
        let store = ObservationStore::new();

        assert!(store.timestamp_range().is_none());

        store.add(create_test_observation("bot_1", 1000)).unwrap();
        store.add(create_test_observation("bot_1", 5000)).unwrap();
        store.add(create_test_observation("bot_1", 3000)).unwrap();

        let (min, max) = store.timestamp_range().unwrap();
        assert_eq!(min, 1000);
        assert_eq!(max, 5000);
    }

    #[test]
    fn test_since_index() {
        let store = ObservationStore::new();

        for i in 0..5 {
            let obs = create_test_observation("bot_1", 1000 + (i * 100) as i64);
            store.add(obs).unwrap();
        }

        let since = store.since_index(2);
        assert_eq!(since, vec![2, 3, 4]);

        let since = store.since_index(5);
        assert!(since.is_empty());

        let since = store.since_index(10);
        assert!(since.is_empty());
    }

    #[test]
    fn test_invalid_observation_rejected() {
        let store = ObservationStore::new();

        // Create invalid observation (invalid location)
        let mut obs = create_test_observation("bot_1", 1000);
        obs.location = GeoPoint::new(95.0, -74.0060); // Invalid lat

        assert!(store.add(obs).is_err());
        assert!(store.is_empty());
    }

    #[test]
    fn test_concurrent_reads() {
        let store = Arc::new(ObservationStore::new());

        for i in 0..10 {
            let obs = create_test_observation("bot_1", 1000 + (i * 100) as i64);
            store.add(obs).unwrap();
        }

        // Multiple readers can access concurrently
        let store_clone = store.clone();
        let handle = std::thread::spawn(move || {
            let _ = store_clone.get(5);
        });

        let _ = store.get(3);
        handle.join().unwrap();

        assert_eq!(store.len(), 10);
    }
}
