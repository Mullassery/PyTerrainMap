//! Integration of Gaussian Splatting with frontier detection
//!
//! Combines Gaussian uncertainty scores with frontier prioritization for
//! intelligent exploration targeting.

use crate::gaussian_splatting::GaussianSplatStore;
use super::frontier::{Frontier, CuriosityScorer, RiskEvaluator};

/// Gaussian-aware frontier scorer
pub struct GaussianFrontierScorer {
    pub curiosity_scorer: CuriosityScorer,
    pub risk_evaluator: RiskEvaluator,
    pub uncertainty_weight: f32,  // How much to weight Gaussian uncertainty in info gain
    pub info_gain_from_uncertainty: bool,  // Use Gaussian uncertainty as primary info gain signal
}

impl GaussianFrontierScorer {
    /// Create new Gaussian-aware frontier scorer
    pub fn new() -> Self {
        GaussianFrontierScorer {
            curiosity_scorer: CuriosityScorer::new(),
            risk_evaluator: RiskEvaluator::new(),
            uncertainty_weight: 0.7,
            info_gain_from_uncertainty: true,
        }
    }

    /// Score a frontier using Gaussian uncertainty
    ///
    /// High uncertainty at frontier = high information gain = high priority
    pub fn score_frontier_with_gaussian(
        &self,
        frontier: &mut Frontier,
        store: &GaussianSplatStore,
    ) {
        let frontier_lat = frontier.location.0;
        let frontier_lon = frontier.location.1;
        let frontier_elev = frontier.location.2;

        // Query Gaussian uncertainty at frontier location
        let uncertainty = store.uncertainty_at([frontier_lat, frontier_lon, frontier_elev as f64]);

        // Information gain = how unknown the region is
        // uncertainty = 0.0 (fully known) to 1.0 (completely unknown)
        let information_gain = if self.info_gain_from_uncertainty {
            // Unknown regions have high information potential
            uncertainty
        } else {
            // Blend with existing information gain
            (frontier.expected_information_gain + (uncertainty * self.uncertainty_weight)) / 2.0
        };

        // Risk assessment includes Gaussian uncertainty
        let risk = self.risk_evaluator.estimate_risk(
            0,  // No nearby failures data from Gaussian store
            uncertainty,  // Use Gaussian uncertainty directly
            0.0,  // Terrain difficulty would come from splat traversability
        );

        // Curiosity = information scarcity is the uncertainty
        let curiosity = self.curiosity_scorer.score_curiosity(
            frontier,
            uncertainty,  // information_scarcity = Gaussian uncertainty
            0.3,  // strategic_importance (default)
            0.3,  // connectivity_potential (default)
            0.2,  // environmental_uniqueness (default)
        );

        // Update frontier scores
        frontier.expected_information_gain = information_gain;
        frontier.risk_estimate = risk;
        frontier.curiosity_score = curiosity;
        frontier.confidence = 1.0 - uncertainty;  // Inverse of uncertainty
        frontier.update_priority();
    }

    /// Score multiple frontiers using Gaussian store
    pub fn score_frontiers_batch(
        &self,
        frontiers: &mut [Frontier],
        store: &GaussianSplatStore,
    ) {
        for frontier in frontiers {
            self.score_frontier_with_gaussian(frontier, store);
        }
    }

    /// Sort frontiers by priority (highest first)
    pub fn rank_by_priority(mut frontiers: Vec<Frontier>) -> Vec<Frontier> {
        frontiers.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });
        frontiers
    }

    /// Filter frontiers: only return those with sufficient uncertainty (information potential)
    pub fn filter_by_uncertainty_threshold(
        frontiers: Vec<Frontier>,
        min_uncertainty: f32,
    ) -> Vec<Frontier> {
        frontiers
            .into_iter()
            .filter(|f| (1.0 - f.confidence) >= min_uncertainty)
            .collect()
    }
}

impl Default for GaussianFrontierScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = GaussianFrontierScorer::new();
        assert!(scorer.uncertainty_weight > 0.0);
        assert!(scorer.info_gain_from_uncertainty);
    }

    #[test]
    fn test_score_frontier_with_high_uncertainty() {
        let scorer = GaussianFrontierScorer::new();
        let mut frontier = Frontier::new("test_frontier".to_string(), (40.0, -74.0, 10.0));

        let store = GaussianSplatStore::new();
        // Empty store = high uncertainty everywhere
        scorer.score_frontier_with_gaussian(&mut frontier, &store);

        // High uncertainty = high information gain
        assert!(frontier.expected_information_gain > 0.5);
        // High uncertainty = high risk
        assert!(frontier.risk_estimate > 0.3);
        // Low confidence in high uncertainty areas
        assert!(frontier.confidence < 0.5);
    }

    #[test]
    fn test_score_frontier_with_observations() {
        let scorer = GaussianFrontierScorer::new();
        let mut frontier = Frontier::new("test_frontier".to_string(), (40.0, -74.0, 10.0));

        let mut store = GaussianSplatStore::new();
        // Add observation to reduce uncertainty
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road");

        scorer.score_frontier_with_gaussian(&mut frontier, &store);

        // With observation: lower uncertainty = lower information gain
        assert!(frontier.expected_information_gain < 0.8);
        // Lower uncertainty = lower risk at observed area
        assert!(frontier.risk_estimate < 0.8);
        // Higher confidence in observed areas
        assert!(frontier.confidence > 0.3);
    }

    #[test]
    fn test_rank_by_priority() {
        let mut f1 = Frontier::new("f1".to_string(), (40.0, -74.0, 10.0));
        f1.priority = 0.9;

        let mut f2 = Frontier::new("f2".to_string(), (40.005, -74.005, 10.0));
        f2.priority = 0.3;

        let mut f3 = Frontier::new("f3".to_string(), (40.01, -74.01, 10.0));
        f3.priority = 0.6;

        let frontiers = vec![f2, f1, f3];
        let ranked = GaussianFrontierScorer::rank_by_priority(frontiers);

        assert_eq!(ranked[0].id, "f1");
        assert_eq!(ranked[1].id, "f3");
        assert_eq!(ranked[2].id, "f2");
    }

    #[test]
    fn test_filter_by_uncertainty_threshold() {
        let mut f1 = Frontier::new("f1".to_string(), (40.0, -74.0, 10.0));
        f1.confidence = 0.2;  // High uncertainty (low confidence)

        let mut f2 = Frontier::new("f2".to_string(), (40.005, -74.005, 10.0));
        f2.confidence = 0.9;  // Low uncertainty (high confidence)

        let frontiers = vec![f1, f2];
        let filtered = GaussianFrontierScorer::filter_by_uncertainty_threshold(frontiers, 0.5);

        // Only f1 should pass (uncertainty = 1.0 - 0.2 = 0.8 > 0.5)
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "f1");
    }
}
