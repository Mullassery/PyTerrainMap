//! Frontier detection, evaluation, and prioritization
//!
//! Intelligently select which unexplored regions to explore based on
//! information gain, curiosity value, risk, and robot capabilities.

use serde::{Deserialize, Serialize};
use crate::traversability::{Node, SpatialGraph};
use super::SemanticContext;

/// A frontier: boundary of mapped region where exploration could continue
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frontier {
    pub id: String,
    pub location: (f64, f64, f32),  // (lat, lon, elevation)
    pub boundary_edges: Vec<String>,
    pub expected_information_gain: f32,  // 0.0-1.0
    pub exploration_cost: f32,           // 0.0-1.0 (normalized distance)
    pub risk_estimate: f32,              // 0.0-1.0 (difficulty)
    pub curiosity_score: f32,            // 0.0-1.0
    pub priority: f32,                   // 0.0-1.0 (combined score)
    pub confidence: f32,
    pub semantic_context: Option<SemanticContext>,
}

impl Frontier {
    /// Create a new frontier
    pub fn new(
        id: String,
        location: (f64, f64, f32),
    ) -> Self {
        Frontier {
            id,
            location,
            boundary_edges: Vec::new(),
            expected_information_gain: 0.5,
            exploration_cost: 0.5,
            risk_estimate: 0.5,
            curiosity_score: 0.5,
            priority: 0.5,
            confidence: 0.5,
            semantic_context: None,
        }
    }

    /// Recalculate priority from components
    pub fn update_priority(&mut self) {
        // Priority = (info_gain * curiosity) / (cost * risk)
        // High gain + high curiosity = prioritize
        // High cost + high risk = deprioritize
        let numerator = self.expected_information_gain * self.curiosity_score;
        let denominator = (self.exploration_cost + 0.1) * (self.risk_estimate + 0.1);
        self.priority = (numerator / denominator).min(1.0).max(0.0);
    }

    /// Set all components and update priority
    pub fn evaluate(
        &mut self,
        information_gain: f32,
        cost: f32,
        risk: f32,
        curiosity: f32,
    ) {
        self.expected_information_gain = information_gain;
        self.exploration_cost = cost;
        self.risk_estimate = risk;
        self.curiosity_score = curiosity;
        self.update_priority();
    }
}

/// Frontier detector
#[derive(Clone, Debug)]
pub struct FrontierDetector {
    pub min_frontier_distance: f32,  // Don't create frontiers within this distance
}

impl FrontierDetector {
    /// Create a new frontier detector
    pub fn new() -> Self {
        FrontierDetector {
            min_frontier_distance: 10.0,
        }
    }

    /// Detect frontiers in a spatial graph
    pub fn detect_frontiers(&self, graph: &SpatialGraph) -> Vec<Frontier> {
        let mut frontiers = Vec::new();

        // Iterate through all nodes to find boundaries
        for node in graph.all_nodes() {
            // Get edges from this node
            let edges = graph.edges_from(&node.id);

            if !edges.is_empty() {
                // Node with edges - it's on a boundary
                for edge in edges {
                    if let Some(to_node) = graph.get_node(&edge.to_node) {
                        // Create frontier at boundary of to_node
                        let frontier_id = format!("frontier_{}", edge.id);
                        let mut frontier = Frontier::new(frontier_id, to_node.position);
                        frontier.boundary_edges.push(edge.id.clone());

                        // Check distance from other frontiers
                        let min_dist = frontiers.iter().map(|f: &Frontier| {
                            self.distance_between(f.location, frontier.location)
                        }).fold(f32::INFINITY, f32::min);

                        if min_dist > self.min_frontier_distance {
                            frontiers.push(frontier);
                        }
                    }
                }
            }
        }

        frontiers
    }

    /// Calculate distance between two locations
    fn distance_between(&self, loc1: (f64, f64, f32), loc2: (f64, f64, f32)) -> f32 {
        let dlat = (loc1.0 - loc2.0) as f32;
        let dlon = (loc1.1 - loc2.1) as f32;
        let delev = loc1.2 - loc2.2;

        let lat_m = dlat * 111000.0;
        let lon_m = dlon * 111000.0 * (loc1.0.to_radians().cos() as f32);

        (lat_m * lat_m + lon_m * lon_m + delev * delev).sqrt()
    }
}

impl Default for FrontierDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Curiosity scorer
#[derive(Clone, Debug)]
pub struct CuriosityScorer {
    pub information_weight: f32,
    pub strategic_weight: f32,
    pub connectivity_weight: f32,
    pub uniqueness_weight: f32,
}

impl CuriosityScorer {
    /// Create a new curiosity scorer
    pub fn new() -> Self {
        CuriosityScorer {
            information_weight: 0.4,
            strategic_weight: 0.2,
            connectivity_weight: 0.2,
            uniqueness_weight: 0.2,
        }
    }

    /// Score curiosity for a frontier
    pub fn score_curiosity(
        &self,
        frontier: &Frontier,
        information_scarcity: f32,
        strategic_importance: f32,
        connectivity_potential: f32,
        environmental_uniqueness: f32,
    ) -> f32 {
        self.information_weight * information_scarcity
            + self.strategic_weight * strategic_importance
            + self.connectivity_weight * connectivity_potential
            + self.uniqueness_weight * environmental_uniqueness
    }
}

impl Default for CuriosityScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk evaluator for frontiers
#[derive(Clone, Debug)]
pub struct RiskEvaluator {
    pub nearby_failure_weight: f32,
    pub uncertainty_weight: f32,
    pub terrain_difficulty_weight: f32,
}

impl RiskEvaluator {
    /// Create a new risk evaluator
    pub fn new() -> Self {
        RiskEvaluator {
            nearby_failure_weight: 0.4,
            uncertainty_weight: 0.3,
            terrain_difficulty_weight: 0.3,
        }
    }

    /// Estimate risk for a frontier
    pub fn estimate_risk(
        &self,
        nearby_failures: u32,
        sensor_uncertainty: f32,
        terrain_difficulty: f32,
    ) -> f32 {
        let failure_score = (nearby_failures as f32 / 10.0).min(1.0);
        self.nearby_failure_weight * failure_score
            + self.uncertainty_weight * sensor_uncertainty
            + self.terrain_difficulty_weight * terrain_difficulty
    }
}

impl Default for RiskEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Frontier prioritizer
#[derive(Clone, Debug)]
pub struct FrontierPrioritizer {
    pub discovery_importance: f32,
    pub cost_importance: f32,
    pub risk_importance: f32,
}

impl FrontierPrioritizer {
    /// Create a new frontier prioritizer
    pub fn new() -> Self {
        FrontierPrioritizer {
            discovery_importance: 0.5,
            cost_importance: 0.3,
            risk_importance: 0.2,
        }
    }

    /// Rank frontiers by priority
    pub fn rank_frontiers(&self, mut frontiers: Vec<Frontier>) -> Vec<Frontier> {
        // Update priority for each frontier
        for frontier in &mut frontiers {
            let discovery_score = frontier.expected_information_gain * frontier.curiosity_score;
            let cost_penalty = frontier.exploration_cost;
            let risk_penalty = frontier.risk_estimate;

            frontier.priority = (
                self.discovery_importance * discovery_score
                - self.cost_importance * cost_penalty
                - self.risk_importance * risk_penalty
            ).max(0.0).min(1.0);
        }

        // Sort by priority (highest first)
        frontiers.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });

        frontiers
    }

    /// Get frontier suited for robot type
    pub fn frontier_for_robot(
        &self,
        frontiers: &[Frontier],
        robot_type: &str,
    ) -> Option<Frontier> {
        match robot_type {
            "aerial" => {
                // Aerial robots prefer high information gain, ignore risk/cost
                frontiers
                    .iter()
                    .max_by(|a, b| {
                        a.expected_information_gain
                            .partial_cmp(&b.expected_information_gain)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .cloned()
            }
            "wheeled" => {
                // Wheeled robots avoid high risk
                frontiers
                    .iter()
                    .filter(|f| f.risk_estimate < 0.7)
                    .max_by(|a, b| {
                        a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .cloned()
            }
            "tracked" => {
                // Tracked robots can handle moderate risk
                frontiers
                    .iter()
                    .max_by(|a, b| {
                        a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .cloned()
            }
            _ => frontiers.first().cloned(),
        }
    }
}

impl Default for FrontierPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontier_creation() {
        let frontier = Frontier::new(
            "frontier_1".to_string(),
            (37.7749, -122.4194, 10.0),
        );

        assert_eq!(frontier.id, "frontier_1");
        assert_eq!(frontier.priority, 0.5);
        assert!(frontier.confidence > 0.0);
    }

    #[test]
    fn test_frontier_evaluation() {
        let mut frontier = Frontier::new(
            "frontier_1".to_string(),
            (0.0, 0.0, 0.0),
        );

        frontier.evaluate(
            0.9,  // high information gain
            0.2,  // low cost
            0.3,  // moderate risk
            0.8,  // high curiosity
        );

        assert!(frontier.priority > 0.5);
    }

    #[test]
    fn test_frontier_priority_calculation() {
        let mut frontier = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));

        // High gain, low cost/risk
        frontier.evaluate(1.0, 0.1, 0.1, 0.9);
        let high_priority = frontier.priority;

        // Low gain, high cost/risk
        frontier.evaluate(0.1, 0.9, 0.9, 0.2);
        let low_priority = frontier.priority;

        assert!(high_priority > low_priority);
    }

    #[test]
    fn test_frontier_detector_creation() {
        let detector = FrontierDetector::new();
        assert_eq!(detector.min_frontier_distance, 10.0);
    }

    #[test]
    fn test_curiosity_scorer_creation() {
        let scorer = CuriosityScorer::new();
        assert_eq!(scorer.information_weight, 0.4);
    }

    #[test]
    fn test_score_curiosity() {
        let scorer = CuriosityScorer::new();
        let frontier = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));

        let score = scorer.score_curiosity(
            &frontier,
            0.9,  // high scarcity
            0.7,  // strategic
            0.6,  // connectivity
            0.8,  // uniqueness
        );

        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_risk_evaluator_creation() {
        let evaluator = RiskEvaluator::new();
        assert_eq!(evaluator.nearby_failure_weight, 0.4);
    }

    #[test]
    fn test_estimate_risk() {
        let evaluator = RiskEvaluator::new();

        let low_risk = evaluator.estimate_risk(0, 0.1, 0.2);
        let high_risk = evaluator.estimate_risk(10, 0.8, 0.9);

        assert!(low_risk < high_risk);
    }

    #[test]
    fn test_frontier_prioritizer_creation() {
        let prioritizer = FrontierPrioritizer::new();
        assert_eq!(prioritizer.discovery_importance, 0.5);
    }

    #[test]
    fn test_rank_frontiers() {
        let prioritizer = FrontierPrioritizer::new();

        let mut frontier1 = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));
        frontier1.evaluate(0.9, 0.2, 0.3, 0.8);

        let mut frontier2 = Frontier::new("f2".to_string(), (0.001, 0.001, 0.0));
        frontier2.evaluate(0.3, 0.8, 0.7, 0.2);

        let ranked = prioritizer.rank_frontiers(vec![frontier2, frontier1]);
        assert_eq!(ranked[0].id, "f1");  // Higher priority first
    }

    #[test]
    fn test_frontier_for_aerial_robot() {
        let prioritizer = FrontierPrioritizer::new();

        let mut f1 = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));
        f1.expected_information_gain = 0.5;

        let mut f2 = Frontier::new("f2".to_string(), (0.001, 0.001, 0.0));
        f2.expected_information_gain = 0.9;

        let frontiers = vec![f1, f2];
        let chosen = prioritizer.frontier_for_robot(&frontiers, "aerial");

        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().id, "f2");  // Highest info gain
    }

    #[test]
    fn test_frontier_for_wheeled_robot() {
        let prioritizer = FrontierPrioritizer::new();

        let mut f1 = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));
        f1.evaluate(0.7, 0.3, 0.3, 0.7);

        let mut f2 = Frontier::new("f2".to_string(), (0.001, 0.001, 0.0));
        f2.evaluate(0.8, 0.2, 0.8, 0.8);  // High risk

        let frontiers = vec![f1.clone(), f2];
        let chosen = prioritizer.frontier_for_robot(&frontiers, "wheeled");

        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().id, "f1");  // Lower risk
    }

    #[test]
    fn test_frontier_for_tracked_robot() {
        let prioritizer = FrontierPrioritizer::new();

        let mut f1 = Frontier::new("f1".to_string(), (0.0, 0.0, 0.0));
        f1.evaluate(0.6, 0.5, 0.5, 0.6);

        let mut f2 = Frontier::new("f2".to_string(), (0.001, 0.001, 0.0));
        f2.evaluate(0.8, 0.2, 0.6, 0.8);  // Better overall

        let frontiers = vec![f1, f2.clone()];
        let chosen = prioritizer.frontier_for_robot(&frontiers, "tracked");

        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().id, "f2");  // Highest priority
    }
}
