# Production-Grade Storage Architecture: Pluggable Backends

## Foundational Principle

**The platform owns the data model. The organization owns the storage strategy.**

PyTerrainMap should not force any storage technology. Organizations have existing:
- Operational databases (PostgreSQL, Oracle, SQL Server)
- Data warehouses (Snowflake, BigQuery, Redshift)
- Graph stores (Neo4j, JanusGraph)
- Object storage (S3, Azure Blob, GCS, MinIO)
- Real-time caches (Redis, Memcached)
- Lakehouses (Iceberg, Delta Lake, Hudi)

The platform should integrate all of these into a unified space-time intelligence architecture while preserving each system's strengths and the organization's autonomy.

---

## Architecture Overview

### Separation of Concerns

```
┌─────────────────────────────────────────────────┐
│      Application Layer (Agents, APIs, ML)       │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│    Unified Query Layer (Query Parser, Router)   │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│   Storage Abstraction Layer (Traits, Adapters)  │
└──┬──────────┬──────────┬──────────┬────────┬────┘
   │          │          │          │        │
┌──▼─┐  ┌─────▼────┐ ┌──▼──┐ ┌────▼──┐ ┌──▼──┐
│ SQL│  │Warehouse │ │Graph│ │Object │ │Cache│
│ DB │  │(BigQuery)│ │  DB │ │Storage│ │ DB  │
└────┘  └──────────┘ └─────┘ └───────┘ └─────┘

Platform owns: Query semantics, data model, schema
Organization owns: Storage selection, deployment, optimization
```

### Three-Tier Data Placement

```
HOT DATA (Operational)
├─ Current world state
├─ Recent observations (<1 day)
├─ Active missions
├─ Real-time cache
└─ Storage: PostgreSQL, Redis

WARM DATA (Analytical)
├─ Recent history (1 day - 90 days)
├─ Aggregations
├─ Derived features
├─ Reports
└─ Storage: ClickHouse, Warehouse (BigQuery/Snowflake)

COLD DATA (Archive)
├─ Long-term history (>90 days)
├─ Audit trails
├─ Compliance records
├─ Raw sensor archives
└─ Storage: Object Storage (S3, GCS, Azure Blob)
```

---

## Storage Abstraction Layer

### Core Trait: StorageBackend

```rust
pub trait StorageBackend: Send + Sync {
    /// Unique identifier for this backend
    fn backend_id(&self) -> &str;

    /// Capabilities (what this backend can do efficiently)
    fn capabilities(&self) -> BackendCapabilities;

    /// Insert single observation
    async fn insert_observation(
        &self,
        observation: &Observation,
        policy: &DataPlacementPolicy,
    ) -> Result<ObservationId>;

    /// Batch insert (preferred for efficiency)
    async fn insert_batch(
        &self,
        observations: Vec<Observation>,
        policy: &DataPlacementPolicy,
    ) -> Result<Vec<ObservationId>>;

    /// Query observations by region and time
    async fn query_spatial_temporal(
        &self,
        region: &Region,
        time_range: &TimeRange,
        limit: usize,
    ) -> Result<Vec<Observation>>;

    /// Query by observation ID
    async fn get_by_id(&self, id: &ObservationId) -> Result<Option<Observation>>;

    /// Aggregate query (count, sum, avg, etc.)
    async fn aggregate(
        &self,
        aggregation: &AggregationQuery,
    ) -> Result<AggregationResult>;

    /// Graph traversal (for graph backends)
    async fn traverse_graph(
        &self,
        start_node: &str,
        traversal_spec: &GraphTraversal,
    ) -> Result<Vec<String>>;

    /// Delete observations by policy
    async fn delete_by_policy(
        &self,
        policy: &RetentionPolicy,
    ) -> Result<usize>;

    /// Export to external format
    async fn export(
        &self,
        filter: &ExportFilter,
        format: &ExportFormat,
    ) -> Result<ExportStream>;

    /// Vacuum/optimize storage
    async fn maintenance(&self) -> Result<MaintenanceStats>;

    /// Check backend health
    async fn health_check(&self) -> Result<HealthStatus>;
}

pub struct BackendCapabilities {
    pub supports_transactions: bool,
    pub supports_graph_traversal: bool,
    pub supports_aggregations: bool,
    pub supports_fulltext_search: bool,
    pub supports_geospatial_queries: bool,
    pub supports_time_series: bool,
    pub supports_ttl: bool,  // Automatic expiration
    pub latency_ms: u32,
    pub throughput_rps: u32,
    pub storage_format: StorageFormat,
}

pub enum StorageFormat {
    Relational,
    Document,
    Graph,
    KeyValue,
    TimeSeries,
    Columnar,
    ObjectStorage,
}
```

### Backend Implementations

#### 1. PostgreSQL (Operational Database)

```rust
pub struct PostgresBackend {
    connection_pool: pgwire::Pool,
    schema_manager: SchemaManager,
    policy_manager: PolicyManager,
}

impl PostgresBackend {
    pub fn new(connection_string: &str) -> Result<Self> {
        let pool = pgwire::Pool::new(connection_string)?;
        
        // Initialize schema
        pool.execute("CREATE TABLE IF NOT EXISTS observations (
            id UUID PRIMARY KEY,
            location GEOMETRY(POINT, 4326),
            time_range TSRANGE,
            sensor_type VARCHAR(50),
            data JSONB,
            trust_score FLOAT,
            provenance JSONB,
            created_at TIMESTAMP,
            INDEX idx_location_time USING GIST (location, time_range)
        )")?;

        Ok(PostgresBackend {
            connection_pool: pool,
            schema_manager: SchemaManager::new(),
            policy_manager: PolicyManager::new(),
        })
    }
}

impl StorageBackend for PostgresBackend {
    fn backend_id(&self) -> &str { "postgres" }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: true,
            supports_graph_traversal: false,
            supports_aggregations: true,
            supports_fulltext_search: true,
            supports_geospatial_queries: true,
            supports_time_series: true,
            supports_ttl: false,
            latency_ms: 10,
            throughput_rps: 10_000,
            storage_format: StorageFormat::Relational,
        }
    }

    async fn insert_observation(&self, obs: &Observation, policy: &DataPlacementPolicy) -> Result<ObservationId> {
        let id = Uuid::new_v4();
        
        // Determine which backend this observation should go to
        let tier = policy.placement_tier(obs);
        
        // If this backend matches the tier, insert
        if tier == DataTier::Hot {
            self.connection_pool.execute(
                "INSERT INTO observations (id, location, time_range, sensor_type, data, trust_score, provenance, created_at)
                 VALUES ($1, ST_GeomFromText($2, 4326), $3, $4, $5, $6, $7, NOW())",
                [&id, &obs.location.to_wkt(), &obs.temporal_metadata.to_tsrange(), ...]
            ).await?;
        }
        
        Ok(id)
    }
}
```

#### 2. BigQuery (Data Warehouse)

```rust
pub struct BigQueryBackend {
    project_id: String,
    dataset_id: String,
    client: bigquery::Client,
}

impl StorageBackend for BigQueryBackend {
    fn backend_id(&self) -> &str { "bigquery" }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: false,  // Not ACID
            supports_graph_traversal: false,
            supports_aggregations: true,  // Excellent for aggregations
            supports_fulltext_search: false,
            supports_geospatial_queries: true,
            supports_time_series: true,
            supports_ttl: true,
            latency_ms: 5000,  // Higher latency but massive throughput
            throughput_rps: 1_000_000,
            storage_format: StorageFormat::Columnar,
        }
    }

    async fn aggregate(&self, agg: &AggregationQuery) -> Result<AggregationResult> {
        // BigQuery excels at this
        let sql = format!(
            "SELECT {} FROM {}.observations WHERE {} GROUP BY {} LIMIT {}",
            agg.aggregation_expr(),
            self.dataset_id,
            agg.filter_expr(),
            agg.group_by_expr(),
            agg.limit
        );

        let result = self.client.query(&sql).await?;
        Ok(AggregationResult::from(result))
    }
}
```

#### 3. Neo4j (Graph Database)

```rust
pub struct Neo4jBackend {
    driver: neo4j::Driver,
    session: neo4j::Session,
}

impl StorageBackend for Neo4jBackend {
    fn backend_id(&self) -> &str { "neo4j" }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: true,
            supports_graph_traversal: true,  // Core strength
            supports_aggregations: true,
            supports_fulltext_search: false,
            supports_geospatial_queries: false,  // Not spatial
            supports_time_series: false,
            supports_ttl: false,
            latency_ms: 50,
            throughput_rps: 50_000,
            storage_format: StorageFormat::Graph,
        }
    }

    async fn traverse_graph(&self, start_node: &str, spec: &GraphTraversal) -> Result<Vec<String>> {
        // Excellent for relationship queries
        let cypher = format!(
            "MATCH (start:Observation {{ id: '{}' }})-[:{}]->* (end:Observation)
             RETURN end.id LIMIT {}",
            start_node,
            spec.relationship_type,
            spec.max_depth
        );

        let result = self.session.run(cypher, None).await?;
        Ok(result.records().iter().map(|r| r.get(0)).collect())
    }
}
```

#### 4. S3/Object Storage (Archive)

```rust
pub struct S3Backend {
    bucket: String,
    client: aws_sdk_s3::Client,
    compression: CompressionFormat,
}

impl StorageBackend for S3Backend {
    fn backend_id(&self) -> &str { "s3" }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: false,
            supports_graph_traversal: false,
            supports_aggregations: false,
            supports_fulltext_search: false,
            supports_geospatial_queries: false,
            supports_time_series: false,
            supports_ttl: true,  // Via lifecycle policies
            latency_ms: 100,
            throughput_rps: 100_000,
            storage_format: StorageFormat::ObjectStorage,
        }
    }

    async fn insert_batch(&self, obs: Vec<Observation>, policy: &DataPlacementPolicy) -> Result<Vec<ObservationId>> {
        let ids: Vec<_> = obs.iter().map(|o| o.id.clone()).collect();

        // Compress and batch
        let compressed = self.compress_batch(&obs, policy)?;
        let key = format!("observations/{}/{}.parquet.gz", 
            chrono::Utc::now().date_naive(), 
            uuid::Uuid::new_v4()
        );

        self.client.put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(aws_sdk_s3::primitives::ByteStream::from(compressed))
            .expire_after(policy.retention_policy.expires_in())
            .send()
            .await?;

        Ok(ids)
    }
}
```

#### 5. Redis (Cache)

```rust
pub struct RedisBackend {
    client: redis::Client,
    ttl_seconds: u64,
}

impl StorageBackend for RedisBackend {
    fn backend_id(&self) -> &str { "redis" }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_transactions: true,
            supports_graph_traversal: false,
            supports_aggregations: false,
            supports_fulltext_search: false,
            supports_geospatial_queries: true,  // Via Redis-Stack
            supports_time_series: true,
            supports_ttl: true,
            latency_ms: 1,  // Sub-millisecond
            throughput_rps: 1_000_000,
            storage_format: StorageFormat::KeyValue,
        }
    }

    async fn insert_observation(&self, obs: &Observation, policy: &DataPlacementPolicy) -> Result<ObservationId> {
        let key = format!("obs:{}", obs.id);
        let ttl = self.ttl_from_policy(policy);

        self.client
            .set_with_expiry(&key, serde_json::to_string(obs)?, ttl)
            .await?;

        Ok(obs.id.clone())
    }

    async fn query_spatial_temporal(&self, region: &Region, time_range: &TimeRange, limit: usize) -> Result<Vec<Observation>> {
        // Use Redis Search capability
        let query = format!(
            "@location:[{} {} {} {}] @time:[{} {}]",
            region.south, region.west, region.north, region.east,
            time_range.start, time_range.end
        );

        let results = self.client.ft_search("observations:idx", &query, None)?;
        Ok(results.into_iter().take(limit).collect())
    }
}
```

---

## Query Federation Layer

### Unified Query Interface

```rust
pub struct FederatedQuery {
    /// Logical query (storage-agnostic)
    pub query: LogicalQuery,
    
    /// Data placement policies
    pub placement_policy: DataPlacementPolicy,
    
    /// Available backends
    pub backends: Vec<Arc<dyn StorageBackend>>,
    
    /// Query optimizer
    pub optimizer: QueryOptimizer,
}

impl FederatedQuery {
    pub async fn execute(&self) -> Result<QueryResult> {
        // 1. Analyze query
        let analysis = self.analyze_query(&self.query)?;

        // 2. Select best backends
        let selected_backends = self.select_backends(&analysis)?;

        // 3. Parallelize execution
        let results = self.execute_parallel(selected_backends).await?;

        // 4. Merge results
        Ok(self.merge_results(results)?)
    }

    fn select_backends(&self, analysis: &QueryAnalysis) -> Result<Vec<Arc<dyn StorageBackend>>> {
        let mut backends = Vec::new();

        if analysis.requires_aggregation {
            // BigQuery is best for aggregations
            if let Some(warehouse) = self.find_backend("bigquery") {
                backends.push(warehouse);
            }
        }

        if analysis.requires_graph_traversal {
            // Neo4j is best for relationships
            if let Some(graph_db) = self.find_backend("neo4j") {
                backends.push(graph_db);
            }
        }

        if analysis.requires_recent_data {
            // PostgreSQL or Redis for hot data
            if let Some(cache) = self.find_backend("redis") {
                backends.push(cache);
            }
        }

        if analysis.requires_historical_data {
            // Object storage for archives
            if let Some(archive) = self.find_backend("s3") {
                backends.push(archive);
            }
        }

        Ok(backends)
    }

    async fn execute_parallel(&self, backends: Vec<Arc<dyn StorageBackend>>) -> Result<Vec<PartialResult>> {
        let futures: Vec<_> = backends
            .iter()
            .map(|backend| self.execute_on_backend(backend.clone()))
            .collect();

        let results = futures::future::join_all(futures).await;
        Ok(results.into_iter().filter_map(|r| r.ok()).collect())
    }

    fn merge_results(&self, partial: Vec<PartialResult>) -> Result<QueryResult> {
        // Combine results from multiple backends
        // Handle deduplication, sorting, aggregation
        match self.query.operation {
            QueryOp::Aggregate(agg_type) => self.merge_aggregations(partial, agg_type),
            QueryOp::Spatial => self.merge_spatial_results(partial),
            QueryOp::Graph => self.merge_graph_results(partial),
        }
    }
}
```

### Query Optimization

```rust
pub struct QueryOptimizer {
    backend_stats: BackendStatistics,
    cost_model: CostModel,
}

impl QueryOptimizer {
    pub fn estimate_cost(&self, query: &LogicalQuery, backend: &dyn StorageBackend) -> QueryCost {
        let capabilities = backend.capabilities();
        
        QueryCost {
            estimated_latency_ms: self.estimate_latency(query, capabilities),
            estimated_cpu_percent: self.estimate_cpu(query, capabilities),
            estimated_memory_mb: self.estimate_memory(query, capabilities),
            capability_match: self.capability_match_score(query, capabilities),
        }
    }

    pub fn select_optimal_backend(
        &self,
        query: &LogicalQuery,
        backends: &[Arc<dyn StorageBackend>],
    ) -> Result<Arc<dyn StorageBackend>> {
        backends
            .iter()
            .map(|b| (b, self.estimate_cost(query, b.as_ref())))
            .min_by(|(_, cost_a), (_, cost_b)| cost_a.total_score().cmp(&cost_b.total_score()))
            .map(|(b, _)| b.clone())
            .ok_or("No suitable backend found".into())
    }
}
```

---

## Data Placement Policies

### Tier Assignment

```rust
pub enum DataTier {
    Hot,    // Current, frequently accessed
    Warm,   // Recent, occasionally accessed
    Cold,   // Old, rarely accessed
    Archive, // Retention only
}

pub struct DataPlacementPolicy {
    pub hot_retention: Duration,     // <1 day → PostgreSQL
    pub warm_retention: Duration,    // 1-90 days → Warehouse
    pub cold_retention: Duration,    // >90 days → S3
    pub archive_retention: Duration, // Permanent → Glacier
    pub compliance_regions: Vec<String>, // Data residency
}

impl DataPlacementPolicy {
    pub fn placement_tier(&self, observation: &Observation) -> DataTier {
        let age = now() - observation.timestamp;

        match age {
            d if d < self.hot_retention => DataTier::Hot,
            d if d < self.warm_retention => DataTier::Warm,
            d if d < self.cold_retention => DataTier::Cold,
            _ => DataTier::Archive,
        }
    }

    pub fn route_to_backend(&self, observation: &Observation) -> &str {
        match self.placement_tier(observation) {
            DataTier::Hot => "postgres",
            DataTier::Warm => "bigquery",
            DataTier::Cold => "s3",
            DataTier::Archive => "glacier",
        }
    }

    pub fn allowed_backends(&self, observation: &Observation) -> Vec<&str> {
        let tier = self.placement_tier(observation);

        // Identify backends in allowed regions
        self.compliance_regions
            .iter()
            .filter_map(|region| self.region_to_backend(region, tier))
            .collect()
    }
}
```

### Automatic Tiering

```rust
pub struct TieringManager {
    policy: Arc<DataPlacementPolicy>,
    backends: Arc<HashMap<String, Arc<dyn StorageBackend>>>,
}

impl TieringManager {
    pub async fn tier_observations(&self) -> Result<TieringStats> {
        // Find all observations that should move tiers
        let to_move = self.find_observations_to_move().await?;

        let mut stats = TieringStats::default();

        for (observation, from_backend, to_backend) in to_move {
            // Read from source backend
            let data = self.backends[from_backend].get_by_id(&observation.id).await??;

            // Write to target backend
            self.backends[to_backend].insert_observation(&data, &self.policy).await?;

            // Delete from source (after confirmation)
            self.backends[from_backend].delete_by_policy(&self.policy).await?;

            stats.moved += 1;
        }

        Ok(stats)
    }

    pub async fn find_observations_to_move(&self) -> Result<Vec<(Observation, String, String)>> {
        // Query all backends for observations that have aged into a different tier
        let mut to_move = Vec::new();

        for (backend_id, backend) in self.backends.iter() {
            let observations = backend.query_spatial_temporal(
                &Region::all(),
                &TimeRange::all(),
                usize::MAX,
            ).await?;

            for obs in observations {
                let target_tier = self.policy.placement_tier(&obs);
                let target_backend = self.policy.route_to_backend(&obs);

                if target_backend != backend_id {
                    to_move.push((obs, backend_id.clone(), target_backend.to_string()));
                }
            }
        }

        Ok(to_move)
    }
}
```

---

## Compliance and Data Residency

### Residency Constraints

```rust
pub struct ResidencyPolicy {
    /// Which backends can store which data
    pub constraints: HashMap<String, Vec<String>>, // data_class → allowed_backends
    
    /// Geographic constraints
    pub regional_constraints: HashMap<String, Vec<String>>, // region → allowed_locations
    
    /// Retention by classification
    pub retention_by_classification: HashMap<DataClassification, RetentionPolicy>,
}

impl ResidencyPolicy {
    pub fn validate_placement(&self, observation: &Observation, backend_id: &str) -> Result<()> {
        let classification = &observation.compliance_metadata.classification;

        // Check data classification constraints
        let allowed = self.constraints
            .get(&classification.to_string())
            .ok_or("No backends allowed for this classification")?;

        if !allowed.contains(&backend_id.to_string()) {
            return Err(format!("Backend {} not allowed for {}", backend_id, classification).into());
        }

        // Check geographic constraints
        let backend_region = self.get_backend_region(backend_id)?;
        let required_regions = &self.regional_constraints;

        if let Some(required) = required_regions.get(&backend_region) {
            if !required.is_empty() && !required.contains(&backend_region) {
                return Err(format!("Data cannot reside in region {}", backend_region).into());
            }
        }

        Ok(())
    }

    pub fn retention_for(&self, observation: &Observation) -> RetentionPolicy {
        let classification = &observation.compliance_metadata.classification;
        self.retention_by_classification
            .get(classification)
            .cloned()
            .unwrap_or(RetentionPolicy::Years(7))
    }
}
```

### Audit Trail for Data Movement

```rust
pub struct DataMovementAudit {
    pub observation_id: Uuid,
    pub from_backend: String,
    pub to_backend: String,
    pub reason: MovementReason,
    pub timestamp_us: i64,
    pub verified_by: String,
}

pub enum MovementReason {
    TieringPolicy,
    UserRequest,
    ComplianceEnforcement,
    DisasterRecovery,
    Rebalancing,
}

pub struct DataMovementLog {
    entries: parking_lot::RwLock<Vec<DataMovementAudit>>,
}

impl DataMovementLog {
    pub fn record_movement(&self, audit: DataMovementAudit) {
        self.entries.write().push(audit);
    }

    pub fn audit_trail(&self, observation_id: &Uuid) -> Vec<DataMovementAudit> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.observation_id == *observation_id)
            .cloned()
            .collect()
    }
}
```

---

## Backend Configuration

### Registry Pattern

```rust
pub struct BackendRegistry {
    backends: HashMap<String, Arc<dyn StorageBackend>>,
    policies: Arc<DataPlacementPolicy>,
    compliance: Arc<ResidencyPolicy>,
}

impl BackendRegistry {
    pub fn builder() -> BackendRegistryBuilder {
        BackendRegistryBuilder::new()
    }

    pub fn register(&mut self, backend: Arc<dyn StorageBackend>) {
        self.backends.insert(backend.backend_id().to_string(), backend);
    }

    pub fn get(&self, id: &str) -> Result<Arc<dyn StorageBackend>> {
        self.backends.get(id).cloned()
            .ok_or(format!("Backend not found: {}", id).into())
    }

    pub fn all_backends(&self) -> Vec<Arc<dyn StorageBackend>> {
        self.backends.values().cloned().collect()
    }

    pub fn backends_for_tier(&self, tier: DataTier) -> Vec<Arc<dyn StorageBackend>> {
        self.backends.values()
            .filter(|b| self.tier_matches_backend(tier, b.as_ref()))
            .cloned()
            .collect()
    }
}

pub struct BackendRegistryBuilder {
    backends: HashMap<String, Arc<dyn StorageBackend>>,
}

impl BackendRegistryBuilder {
    pub fn new() -> Self {
        BackendRegistryBuilder {
            backends: HashMap::new(),
        }
    }

    pub fn with_postgres(mut self, url: &str) -> Result<Self> {
        let backend = Arc::new(PostgresBackend::new(url)?);
        self.backends.insert("postgres".to_string(), backend);
        Ok(self)
    }

    pub fn with_bigquery(mut self, project_id: &str, dataset_id: &str) -> Result<Self> {
        let backend = Arc::new(BigQueryBackend::new(project_id, dataset_id)?);
        self.backends.insert("bigquery".to_string(), backend);
        Ok(self)
    }

    pub fn with_neo4j(mut self, uri: &str) -> Result<Self> {
        let backend = Arc::new(Neo4jBackend::new(uri)?);
        self.backends.insert("neo4j".to_string(), backend);
        Ok(self)
    }

    pub fn with_s3(mut self, bucket: &str, region: &str) -> Result<Self> {
        let backend = Arc::new(S3Backend::new(bucket, region)?);
        self.backends.insert("s3".to_string(), backend);
        Ok(self)
    }

    pub fn with_redis(mut self, url: &str) -> Result<Self> {
        let backend = Arc::new(RedisBackend::new(url)?);
        self.backends.insert("redis".to_string(), backend);
        Ok(self)
    }

    pub fn build(self) -> BackendRegistry {
        BackendRegistry {
            backends: self.backends,
            policies: Arc::new(DataPlacementPolicy::default()),
            compliance: Arc::new(ResidencyPolicy::default()),
        }
    }
}
```

---

## Query Examples

### Example 1: Simple Spatial Query

```rust
// Logical query (application layer)
world.query()
    .location(&region)
    .time_range(&last_24h)
    .execute()
    .await?

// Federation layer:
// 1. Analyzes: recent data (~24h) → hot tier
// 2. Selects: PostgreSQL (hot tier backend)
// 3. Executes: direct query to PostgreSQL
// 4. Returns: observations from PostgreSQL
```

### Example 2: Historical Aggregation

```rust
world.query()
    .aggregate(Aggregation::Count)
    .region(&region)
    .time_range(&last_year)
    .execute()
    .await?

// Federation layer:
// 1. Analyzes: aggregation over year of data
// 2. Selects: BigQuery (best for aggregations)
// 3. Routes queries:
//    - BigQuery: data from 1-365 days ago (warm/cold)
//    - Postgres: data from 0-1 days ago (hot)
// 4. Executes in parallel
// 5. Merges results locally
```

### Example 3: Graph Relationship Query

```rust
world.query()
    .observation(&obs_id)
    .related_by("sensor_fusion")
    .depth(3)
    .execute()
    .await?

// Federation layer:
// 1. Analyzes: relationship traversal
// 2. Selects: Neo4j (built for this)
// 3. Executes: Cypher query on Neo4j
// 4. Returns: relationship graph
```

### Example 4: Compliance-Aware Query

```rust
world.query()
    .classification(DataClassification::Restricted)
    .location(&region)
    .execute()
    .await?

// Federation layer:
// 1. Analyzes: restricted data
// 2. Consults: ResidencyPolicy
// 3. Selects: Only backends allowed for restricted data
//    (maybe only on-premises PostgreSQL)
// 4. Executes: query only on compliant backend
// 5. Ensures: audit trail of access
```

---

## Migration Between Backends

### Seamless Backend Swap

```rust
impl BackendRegistry {
    pub async fn migrate_backend(
        &self,
        from: &str,
        to: &str,
        filter: &ObservationFilter,
    ) -> Result<MigrationStats> {
        let source = self.get(from)?;
        let target = self.get(to)?;

        let observations = source.query_spatial_temporal(
            &filter.region,
            &filter.time_range,
            usize::MAX,
        ).await?;

        let mut stats = MigrationStats::default();
        const BATCH_SIZE: usize = 10_000;

        for batch in observations.chunks(BATCH_SIZE) {
            let batch_vec = batch.to_vec();
            target.insert_batch(batch_vec, &self.policies).await?;
            stats.migrated += batch.len();
        }

        // Verify migration complete before deleting from source
        for obs in observations {
            if let Ok(Some(_)) = target.get_by_id(&obs.id).await {
                source.delete_by_policy(&self.policies).await?;
                stats.verified += 1;
            }
        }

        Ok(stats)
    }
}
```

### Zero-Downtime Replication

```rust
pub struct BackendReplicator {
    primary: Arc<dyn StorageBackend>,
    replicas: Vec<Arc<dyn StorageBackend>>,
}

impl BackendReplicator {
    pub async fn insert_with_replication(&self, obs: &Observation) -> Result<()> {
        // Write to primary first
        self.primary.insert_observation(obs, &self.policies).await?;

        // Replicate to secondaries (non-blocking)
        let replicas = self.replicas.clone();
        let obs = obs.clone();
        tokio::spawn(async move {
            for replica in replicas {
                let _ = replica.insert_observation(&obs, &self.policies).await;
            }
        });

        Ok(())
    }
}
```

---

## Testing and Validation

### Backend Compatibility Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_backend() {
        let backend = PostgresBackend::new("postgres://localhost/test")?;
        assert_eq!(backend.backend_id(), "postgres");
        assert!(backend.capabilities().supports_transactions);
    }

    #[tokio::test]
    async fn test_spatial_temporal_query() {
        let backends = vec![
            Arc::new(PostgresBackend::new("...")?) as Arc<dyn StorageBackend>,
            Arc::new(BigQueryBackend::new("...", "...")?) as Arc<dyn StorageBackend>,
        ];

        let query = LogicalQuery::SpatialTemporal {
            region: test_region(),
            time_range: last_24h(),
        };

        for backend in backends {
            let results = backend.query_spatial_temporal(
                &query.region,
                &query.time_range,
                1000,
            ).await?;

            assert!(!results.is_empty());
        }
    }

    #[tokio::test]
    async fn test_federation_layer() {
        let registry = BackendRegistry::builder()
            .with_postgres("...")?
            .with_bigquery("...", "...")?
            .with_s3("...", "...")?
            .build();

        let query = LogicalQuery::Aggregate {
            aggregation: Aggregation::Count,
            time_range: TimeRange::years(1),
        };

        let result = FederatedQuery {
            query,
            placement_policy: default_policy(),
            backends: registry.all_backends(),
            optimizer: QueryOptimizer::new(),
        }.execute().await?;

        assert!(result.count > 0);
    }

    #[test]
    fn test_compliance_validation() {
        let policy = ResidencyPolicy::restricted_to_eu();
        let obs = test_observation_with_classification(DataClassification::Restricted);

        // EU backend should pass
        assert!(policy.validate_placement(&obs, "postgres-eu").is_ok());

        // US backend should fail
        assert!(policy.validate_placement(&obs, "postgres-us").is_err());
    }
}
```

---

## Implementation Roadmap

### Phase 1: Core Abstraction (Week 21)
- ✓ StorageBackend trait definition
- ✓ PostgreSQL backend (reference implementation)
- ✓ Query federation layer
- ✓ Data placement policies
- 20 tests

### Phase 2: Additional Backends (Week 22)
- ✓ BigQuery backend
- ✓ S3 backend
- ✓ Redis backend
- 15 tests

### Phase 3: Compliance & Governance (Week 23)
- ✓ Residency policies
- ✓ Audit trails for data movement
- ✓ Compliance validation
- 10 tests

### Phase 4: Query Optimization (Week 24)
- ✓ Query cost model
- ✓ Backend selection algorithm
- ✓ Result merging
- 15 tests

### Phase 5: Enterprise Features (Week 25+)
- ✓ Replication and failover
- ✓ Zero-downtime migration
- ✓ Multi-region deployment
- ✓ Backup and recovery

---

## Deployment Scenarios

### Scenario 1: Local Development
```yaml
backends:
  postgres:
    url: postgres://localhost/pyterrain
  redis:
    url: redis://localhost:6379

policies:
  hot_retention: 7 days
  warm_retention: 90 days
```

### Scenario 2: Team Environment
```yaml
backends:
  postgres:
    url: postgres://prod-db.internal/pyterrain
    pool_size: 50
  redis:
    url: redis://cache.internal
    ttl_seconds: 3600

policies:
  hot_retention: 14 days
  warm_retention: 180 days
```

### Scenario 3: Production Polyglot
```yaml
backends:
  postgres:
    url: postgres://operational-db.aws/pyterrain
    role: hot_data
  bigquery:
    project: my-project
    dataset: pyterrain_warehouse
    role: warm_data
  neo4j:
    uri: neo4j://graph-db.aws:7687
    role: relationships
  s3:
    bucket: pyterrain-archive
    role: cold_data
  redis:
    url: redis://cache.aws:6379
    role: real_time

compliance:
  regional_constraints:
    us: [postgres-us-east, bigquery-us, s3-us]
    eu: [postgres-eu, bigquery-eu, s3-eu]
  restricted_to_region: eu
  retention_policy:
    public: 7 years
    internal: 5 years
    confidential: 3 years
    restricted: permanent
```

---

## Guiding Philosophy Summary

**The platform owns the data model. The organization owns the storage strategy.**

1. **Abstraction**: Storage is an implementation detail, not a dependency
2. **Flexibility**: Use any combination of backends simultaneously
3. **Evolution**: Start simple (PostgreSQL), scale to complex (polyglot persistent)
4. **Portability**: Data remains accessible, never vendor-locked
5. **Compliance**: Storage choices respect regulatory constraints
6. **Performance**: Optimization based on backend capabilities, not one-size-fits-all
7. **Governance**: Data placement is policy-driven, auditable, enforceable

Organizations should never face the choice:
- Keep using our framework (and redesign everything), OR
- Migrate to a different system (because ours doesn't support their storage)

Instead, they choose:
- Continue with PyTerrainMap (and use whatever storage makes sense)

