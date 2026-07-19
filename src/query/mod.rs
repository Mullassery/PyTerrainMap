//! Unified query API combining spatial and temporal indexing
//!
//! Provides high-level queries that combine H3 spatial cells,
//! elevation buckets, and temporal decay in a single operation.

use crate::types::{Observation, Result, Error, ElevationBucket};
use crate::spatial::{SpatialIndex, SpatialKey};
use crate::temporal::TemporalIndex;
use crate::storage::ObservationStore;
use std::sync::Arc;

/// Query result with decayed confidence
#[derive(Clone, Debug)]
pub struct QueryResult {
    /// The observation
    pub observation: Arc<Observation>,
    /// Confidence after temporal decay
    pub decayed_confidence: f32,
}

/// Spatial-temporal query builder
pub struct Query {
    spatial: SpatialIndex,
    temporal: TemporalIndex,
    store: Arc<ObservationStore>,
    current_time_us: i64,
}

impl Query {
    /// Create new query with spatial/temporal indices and store
    pub fn new(
        spatial: SpatialIndex,
        temporal: TemporalIndex,
        store: Arc<ObservationStore>,
        current_time_us: i64,
    ) -> Self {
        Query {
            spatial,
            temporal,
            store,
            current_time_us,
        }
    }

    /// Query observations in radius at current time
    pub fn radius(
        &self,
        lat: f64,
        lon: f64,
        radius_m: f32,
    ) -> Result<Vec<QueryResult>> {
        let location = crate::types::GeoPoint::new(lat, lon);

        // Get indices from spatial index
        let spatial_indices = self.spatial.query_radius(location, radius_m, None)?;

        // Fetch and decay
        self.apply_decay(&spatial_indices)
    }

    /// Query observations in radius with elevation filter
    pub fn radius_with_elevation(
        &self,
        lat: f64,
        lon: f64,
        radius_m: f32,
        elevation: Option<ElevationBucket>,
    ) -> Result<Vec<QueryResult>> {
        let location = crate::types::GeoPoint::new(lat, lon);

        // Get indices from spatial index
        let spatial_indices = self.spatial.query_radius(location, radius_m, elevation)?;

        // Fetch and decay
        self.apply_decay(&spatial_indices)
    }

    /// Query observations in time range
    pub fn time_range(&self, from_us: i64, to_us: i64) -> Result<Vec<QueryResult>> {
        // Get indices from temporal index
        let temporal_indices = self.temporal.range_query(from_us, to_us)?;

        // Fetch and decay
        self.apply_decay(&temporal_indices)
    }

    /// Query observations since timestamp
    pub fn since(&self, timestamp_us: i64) -> Result<Vec<QueryResult>> {
        // Get indices from temporal index
        let temporal_indices = self.temporal.since(timestamp_us)?;

        // Fetch and decay
        self.apply_decay(&temporal_indices)
    }

    /// Get observations in both spatial radius AND time range
    pub fn spatial_temporal(
        &self,
        lat: f64,
        lon: f64,
        radius_m: f32,
        from_time_us: i64,
        to_time_us: i64,
    ) -> Result<Vec<QueryResult>> {
        let location = crate::types::GeoPoint::new(lat, lon);

        // Get spatial candidates
        let spatial_indices = self.spatial.query_radius(location, radius_m, None)?;

        // Get temporal candidates
        let temporal_indices = self.temporal.range_query(from_time_us, to_time_us)?;

        // Intersect: keep only indices in both sets
        let temporal_set: std::collections::HashSet<_> = temporal_indices.into_iter().collect();
        let intersect: Vec<_> = spatial_indices
            .into_iter()
            .filter(|idx| temporal_set.contains(idx))
            .collect();

        // Fetch and decay
        self.apply_decay(&intersect)
    }

    /// Get newest N observations in radius
    pub fn newest_in_radius(&self, lat: f64, lon: f64, radius_m: f32, n: usize) -> Result<Vec<QueryResult>> {
        let location = crate::types::GeoPoint::new(lat, lon);
        let spatial_indices = self.spatial.query_radius(location, radius_m, None)?;

        // Get newest N overall
        let newest = self.temporal.newest_n(n);

        // Intersect with spatial
        let spatial_set: std::collections::HashSet<_> = spatial_indices.into_iter().collect();
        let intersect: Vec<_> = newest
            .into_iter()
            .filter(|idx| spatial_set.contains(idx))
            .take(n)
            .collect();

        self.apply_decay(&intersect)
    }

    /// Get oldest N observations in radius
    pub fn oldest_in_radius(&self, lat: f64, lon: f64, radius_m: f32, n: usize) -> Result<Vec<QueryResult>> {
        let location = crate::types::GeoPoint::new(lat, lon);
        let spatial_indices = self.spatial.query_radius(location, radius_m, None)?;

        // Get oldest N overall
        let oldest = self.temporal.oldest_n(n);

        // Intersect with spatial
        let spatial_set: std::collections::HashSet<_> = spatial_indices.into_iter().collect();
        let intersect: Vec<_> = oldest
            .into_iter()
            .filter(|idx| spatial_set.contains(idx))
            .take(n)
            .collect();

        self.apply_decay(&intersect)
    }

    /// Set current time for decay calculations
    pub fn with_time(mut self, current_time_us: i64) -> Self {
        self.current_time_us = current_time_us;
        self
    }

    /// Apply temporal decay to a set of observation indices
    fn apply_decay(&self, indices: &[usize]) -> Result<Vec<QueryResult>> {
        let observations = self.store.get_batch(&indices.to_vec())?;
        let mut results = Vec::with_capacity(observations.len());

        for (original_idx, obs) in indices.iter().zip(observations.iter()) {
            let decayed_conf = self
                .temporal
                .decayed_confidence(*original_idx, self.current_time_us, obs.confidence)?;

            results.push(QueryResult {
                observation: obs.clone(),
                decayed_confidence: decayed_conf,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType, SensorValue};
    use crate::temporal::DecayFunction;

    fn create_store_with_observations() -> (Arc<ObservationStore>, Vec<usize>) {
        let store = Arc::new(ObservationStore::new());
        let mut indices = Vec::new();

        for i in 0..5 {
            let obs = Observation::new(
                format!("bot_{}", i % 2),
                1000_000 + (i * 100_000) as i64,
                GeoPoint::new(40.7128, -74.0060),
                Some(100.0),
                SensorType::Thermal,
                SensorValue::Temperature { celsius: 22.5 + i as f32 },
                0.95 - (i as f32 * 0.05),
            );
            let idx = store.add(obs).unwrap();
            indices.push(idx);
        }

        (store, indices)
    }

    fn setup_query() -> Query {
        let (store, _) = create_store_with_observations();

        let mut spatial = SpatialIndex::new();
        let mut temporal = TemporalIndex::new(DecayFunction::None);

        // Add all observations to indices
        for i in 0..5 {
            let obs = store.get(i).unwrap();
            spatial.insert(i, obs.location, obs.elevation_asl).unwrap();
            temporal.insert(obs.timestamp).unwrap();
        }

        Query::new(spatial, temporal, store, 5_000_000)
    }

    #[test]
    fn test_query_creation() {
        let query = setup_query();
        assert_eq!(query.current_time_us, 5_000_000);
    }

    #[test]
    fn test_query_radius() {
        let query = setup_query();

        let results = query.radius(40.7128, -74.0060, 1000.0).unwrap();
        assert!(!results.is_empty());

        // Check that results have observations
        for result in results {
            assert!(result.observation.confidence > 0.0);
            assert!(result.decayed_confidence >= 0.0);
        }
    }

    #[test]
    fn test_query_time_range() {
        let query = setup_query();

        // Query in time range [1000_000, 1300_000] to get obs 0, 1, 2, 3
        let results = query.time_range(1_000_000, 1_300_000).unwrap();
        assert_eq!(results.len(), 4); // obs 0, 1, 2, 3

        // Verify timestamps are in range
        for result in results {
            assert!(result.observation.timestamp >= 1_000_000);
            assert!(result.observation.timestamp <= 1_300_000);
        }
    }

    #[test]
    fn test_query_since() {
        let query = setup_query();

        let results = query.since(1_200_000).unwrap();
        assert_eq!(results.len(), 3); // Should get obs 2, 3, 4 (timestamps >= 1_200_000)

        for result in results {
            assert!(result.observation.timestamp >= 1_200_000);
        }
    }

    #[test]
    fn test_query_spatial_temporal() {
        let query = setup_query();

        let results = query.spatial_temporal(40.7128, -74.0060, 1000.0, 1_000_000, 1_300_000).unwrap();
        // All observations are at same location, so spatial doesn't filter
        // Temporal filters to [1_000_000, 1_300_000], which includes obs 0-3
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_query_newest_in_radius() {
        let query = setup_query();

        let results = query.newest_in_radius(40.7128, -74.0060, 1000.0, 2).unwrap();
        assert!(results.len() <= 2);

        // Should be newest observations (highest timestamps)
        if results.len() > 1 {
            assert!(results[0].observation.timestamp <= results[1].observation.timestamp);
        }
    }

    #[test]
    fn test_query_with_decay() {
        let store = Arc::new(ObservationStore::new());

        // Add observation at t=0 microseconds
        let obs = Observation::new(
            "bot_1".to_string(),
            100_000, // Use non-zero timestamp
            GeoPoint::new(40.7128, -74.0060),
            None,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            1.0,
        );
        store.add(obs).unwrap();

        let mut spatial = SpatialIndex::new();
        let mut temporal = TemporalIndex::new(DecayFunction::Exponential { half_life_ms: 1000 });

        spatial.insert(0, GeoPoint::new(40.7128, -74.0060), None).unwrap();
        temporal.insert(100_000).unwrap();

        // Query at t = 100_000 + 1_000_000 = 1_100_000 microseconds (1000ms later)
        let query = Query::new(spatial, temporal, store, 1_100_000);

        let results = query.radius(40.7128, -74.0060, 1000.0).unwrap();
        assert_eq!(results.len(), 1);

        // After 1 half-life (1000ms), exponential decay should give ~0.5
        assert!((results[0].decayed_confidence - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_query_set_time() {
        let query = setup_query();

        let query_later = query.with_time(6_000_000);
        assert_eq!(query_later.current_time_us, 6_000_000);
    }

    #[test]
    fn test_query_invalid_coordinates() {
        let query = setup_query();

        let result = query.radius(95.0, -74.0060, 1000.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_empty_results() {
        let store = Arc::new(ObservationStore::new());
        let spatial = SpatialIndex::new();
        let temporal = TemporalIndex::new(DecayFunction::None);

        let query = Query::new(spatial, temporal, store, 1_000_000);

        let results = query.radius(40.7128, -74.0060, 1000.0).unwrap();
        assert!(results.is_empty());
    }
}
