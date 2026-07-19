//! Query Federation Layer
//!
//! Routes queries to the most appropriate storage backend based on:
//! - Query type
//! - Data freshness requirements
//! - Backend capabilities
//! - Cost model

use std::sync::Arc;
use crate::storage::backends::{
    StorageBackend, BackendRegistry, QueryOptimizer, FederatedQueryExecutor,
    DataPlacementPolicy, DataTier, Region, TimeRange, AggregationQuery, StorageObservation,
    BackendResult, BackendError,
};

/// Query router that dispatches to appropriate backends
pub struct QueryRouter {
    registry: Arc<BackendRegistry>,
    optimizer: Arc<QueryOptimizer>,
    policy: Arc<DataPlacementPolicy>,
}

impl QueryRouter {
    /// Create new query router
    pub fn new(
        registry: Arc<BackendRegistry>,
        optimizer: Arc<QueryOptimizer>,
        policy: Arc<DataPlacementPolicy>,
    ) -> Self {
        QueryRouter {
            registry,
            optimizer,
            policy,
        }
    }

    /// Route observation insert to appropriate backend
    pub async fn route_insert(&self, obs: &StorageObservation) -> BackendResult<String> {
        // Determine which tier this observation belongs to
        let tier = self._estimate_tier(obs.timestamp);

        // Select backend for this tier
        let backend = self._select_backend_for_tier(tier)?;

        // Insert
        backend.insert_observation(obs).await.map(|id| id.0)
    }

    /// Route spatial-temporal query to best backend
    pub async fn route_query(
        &self,
        region: &Region,
        time_range: &TimeRange,
        limit: usize,
    ) -> BackendResult<Vec<StorageObservation>> {
        // Analyze query
        let is_recent = self._is_recent_query(time_range);
        let is_small_region = self._is_small_region(region);

        // Select best backend based on query characteristics
        let backend = if is_recent && is_small_region {
            // Hot tier: PostgreSQL for fast access
            self.registry.get("postgres")?
        } else if !is_recent {
            // Cold tier: Check for BigQuery/warehouse
            self.registry
                .get("bigquery")
                .or_else(|_| self.registry.get("postgres"))?
        } else {
            // Default: PostgreSQL
            self.registry.get("postgres")?
        };

        // Execute query
        backend.query_spatial_temporal(region, time_range, limit).await
    }

    /// Route aggregation query
    pub async fn route_aggregation(
        &self,
        query: &AggregationQuery,
    ) -> BackendResult<(f64, u64)> {
        // Aggregations are best served by column-oriented databases
        let backend = self.registry
            .get("bigquery")
            .or_else(|_| self.registry.get("postgres"))?;

        let result = backend.aggregate(query).await?;
        Ok((result.value, result.count))
    }

    /// Determine data tier from timestamp
    fn _estimate_tier(&self, timestamp_us: i64) -> DataTier {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        let age_days = (now - timestamp_us) / (24 * 3600 * 1_000_000);

        match age_days {
            d if d < 1 => DataTier::Hot,
            d if d < 90 => DataTier::Warm,
            d if d < 365 => DataTier::Cold,
            _ => DataTier::Archive,
        }
    }

    /// Check if query is for recent data
    fn _is_recent_query(&self, time_range: &TimeRange) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        // Consider "recent" if querying last 24 hours
        (now - time_range.end_us) < (24 * 3600 * 1_000_000)
    }

    /// Check if region is small (optimization hint)
    fn _is_small_region(&self, region: &Region) -> bool {
        let lat_range = (region.north - region.south).abs();
        let lon_range = (region.east - region.west).abs();

        // Small region: less than 10km in each direction
        lat_range < 0.1 && lon_range < 0.1
    }

    /// Select backend for data tier
    fn _select_backend_for_tier(&self, tier: DataTier) -> BackendResult<Arc<dyn StorageBackend>> {
        match tier {
            DataTier::Hot => self.registry.get("postgres"),
            DataTier::Warm => self.registry
                .get("bigquery")
                .or_else(|_| self.registry.get("postgres")),
            DataTier::Cold => self.registry
                .get("s3")
                .or_else(|_| self.registry.get("postgres")),
            DataTier::Archive => self.registry
                .get("glacier")
                .or_else(|_| self.registry.get("s3")),
        }
    }
}

/// Multi-backend insert strategy
pub struct MultiBackendInserter {
    router: Arc<QueryRouter>,
    replicate_hot_to_warm: bool,
}

impl MultiBackendInserter {
    pub fn new(router: Arc<QueryRouter>, replicate: bool) -> Self {
        MultiBackendInserter {
            router,
            replicate_hot_to_warm: replicate,
        }
    }

    /// Insert with replication policy
    pub async fn insert_with_replication(&self, obs: &StorageObservation) -> BackendResult<String> {
        // Always insert to hot tier (PostgreSQL)
        let id = self.router.route_insert(obs).await?;

        // Optionally replicate to warm tier (BigQuery) asynchronously
        if self.replicate_hot_to_warm {
            // Fire-and-forget replication
            // In production, this would be queued for async processing
            // let _ = tokio::spawn(self._replicate_to_warm(obs.clone()));
        }

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_estimation() {
        let router = QueryRouter {
            registry: Arc::new(BackendRegistry::new()),
            optimizer: Arc::new(QueryOptimizer::new()),
            policy: Arc::new(DataPlacementPolicy::default()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        // Recent observation → Hot tier
        let recent_tier = router._estimate_tier(now);
        assert_eq!(recent_tier, DataTier::Hot);

        // Old observation → Archive tier
        let old_time = now - (400 * 24 * 3600 * 1_000_000);
        let old_tier = router._estimate_tier(old_time);
        assert_eq!(old_tier, DataTier::Archive);
    }

    #[test]
    fn test_region_size_detection() {
        let router = QueryRouter {
            registry: Arc::new(BackendRegistry::new()),
            optimizer: Arc::new(QueryOptimizer::new()),
            policy: Arc::new(DataPlacementPolicy::default()),
        };

        // Small region
        let small = Region {
            north: 40.715,
            south: 40.710,
            east: -74.005,
            west: -74.010,
        };
        assert!(router._is_small_region(&small));

        // Large region
        let large = Region {
            north: 41.0,
            south: 40.0,
            east: -73.0,
            west: -74.0,
        };
        assert!(!router._is_small_region(&large));
    }

    #[test]
    fn test_recent_query_detection() {
        let router = QueryRouter {
            registry: Arc::new(BackendRegistry::new()),
            optimizer: Arc::new(QueryOptimizer::new()),
            policy: Arc::new(DataPlacementPolicy::default()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        // Recent query
        let recent = TimeRange {
            start_us: now - 3600 * 1_000_000,  // 1 hour ago
            end_us: now,
        };
        assert!(router._is_recent_query(&recent));

        // Old query
        let old = TimeRange {
            start_us: now - (100 * 24 * 3600 * 1_000_000),  // 100 days ago
            end_us: now - (90 * 24 * 3600 * 1_000_000),
        };
        assert!(!router._is_recent_query(&old));
    }
}
