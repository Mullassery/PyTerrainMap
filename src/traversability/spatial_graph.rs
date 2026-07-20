//! Spatial Knowledge Graph
//!
//! Main graph structure managing nodes, edges, and traversability observations.

use crate::traversability::{
    Node, NodeType, Edge, EdgeType, TraversabilityObservation, TraversalOutcome,
    ConsensusResult, SpatialMetadata,
};
use std::collections::HashMap;

/// Spatial knowledge graph for traversability intelligence
pub struct SpatialGraph {
    pub metadata: SpatialMetadata,
    nodes: HashMap<String, Node>,
    edges: HashMap<String, Edge>,
    observations: Vec<TraversabilityObservation>,
    consensus_cache: HashMap<(String, String), ConsensusResult>,
}

impl SpatialGraph {
    /// Create a new spatial graph
    pub fn new(environment_id: String, origin: (f64, f64, f32)) -> Self {
        let metadata = SpatialMetadata::new(environment_id, origin);
        SpatialGraph {
            metadata,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            observations: Vec::new(),
            consensus_cache: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> Result<(), String> {
        if self.nodes.contains_key(&node.id) {
            return Err(format!("Node {} already exists", node.id));
        }
        self.nodes.insert(node.id.clone(), node);
        self.metadata.total_nodes = self.nodes.len() as u32;
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Get mutable reference to a node
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    /// Remove a node (cascades to edges)
    pub fn remove_node(&mut self, id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(id) {
            return Err(format!("Node {} not found", id));
        }

        // Remove all edges connected to this node
        self.edges.retain(|_, edge| {
            edge.from_node != id && edge.to_node != id
        });

        self.nodes.remove(id);
        self.metadata.total_nodes = self.nodes.len() as u32;
        self.metadata.total_edges = self.edges.len() as u32;
        Ok(())
    }

    /// Get all nodes
    pub fn all_nodes(&self) -> Vec<&Node> {
        self.nodes.values().collect()
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), String> {
        if !self.nodes.contains_key(&edge.from_node) {
            return Err(format!("From node {} not found", edge.from_node));
        }
        if !self.nodes.contains_key(&edge.to_node) {
            return Err(format!("To node {} not found", edge.to_node));
        }
        if self.edges.contains_key(&edge.id) {
            return Err(format!("Edge {} already exists", edge.id));
        }

        self.edges.insert(edge.id.clone(), edge);
        self.metadata.total_edges = self.edges.len() as u32;
        Ok(())
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &str) -> Option<&Edge> {
        self.edges.get(id)
    }

    /// Get mutable reference to an edge
    pub fn get_edge_mut(&mut self, id: &str) -> Option<&mut Edge> {
        self.edges.get_mut(id)
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, id: &str) -> Result<(), String> {
        if !self.edges.contains_key(id) {
            return Err(format!("Edge {} not found", id));
        }
        self.edges.remove(id);
        self.metadata.total_edges = self.edges.len() as u32;
        Ok(())
    }

    /// Get all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Vec<&Edge> {
        self.edges
            .values()
            .filter(|e| e.from_node == node_id)
            .collect()
    }

    /// Get neighbors of a node
    pub fn neighbors(&self, node_id: &str) -> Vec<(String, &Edge)> {
        self.edges_from(node_id)
            .into_iter()
            .map(|e| (e.to_node.clone(), e))
            .collect()
    }

    /// Get connected component from a starting node
    pub fn connected_component(&self, start: &str) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![start.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for (neighbor, _) in self.neighbors(&current) {
                if !visited.contains(&neighbor) {
                    queue.push(neighbor);
                }
            }
        }

        visited.into_iter().collect()
    }

    /// Record a traversability observation
    pub fn record_observation(&mut self, obs: TraversabilityObservation) -> Result<(), String> {
        if !self.edges.contains_key(&obs.edge_id) {
            return Err(format!("Edge {} not found", obs.edge_id));
        }
        self.observations.push(obs);
        self.metadata.observation_count = self.observations.len() as u32;
        Ok(())
    }

    /// Compute consensus for an edge and robot type
    pub fn compute_consensus(&mut self, edge_id: &str, robot_type: &str) -> Result<ConsensusResult, String> {
        if !self.edges.contains_key(edge_id) {
            return Err(format!("Edge {} not found", edge_id));
        }

        let relevant_obs: Vec<TraversabilityObservation> = self.observations
            .iter()
            .filter(|o| o.edge_id == edge_id && o.robot_type == robot_type)
            .cloned()
            .collect();

        let consensus = ConsensusResult::from_observations(
            edge_id.to_string(),
            robot_type.to_string(),
            &relevant_obs,
        );

        self.consensus_cache.insert(
            (edge_id.to_string(), robot_type.to_string()),
            consensus.clone(),
        );

        Ok(consensus)
    }

    /// Get cached consensus (returns None if not computed)
    pub fn get_consensus(&self, edge_id: &str, robot_type: &str) -> Option<&ConsensusResult> {
        self.consensus_cache.get(&(edge_id.to_string(), robot_type.to_string()))
    }

    /// Get traversability score for an edge and robot type
    pub fn get_traversability_score(&self, edge_id: &str, robot_type: &str) -> Option<f32> {
        self.get_consensus(edge_id, robot_type)
            .map(|c| c.traversability_score())
    }

    /// Get all observations for an edge
    pub fn observations_for_edge(&self, edge_id: &str) -> Vec<&TraversabilityObservation> {
        self.observations
            .iter()
            .filter(|o| o.edge_id == edge_id)
            .collect()
    }

    /// Get count of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get count of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get count of observations
    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    /// Update average confidence metrics
    pub fn update_confidence_metrics(&mut self) {
        let nodes = self.all_nodes();
        let avg_node_conf = if !nodes.is_empty() {
            nodes.iter().map(|n| n.confidence).sum::<f32>() / nodes.len() as f32
        } else {
            0.5
        };

        let edges: Vec<_> = self.edges.values().collect();
        let avg_edge_conf = if !edges.is_empty() {
            edges.iter().map(|e| e.confidence).sum::<f32>() / edges.len() as f32
        } else {
            0.5
        };

        self.metadata.average_node_confidence = avg_node_conf;
        self.metadata.average_edge_confidence = avg_edge_conf;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> SpatialGraph {
        SpatialGraph::new("test_env".to_string(), (0.0, 0.0, 0.0))
    }

    #[test]
    fn test_graph_creation() {
        let graph = create_test_graph();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.metadata.environment_id, "test_env");
    }

    #[test]
    fn test_add_node() {
        let mut graph = create_test_graph();
        let node = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );

        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut graph = create_test_graph();
        let node = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );

        assert!(graph.add_node(node.clone()).is_ok());
        assert!(graph.add_node(node).is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = create_test_graph();
        let node1 = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );
        let node2 = Node::new(
            "room_2".to_string(),
            NodeType::IndoorRoom {
                width: 4.0,
                depth: 3.0,
                height: 3.0,
                floor_material: "carpet".to_string(),
            },
            (1.0, 1.0, 0.0),
        );

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

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

        assert!(graph.add_edge(edge).is_ok());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_neighbors() {
        let mut graph = create_test_graph();
        let node1 = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );
        let node2 = Node::new(
            "room_2".to_string(),
            NodeType::IndoorRoom {
                width: 4.0,
                depth: 3.0,
                height: 3.0,
                floor_material: "carpet".to_string(),
            },
            (1.0, 1.0, 0.0),
        );

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

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

        graph.add_edge(edge).unwrap();
        let neighbors = graph.neighbors("room_1");
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0].0, "room_2");
    }

    #[test]
    fn test_record_observation() {
        let mut graph = create_test_graph();
        let node1 = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );
        let node2 = Node::new(
            "room_2".to_string(),
            NodeType::IndoorRoom {
                width: 4.0,
                depth: 3.0,
                height: 3.0,
                floor_material: "carpet".to_string(),
            },
            (1.0, 1.0, 0.0),
        );

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

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

        graph.add_edge(edge).unwrap();

        let obs = TraversabilityObservation::new(
            "obs_1".to_string(),
            "edge_1".to_string(),
            "robot_1".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Success {
                time_ms: 5000,
                energy_used: 0.1,
            },
        );

        assert!(graph.record_observation(obs).is_ok());
        assert_eq!(graph.observation_count(), 1);
    }

    #[test]
    fn test_compute_consensus() {
        let mut graph = create_test_graph();
        let node1 = Node::new(
            "room_1".to_string(),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (0.0, 0.0, 0.0),
        );
        let node2 = Node::new(
            "room_2".to_string(),
            NodeType::IndoorRoom {
                width: 4.0,
                depth: 3.0,
                height: 3.0,
                floor_material: "carpet".to_string(),
            },
            (1.0, 1.0, 0.0),
        );

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

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

        graph.add_edge(edge).unwrap();

        let obs1 = TraversabilityObservation::new(
            "obs_1".to_string(),
            "edge_1".to_string(),
            "robot_1".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Success {
                time_ms: 5000,
                energy_used: 0.1,
            },
        );

        let obs2 = TraversabilityObservation::new(
            "obs_2".to_string(),
            "edge_1".to_string(),
            "robot_2".to_string(),
            "wheeled".to_string(),
            TraversalOutcome::Success {
                time_ms: 6000,
                energy_used: 0.12,
            },
        );

        graph.record_observation(obs1).unwrap();
        graph.record_observation(obs2).unwrap();

        let consensus = graph.compute_consensus("edge_1", "wheeled");
        assert!(consensus.is_ok());

        let c = consensus.unwrap();
        assert_eq!(c.total_observations, 2);
        assert_eq!(c.success_count, 2);
    }
}
