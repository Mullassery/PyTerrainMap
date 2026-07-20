//! Unified path planning integrating traversability graphs and Gaussian splatting
//!
//! Combines discrete graph-based routes with continuous probabilistic terrain knowledge
//! for informed navigation decisions.

use crate::gaussian_splatting::{
    GaussianSplatStore, TraversabilityDistanceEngine, PathCost,
};
use crate::traversability::{Node, Edge};
use serde::{Deserialize, Serialize};

/// Combined path cost from graph edges and Gaussian observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnifiedPathCost {
    /// Base cost from traversability graph (edge traversal history)
    pub graph_cost: f32,
    /// Cost from Gaussian terrain observations
    pub gaussian_cost: f32,
    /// Passage probability cost (if any)
    pub passage_cost: f32,
    /// Total combined cost
    pub total_cost: f32,
    /// Confidence in the estimate (how well-observed the path is)
    pub confidence: f32,
}

/// Unified path planning engine combining graph + Gaussian approaches
pub struct UnifiedPathPlanner {
    distance_engine: TraversabilityDistanceEngine,
}

impl UnifiedPathPlanner {
    /// Create new planner
    pub fn new() -> Self {
        UnifiedPathPlanner {
            distance_engine: TraversabilityDistanceEngine::new(),
        }
    }

    /// Compute unified path cost between two positions
    ///
    /// Combines:
    /// 1. Traversability graph edge costs (historical robot success)
    /// 2. Gaussian terrain costs (continuous probabilistic model)
    /// 3. Passage probability costs (door open rates, etc.)
    pub fn path_cost(
        &self,
        from_pos: [f64; 3],
        to_pos: [f64; 3],
        gaussian_store: &GaussianSplatStore,
    ) -> UnifiedPathCost {
        // Get Gaussian-based cost
        let gaussian_path_cost = self.distance_engine.path_cost(from_pos, to_pos, gaussian_store);

        // Get uncertainty from Gaussian layer
        let from_uncertainty = gaussian_store.uncertainty_at(from_pos);
        let to_uncertainty = gaussian_store.uncertainty_at(to_pos);
        let avg_uncertainty = (from_uncertainty + to_uncertainty) / 2.0;

        // Confidence is inverse of uncertainty (well-observed regions have high confidence)
        let confidence = 1.0 - avg_uncertainty;

        UnifiedPathCost {
            graph_cost: 0.0,  // Placeholder: would query graph edges if available
            gaussian_cost: gaussian_path_cost.terrain_cost
                + gaussian_path_cost.elevation_cost
                + gaussian_path_cost.uncertainty_cost,
            passage_cost: gaussian_path_cost.passage_cost,
            total_cost: gaussian_path_cost.total,
            confidence,
        }
    }

    /// Compute path cost with graph edge history integration
    ///
    /// For known graph edges, blend graph-based costs (from historical traversals)
    /// with Gaussian-based costs for more robust estimates.
    pub fn path_cost_with_graph_edges(
        &self,
        from_node: &Node,
        to_node: &Node,
        edge: &Edge,
        gaussian_store: &GaussianSplatStore,
    ) -> UnifiedPathCost {
        let from_pos = [from_node.position.0, from_node.position.1, from_node.position.2 as f64];
        let to_pos = [to_node.position.0, to_node.position.1, to_node.position.2 as f64];

        // Base Gaussian cost
        let mut unified = self.path_cost(from_pos, to_pos, gaussian_store);

        // Blend with graph edge confidence
        // High-confidence edges reduce overall cost
        unified.graph_cost = edge.confidence * 0.2;  // Edge confidence contributes ~20% to cost
        unified.total_cost = (unified.gaussian_cost * 0.8) + (unified.graph_cost * 0.2);

        // Confidence is product of graph and Gaussian confidence
        unified.confidence *= edge.confidence;

        unified
    }

    /// Find lowest-cost path considering both graph structure and Gaussian uncertainty
    ///
    /// This is a simplified version that finds direct cost between two points.
    /// A full implementation would do graph-based pathfinding with Gaussian-weighted edges.
    pub fn find_best_route(
        &self,
        from_pos: [f64; 3],
        to_pos: [f64; 3],
        gaussian_store: &GaussianSplatStore,
    ) -> UnifiedPathCost {
        self.path_cost(from_pos, to_pos, gaussian_store)
    }
}

impl Default for UnifiedPathPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_creation() {
        let planner = UnifiedPathPlanner::new();
        assert!(planner.distance_engine.uncertainty_penalty > 0.0);
    }

    #[test]
    fn test_path_cost_with_uncertainty() {
        let planner = UnifiedPathPlanner::new();
        let store = GaussianSplatStore::new();

        let from = [40.0, -74.0, 10.0];
        let to = [40.005, -74.005, 10.0];

        let cost = planner.path_cost(from, to, &store);

        // Empty store: high uncertainty = low confidence
        assert!(cost.confidence < 0.5);
        assert!(cost.total_cost > 0.0);
    }

    #[test]
    fn test_path_cost_with_observations() {
        let planner = UnifiedPathPlanner::new();
        let mut store = GaussianSplatStore::new();

        // Add observations to reduce uncertainty
        store.insert(crate::gaussian_splatting::TerrainGaussian::from_point_observation(
            [40.0, -74.0, 10.0],
            "bot_01",
            0.85,
        ));

        let from = [40.0, -74.0, 10.0];
        let to = [40.005, -74.005, 10.0];

        let cost = planner.path_cost(from, to, &store);

        // With observations: lower uncertainty = higher confidence
        assert!(cost.confidence > 0.3);
    }
}
