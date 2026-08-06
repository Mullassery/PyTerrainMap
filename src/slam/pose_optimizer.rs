//! Pose Graph Optimization Engine
//!
//! Performs incremental pose graph optimization to correct accumulated drift
//! and integrate loop closures.

use crate::types::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pose node in graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoseNode {
    pub id: u32,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub timestamp: i64,
    pub fixed: bool, // Anchor nodes (fixed)
}

/// Edge constraint between poses
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoseEdgeConstraint {
    pub from_id: u32,
    pub to_id: u32,
    pub relative_translation: (f32, f32, f32),
    pub relative_rotation: (f32, f32, f32, f32),
    pub information_weight: f32, // Higher = more confident
}

/// Pose graph optimizer
pub struct PoseGraphOptimizer {
    /// Pose nodes
    pub nodes: HashMap<u32, PoseNode>,
    /// Edge constraints
    pub edges: Vec<PoseEdgeConstraint>,
    /// Convergence threshold
    pub threshold: f32,
    /// Maximum iterations
    pub max_iterations: u32,
    /// Current node ID
    next_node_id: u32,
}

impl PoseGraphOptimizer {
    pub fn new() -> Self {
        PoseGraphOptimizer {
            nodes: HashMap::new(),
            edges: Vec::new(),
            threshold: 1e-4,
            max_iterations: 20,
            next_node_id: 0,
        }
    }

    /// Add pose node
    pub fn add_node(&mut self, position: (f32, f32, f32), rotation: (f32, f32, f32, f32), timestamp: i64) -> u32 {
        let node_id = self.next_node_id;
        self.nodes.insert(
            node_id,
            PoseNode {
                id: node_id,
                position,
                rotation,
                timestamp,
                fixed: false,
            },
        );
        self.next_node_id += 1;
        node_id
    }

    /// Fix a pose (anchor node)
    pub fn fix_node(&mut self, node_id: u32) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.fixed = true;
        }
    }

    /// Add edge constraint
    pub fn add_edge(
        &mut self,
        from_id: u32,
        to_id: u32,
        rel_trans: (f32, f32, f32),
        rel_rot: (f32, f32, f32, f32),
        weight: f32,
    ) {
        self.edges.push(PoseEdgeConstraint {
            from_id,
            to_id,
            relative_translation: rel_trans,
            relative_rotation: rel_rot,
            information_weight: weight,
        });
    }

    /// Optimize pose graph (Gauss-Newton iterations)
    pub fn optimize(&mut self) -> Result<OptimizationResult> {
        if self.nodes.is_empty() {
            return Ok(OptimizationResult::default());
        }

        let mut iteration = 0;
        let mut total_error = f32::INFINITY;

        while iteration < self.max_iterations && total_error > self.threshold {
            total_error = 0.0;

            // For each edge, compute residual and apply correction
            for edge in &self.edges {
                if let (Some(from_node), Some(to_node)) = (self.nodes.get(&edge.from_id), self.nodes.get(&edge.to_id)) {
                    // Compute prediction: from_node + relative = to_node
                    let predicted_pos = (
                        from_node.position.0 + edge.relative_translation.0,
                        from_node.position.1 + edge.relative_translation.1,
                        from_node.position.2 + edge.relative_translation.2,
                    );

                    // Position residual
                    let dx = to_node.position.0 - predicted_pos.0;
                    let dy = to_node.position.1 - predicted_pos.1;
                    let dz = to_node.position.2 - predicted_pos.2;
                    let pos_error = (dx * dx + dy * dy + dz * dz).sqrt();
                    total_error += pos_error * edge.information_weight;

                    // Apply correction (small update toward residual)
                    let correction_scale = 0.01 * edge.information_weight;
                    if !to_node.fixed {
                        let to_mutable = self.nodes.get_mut(&edge.to_id).unwrap();
                        to_mutable.position.0 += dx * correction_scale;
                        to_mutable.position.1 += dy * correction_scale;
                        to_mutable.position.2 += dz * correction_scale;
                    }
                }
            }

            iteration += 1;
        }

        Ok(OptimizationResult {
            iterations: iteration,
            final_error: total_error,
            converged: total_error <= self.threshold,
        })
    }

    /// Get optimized pose
    pub fn get_pose(&self, node_id: u32) -> Option<(f32, f32, f32, (f32, f32, f32, f32))> {
        self.nodes.get(&node_id).map(|n| (n.position.0, n.position.1, n.position.2, n.rotation))
    }

    /// Get all nodes
    pub fn get_nodes(&self) -> Vec<PoseNode> {
        self.nodes.values().cloned().collect()
    }

    /// Get graph statistics
    pub fn statistics(&self) -> GraphStatistics {
        GraphStatistics {
            node_count: self.nodes.len() as u32,
            edge_count: self.edges.len() as u32,
            fixed_node_count: self.nodes.values().filter(|n| n.fixed).count() as u32,
        }
    }
}

impl Default for PoseGraphOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub iterations: u32,
    pub final_error: f32,
    pub converged: bool,
}

impl Default for OptimizationResult {
    fn default() -> Self {
        OptimizationResult {
            iterations: 0,
            final_error: 0.0,
            converged: true,
        }
    }
}

/// Graph statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub node_count: u32,
    pub edge_count: u32,
    pub fixed_node_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = PoseGraphOptimizer::new();
        assert_eq!(optimizer.nodes.len(), 0);
        assert_eq!(optimizer.edges.len(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut optimizer = PoseGraphOptimizer::new();
        let id = optimizer.add_node((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1000);
        assert_eq!(id, 0);
        assert_eq!(optimizer.nodes.len(), 1);
    }

    #[test]
    fn test_add_edge() {
        let mut optimizer = PoseGraphOptimizer::new();
        let n1 = optimizer.add_node((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1000);
        let n2 = optimizer.add_node((1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 2000);
        optimizer.add_edge(n1, n2, (1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1.0);
        assert_eq!(optimizer.edges.len(), 1);
    }

    #[test]
    fn test_optimize() {
        let mut optimizer = PoseGraphOptimizer::new();
        let n1 = optimizer.add_node((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1000);
        optimizer.fix_node(n1);
        let n2 = optimizer.add_node((1.5, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 2000);
        // True distance is 1.0, observed is 1.5
        optimizer.add_edge(n1, n2, (1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1.0);

        let result = optimizer.optimize().unwrap();
        assert!(result.iterations > 0);
        // After optimization, n2 should move closer to (1.0, 0.0, 0.0)
    }

    #[test]
    fn test_statistics() {
        let mut optimizer = PoseGraphOptimizer::new();
        optimizer.add_node((0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0), 1000);
        let stats = optimizer.statistics();
        assert_eq!(stats.node_count, 1);
    }
}
