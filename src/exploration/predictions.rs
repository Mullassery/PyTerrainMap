//! Predictive traversability modeling
//!
//! Estimate traversability probability for unexplored edges using Bayesian inference.

use serde::{Deserialize, Serialize};
use crate::traversability::Node;
use super::patterns::{EnvironmentType, PatternLibrary};

/// Predicted properties for an unexplored edge
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictiveModel {
    pub edge_id: String,
    pub from_node: String,
    pub to_node: String,

    // Predictions (probability 0.0-1.0)
    pub traversability_prob: f32,
    pub estimated_width: (f32, f32),  // (mean, std_dev)
    pub estimated_height: (f32, f32),
    pub estimated_surface: Vec<(String, f32)>,  // (type, confidence)
    pub estimated_connector_type: Vec<(String, f32)>,  // (door/corridor, confidence)
    pub estimated_distance: (f32, f32),
    pub estimated_traversal_time_ms: (u32, u32),
    pub estimated_energy_cost: (f32, f32),

    // Metadata
    pub prediction_basis: Vec<String>,
    pub confidence: f32,
    pub predicted_at: i64,
    pub validated_at: Option<i64>,
    pub validation_error: Option<f32>,
}

impl PredictiveModel {
    /// Create a new predictive model
    pub fn new(edge_id: String, from_node: String, to_node: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        PredictiveModel {
            edge_id,
            from_node,
            to_node,
            traversability_prob: 0.5,
            estimated_width: (0.0, 0.0),
            estimated_height: (0.0, 0.0),
            estimated_surface: Vec::new(),
            estimated_connector_type: Vec::new(),
            estimated_distance: (0.0, 0.0),
            estimated_traversal_time_ms: (0, 0),
            estimated_energy_cost: (0.0, 0.0),
            prediction_basis: Vec::new(),
            confidence: 0.5,
            predicted_at: now,
            validated_at: None,
            validation_error: None,
        }
    }

    /// Add supporting evidence for this prediction
    pub fn add_evidence(&mut self, reason: String) {
        self.prediction_basis.push(reason);
        // Confidence increases with evidence: 1 evidence = 0.6, 5 = 0.75, 10+ = 1.0
        self.confidence = 0.5 + (self.prediction_basis.len() as f32 / 20.0).min(0.5);
    }

    /// Mark as validated with error magnitude
    pub fn validate(&mut self, error: f32) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.validated_at = Some(now);
        self.validation_error = Some(error);
    }
}

/// Bayesian traversability predictor
#[derive(Clone, Debug)]
pub struct TraversabilityPredictor {
    pub pattern_library: PatternLibrary,
}

impl TraversabilityPredictor {
    /// Create a new predictor
    pub fn new(pattern_library: PatternLibrary) -> Self {
        TraversabilityPredictor { pattern_library }
    }

    /// Predict edge properties
    pub fn predict_edge(
        &self,
        from_node: &Node,
        to_node: &Node,
        local_context: &[Node],
    ) -> PredictiveModel {
        let mut model = PredictiveModel::new(
            format!("edge_{}_{}", from_node.id, to_node.id),
            from_node.id.clone(),
            to_node.id.clone(),
        );

        // Classify local environment
        let (env_type, env_confidence) = self.pattern_library.classify_region(local_context);

        // Get pattern for this environment
        if let Some(pattern) = self.pattern_library.get_pattern(env_type.clone()) {
            // Width prediction
            model.estimated_width = pattern.typical_widths;
            model.add_evidence(format!(
                "pattern_{}_width_{}%",
                pattern.env_type.as_str(),
                (env_confidence * 100.0) as u32
            ));

            // Height prediction
            model.estimated_height = pattern.typical_heights;
            model.add_evidence(format!(
                "pattern_{}_height_{}%",
                pattern.env_type.as_str(),
                (env_confidence * 100.0) as u32
            ));

            // Surface prediction
            model.estimated_surface = pattern.typical_surfaces.clone();
            model.add_evidence(format!(
                "pattern_{}_surfaces_{}%",
                pattern.env_type.as_str(),
                (env_confidence * 100.0) as u32
            ));

            // Connector type prediction
            let connector_probs = self.pattern_library.predict_next_connector(env_type);
            model.estimated_connector_type = connector_probs.clone();
            model.add_evidence(format!(
                "pattern_connector_{}%",
                (env_confidence * 100.0) as u32
            ));

            // Traversability probability (higher confidence = higher probability)
            model.traversability_prob = 0.5 + (env_confidence * 0.4);
        } else {
            // Default predictions for unknown environments
            model.traversability_prob = 0.6;
            model.estimated_width = (1.0, 3.0);
            model.estimated_height = (2.0, 3.5);
            model.estimated_connector_type = vec![
                ("door".to_string(), 0.4),
                ("corridor".to_string(), 0.4),
                ("path".to_string(), 0.2),
            ];
            model.add_evidence("default_unknown".to_string());
        }

        // Distance estimation (simple: node distance + 10%)
        let dist = from_node.distance_to(to_node);
        model.estimated_distance = (dist, dist * 0.1);
        model.add_evidence(format!(
            "geometric_distance_{}m",
            dist as u32
        ));

        // Traversal time estimation (assume 1 m/s base speed)
        let base_time = (dist * 1000.0) as u32;
        model.estimated_traversal_time_ms = (base_time, base_time / 10);
        model.add_evidence("time_1ms_per_meter".to_string());

        // Energy cost estimation (proportional to distance)
        model.estimated_energy_cost = (dist * 0.01, dist * 0.002);
        model.add_evidence("energy_proportional_distance".to_string());

        model.confidence = (model.prediction_basis.len() as f32 / 10.0).min(1.0);
        model
    }

    /// Predict connector type for an environment
    pub fn predict_connector_type(
        &self,
        current_env: EnvironmentType,
    ) -> Vec<(String, f32)> {
        self.pattern_library.predict_next_connector(current_env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traversability::{NodeType};

    #[test]
    fn test_predictive_model_creation() {
        let model = PredictiveModel::new(
            "edge_1".to_string(),
            "node_1".to_string(),
            "node_2".to_string(),
        );

        assert_eq!(model.edge_id, "edge_1");
        assert_eq!(model.from_node, "node_1");
        assert_eq!(model.to_node, "node_2");
        assert_eq!(model.traversability_prob, 0.5);
        assert_eq!(model.confidence, 0.5);
    }

    #[test]
    fn test_add_evidence() {
        let mut model = PredictiveModel::new(
            "edge_1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
        );

        model.add_evidence("reason_1".to_string());
        assert_eq!(model.prediction_basis.len(), 1);
        assert!(model.confidence > 0.5);

        model.add_evidence("reason_2".to_string());
        assert_eq!(model.prediction_basis.len(), 2);
    }

    #[test]
    fn test_validate_prediction() {
        let mut model = PredictiveModel::new(
            "edge_1".to_string(),
            "n1".to_string(),
            "n2".to_string(),
        );

        model.validate(0.15);
        assert!(model.validated_at.is_some());
        assert_eq!(model.validation_error, Some(0.15));
    }

    #[test]
    fn test_predictor_creation() {
        let lib = PatternLibrary::new();
        let predictor = TraversabilityPredictor::new(lib);
        assert_eq!(predictor.pattern_library.patterns.len(), 0);
    }

    #[test]
    fn test_predict_edge_default() {
        let lib = PatternLibrary::new();
        let predictor = TraversabilityPredictor::new(lib);

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
            (0.001, 0.001, 0.0),
        );

        let model = predictor.predict_edge(&node1, &node2, &[]);
        assert!(model.traversability_prob > 0.5);
        assert!(!model.estimated_connector_type.is_empty());
        assert!(model.confidence > 0.0);
    }

    #[test]
    fn test_predict_connector_type() {
        let lib = PatternLibrary::new();
        let predictor = TraversabilityPredictor::new(lib);
        let predictions = predictor.predict_connector_type(EnvironmentType::OfficeBuilding);
        assert!(!predictions.is_empty());
    }
}
