//! Spatial edge types and representation
//!
//! Edges represent navigable connections between nodes with various constraint types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of connection between nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EdgeType {
    /// Door between rooms
    Door {
        width: f32,
        height: f32,
        is_open: bool,
        requires_key: bool,
        one_way: bool,
    },

    /// Corridor or hallway
    Corridor {
        width: f32,
        length: f32,
        surface: String,
        obstacles: Vec<String>,
    },

    /// Outdoor path
    Path {
        surface_type: String,
        distance: f32,
        clearance_height: Option<f32>,
    },

    /// Bridge or overpass
    Bridge {
        span_length: f32,
        width: f32,
        weight_limit_kg: Option<f32>,
        surface: String,
    },

    /// Elevator or lift
    Elevator {
        capacity_kg: f32,
        height: f32,
        accessible: bool,
    },

    /// Stairs
    Stairs {
        step_height: f32,
        width: f32,
        count: u32,
    },

    /// Ramp
    Ramp {
        length: f32,
        height: f32,
        slope: f32,
        surface: String,
    },

    /// Generic connection
    Generic {
        distance: f32,
        traversability_score: f32,
    },
}

impl EdgeType {
    /// Get human-readable type name
    pub fn type_name(&self) -> &str {
        match self {
            EdgeType::Door { .. } => "door",
            EdgeType::Corridor { .. } => "corridor",
            EdgeType::Path { .. } => "path",
            EdgeType::Bridge { .. } => "bridge",
            EdgeType::Elevator { .. } => "elevator",
            EdgeType::Stairs { .. } => "stairs",
            EdgeType::Ramp { .. } => "ramp",
            EdgeType::Generic { .. } => "generic",
        }
    }

    /// Get metadata as JSON-serializable map
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self {
            EdgeType::Door { width, height, is_open, requires_key, one_way } => {
                map.insert("width".to_string(), width.to_string());
                map.insert("height".to_string(), height.to_string());
                map.insert("is_open".to_string(), is_open.to_string());
                map.insert("requires_key".to_string(), requires_key.to_string());
                map.insert("one_way".to_string(), one_way.to_string());
            }
            EdgeType::Corridor { width, length, surface, obstacles } => {
                map.insert("width".to_string(), width.to_string());
                map.insert("length".to_string(), length.to_string());
                map.insert("surface".to_string(), surface.clone());
                map.insert("obstacles".to_string(), obstacles.join("; "));
            }
            EdgeType::Path { surface_type, distance, clearance_height } => {
                map.insert("surface_type".to_string(), surface_type.clone());
                map.insert("distance".to_string(), distance.to_string());
                if let Some(h) = clearance_height {
                    map.insert("clearance_height".to_string(), h.to_string());
                }
            }
            EdgeType::Bridge { span_length, width, weight_limit_kg, surface } => {
                map.insert("span_length".to_string(), span_length.to_string());
                map.insert("width".to_string(), width.to_string());
                if let Some(w) = weight_limit_kg {
                    map.insert("weight_limit_kg".to_string(), w.to_string());
                }
                map.insert("surface".to_string(), surface.clone());
            }
            EdgeType::Elevator { capacity_kg, height, accessible } => {
                map.insert("capacity_kg".to_string(), capacity_kg.to_string());
                map.insert("height".to_string(), height.to_string());
                map.insert("accessible".to_string(), accessible.to_string());
            }
            EdgeType::Stairs { step_height, width, count } => {
                map.insert("step_height".to_string(), step_height.to_string());
                map.insert("width".to_string(), width.to_string());
                map.insert("count".to_string(), count.to_string());
            }
            EdgeType::Ramp { length, height, slope, surface } => {
                map.insert("length".to_string(), length.to_string());
                map.insert("height".to_string(), height.to_string());
                map.insert("slope".to_string(), slope.to_string());
                map.insert("surface".to_string(), surface.clone());
            }
            EdgeType::Generic { distance, traversability_score } => {
                map.insert("distance".to_string(), distance.to_string());
                map.insert("traversability_score".to_string(), traversability_score.to_string());
            }
        }
        map
    }

    /// Check if this edge is passable for a given width
    pub fn passable_for_width(&self, width: f32) -> bool {
        match self {
            EdgeType::Door { width: w, .. } => width <= *w,
            EdgeType::Corridor { width: w, .. } => width <= *w,
            EdgeType::Path { clearance_height: Some(h), .. } => width <= *h,
            EdgeType::Bridge { width: w, .. } => width <= *w,
            EdgeType::Stairs { width: w, .. } => width <= *w,
            EdgeType::Ramp { .. } => true,
            EdgeType::Elevator { .. } => true,
            EdgeType::Generic { .. } => true,
            EdgeType::Path { clearance_height: None, .. } => true,
        }
    }

    /// Check if this edge supports a given weight
    pub fn supports_weight(&self, weight_kg: f32) -> bool {
        match self {
            EdgeType::Bridge { weight_limit_kg: Some(limit), .. } => weight_kg <= *limit,
            EdgeType::Elevator { capacity_kg, .. } => weight_kg <= *capacity_kg,
            _ => true,
        }
    }
}

/// A spatial edge representing a navigable connection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from_node: String,
    pub to_node: String,
    pub edge_type: EdgeType,
    pub distance: f32,
    pub bidirectional: bool,
    pub created_at: i64,
    pub last_updated: i64,
    pub confidence: f32,  // 0.0-1.0
}

impl Edge {
    /// Create a new edge
    pub fn new(
        id: String,
        from_node: String,
        to_node: String,
        edge_type: EdgeType,
        distance: f32,
        bidirectional: bool,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Edge {
            id,
            from_node,
            to_node,
            edge_type,
            distance,
            bidirectional,
            created_at: now,
            last_updated: now,
            confidence: 0.7,
        }
    }

    /// Update the edge's last-modified timestamp
    pub fn touch(&mut self) {
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
    }

    /// Increase confidence (after successful traversal)
    pub fn increase_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence + delta).min(1.0);
        self.touch();
    }

    /// Decrease confidence (after failed traversal)
    pub fn decrease_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence - delta).max(0.0);
        self.touch();
    }

    /// Get reverse edge (for bidirectional edges)
    pub fn reverse(&self) -> Self {
        Edge {
            id: format!("{}_reverse", self.id),
            from_node: self.to_node.clone(),
            to_node: self.from_node.clone(),
            edge_type: self.edge_type.clone(),
            distance: self.distance,
            bidirectional: self.bidirectional,
            created_at: self.created_at,
            last_updated: self.last_updated,
            confidence: self.confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(
            "edge_1".to_string(),
            "room_1".to_string(),
            "room_2".to_string(),
            EdgeType::Door {
                width: 0.9,
                height: 2.1,
                is_open: true,
                requires_key: false,
                one_way: false,
            },
            2.0,
            true,
        );

        assert_eq!(edge.from_node, "room_1");
        assert_eq!(edge.to_node, "room_2");
        assert_eq!(edge.distance, 2.0);
        assert!(edge.bidirectional);
    }

    #[test]
    fn test_door_width_passable() {
        let door = EdgeType::Door {
            width: 0.9,
            height: 2.1,
            is_open: true,
            requires_key: false,
            one_way: false,
        };

        assert!(door.passable_for_width(0.8));
        assert!(!door.passable_for_width(1.0));
    }

    #[test]
    fn test_bridge_weight_limit() {
        let bridge = EdgeType::Bridge {
            span_length: 50.0,
            width: 4.0,
            weight_limit_kg: Some(5000.0),
            surface: "asphalt".to_string(),
        };

        assert!(bridge.supports_weight(4000.0));
        assert!(!bridge.supports_weight(6000.0));
    }

    #[test]
    fn test_edge_confidence_bounds() {
        let mut edge = Edge::new(
            "edge_1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
            EdgeType::Generic {
                distance: 10.0,
                traversability_score: 0.5,
            },
            10.0,
            true,
        );

        edge.increase_confidence(0.4);
        assert!(edge.confidence <= 1.0);

        edge.decrease_confidence(2.0);
        assert!(edge.confidence >= 0.0);
    }

    #[test]
    fn test_edge_reverse() {
        let edge = Edge::new(
            "edge_1".to_string(),
            "room_1".to_string(),
            "room_2".to_string(),
            EdgeType::Path {
                surface_type: "gravel".to_string(),
                distance: 15.0,
                clearance_height: None,
            },
            15.0,
            true,
        );

        let reversed = edge.reverse();
        assert_eq!(reversed.from_node, "room_2");
        assert_eq!(reversed.to_node, "room_1");
        assert_eq!(reversed.distance, 15.0);
    }

    #[test]
    fn test_edge_type_metadata() {
        let door = EdgeType::Door {
            width: 0.9,
            height: 2.1,
            is_open: true,
            requires_key: false,
            one_way: false,
        };

        let metadata = door.metadata();
        assert_eq!(metadata.get("width").unwrap(), "0.9");
        assert_eq!(metadata.get("is_open").unwrap(), "true");
    }
}
