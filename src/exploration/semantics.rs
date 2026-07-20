//! Semantic classification of environments
//!
//! Assign meaning to unexplored regions (indoor/outdoor, structured/organic, etc.)

use serde::{Deserialize, Serialize};
use crate::traversability::Node;

/// Semantic context of an environment
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticContext {
    IndoorStructured,   // Offices, warehouses (grid-like)
    IndoorOrganic,      // Hospitals, retail (non-grid)
    OutdoorPaved,       // Roads, sidewalks
    OutdoorNatural,     // Fields, trails, forests
    OutdoorMixed,       // Parks, industrial zones
}

impl SemanticContext {
    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            SemanticContext::IndoorStructured => "indoor_structured",
            SemanticContext::IndoorOrganic => "indoor_organic",
            SemanticContext::OutdoorPaved => "outdoor_paved",
            SemanticContext::OutdoorNatural => "outdoor_natural",
            SemanticContext::OutdoorMixed => "outdoor_mixed",
        }
    }
}

/// Structural template for an environment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructureTemplate {
    pub connectivity_pattern: Vec<Vec<String>>,
    pub typical_dimensions: (f32, f32, f32),  // (width, depth, height)
    pub obstacle_likelihood: f32,
}

impl StructureTemplate {
    /// Create a new structure template
    pub fn new() -> Self {
        StructureTemplate {
            connectivity_pattern: vec![vec!["room".to_string(), "corridor".to_string()]],
            typical_dimensions: (5.0, 5.0, 3.0),
            obstacle_likelihood: 0.3,
        }
    }
}

impl Default for StructureTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic classifier for environments
#[derive(Clone, Debug)]
pub struct SemanticClassifier {
    pub context_confidence: f32,
}

impl SemanticClassifier {
    /// Create a new semantic classifier
    pub fn new() -> Self {
        SemanticClassifier {
            context_confidence: 0.7,
        }
    }

    /// Classify a region based on nodes
    pub fn classify_region(&self, nodes: &[Node]) -> (SemanticContext, f32) {
        if nodes.is_empty() {
            return (SemanticContext::OutdoorNatural, 0.5);
        }

        // Count different node types
        let mut indoor_count = 0;
        let mut terrain_count = 0;
        let mut landmark_count = 0;

        for node in nodes {
            match &node.node_type {
                crate::traversability::NodeType::IndoorRoom { .. } => indoor_count += 1,
                crate::traversability::NodeType::TerrainCell { .. } => terrain_count += 1,
                crate::traversability::NodeType::Landmark { .. } => landmark_count += 1,
                _ => {}
            }
        }

        let total = nodes.len() as f32;
        let indoor_ratio = indoor_count as f32 / total;
        let terrain_ratio = terrain_count as f32 / total;
        let landmark_ratio = landmark_count as f32 / total;

        // Classify based on ratios
        if indoor_ratio > 0.7 {
            // Mostly indoor
            if terrain_ratio > 0.2 {
                (SemanticContext::IndoorOrganic, self.context_confidence)
            } else {
                (SemanticContext::IndoorStructured, self.context_confidence)
            }
        } else if terrain_ratio > 0.7 {
            // Mostly terrain
            if landmark_ratio > 0.2 {
                (SemanticContext::OutdoorMixed, self.context_confidence)
            } else {
                (SemanticContext::OutdoorNatural, self.context_confidence)
            }
        } else {
            // Mixed
            (SemanticContext::OutdoorMixed, self.context_confidence - 0.1)
        }
    }

    /// Predict structure from semantic context
    pub fn predict_structure_from_context(&self, context: SemanticContext) -> StructureTemplate {
        match context {
            SemanticContext::IndoorStructured => {
                // Office/warehouse pattern: grid-like
                StructureTemplate {
                    connectivity_pattern: vec![
                        vec!["room".to_string(), "corridor".to_string()],
                        vec!["corridor".to_string(), "room".to_string()],
                    ],
                    typical_dimensions: (5.0, 5.0, 3.0),
                    obstacle_likelihood: 0.2,
                }
            }
            SemanticContext::IndoorOrganic => {
                // Hospital/retail: organic flows
                StructureTemplate {
                    connectivity_pattern: vec![
                        vec!["room".to_string(), "corridor".to_string()],
                        vec!["room".to_string(), "room".to_string()],
                    ],
                    typical_dimensions: (8.0, 6.0, 3.5),
                    obstacle_likelihood: 0.4,
                }
            }
            SemanticContext::OutdoorPaved => {
                // Roads: linear paths
                StructureTemplate {
                    connectivity_pattern: vec![
                        vec!["path".to_string(), "path".to_string()],
                        vec!["sidewalk".to_string(), "intersection".to_string()],
                    ],
                    typical_dimensions: (15.0, 10.0, 0.0),
                    obstacle_likelihood: 0.15,
                }
            }
            SemanticContext::OutdoorNatural => {
                // Fields/trails: sparse connections
                StructureTemplate {
                    connectivity_pattern: vec![
                        vec!["terrain".to_string(), "terrain".to_string()],
                    ],
                    typical_dimensions: (50.0, 50.0, 0.0),
                    obstacle_likelihood: 0.6,
                }
            }
            SemanticContext::OutdoorMixed => {
                // Parks/industrial: mixed patterns
                StructureTemplate {
                    connectivity_pattern: vec![
                        vec!["path".to_string(), "terrain".to_string()],
                        vec!["zone".to_string(), "zone".to_string()],
                    ],
                    typical_dimensions: (25.0, 20.0, 0.0),
                    obstacle_likelihood: 0.35,
                }
            }
        }
    }

    /// Estimate obstacle likelihood in a context
    pub fn obstacle_likelihood_in(&self, context: SemanticContext) -> f32 {
        let template = self.predict_structure_from_context(context);
        template.obstacle_likelihood
    }
}

impl Default for SemanticClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traversability::{Node, NodeType};

    #[test]
    fn test_semantic_context_as_str() {
        assert_eq!(SemanticContext::IndoorStructured.as_str(), "indoor_structured");
        assert_eq!(SemanticContext::OutdoorNatural.as_str(), "outdoor_natural");
    }

    #[test]
    fn test_structure_template_creation() {
        let template = StructureTemplate::new();
        assert!(!template.connectivity_pattern.is_empty());
        assert!(template.obstacle_likelihood > 0.0 && template.obstacle_likelihood < 1.0);
    }

    #[test]
    fn test_classifier_creation() {
        let classifier = SemanticClassifier::new();
        assert_eq!(classifier.context_confidence, 0.7);
    }

    #[test]
    fn test_classify_empty_region() {
        let classifier = SemanticClassifier::new();
        let (context, confidence) = classifier.classify_region(&[]);
        assert_eq!(context, SemanticContext::OutdoorNatural);
        assert_eq!(confidence, 0.5);
    }

    #[test]
    fn test_classify_indoor_region() {
        let classifier = SemanticClassifier::new();
        let mut nodes = vec![];

        for i in 0..8 {
            nodes.push(Node::new(
                format!("room_{}", i),
                NodeType::IndoorRoom {
                    width: 5.0,
                    depth: 4.0,
                    height: 3.0,
                    floor_material: "tile".to_string(),
                },
                (0.0, 0.0, 0.0),
            ));
        }

        let (context, confidence) = classifier.classify_region(&nodes);
        assert_eq!(context, SemanticContext::IndoorStructured);
        assert!(confidence > 0.6);
    }

    #[test]
    fn test_predict_structure_structured() {
        let classifier = SemanticClassifier::new();
        let template = classifier.predict_structure_from_context(SemanticContext::IndoorStructured);
        assert_eq!(template.typical_dimensions, (5.0, 5.0, 3.0));
        assert!(template.obstacle_likelihood < 0.3);
    }

    #[test]
    fn test_predict_structure_natural() {
        let classifier = SemanticClassifier::new();
        let template = classifier.predict_structure_from_context(SemanticContext::OutdoorNatural);
        assert_eq!(template.typical_dimensions, (50.0, 50.0, 0.0));
        assert!(template.obstacle_likelihood > 0.5);
    }

    #[test]
    fn test_obstacle_likelihood() {
        let classifier = SemanticClassifier::new();
        let likelihood = classifier.obstacle_likelihood_in(SemanticContext::OutdoorNatural);
        assert!(likelihood > 0.5);  // Natural outdoor has higher obstacles
    }
}
