//! Replication & Consistency
//!
//! Phase 7.2: Manage data replication across nodes.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub replication_factor: u32,
    pub consistency_level: ConsistencyLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConsistencyLevel {
    Weak,
    Eventual,
    Strong,
}

pub struct ReplicationManager {
    pub config: ReplicationConfig,
}

impl ReplicationManager {
    pub fn new(replication_factor: u32) -> Self {
        ReplicationManager {
            config: ReplicationConfig {
                replication_factor,
                consistency_level: ConsistencyLevel::Eventual,
            },
        }
    }
}

impl Default for ReplicationManager {
    fn default() -> Self {
        Self::new(3)
    }
}
