//! Storage backend abstraction and implementations
//!
//! Defines the StorageBackend trait and provides:
//! - Backend capabilities model
//! - Data tiering and routing
//! - Query optimization
//! - In-memory reference implementation

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use async_trait::async_trait;

// ============================================================================
// Error Types
// ============================================================================

pub type BackendResult<T> = std::result::Result<T, BackendError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendError {
    pub backend: String,
    pub operation: String,
    pub message: String,
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}:{})", self.message, self.backend, self.operation)
    }
}

impl std::error::Error for BackendError {}

// ============================================================================
// Observation ID
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObservationId(pub String);

// ============================================================================
// Storage Observation (independent of PyO3)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageObservation {
    pub id: String,
    pub robot_id: String,
    pub timestamp: i64,
    pub location_lat: f64,
    pub location_lon: f64,
    pub sensor_type: String,
    pub value_json: String,
    pub confidence: f32,
}

// ============================================================================
// Storage Formats
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageFormat {
    Relational,
    Document,
    Graph,
    KeyValue,
    TimeSeries,
    Columnar,
    ObjectStorage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataTier {
    Hot,
    Warm,
    Cold,
    Archive,
}

// ============================================================================
// Query Types
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_us: i64,
    pub end_us: i64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum AggregationOp {
    Count,
    Sum(u32),
    Avg(u32),
    Min(u32),
    Max(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregationQuery {
    pub operation: AggregationOp,
    pub time_range: TimeRange,
    pub region: Option<Region>,
    pub group_by: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregationResult {
    pub value: f64,
    pub count: u64,
    pub groups: std::collections::HashMap<String, f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub backend_id: String,
    pub is_healthy: bool,
    pub latency_ms: u32,
    pub error_rate: f32,
}

// ============================================================================
// Backend Capabilities
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub supports_transactions: bool,
    pub supports_graph_traversal: bool,
    pub supports_aggregations: bool,
    pub supports_fulltext_search: bool,
    pub supports_geospatial_queries: bool,
    pub supports_time_series: bool,
    pub supports_ttl: bool,
    pub latency_ms: u32,
    pub throughput_rps: u32,
    pub storage_format: StorageFormat,
}

// ============================================================================
// Storage Backend Trait
// ============================================================================

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn backend_id(&self) -> &str;
    fn capabilities(&self) -> BackendCapabilities;

    async fn insert_observation(
        &self,
        observation: &StorageObservation,
    ) -> BackendResult<ObservationId>;

    async fn insert_batch(
        &self,
        observations: Vec<StorageObservation>,
    ) -> BackendResult<Vec<ObservationId>>;

    async fn query_spatial_temporal(
        &self,
        region: &Region,
        time_range: &TimeRange,
        limit: usize,
    ) -> BackendResult<Vec<StorageObservation>>;

    async fn get_by_id(
        &self,
        id: &ObservationId,
    ) -> BackendResult<Option<StorageObservation>>;

    async fn aggregate(
        &self,
        query: &AggregationQuery,
    ) -> BackendResult<AggregationResult>;

    async fn delete_expired(&self) -> BackendResult<usize>;
    async fn maintenance(&self) -> BackendResult<()>;
    async fn health_check(&self) -> BackendResult<HealthStatus>;
}

// ============================================================================
// Backend Registry
// ============================================================================

pub struct BackendRegistry {
    backends: parking_lot::RwLock<std::collections::HashMap<String, Arc<dyn StorageBackend>>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        BackendRegistry {
            backends: parking_lot::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn register(&self, backend: Arc<dyn StorageBackend>) {
        self.backends.write().insert(backend.backend_id().to_string(), backend);
    }

    pub fn get(&self, id: &str) -> BackendResult<Arc<dyn StorageBackend>> {
        self.backends
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| BackendError {
                backend: id.to_string(),
                operation: "get".to_string(),
                message: "Backend not registered".to_string(),
            })
    }

    pub fn all_backends(&self) -> Vec<Arc<dyn StorageBackend>> {
        self.backends.read().values().cloned().collect()
    }
}

impl Default for BackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Query Optimizer
// ============================================================================

pub struct QueryOptimizer;

impl QueryOptimizer {
    pub fn new() -> Self {
        QueryOptimizer
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Data Placement Policy
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataPlacementPolicy {
    pub hot_retention_days: u32,
    pub warm_retention_days: u32,
    pub cold_retention_days: u32,
}

impl Default for DataPlacementPolicy {
    fn default() -> Self {
        DataPlacementPolicy {
            hot_retention_days: 1,
            warm_retention_days: 90,
            cold_retention_days: 365,
        }
    }
}

// ============================================================================
// Federated Query Executor
// ============================================================================

pub struct FederatedQueryExecutor {
    registry: Arc<BackendRegistry>,
    optimizer: Arc<QueryOptimizer>,
    policy: Arc<DataPlacementPolicy>,
}

impl FederatedQueryExecutor {
    pub fn new(
        registry: Arc<BackendRegistry>,
        optimizer: Arc<QueryOptimizer>,
        policy: Arc<DataPlacementPolicy>,
    ) -> Self {
        FederatedQueryExecutor {
            registry,
            optimizer,
            policy,
        }
    }
}

// ============================================================================
// In-Memory Backend (Testing)
// ============================================================================

pub struct InMemoryBackend {
    observations: parking_lot::RwLock<Vec<StorageObservation>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        InMemoryBackend {
            observations: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl StorageBackend for InMemoryBackend {
    fn backend_id(&self) -> &str {
        "memory"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: true,
            supports_graph_traversal: false,
            supports_aggregations: true,
            supports_fulltext_search: false,
            supports_geospatial_queries: true,
            supports_time_series: true,
            supports_ttl: false,
            latency_ms: 1,
            throughput_rps: 1_000_000,
            storage_format: StorageFormat::KeyValue,
        }
    }

    async fn insert_observation(
        &self,
        observation: &StorageObservation,
    ) -> BackendResult<ObservationId> {
        self.observations.write().push(observation.clone());
        Ok(ObservationId(uuid::Uuid::new_v4().to_string()))
    }

    async fn insert_batch(
        &self,
        observations: Vec<StorageObservation>,
    ) -> BackendResult<Vec<ObservationId>> {
        let ids: Vec<_> = observations
            .iter()
            .map(|_| ObservationId(uuid::Uuid::new_v4().to_string()))
            .collect();
        self.observations.write().extend(observations);
        Ok(ids)
    }

    async fn query_spatial_temporal(
        &self,
        region: &Region,
        time_range: &TimeRange,
        limit: usize,
    ) -> BackendResult<Vec<StorageObservation>> {
        let results: Vec<_> = self
            .observations
            .read()
            .iter()
            .filter(|o| {
                o.location_lat >= region.south && o.location_lat <= region.north
                    && o.location_lon >= region.west && o.location_lon <= region.east
                    && o.timestamp >= time_range.start_us && o.timestamp <= time_range.end_us
            })
            .take(limit)
            .cloned()
            .collect();

        Ok(results)
    }

    async fn get_by_id(
        &self,
        _id: &ObservationId,
    ) -> BackendResult<Option<StorageObservation>> {
        Ok(None)
    }

    async fn aggregate(
        &self,
        query: &AggregationQuery,
    ) -> BackendResult<AggregationResult> {
        let observations = self.observations.read();
        let count: usize = observations
            .iter()
            .filter(|o| {
                o.timestamp >= query.time_range.start_us && o.timestamp <= query.time_range.end_us
            })
            .count();

        Ok(AggregationResult {
            value: count as f64,
            count: count as u64,
            groups: std::collections::HashMap::new(),
        })
    }

    async fn delete_expired(&self) -> BackendResult<usize> {
        Ok(0)
    }

    async fn maintenance(&self) -> BackendResult<()> {
        Ok(())
    }

    async fn health_check(&self) -> BackendResult<HealthStatus> {
        Ok(HealthStatus {
            backend_id: "memory".to_string(),
            is_healthy: true,
            latency_ms: 1,
            error_rate: 0.0,
        })
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_registry() {
        let registry = BackendRegistry::new();
        let backend = Arc::new(InMemoryBackend::new());
        registry.register(backend);

        assert!(registry.get("memory").is_ok());
        assert!(registry.get("nonexistent").is_err());
    }

    #[tokio::test]
    async fn test_in_memory_backend_capabilities() {
        let backend = InMemoryBackend::new();
        let caps = backend.capabilities();

        assert!(caps.supports_transactions);
        assert!(caps.supports_geospatial_queries);
        assert!(caps.supports_aggregations);
        assert_eq!(caps.latency_ms, 1);
    }

    #[tokio::test]
    async fn test_in_memory_backend_health_check() {
        let backend = InMemoryBackend::new();
        let health = backend.health_check().await;
        assert!(health.is_ok());
        assert!(health.unwrap().is_healthy);
    }

    #[test]
    fn test_data_tier_default_policy() {
        let policy = DataPlacementPolicy::default();
        assert_eq!(policy.hot_retention_days, 1);
        assert_eq!(policy.warm_retention_days, 90);
        assert_eq!(policy.cold_retention_days, 365);
    }
}
