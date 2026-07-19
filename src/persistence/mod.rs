//! Persistence layer for PyTerrainMap
//!
//! Supports SQLite for local development and PostgreSQL for production.
//! Handles observation archival, querying, and data lifecycle management.

use serde::{Deserialize, Serialize};

/// Database backend type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseBackend {
    /// SQLite (file-based, single process)
    SQLite,
    /// PostgreSQL (server-based, multi-process)
    PostgreSQL,
}

/// Database connection configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Backend type
    pub backend: DatabaseBackend,
    /// Connection string (SQLite path or PostgreSQL URL)
    pub connection_string: String,
    /// Maximum connection pool size
    pub max_connections: u32,
    /// Enable query logging
    pub query_logging: bool,
    /// Automatic archival after N days
    pub auto_archive_days: Option<u32>,
}

impl DatabaseConfig {
    /// Create SQLite config (local development)
    pub fn sqlite(path: &str) -> Self {
        DatabaseConfig {
            backend: DatabaseBackend::SQLite,
            connection_string: path.to_string(),
            max_connections: 1,
            query_logging: false,
            auto_archive_days: None,
        }
    }

    /// Create PostgreSQL config (production)
    pub fn postgresql(url: &str) -> Self {
        DatabaseConfig {
            backend: DatabaseBackend::PostgreSQL,
            connection_string: url.to_string(),
            max_connections: 16,
            query_logging: false,
            auto_archive_days: Some(90),
        }
    }

    /// Enable query logging
    pub fn with_logging(mut self) -> Self {
        self.query_logging = true;
        self
    }

    /// Set auto-archival
    pub fn with_auto_archive(mut self, days: u32) -> Self {
        self.auto_archive_days = Some(days);
        self
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::sqlite("pyterrain.db")
    }
}

/// Observation storage record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageRecord {
    /// Observation ID (UUID)
    pub observation_id: String,
    /// Robot ID
    pub robot_id: String,
    /// Sensor type
    pub sensor_type: String,
    /// Location (WGS84)
    pub location: String, // GeoJSON Point as string
    /// Timestamp (microseconds since epoch)
    pub timestamp_us: i64,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
    /// Value (JSON)
    pub value: String,
    /// Metadata (JSON)
    pub metadata: String,
    /// Archived (soft delete)
    pub archived: bool,
    /// Archive timestamp
    pub archived_at: Option<i64>,
}

/// Query for historical observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageQuery {
    /// Start timestamp (microseconds)
    pub from_timestamp_us: i64,
    /// End timestamp (microseconds)
    pub to_timestamp_us: i64,
    /// Filter by robot IDs (optional)
    pub robot_ids: Option<Vec<String>>,
    /// Filter by sensor types (optional)
    pub sensor_types: Option<Vec<String>>,
    /// Geographic bounds (optional)
    pub bounds: Option<GeographicBounds>,
    /// Minimum confidence (0.0-1.0)
    pub min_confidence: Option<f32>,
    /// Limit result count
    pub limit: Option<u32>,
}

impl StorageQuery {
    /// Create query
    pub fn new(from: i64, to: i64) -> Self {
        StorageQuery {
            from_timestamp_us: from,
            to_timestamp_us: to,
            robot_ids: None,
            sensor_types: None,
            bounds: None,
            min_confidence: None,
            limit: None,
        }
    }

    /// Filter by robot IDs
    pub fn with_robots(mut self, robots: Vec<String>) -> Self {
        self.robot_ids = Some(robots);
        self
    }

    /// Filter by sensor types
    pub fn with_sensors(mut self, sensors: Vec<String>) -> Self {
        self.sensor_types = Some(sensors);
        self
    }

    /// Filter by geographic bounds
    pub fn with_bounds(mut self, bounds: GeographicBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Filter by minimum confidence
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = Some(confidence);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// Geographic bounds for spatial queries
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GeographicBounds {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl GeographicBounds {
    /// Create bounds
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        GeographicBounds { west, south, east, north }
    }
}

/// Database statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    /// Total observations
    pub total_observations: u64,
    /// Active observations (not archived)
    pub active_observations: u64,
    /// Archived observations
    pub archived_observations: u64,
    /// Unique robots
    pub unique_robots: u32,
    /// Unique sensor types
    pub unique_sensor_types: u32,
    /// Database size (bytes)
    pub database_size_bytes: u64,
    /// Oldest observation timestamp
    pub oldest_timestamp_us: Option<i64>,
    /// Newest observation timestamp
    pub newest_timestamp_us: Option<i64>,
}

impl Default for DatabaseStats {
    fn default() -> Self {
        DatabaseStats {
            total_observations: 0,
            active_observations: 0,
            archived_observations: 0,
            unique_robots: 0,
            unique_sensor_types: 0,
            database_size_bytes: 0,
            oldest_timestamp_us: None,
            newest_timestamp_us: None,
        }
    }
}

/// Index configuration for performance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index on timestamp (for time-range queries)
    pub index_timestamp: bool,
    /// Index on robot_id (for robot-specific queries)
    pub index_robot_id: bool,
    /// Index on sensor_type
    pub index_sensor_type: bool,
    /// Spatial index on location (if supported)
    pub index_location: bool,
    /// Index on confidence
    pub index_confidence: bool,
}

impl IndexConfig {
    /// Create with all indexes enabled
    pub fn all() -> Self {
        IndexConfig {
            index_timestamp: true,
            index_robot_id: true,
            index_sensor_type: true,
            index_location: true,
            index_confidence: true,
        }
    }

    /// Create with minimal indexes (faster writes)
    pub fn minimal() -> Self {
        IndexConfig {
            index_timestamp: true,
            index_robot_id: false,
            index_sensor_type: false,
            index_location: false,
            index_confidence: false,
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self::all()
    }
}

/// Archival policy for data lifecycle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchivalPolicy {
    /// Archive observations older than N days
    pub archive_after_days: u32,
    /// Delete archived observations after N days
    pub delete_after_days: u32,
    /// Enable compression for archived data
    pub compress_archived: bool,
    /// Compression level (1-9)
    pub compression_level: u8,
}

impl ArchivalPolicy {
    /// Create policy
    pub fn new(archive_days: u32, delete_days: u32) -> Self {
        ArchivalPolicy {
            archive_after_days: archive_days,
            delete_after_days: delete_days,
            compress_archived: true,
            compression_level: 6,
        }
    }
}

impl Default for ArchivalPolicy {
    fn default() -> Self {
        Self::new(90, 365) // Archive after 90 days, delete after 1 year
    }
}

/// Database performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average query time (milliseconds)
    pub avg_query_time_ms: f32,
    /// Queries per second
    pub queries_per_second: f32,
    /// Write throughput (observations per second)
    pub writes_per_second: f32,
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f32,
    /// Index efficiency (how well indexes are used)
    pub index_efficiency: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            avg_query_time_ms: 0.0,
            queries_per_second: 0.0,
            writes_per_second: 0.0,
            cache_hit_rate: 0.0,
            index_efficiency: 0.0,
        }
    }
}

/// Backup configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub enabled: bool,
    /// Backup interval (hours)
    pub interval_hours: u32,
    /// Backup directory path
    pub backup_directory: String,
    /// Number of backups to retain
    pub retain_count: u32,
    /// Compress backups
    pub compress: bool,
}

impl BackupConfig {
    /// Create backup config
    pub fn new(directory: &str) -> Self {
        BackupConfig {
            enabled: true,
            interval_hours: 24,
            backup_directory: directory.to_string(),
            retain_count: 7,
            compress: true,
        }
    }

    /// Disable backups
    pub fn disabled() -> Self {
        BackupConfig {
            enabled: false,
            interval_hours: 24,
            backup_directory: String::new(),
            retain_count: 0,
            compress: false,
        }
    }
}

/// Persistence manager (trait for swappable backends)
pub struct PersistenceManager {
    pub config: DatabaseConfig,
    pub index_config: IndexConfig,
    pub archival_policy: ArchivalPolicy,
    pub backup_config: BackupConfig,
    /// Recent stats cache
    pub stats: Option<DatabaseStats>,
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

impl PersistenceManager {
    /// Create manager with SQLite
    pub fn sqlite(path: &str) -> Self {
        PersistenceManager {
            config: DatabaseConfig::sqlite(path),
            index_config: IndexConfig::default(),
            archival_policy: ArchivalPolicy::default(),
            backup_config: BackupConfig::new("./backups"),
            stats: None,
            metrics: PerformanceMetrics::default(),
        }
    }

    /// Create manager with PostgreSQL
    pub fn postgresql(url: &str) -> Self {
        PersistenceManager {
            config: DatabaseConfig::postgresql(url),
            index_config: IndexConfig::default(),
            archival_policy: ArchivalPolicy::default(),
            backup_config: BackupConfig::new("./backups"),
            stats: None,
            metrics: PerformanceMetrics::default(),
        }
    }

    /// Configure indexes
    pub fn with_indexes(mut self, config: IndexConfig) -> Self {
        self.index_config = config;
        self
    }

    /// Configure archival
    pub fn with_archival(mut self, policy: ArchivalPolicy) -> Self {
        self.archival_policy = policy;
        self
    }

    /// Configure backups
    pub fn with_backups(mut self, config: BackupConfig) -> Self {
        self.backup_config = config;
        self
    }

    /// Get schema creation SQL (SQLite)
    pub fn get_sqlite_schema() -> &'static str {
        r#"
CREATE TABLE IF NOT EXISTS observations (
    observation_id TEXT PRIMARY KEY,
    robot_id TEXT NOT NULL,
    sensor_type TEXT NOT NULL,
    location TEXT NOT NULL,
    timestamp_us INTEGER NOT NULL,
    confidence REAL NOT NULL,
    value TEXT NOT NULL,
    metadata TEXT,
    archived BOOLEAN DEFAULT 0,
    archived_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_timestamp ON observations(timestamp_us);
CREATE INDEX IF NOT EXISTS idx_robot_id ON observations(robot_id);
CREATE INDEX IF NOT EXISTS idx_sensor_type ON observations(sensor_type);
CREATE INDEX IF NOT EXISTS idx_archived ON observations(archived);

CREATE TABLE IF NOT EXISTS snapshots (
    snapshot_id TEXT PRIMARY KEY,
    timestamp_us INTEGER NOT NULL,
    point_count INTEGER,
    bounds_min_x REAL,
    bounds_min_y REAL,
    bounds_min_z REAL,
    bounds_max_x REAL,
    bounds_max_y REAL,
    bounds_max_z REAL,
    metadata TEXT
);

CREATE INDEX IF NOT EXISTS idx_snapshot_timestamp ON snapshots(timestamp_us);

CREATE TABLE IF NOT EXISTS changes (
    change_id TEXT PRIMARY KEY,
    baseline_snapshot_id TEXT NOT NULL,
    current_snapshot_id TEXT NOT NULL,
    timestamp_us INTEGER NOT NULL,
    changed_count INTEGER,
    added_count INTEGER,
    removed_count INTEGER,
    change_percentage REAL,
    metadata TEXT,
    FOREIGN KEY(baseline_snapshot_id) REFERENCES snapshots(snapshot_id),
    FOREIGN KEY(current_snapshot_id) REFERENCES snapshots(snapshot_id)
);

CREATE INDEX IF NOT EXISTS idx_change_timestamp ON changes(timestamp_us);
        "#
    }

    /// Get schema creation SQL (PostgreSQL)
    pub fn get_postgresql_schema() -> &'static str {
        r#"
CREATE TABLE IF NOT EXISTS observations (
    observation_id TEXT PRIMARY KEY,
    robot_id TEXT NOT NULL,
    sensor_type TEXT NOT NULL,
    location GEOMETRY NOT NULL,
    timestamp_us BIGINT NOT NULL,
    confidence REAL NOT NULL,
    value JSONB NOT NULL,
    metadata JSONB,
    archived BOOLEAN DEFAULT FALSE,
    archived_at BIGINT
);

CREATE INDEX IF NOT EXISTS idx_timestamp ON observations(timestamp_us);
CREATE INDEX IF NOT EXISTS idx_robot_id ON observations(robot_id);
CREATE INDEX IF NOT EXISTS idx_sensor_type ON observations(sensor_type);
CREATE INDEX IF NOT EXISTS idx_archived ON observations(archived);
CREATE INDEX IF NOT EXISTS idx_location ON observations USING GIST(location);

CREATE TABLE IF NOT EXISTS snapshots (
    snapshot_id TEXT PRIMARY KEY,
    timestamp_us BIGINT NOT NULL,
    point_count INTEGER,
    bounds BOX3D,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_snapshot_timestamp ON snapshots(timestamp_us);

CREATE TABLE IF NOT EXISTS changes (
    change_id TEXT PRIMARY KEY,
    baseline_snapshot_id TEXT NOT NULL REFERENCES snapshots(snapshot_id),
    current_snapshot_id TEXT NOT NULL REFERENCES snapshots(snapshot_id),
    timestamp_us BIGINT NOT NULL,
    changed_count INTEGER,
    added_count INTEGER,
    removed_count INTEGER,
    change_percentage REAL,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_change_timestamp ON changes(timestamp_us);
        "#
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        Self::sqlite("pyterrain.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_sqlite() {
        let config = DatabaseConfig::sqlite("test.db");
        assert_eq!(config.backend, DatabaseBackend::SQLite);
        assert_eq!(config.connection_string, "test.db");
        assert_eq!(config.max_connections, 1);
    }

    #[test]
    fn test_database_config_postgresql() {
        let config = DatabaseConfig::postgresql("postgresql://localhost/terrain");
        assert_eq!(config.backend, DatabaseBackend::PostgreSQL);
        assert_eq!(config.max_connections, 16);
    }

    #[test]
    fn test_database_config_with_logging() {
        let config = DatabaseConfig::sqlite("test.db").with_logging();
        assert!(config.query_logging);
    }

    #[test]
    fn test_database_config_with_archive() {
        let config = DatabaseConfig::sqlite("test.db").with_auto_archive(30);
        assert_eq!(config.auto_archive_days, Some(30));
    }

    #[test]
    fn test_storage_record_creation() {
        let record = StorageRecord {
            observation_id: "obs1".to_string(),
            robot_id: "robot1".to_string(),
            sensor_type: "thermal".to_string(),
            location: "{\"type\":\"Point\"}".to_string(),
            timestamp_us: 1000,
            confidence: 0.95,
            value: "{}".to_string(),
            metadata: "{}".to_string(),
            archived: false,
            archived_at: None,
        };
        assert_eq!(record.observation_id, "obs1");
        assert!(!record.archived);
    }

    #[test]
    fn test_storage_query_creation() {
        let query = StorageQuery::new(1000, 2000);
        assert_eq!(query.from_timestamp_us, 1000);
        assert_eq!(query.to_timestamp_us, 2000);
        assert!(query.robot_ids.is_none());
    }

    #[test]
    fn test_storage_query_with_robots() {
        let query = StorageQuery::new(1000, 2000)
            .with_robots(vec!["robot1".to_string(), "robot2".to_string()]);
        assert_eq!(query.robot_ids.unwrap().len(), 2);
    }

    #[test]
    fn test_geographic_bounds_creation() {
        let bounds = GeographicBounds::new(-74.0, 40.0, -73.0, 41.0);
        assert_eq!(bounds.west, -74.0);
        assert_eq!(bounds.north, 41.0);
    }

    #[test]
    fn test_database_stats_default() {
        let stats = DatabaseStats::default();
        assert_eq!(stats.total_observations, 0);
        assert_eq!(stats.archived_observations, 0);
    }

    #[test]
    fn test_index_config_all() {
        let config = IndexConfig::all();
        assert!(config.index_timestamp);
        assert!(config.index_location);
    }

    #[test]
    fn test_index_config_minimal() {
        let config = IndexConfig::minimal();
        assert!(config.index_timestamp);
        assert!(!config.index_robot_id);
    }

    #[test]
    fn test_archival_policy_creation() {
        let policy = ArchivalPolicy::new(30, 90);
        assert_eq!(policy.archive_after_days, 30);
        assert_eq!(policy.delete_after_days, 90);
    }

    #[test]
    fn test_backup_config_creation() {
        let config = BackupConfig::new("./backups");
        assert!(config.enabled);
        assert_eq!(config.interval_hours, 24);
        assert_eq!(config.retain_count, 7);
    }

    #[test]
    fn test_backup_config_disabled() {
        let config = BackupConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_persistence_manager_sqlite() {
        let manager = PersistenceManager::sqlite("test.db");
        assert_eq!(manager.config.backend, DatabaseBackend::SQLite);
    }

    #[test]
    fn test_persistence_manager_postgresql() {
        let manager = PersistenceManager::postgresql("postgresql://localhost/terrain");
        assert_eq!(manager.config.backend, DatabaseBackend::PostgreSQL);
    }

    #[test]
    fn test_persistence_manager_with_indexes() {
        let config = IndexConfig::minimal();
        let manager = PersistenceManager::sqlite("test.db").with_indexes(config);
        assert!(!manager.index_config.index_robot_id);
    }

    #[test]
    fn test_sqlite_schema() {
        let schema = PersistenceManager::get_sqlite_schema();
        assert!(schema.contains("CREATE TABLE"));
        assert!(schema.contains("observations"));
        assert!(schema.contains("snapshots"));
    }

    #[test]
    fn test_postgresql_schema() {
        let schema = PersistenceManager::get_postgresql_schema();
        assert!(schema.contains("CREATE TABLE"));
        assert!(schema.contains("GEOMETRY"));
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.avg_query_time_ms, 0.0);
        assert_eq!(metrics.cache_hit_rate, 0.0);
    }
}
