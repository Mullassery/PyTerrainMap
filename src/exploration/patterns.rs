//! Environmental pattern learning and recognition
//!
//! Learn recurring spatial structures (offices, warehouses, fields) and use them
//! to predict unexplored regions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::traversability::{Node, NodeType, Edge, EdgeType};

/// Types of environments
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EnvironmentType {
    // Indoor
    OfficeBuilding,
    Warehouse,
    Hospital,
    RetailStore,
    Factory,
    School,

    // Outdoor
    Road,
    Sidewalk,
    Trail,
    AgriculturalField,
    IndustrialZone,
    Park,

    // Hybrid
    CarPark,
    OpenCorridor,
    Unknown,
}

impl EnvironmentType {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            EnvironmentType::OfficeBuilding => "office_building",
            EnvironmentType::Warehouse => "warehouse",
            EnvironmentType::Hospital => "hospital",
            EnvironmentType::RetailStore => "retail_store",
            EnvironmentType::Factory => "factory",
            EnvironmentType::School => "school",
            EnvironmentType::Road => "road",
            EnvironmentType::Sidewalk => "sidewalk",
            EnvironmentType::Trail => "trail",
            EnvironmentType::AgriculturalField => "agricultural_field",
            EnvironmentType::IndustrialZone => "industrial_zone",
            EnvironmentType::Park => "park",
            EnvironmentType::CarPark => "car_park",
            EnvironmentType::OpenCorridor => "open_corridor",
            EnvironmentType::Unknown => "unknown",
        }
    }
}

/// Learned pattern for an environment type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentPattern {
    pub env_type: EnvironmentType,
    pub features: Vec<String>,
    pub common_sequences: Vec<Vec<String>>,  // room->corridor->room
    pub typical_widths: (f32, f32),          // min, max
    pub typical_heights: (f32, f32),
    pub typical_surfaces: Vec<(String, f32)>,  // (surface, frequency)
    pub connector_frequency: HashMap<String, f32>,  // door->0.85, stairs->0.2
    pub average_grid_density: f32,           // nodes per 100m²
    pub observation_count: u32,
    pub confidence: f32,
}

impl EnvironmentPattern {
    /// Create a new pattern
    pub fn new(env_type: EnvironmentType) -> Self {
        EnvironmentPattern {
            env_type,
            features: Vec::new(),
            common_sequences: Vec::new(),
            typical_widths: (0.0, 0.0),
            typical_heights: (0.0, 0.0),
            typical_surfaces: Vec::new(),
            connector_frequency: HashMap::new(),
            average_grid_density: 0.0,
            observation_count: 0,
            confidence: 0.0,
        }
    }

    /// Update pattern from observations
    pub fn update_from_observations(
        &mut self,
        nodes: &[Node],
        edges: &[Edge],
    ) {
        if nodes.is_empty() {
            return;
        }

        // Extract widths and heights from room nodes
        let mut widths = Vec::new();
        let mut heights = Vec::new();
        let mut surfaces = HashMap::new();

        for node in nodes {
            match &node.node_type {
                NodeType::IndoorRoom { width, height, floor_material, .. } => {
                    widths.push(*width);
                    heights.push(*height);
                    *surfaces.entry(floor_material.clone()).or_insert(0) += 1;
                }
                NodeType::TerrainCell { surface_type, .. } => {
                    *surfaces.entry(surface_type.clone()).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        // Calculate statistics
        if !widths.is_empty() {
            let min_w = widths.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_w = widths.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            self.typical_widths = (min_w, max_w);
        }

        if !heights.is_empty() {
            let min_h = heights.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_h = heights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            self.typical_heights = (min_h, max_h);
        }

        // Normalize surface frequencies
        let total_surfaces: u32 = surfaces.values().sum();
        if total_surfaces > 0 {
            self.typical_surfaces = surfaces
                .iter()
                .map(|(k, v)| (k.clone(), *v as f32 / total_surfaces as f32))
                .collect();
        }

        // Count edge types
        let total_edges = edges.len().max(1);
        let mut edge_counts: HashMap<String, u32> = HashMap::new();
        for edge in edges {
            let edge_type = edge.edge_type.type_name();
            *edge_counts.entry(edge_type.to_string()).or_insert(0) += 1;
        }

        self.connector_frequency = edge_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v as f32 / total_edges as f32))
            .collect();

        // Grid density (nodes per 100m²)
        let area = (self.typical_widths.1 - self.typical_widths.0).abs()
            * (self.typical_heights.1 - self.typical_heights.0).abs();
        if area > 0.0 {
            self.average_grid_density = (nodes.len() as f32 / area) * 100.0;
        }

        self.observation_count += nodes.len() as u32;
        self.confidence = (self.observation_count as f32 / 10.0).min(1.0);
    }
}

/// Library of learned environment patterns
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternLibrary {
    pub patterns: HashMap<String, EnvironmentPattern>,
    pub similarity_threshold: f32,
}

impl PatternLibrary {
    /// Create a new pattern library
    pub fn new() -> Self {
        PatternLibrary {
            patterns: HashMap::new(),
            similarity_threshold: 0.65,
        }
    }

    /// Learn from observations
    pub fn learn_from_observations(
        &mut self,
        env_type: EnvironmentType,
        nodes: &[Node],
        edges: &[Edge],
    ) {
        let key = env_type.as_str().to_string();
        let mut pattern = self.patterns
            .entry(key)
            .or_insert_with(|| EnvironmentPattern::new(env_type.clone()));

        pattern.update_from_observations(nodes, edges);
    }

    /// Classify a region
    pub fn classify_region(&self, nodes: &[Node]) -> (EnvironmentType, f32) {
        if nodes.is_empty() {
            return (EnvironmentType::Unknown, 0.0);
        }

        // Count node types
        let mut room_count = 0;
        let mut terrain_count = 0;
        let mut corridor_count = 0;

        for node in nodes {
            match &node.node_type {
                NodeType::IndoorRoom { .. } => room_count += 1,
                NodeType::TerrainCell { .. } => terrain_count += 1,
                _ => corridor_count += 1,
            }
        }

        let total = nodes.len() as f32;
        let room_ratio = room_count as f32 / total;
        let terrain_ratio = terrain_count as f32 / total;

        // Simple classification heuristic
        if terrain_ratio > 0.7 {
            (EnvironmentType::AgriculturalField, 0.8)
        } else if room_ratio > 0.5 {
            (EnvironmentType::OfficeBuilding, 0.7)
        } else if corridor_count as f32 / total > 0.6 {
            (EnvironmentType::Warehouse, 0.75)
        } else {
            (EnvironmentType::Unknown, 0.5)
        }
    }

    /// Predict next connector type based on environment pattern
    pub fn predict_next_connector(
        &self,
        current_type: EnvironmentType,
    ) -> Vec<(String, f32)> {
        let key = current_type.as_str();
        if let Some(pattern) = self.patterns.get(key) {
            pattern
                .connector_frequency
                .iter()
                .map(|(k, v)| (k.clone(), *v))
                .collect()
        } else {
            // Default predictions
            vec![
                ("door".to_string(), 0.6),
                ("corridor".to_string(), 0.3),
                ("path".to_string(), 0.1),
            ]
        }
    }

    /// Get pattern for environment type
    pub fn get_pattern(&self, env_type: EnvironmentType) -> Option<&EnvironmentPattern> {
        self.patterns.get(env_type.as_str())
    }
}

impl Default for PatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_type_as_str() {
        assert_eq!(EnvironmentType::OfficeBuilding.as_str(), "office_building");
        assert_eq!(EnvironmentType::Warehouse.as_str(), "warehouse");
        assert_eq!(EnvironmentType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_pattern_creation() {
        let pattern = EnvironmentPattern::new(EnvironmentType::OfficeBuilding);
        assert_eq!(pattern.env_type, EnvironmentType::OfficeBuilding);
        assert_eq!(pattern.observation_count, 0);
        assert_eq!(pattern.confidence, 0.0);
    }

    #[test]
    fn test_pattern_library_creation() {
        let lib = PatternLibrary::new();
        assert_eq!(lib.patterns.len(), 0);
        assert_eq!(lib.similarity_threshold, 0.65);
    }

    #[test]
    fn test_classify_empty_region() {
        let lib = PatternLibrary::new();
        let (env_type, confidence) = lib.classify_region(&[]);
        assert_eq!(env_type, EnvironmentType::Unknown);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn test_predict_next_connector_unknown_env() {
        let lib = PatternLibrary::new();
        let predictions = lib.predict_next_connector(EnvironmentType::Unknown);
        assert!(!predictions.is_empty());
        assert!(predictions.iter().any(|(t, _)| t == "door"));
    }
}
