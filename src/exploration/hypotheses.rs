//! Hypothesis generation and management
//!
//! Maintain competing hypotheses for unexplored regions with confidence scores.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::traversability::{TraversabilityObservation, Node};

/// A hypothesis prediction value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PredictionValue {
    Categorical(Vec<(String, f32)>),  // (value, confidence)
    Numerical { mean: f32, std_dev: f32 },
    Boolean(f32),  // confidence of true
}

/// Type of hypothesis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HypothesisType {
    ConnectorExists(String),  // door, corridor, stairs, etc.
    NodeType(String),         // room, terrain_cell, landmark
    RouteExists { from: String, to: String },
    ObstacleExists { location: String },
}

/// A hypothesis about an unexplored element
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub element_id: String,
    pub hypothesis_type: HypothesisType,
    pub predictions: HashMap<String, PredictionValue>,
    pub confidence: f32,  // 0.0-1.0
    pub created_at: i64,
    pub supporting_evidence: Vec<String>,
    pub contradicting_evidence: Vec<String>,
}

impl Hypothesis {
    /// Create a new hypothesis
    pub fn new(
        id: String,
        element_id: String,
        hypothesis_type: HypothesisType,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Hypothesis {
            id,
            element_id,
            hypothesis_type,
            predictions: HashMap::new(),
            confidence: 0.5,
            created_at: now,
            supporting_evidence: Vec::new(),
            contradicting_evidence: Vec::new(),
        }
    }

    /// Add supporting evidence
    pub fn add_evidence(&mut self, evidence: String) {
        self.supporting_evidence.push(evidence);
        self.update_confidence();
    }

    /// Add contradicting evidence
    pub fn add_contradiction(&mut self, evidence: String) {
        self.contradicting_evidence.push(evidence);
        self.update_confidence();
    }

    /// Update confidence based on evidence ratio
    fn update_confidence(&mut self) {
        let support = self.supporting_evidence.len() as f32;
        let contradiction = self.contradicting_evidence.len() as f32;
        let total = support + contradiction;

        if total > 0.0 {
            self.confidence = support / total;
        } else {
            self.confidence = 0.5;
        }
    }

    /// Check if hypothesis is still viable
    pub fn is_viable(&self) -> bool {
        self.confidence > 0.1
    }
}

/// Outcome of a prediction validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionOutcome {
    pub hypothesis_id: String,
    pub predicted_value: String,
    pub actual_value: String,
    pub accuracy: f32,
    pub prediction_confidence: f32,
    pub outcome_timestamp: i64,
}

/// Hypothesis manager
#[derive(Clone, Debug)]
pub struct HypothesisManager {
    pub hypotheses: HashMap<String, Vec<Hypothesis>>,  // element_id -> hypotheses
    pub prediction_history: Vec<PredictionOutcome>,
}

impl HypothesisManager {
    /// Create a new hypothesis manager
    pub fn new() -> Self {
        HypothesisManager {
            hypotheses: HashMap::new(),
            prediction_history: Vec::new(),
        }
    }

    /// Generate hypotheses for an element
    pub fn generate_hypotheses(
        &mut self,
        element_id: &str,
        context: &[Node],
    ) -> Vec<Hypothesis> {
        let mut hypotheses = Vec::new();

        // Hypothesis 1: Connector exists (door)
        let mut h1 = Hypothesis::new(
            format!("{}_door", element_id),
            element_id.to_string(),
            HypothesisType::ConnectorExists("door".to_string()),
        );
        h1.add_evidence("typical_office_pattern".to_string());
        h1.confidence = 0.6;
        hypotheses.push(h1);

        // Hypothesis 2: Connector exists (corridor)
        let mut h2 = Hypothesis::new(
            format!("{}_corridor", element_id),
            element_id.to_string(),
            HypothesisType::ConnectorExists("corridor".to_string()),
        );
        h2.add_evidence("open_layout".to_string());
        h2.confidence = 0.3;
        hypotheses.push(h2);

        // Hypothesis 3: Dead end (no connector)
        let mut h3 = Hypothesis::new(
            format!("{}_dead_end", element_id),
            element_id.to_string(),
            HypothesisType::ConnectorExists("dead_end".to_string()),
        );
        h3.add_evidence("boundary_region".to_string());
        h3.confidence = 0.1;
        hypotheses.push(h3);

        // Store hypotheses
        self.hypotheses
            .entry(element_id.to_string())
            .or_insert_with(Vec::new)
            .extend(hypotheses.clone());

        hypotheses
    }

    /// Update hypothesis based on observation
    pub fn update_hypothesis(
        &mut self,
        element_id: &str,
        observation: &TraversabilityObservation,
    ) -> f32 {
        if let Some(hypotheses) = self.hypotheses.get_mut(element_id) {
            // Update all hypotheses based on observation
            for hypothesis in hypotheses.iter_mut() {
                if observation.is_success() {
                    hypothesis.add_evidence(format!("observation_{}", observation.id));
                } else {
                    hypothesis.add_contradiction(format!("observation_{}", observation.id));
                }
            }

            // Return confidence of top hypothesis
            hypotheses
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                .map(|h| h.confidence)
                .unwrap_or(0.5)
        } else {
            0.5
        }
    }

    /// Get top hypothesis for an element
    pub fn get_top_hypothesis(&self, element_id: &str) -> Option<Hypothesis> {
        self.hypotheses
            .get(element_id)
            .and_then(|hs| {
                hs.iter()
                    .filter(|h| h.is_viable())
                    .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                    .cloned()
            })
    }

    /// Remove a hypothesis
    pub fn remove_hypothesis(&mut self, element_id: &str, hypothesis_id: &str) {
        if let Some(hypotheses) = self.hypotheses.get_mut(element_id) {
            hypotheses.retain(|h| h.id != hypothesis_id);
        }
    }

    /// Get all hypotheses for an element
    pub fn get_hypotheses(&self, element_id: &str) -> Vec<Hypothesis> {
        self.hypotheses
            .get(element_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Record a prediction outcome
    pub fn record_outcome(&mut self, outcome: PredictionOutcome) {
        self.prediction_history.push(outcome);
    }

    /// Get prediction accuracy (across all predictions)
    pub fn average_accuracy(&self) -> f32 {
        if self.prediction_history.is_empty() {
            return 0.5;
        }
        let total: f32 = self.prediction_history.iter().map(|p| p.accuracy).sum();
        total / self.prediction_history.len() as f32
    }
}

impl Default for HypothesisManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_creation() {
        let h = Hypothesis::new(
            "h1".to_string(),
            "elem_1".to_string(),
            HypothesisType::ConnectorExists("door".to_string()),
        );

        assert_eq!(h.id, "h1");
        assert_eq!(h.element_id, "elem_1");
        assert_eq!(h.confidence, 0.5);
    }

    #[test]
    fn test_hypothesis_evidence() {
        let mut h = Hypothesis::new(
            "h1".to_string(),
            "elem_1".to_string(),
            HypothesisType::ConnectorExists("door".to_string()),
        );

        h.add_evidence("reason1".to_string());
        assert_eq!(h.supporting_evidence.len(), 1);
        assert!(h.confidence > 0.5);
    }

    #[test]
    fn test_hypothesis_contradiction() {
        let mut h = Hypothesis::new(
            "h1".to_string(),
            "elem_1".to_string(),
            HypothesisType::ConnectorExists("door".to_string()),
        );

        h.add_evidence("support".to_string());
        h.add_contradiction("contradiction".to_string());
        assert_eq!(h.supporting_evidence.len(), 1);
        assert_eq!(h.contradicting_evidence.len(), 1);
        assert_eq!(h.confidence, 0.5);  // 1:1 ratio
    }

    #[test]
    fn test_hypothesis_viability() {
        let mut h = Hypothesis::new(
            "h1".to_string(),
            "elem_1".to_string(),
            HypothesisType::ConnectorExists("door".to_string()),
        );

        assert!(h.is_viable());

        for _ in 0..10 {
            h.add_contradiction("evidence".to_string());
        }
        assert!(!h.is_viable());
    }

    #[test]
    fn test_hypothesis_manager_creation() {
        let manager = HypothesisManager::new();
        assert_eq!(manager.hypotheses.len(), 0);
        assert_eq!(manager.prediction_history.len(), 0);
    }

    #[test]
    fn test_generate_hypotheses() {
        let mut manager = HypothesisManager::new();
        let hypotheses = manager.generate_hypotheses("elem_1", &[]);

        assert_eq!(hypotheses.len(), 3);
        assert!(hypotheses.iter().any(|h| h.confidence == 0.6));
        assert!(hypotheses.iter().any(|h| h.confidence == 0.3));
        assert!(hypotheses.iter().any(|h| h.confidence == 0.1));
    }

    #[test]
    fn test_get_top_hypothesis() {
        let mut manager = HypothesisManager::new();
        manager.generate_hypotheses("elem_1", &[]);

        let top = manager.get_top_hypothesis("elem_1");
        assert!(top.is_some());
        assert_eq!(top.unwrap().confidence, 0.6);
    }

    #[test]
    fn test_remove_hypothesis() {
        let mut manager = HypothesisManager::new();
        manager.generate_hypotheses("elem_1", &[]);

        let before = manager.get_hypotheses("elem_1").len();
        manager.remove_hypothesis("elem_1", "elem_1_door");
        let after = manager.get_hypotheses("elem_1").len();

        assert_eq!(before, 3);
        assert_eq!(after, 2);
    }

    #[test]
    fn test_average_accuracy() {
        let mut manager = HypothesisManager::new();

        manager.record_outcome(PredictionOutcome {
            hypothesis_id: "h1".to_string(),
            predicted_value: "door".to_string(),
            actual_value: "door".to_string(),
            accuracy: 1.0,
            prediction_confidence: 0.8,
            outcome_timestamp: 0,
        });

        manager.record_outcome(PredictionOutcome {
            hypothesis_id: "h2".to_string(),
            predicted_value: "corridor".to_string(),
            actual_value: "door".to_string(),
            accuracy: 0.0,
            prediction_confidence: 0.6,
            outcome_timestamp: 0,
        });

        assert_eq!(manager.average_accuracy(), 0.5);
    }
}
