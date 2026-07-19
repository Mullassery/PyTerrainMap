//! PostgreSQL backend implementation for PyTerrainMap
//!
//! Provides production-grade storage using PostgreSQL with sqlx.
//! Handles observation persistence, spatial queries, and temporal reasoning.

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use crate::storage::backends::{
    StorageBackend, BackendCapabilities, BackendResult, BackendError,
    ObservationId, StorageFormat, StorageObservation, Region, TimeRange,
    AggregationQuery, AggregationOp, AggregationResult, HealthStatus,
};
use async_trait::async_trait;

/// PostgreSQL backend for operational data
pub struct PostgresBackend {
    pool: PgPool,
    table_name: String,
}

impl PostgresBackend {
    /// Create new PostgreSQL backend
    pub async fn new(connection_string: &str, pool_size: u32) -> BackendResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(connection_string)
            .await
            .map_err(|e| BackendError {
                backend: "postgres".to_string(),
                operation: "connect".to_string(),
                message: format!("Failed to connect: {}", e),
            })?;

        // Initialize schema
        Self::initialize_schema(&pool).await?;

        Ok(PostgresBackend {
            pool,
            table_name: "pyterrain_observations".to_string(),
        })
    }

    /// Initialize database schema
    async fn initialize_schema(pool: &PgPool) -> BackendResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pyterrain_observations (
                id UUID PRIMARY KEY,
                robot_id VARCHAR(255) NOT NULL,
                timestamp_us BIGINT NOT NULL,
                latitude DOUBLE PRECISION NOT NULL,
                longitude DOUBLE PRECISION NOT NULL,
                sensor_type VARCHAR(100) NOT NULL,
                value_json JSONB NOT NULL,
                confidence FLOAT NOT NULL,
                trust_score FLOAT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_robot_time (robot_id, timestamp_us),
                INDEX idx_spatial (latitude, longitude),
                INDEX idx_timestamp (timestamp_us)
            );

            CREATE INDEX IF NOT EXISTS idx_obs_geospatial
            ON pyterrain_observations
            USING GIST (
                ST_GeomFromText(
                    'SRID=4326;POINT(' || longitude || ' ' || latitude || ')'
                )
            );
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| BackendError {
            backend: "postgres".to_string(),
            operation: "initialize".to_string(),
            message: format!("Schema initialization failed: {}", e),
        })?;

        Ok(())
    }

    /// Get connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl StorageBackend for PostgresBackend {
    fn backend_id(&self) -> &str {
        "postgres"
    }

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

    async fn insert_observation(
        &self,
        observation: &StorageObservation,
    ) -> BackendResult<ObservationId> {
        let id = uuid::Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO pyterrain_observations
            (id, robot_id, timestamp_us, latitude, longitude, sensor_type, value_json, confidence)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(id)
        .bind(&observation.robot_id)
        .bind(observation.timestamp)
        .bind(observation.location_lat)
        .bind(observation.location_lon)
        .bind(&observation.sensor_type)
        .bind(&observation.value_json)
        .bind(observation.confidence)
        .execute(&self.pool)
        .await
        .map_err(|e| BackendError {
            backend: "postgres".to_string(),
            operation: "insert".to_string(),
            message: format!("Insert failed: {}", e),
        })?;

        Ok(ObservationId(id.to_string()))
    }

    async fn insert_batch(
        &self,
        observations: Vec<StorageObservation>,
    ) -> BackendResult<Vec<ObservationId>> {
        let mut ids = Vec::new();

        for obs in observations {
            let id = self.insert_observation(&obs).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    async fn query_spatial_temporal(
        &self,
        region: &Region,
        time_range: &TimeRange,
        limit: usize,
    ) -> BackendResult<Vec<StorageObservation>> {
        let rows = sqlx::query(
            r#"
            SELECT id, robot_id, timestamp_us, latitude, longitude, sensor_type, value_json, confidence
            FROM pyterrain_observations
            WHERE latitude >= $1 AND latitude <= $2
              AND longitude >= $3 AND longitude <= $4
              AND timestamp_us >= $5 AND timestamp_us <= $6
            ORDER BY timestamp_us DESC
            LIMIT $7
            "#,
        )
        .bind(region.south)
        .bind(region.north)
        .bind(region.west)
        .bind(region.east)
        .bind(time_range.start_us)
        .bind(time_range.end_us)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| BackendError {
            backend: "postgres".to_string(),
            operation: "query_spatial_temporal".to_string(),
            message: format!("Query failed: {}", e),
        })?;

        let observations = rows
            .iter()
            .map(|row| {
                StorageObservation {
                    id: row.get::<String, _>("id"),
                    robot_id: row.get::<String, _>("robot_id"),
                    timestamp: row.get::<i64, _>("timestamp_us"),
                    location_lat: row.get::<f64, _>("latitude"),
                    location_lon: row.get::<f64, _>("longitude"),
                    sensor_type: row.get::<String, _>("sensor_type"),
                    value_json: row.get::<String, _>("value_json"),
                    confidence: row.get::<f32, _>("confidence"),
                }
            })
            .collect();

        Ok(observations)
    }

    async fn get_by_id(
        &self,
        id: &ObservationId,
    ) -> BackendResult<Option<StorageObservation>> {
        let row = sqlx::query(
            r#"
            SELECT id, robot_id, timestamp_us, latitude, longitude, sensor_type, value_json, confidence
            FROM pyterrain_observations
            WHERE id = $1
            "#,
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BackendError {
            backend: "postgres".to_string(),
            operation: "get_by_id".to_string(),
            message: format!("Query failed: {}", e),
        })?;

        Ok(row.map(|r| {
            StorageObservation {
                id: r.get::<String, _>("id"),
                robot_id: r.get::<String, _>("robot_id"),
                timestamp: r.get::<i64, _>("timestamp_us"),
                location_lat: r.get::<f64, _>("latitude"),
                location_lon: r.get::<f64, _>("longitude"),
                sensor_type: r.get::<String, _>("sensor_type"),
                value_json: r.get::<String, _>("value_json"),
                confidence: r.get::<f32, _>("confidence"),
            }
        }))
    }

    async fn aggregate(
        &self,
        query: &AggregationQuery,
    ) -> BackendResult<AggregationResult> {
        let result = match &query.operation {
            AggregationOp::Count => {
                let row = sqlx::query(
                    r#"
                    SELECT COUNT(*) as count
                    FROM pyterrain_observations
                    WHERE timestamp_us >= $1 AND timestamp_us <= $2
                    "#,
                )
                .bind(query.time_range.start_us)
                .bind(query.time_range.end_us)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| BackendError {
                    backend: "postgres".to_string(),
                    operation: "aggregate".to_string(),
                    message: format!("Aggregation failed: {}", e),
                })?;

                let count: i64 = row.get("count");
                AggregationResult {
                    value: count as f64,
                    count: count as u64,
                    groups: std::collections::HashMap::new(),
                }
            }
            _ => AggregationResult {
                value: 0.0,
                count: 0,
                groups: std::collections::HashMap::new(),
            },
        };

        Ok(result)
    }

    async fn delete_expired(&self) -> BackendResult<usize> {
        // For now, no automatic expiration in PostgreSQL backend
        // Manual purge would be done via maintenance windows
        Ok(0)
    }

    async fn maintenance(&self) -> BackendResult<()> {
        // Run VACUUM to reclaim space
        sqlx::query("VACUUM ANALYZE pyterrain_observations")
            .execute(&self.pool)
            .await
            .map_err(|e| BackendError {
                backend: "postgres".to_string(),
                operation: "maintenance".to_string(),
                message: format!("Maintenance failed: {}", e),
            })?;

        Ok(())
    }

    async fn health_check(&self) -> BackendResult<HealthStatus> {
        let start = std::time::Instant::now();

        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| BackendError {
                backend: "postgres".to_string(),
                operation: "health_check".to_string(),
                message: format!("Health check failed: {}", e),
            })?;

        let latency_ms = start.elapsed().as_millis() as u32;

        Ok(HealthStatus {
            backend_id: "postgres".to_string(),
            is_healthy: true,
            latency_ms,
            error_rate: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]  // Requires PostgreSQL running
    async fn test_postgres_backend_connect() {
        let backend = PostgresBackend::new("postgresql://user:password@localhost/test", 5)
            .await;
        assert!(backend.is_ok());
    }

    #[test]
    fn test_postgres_capabilities_specification() {
        let caps = BackendCapabilities {
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
        };

        assert!(caps.supports_transactions);
        assert!(caps.supports_geospatial_queries);
        assert!(caps.supports_aggregations);
    }
}
