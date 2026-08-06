//! Cluster Orchestration
//!
//! Phase 7.5: Manage cluster configuration and membership.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClusterNode {
    pub node_id: u32,
    pub address: String,
    pub status: NodeStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Healthy,
    Degraded,
    Failed,
}

pub struct ClusterManager {
    pub nodes: HashMap<u32, ClusterNode>,
}

impl ClusterManager {
    pub fn new() -> Self {
        ClusterManager {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node_id: u32, address: String) {
        self.nodes.insert(node_id, ClusterNode {
            node_id,
            address,
            status: NodeStatus::Healthy,
        });
    }

    pub fn get_healthy_nodes(&self) -> Vec<u32> {
        self.nodes.values()
            .filter(|n| n.status == NodeStatus::Healthy)
            .map(|n| n.node_id)
            .collect()
    }
}

impl Default for ClusterManager {
    fn default() -> Self {
        Self::new()
    }
}
