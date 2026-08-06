//! Multi-robot Fleet Management & Consensus
//!
//! Enables multiple robots to share observations, reach consensus on terrain state,
//! and learn collectively from merged observations.
//!
//! Phase 4 (Multi-robot Consensus & Fleet Learning):
//! - Multi-robot observation protocol (sharing & synchronization)
//! - Consensus algorithms (agreement on terrain state)
//! - Fleet learning (collective learning across robots)
//! - Conflict resolution (handling disagreements)
//! - Map synchronization (coordinating local maps)

pub mod protocol;    // Phase 4.1: Multi-robot observation protocol
pub mod consensus;   // Phase 4.2: Consensus algorithms
pub mod learning;    // Phase 4.3: Fleet learning mechanisms
pub mod conflict;    // Phase 4.4: Conflict resolution
pub mod sync;        // Phase 4.5: Map synchronization

use serde::{Deserialize, Serialize};

/// Robot identifier
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct RobotId(pub u32);

impl RobotId {
    /// Create new robot ID
    pub fn new(id: u32) -> Self {
        RobotId(id)
    }
}

/// Fleet statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStatistics {
    pub active_robots: u32,
    pub total_observations: u64,
    pub consensus_agreements: u32,
    pub conflicts_resolved: u32,
    pub sync_cycles: u64,
}

impl FleetStatistics {
    /// Create new statistics
    pub fn new() -> Self {
        FleetStatistics {
            active_robots: 0,
            total_observations: 0,
            consensus_agreements: 0,
            conflicts_resolved: 0,
            sync_cycles: 0,
        }
    }
}

impl Default for FleetStatistics {
    fn default() -> Self {
        Self::new()
    }
}
