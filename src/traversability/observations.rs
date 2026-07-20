//! Traversability observations and fleet consensus
//!
//! Track robot attempts to traverse edges and compute fleet-wide consensus.

use serde::{Deserialize, Serialize};

/// Outcome of a traversal attempt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TraversalOutcome {
    /// Successfully traversed
    Success {
        time_ms: u32,
        energy_used: f32,
    },
    /// Failed to traverse
    Failure {
        reason: String,
    },
    /// Traversed but with difficulty
    Difficulty {
        score: f32,  // 0.0-1.0 (1.0 = extremely difficult)
    },
}

/// A single robot's observation of traversability
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraversabilityObservation {
    pub id: String,
    pub edge_id: String,
    pub robot_id: String,
    pub robot_type: String,  // "wheeled", "legged", "aerial"
    pub outcome: TraversalOutcome,
    pub timestamp: i64,
    pub confidence: f32,  // 0.0-1.0
    pub notes: Option<String>,
}

impl TraversabilityObservation {
    /// Create a new observation
    pub fn new(
        id: String,
        edge_id: String,
        robot_id: String,
        robot_type: String,
        outcome: TraversalOutcome,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        TraversabilityObservation {
            id,
            edge_id,
            robot_id,
            robot_type,
            outcome,
            timestamp: now,
            confidence: 0.8,
            notes: None,
        }
    }

    /// Check if observation indicates success
    pub fn is_success(&self) -> bool {
        matches!(self.outcome, TraversalOutcome::Success { .. })
    }

    /// Check if observation indicates failure
    pub fn is_failure(&self) -> bool {
        matches!(self.outcome, TraversalOutcome::Failure { .. })
    }

    /// Get difficulty score (0.0 = easy, 1.0 = impossible)
    pub fn difficulty_score(&self) -> f32 {
        match &self.outcome {
            TraversalOutcome::Success { .. } => 0.0,
            TraversalOutcome::Failure { .. } => 1.0,
            TraversalOutcome::Difficulty { score } => *score,
        }
    }
}

/// Consensus result for an edge traversed by robots of a specific type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub edge_id: String,
    pub robot_type: String,
    pub total_observations: u32,
    pub success_count: u32,
    pub failure_count: u32,
    pub difficulty_count: u32,
    pub average_difficulty: f32,  // 0.0-1.0
    pub confidence: f32,           // 0.0-1.0 (higher = more certain)
    pub last_updated: i64,
}

impl ConsensusResult {
    /// Create consensus from observations
    pub fn from_observations(
        edge_id: String,
        robot_type: String,
        observations: &[TraversabilityObservation],
    ) -> Self {
        let total = observations.len() as u32;
        let success_count = observations.iter().filter(|o| o.is_success()).count() as u32;
        let failure_count = observations.iter().filter(|o| o.is_failure()).count() as u32;
        let difficulty_count = total - success_count - failure_count;

        let total_difficulty: f32 = observations
            .iter()
            .map(|o| o.difficulty_score())
            .sum();
        let average_difficulty = if total > 0 {
            total_difficulty / total as f32
        } else {
            0.5
        };

        // Confidence is higher with more observations
        let confidence = if total > 0 {
            (total as f32 / 10.0).min(1.0)
        } else {
            0.0
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        ConsensusResult {
            edge_id,
            robot_type,
            total_observations: total,
            success_count,
            failure_count,
            difficulty_count,
            average_difficulty,
            confidence,
            last_updated: now,
        }
    }

    /// Get success rate (0.0-1.0)
    pub fn success_rate(&self) -> f32 {
        if self.total_observations > 0 {
            self.success_count as f32 / self.total_observations as f32
        } else {
            0.5
        }
    }

    /// Get traversability score for this edge/robot_type combination
    /// Higher = easier to traverse
    pub fn traversability_score(&self) -> f32 {
        1.0 - self.average_difficulty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_creation() {
        let obs = TraversabilityObservation::new(
            "obs_1".to_string(),
            "edge_42".to_string(),
            "robot_alpha".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Success {
                time_ms: 5000,
                energy_used: 0.15,
            },
        );

        assert_eq!(obs.edge_id, "edge_42");
        assert_eq!(obs.robot_id, "robot_alpha");
        assert!(obs.is_success());
        assert!(!obs.is_failure());
    }

    #[test]
    fn test_observation_failure() {
        let obs = TraversabilityObservation::new(
            "obs_2".to_string(),
            "edge_42".to_string(),
            "robot_beta".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Failure {
                reason: "high_slip".to_string(),
            },
        );

        assert!(obs.is_failure());
        assert!(!obs.is_success());
        assert_eq!(obs.difficulty_score(), 1.0);
    }

    #[test]
    fn test_observation_difficulty() {
        let obs = TraversabilityObservation::new(
            "obs_3".to_string(),
            "edge_42".to_string(),
            "robot_gamma".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Difficulty { score: 0.6 },
        );

        assert_eq!(obs.difficulty_score(), 0.6);
        assert!(!obs.is_success());
        assert!(!obs.is_failure());
    }

    #[test]
    fn test_consensus_from_observations() {
        let observations = vec![
            TraversabilityObservation::new(
                "obs_1".to_string(),
                "edge_1".to_string(),
                "robot_1".to_string(),
                "wheeled".to_string(),
                TraversalOutcome::Success {
                    time_ms: 5000,
                    energy_used: 0.1,
                },
            ),
            TraversabilityObservation::new(
                "obs_2".to_string(),
                "edge_1".to_string(),
                "robot_2".to_string(),
                "wheeled".to_string(),
                TraversalOutcome::Success {
                    time_ms: 6000,
                    energy_used: 0.12,
                },
            ),
            TraversabilityObservation::new(
                "obs_3".to_string(),
                "edge_1".to_string(),
                "robot_3".to_string(),
                "wheeled".to_string(),
                TraversalOutcome::Failure {
                    reason: "weight_limit".to_string(),
                },
            ),
        ];

        let consensus = ConsensusResult::from_observations(
            "edge_1".to_string(),
            "wheeled".to_string(),
            &observations,
        );

        assert_eq!(consensus.total_observations, 3);
        assert_eq!(consensus.success_count, 2);
        assert_eq!(consensus.failure_count, 1);
        assert!(consensus.success_rate() > 0.6 && consensus.success_rate() < 0.7);
    }

    #[test]
    fn test_consensus_empty() {
        let consensus = ConsensusResult::from_observations(
            "edge_1".to_string(),
            "aerial".to_string(),
            &[],
        );

        assert_eq!(consensus.total_observations, 0);
        assert_eq!(consensus.confidence, 0.0);
        assert!(consensus.average_difficulty > 0.4 && consensus.average_difficulty < 0.6);
    }

    #[test]
    fn test_traversability_score() {
        let observations = vec![
            TraversabilityObservation::new(
                "obs_1".to_string(),
                "edge_1".to_string(),
                "robot_1".to_string(),
                "wheeled".to_string(),
                TraversalOutcome::Success {
                    time_ms: 5000,
                    energy_used: 0.1,
                },
            ),
        ];

        let consensus = ConsensusResult::from_observations(
            "edge_1".to_string(),
            "wheeled".to_string(),
            &observations,
        );

        assert_eq!(consensus.traversability_score(), 1.0 - consensus.average_difficulty);
    }

    #[test]
    fn test_consensus_confidence_increases_with_observations() {
        let mut observations = vec![];
        for i in 0..15 {
            observations.push(TraversabilityObservation::new(
                format!("obs_{}", i),
                "edge_1".to_string(),
                format!("robot_{}", i),
                "wheeled".to_string(),
                TraversalOutcome::Success {
                    time_ms: 5000,
                    energy_used: 0.1,
                },
            ));
        }

        let consensus =
            ConsensusResult::from_observations("edge_1".to_string(), "wheeled".to_string(), &observations);
        assert!(consensus.confidence > 0.9);  // 15 observations
    }
}
