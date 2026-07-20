//! Spatial metadata and environment versioning
//!
//! Track environment versions, coordinate systems, and aggregate statistics.

use serde::{Deserialize, Serialize};

/// A version of the spatial environment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentVersion {
    pub id: String,
    pub version: u32,
    pub created_at: i64,
    pub description: String,
}

/// Metadata about a spatial environment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpatialMetadata {
    pub environment_id: String,
    pub version: u32,
    pub created_at: i64,
    pub description: String,

    // Reference frame
    pub origin_lat: f64,
    pub origin_lon: f64,
    pub origin_elevation: f32,
    pub coordinate_system: String,  // "WGS84", "UTM_10N", etc.

    // Statistics
    pub total_nodes: u32,
    pub total_edges: u32,
    pub average_node_confidence: f32,
    pub average_edge_confidence: f32,

    // Temporal info
    pub last_update: i64,
    pub sensor_types: Vec<String>,  // ["camera", "lidar", "gps"]
    pub observation_count: u32,
}

impl SpatialMetadata {
    /// Create new metadata for an environment
    pub fn new(environment_id: String, origin: (f64, f64, f32)) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        SpatialMetadata {
            environment_id,
            version: 1,
            created_at: now,
            description: "Spatial environment".to_string(),
            origin_lat: origin.0,
            origin_lon: origin.1,
            origin_elevation: origin.2,
            coordinate_system: "WGS84".to_string(),
            total_nodes: 0,
            total_edges: 0,
            average_node_confidence: 0.5,
            average_edge_confidence: 0.5,
            last_update: now,
            sensor_types: vec![],
            observation_count: 0,
        }
    }

    /// Increment version
    pub fn bump_version(&mut self) {
        self.version += 1;
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Update statistics
    pub fn update_stats(
        &mut self,
        total_nodes: u32,
        total_edges: u32,
        avg_node_conf: f32,
        avg_edge_conf: f32,
        obs_count: u32,
    ) {
        self.total_nodes = total_nodes;
        self.total_edges = total_edges;
        self.average_node_confidence = avg_node_conf;
        self.average_edge_confidence = avg_edge_conf;
        self.observation_count = obs_count;
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Add a sensor type if not already present
    pub fn add_sensor_type(&mut self, sensor: String) {
        if !self.sensor_types.contains(&sensor) {
            self.sensor_types.push(sensor);
        }
    }

    /// Get age in seconds
    pub fn age_seconds(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        now - self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = SpatialMetadata::new(
            "farm_2026".to_string(),
            (37.7749, -122.4194, 10.0),
        );

        assert_eq!(meta.environment_id, "farm_2026");
        assert_eq!(meta.version, 1);
        assert_eq!(meta.origin_lat, 37.7749);
        assert_eq!(meta.coordinate_system, "WGS84");
        assert_eq!(meta.total_nodes, 0);
    }

    #[test]
    fn test_version_bump() {
        let mut meta = SpatialMetadata::new(
            "env_1".to_string(),
            (0.0, 0.0, 0.0),
        );

        assert_eq!(meta.version, 1);
        meta.bump_version();
        assert_eq!(meta.version, 2);
        meta.bump_version();
        assert_eq!(meta.version, 3);
    }

    #[test]
    fn test_sensor_type_addition() {
        let mut meta = SpatialMetadata::new(
            "env_1".to_string(),
            (0.0, 0.0, 0.0),
        );

        meta.add_sensor_type("camera".to_string());
        meta.add_sensor_type("lidar".to_string());
        meta.add_sensor_type("camera".to_string());  // Duplicate

        assert_eq!(meta.sensor_types.len(), 2);
        assert!(meta.sensor_types.contains(&"camera".to_string()));
        assert!(meta.sensor_types.contains(&"lidar".to_string()));
    }

    #[test]
    fn test_stats_update() {
        let mut meta = SpatialMetadata::new(
            "env_1".to_string(),
            (0.0, 0.0, 0.0),
        );

        meta.update_stats(100, 150, 0.9, 0.85, 250);
        assert_eq!(meta.total_nodes, 100);
        assert_eq!(meta.total_edges, 150);
        assert_eq!(meta.average_node_confidence, 0.9);
        assert_eq!(meta.average_edge_confidence, 0.85);
        assert_eq!(meta.observation_count, 250);
    }

    #[test]
    fn test_environment_version() {
        let env = EnvironmentVersion {
            id: "farm_v1".to_string(),
            version: 1,
            created_at: 1700000000,
            description: "Initial survey".to_string(),
        };

        assert_eq!(env.version, 1);
        assert_eq!(env.description, "Initial survey");
    }

    #[test]
    fn test_age_calculation() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut meta = SpatialMetadata::new(
            "env_1".to_string(),
            (0.0, 0.0, 0.0),
        );
        meta.created_at = now - 3600;  // 1 hour ago

        let age = meta.age_seconds();
        assert!(age >= 3600 && age <= 3610);  // Allow small timing variance
    }
}
