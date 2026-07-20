//! Spatial node types and representation
//!
//! Nodes represent distinct spatial regions: rooms, terrain cells, landmarks, zones, vertical transitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of spatial node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    /// Indoor room with walls and dimensions
    IndoorRoom {
        width: f32,
        depth: f32,
        height: f32,
        floor_material: String,
    },

    /// Outdoor terrain cell (typically 10m x 10m grid)
    TerrainCell {
        surface_type: String,
        elevation: f32,
        slope: f32,
        roughness: f32,
    },

    /// Landmark (tree, pole, corner, etc.)
    Landmark {
        landmark_type: String,
        height: Option<f32>,
        radius: f32,
    },

    /// Named zone (parking lot, field, warehouse)
    Zone {
        zone_type: String,
        area_m2: f32,
    },

    /// Vertical transition (stairs, ramp, elevator)
    VerticalTransition {
        transition_type: String,
        vertical_rise: f32,
        slope_angle: f32,
    },
}

impl NodeType {
    /// Get human-readable type name
    pub fn type_name(&self) -> &str {
        match self {
            NodeType::IndoorRoom { .. } => "indoor_room",
            NodeType::TerrainCell { .. } => "terrain_cell",
            NodeType::Landmark { .. } => "landmark",
            NodeType::Zone { .. } => "zone",
            NodeType::VerticalTransition { .. } => "vertical_transition",
        }
    }

    /// Get metadata as JSON-serializable map
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self {
            NodeType::IndoorRoom { width, depth, height, floor_material } => {
                map.insert("width".to_string(), width.to_string());
                map.insert("depth".to_string(), depth.to_string());
                map.insert("height".to_string(), height.to_string());
                map.insert("floor_material".to_string(), floor_material.clone());
            }
            NodeType::TerrainCell { surface_type, elevation, slope, roughness } => {
                map.insert("surface_type".to_string(), surface_type.clone());
                map.insert("elevation".to_string(), elevation.to_string());
                map.insert("slope".to_string(), slope.to_string());
                map.insert("roughness".to_string(), roughness.to_string());
            }
            NodeType::Landmark { landmark_type, height, radius } => {
                map.insert("landmark_type".to_string(), landmark_type.clone());
                if let Some(h) = height {
                    map.insert("height".to_string(), h.to_string());
                }
                map.insert("radius".to_string(), radius.to_string());
            }
            NodeType::Zone { zone_type, area_m2 } => {
                map.insert("zone_type".to_string(), zone_type.clone());
                map.insert("area_m2".to_string(), area_m2.to_string());
            }
            NodeType::VerticalTransition { transition_type, vertical_rise, slope_angle } => {
                map.insert("transition_type".to_string(), transition_type.clone());
                map.insert("vertical_rise".to_string(), vertical_rise.to_string());
                map.insert("slope_angle".to_string(), slope_angle.to_string());
            }
        }
        map
    }
}

/// A spatial node representing a distinct region
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub position: (f64, f64, f32),  // (lat, lon, elevation_m)
    pub created_at: i64,             // Unix timestamp
    pub last_observed: i64,          // Unix timestamp
    pub confidence: f32,             // 0.0-1.0
}

impl Node {
    /// Create a new node
    pub fn new(id: String, node_type: NodeType, position: (f64, f64, f32)) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Node {
            id,
            node_type,
            position,
            created_at: now,
            last_observed: now,
            confidence: 0.8,
        }
    }

    /// Update last observed timestamp
    pub fn touch(&mut self) {
        self.last_observed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Increase confidence (e.g., after observation)
    pub fn increase_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence + delta).min(1.0);
    }

    /// Decrease confidence (e.g., after failed traversal)
    pub fn decrease_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence - delta).max(0.0);
    }

    /// Get latitude
    pub fn lat(&self) -> f64 {
        self.position.0
    }

    /// Get longitude
    pub fn lon(&self) -> f64 {
        self.position.1
    }

    /// Get elevation in meters
    pub fn elevation(&self) -> f32 {
        self.position.2
    }

    /// Distance to another node (Euclidean, ignoring curvature)
    pub fn distance_to(&self, other: &Node) -> f32 {
        let dlat = (self.position.0 - other.position.0) as f32;
        let dlon = (self.position.1 - other.position.1) as f32;
        let delev = self.position.2 - other.position.2;

        // Approximate: 1° ≈ 111 km = 111,000 m
        let lat_m = dlat * 111000.0;
        let lon_m = dlon * 111000.0 * (self.position.0.to_radians().cos() as f32);

        (lat_m * lat_m + lon_m * lon_m + delev * delev).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(
            "room_42".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (37.7749, -122.4194, 10.0),
        );

        assert_eq!(node.id, "room_42");
        assert_eq!(node.lat(), 37.7749);
        assert_eq!(node.lon(), -122.4194);
        assert_eq!(node.elevation(), 10.0);
        assert!(node.confidence > 0.0 && node.confidence <= 1.0);
    }

    #[test]
    fn test_node_type_name() {
        let room = NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "carpet".to_string(),
        };
        assert_eq!(room.type_name(), "indoor_room");

        let terrain = NodeType::TerrainCell {
            surface_type: "grass".to_string(),
            elevation: 100.0,
            slope: 5.0,
            roughness: 0.3,
        };
        assert_eq!(terrain.type_name(), "terrain_cell");
    }

    #[test]
    fn test_node_confidence_bounds() {
        let mut node = Node::new(
            "test".to_string(),
            NodeType::Landmark {
                landmark_type: "tree".to_string(),
                height: Some(5.0),
                radius: 1.0,
            },
            (0.0, 0.0, 0.0),
        );

        node.increase_confidence(0.3);
        assert!(node.confidence <= 1.0);

        node.decrease_confidence(1.5);
        assert!(node.confidence >= 0.0);
    }

    #[test]
    fn test_distance_calculation() {
        let node1 = Node::new(
            "n1".to_string(),
            NodeType::TerrainCell {
                surface_type: "grass".to_string(),
                elevation: 0.0,
                slope: 0.0,
                roughness: 0.2,
            },
            (0.0, 0.0, 0.0),
        );

        let node2 = Node::new(
            "n2".to_string(),
            NodeType::TerrainCell {
                surface_type: "grass".to_string(),
                elevation: 0.0,
                slope: 0.0,
                roughness: 0.2,
            },
            (0.0, 0.0, 0.0),
        );

        assert!(node1.distance_to(&node2) < 0.1);
    }

    #[test]
    fn test_node_metadata() {
        let node_type = NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        };

        let metadata = node_type.metadata();
        assert_eq!(metadata.get("floor_material").unwrap(), "tile");
        assert_eq!(metadata.get("width").unwrap(), "5");
    }
}
