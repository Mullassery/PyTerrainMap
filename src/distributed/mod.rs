//! Distributed Deployment & Scaling
//!
//! Phase 7: Enable deployment across multiple nodes with sharding,
//! replication, load balancing, and disaster recovery.
//!
//! Phase 7 (Distributed - 10-12 dev-days):
//! - Data sharding & partitioning
//! - Replication & consistency
//! - Load balancing & routing
//! - Disaster recovery & failover
//! - Cluster orchestration

pub mod sharding;      // Phase 7.1: Data sharding strategy
pub mod replication;   // Phase 7.2: Replication & consistency
pub mod balancing;     // Phase 7.3: Load balancing
pub mod recovery;      // Phase 7.4: Disaster recovery
pub mod cluster;       // Phase 7.5: Cluster orchestration

use serde::{Deserialize, Serialize};

/// Node identifier
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeId(pub u32);

impl NodeId {
    pub fn new(id: u32) -> Self {
        NodeId(id)
    }
}

/// Deployment mode
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeploymentMode {
    Standalone,
    Distributed,
    HighAvailability,
}
