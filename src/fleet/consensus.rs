//! Consensus Algorithms for Multi-robot Terrain State Agreement
//!
//! Phase 4.2: Enables robots to reach consensus on terrain observations
//! despite differences in local measurements and temporal ordering.

use crate::temporal::TemporalPoint;
use super::{RobotId, protocol::RobotObservation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Consensus decision for terrain state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusDecision {
    pub point: TemporalPoint,
    pub agreeing_robots: Vec<RobotId>,
    pub confidence: f32,
    pub consensus_method: ConsensusMethod,
}

/// Consensus method used
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConsensusMethod {
    Majority,       // >50% of robots agree
    Supermajority,  // >66% of robots agree
    WeightedVote,   // Weighted by robot reliability scores
    BayesianFusion, // Bayesian combination of estimates
}

/// Stub: Consensus state (to be implemented in Phase 4.2)
pub struct ConsensusEngine {
    robot_id: RobotId,
    method: ConsensusMethod,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(robot_id: RobotId) -> Self {
        ConsensusEngine {
            robot_id,
            method: ConsensusMethod::Majority,
        }
    }
}
