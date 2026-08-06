//! Load Balancing & Routing
//!
//! Phase 7.3: Distribute load across cluster nodes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Geographic,
}

pub struct LoadBalancer {
    pub strategy: BalancingStrategy,
    pub nodes: Vec<u32>,
}

impl LoadBalancer {
    pub fn new(strategy: BalancingStrategy, node_count: u32) -> Self {
        LoadBalancer {
            strategy,
            nodes: (0..node_count).collect(),
        }
    }

    pub fn select_node(&self) -> Option<u32> {
        self.nodes.first().copied()
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(BalancingStrategy::RoundRobin, 3)
    }
}
