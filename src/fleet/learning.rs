//! Fleet Learning - Collective Learning Across Multiple Robots
//!
//! Phase 4.3: Enables robots to learn from shared observations,
//! improving terrain models through accumulated fleet knowledge.

use super::RobotId;
use serde::{Deserialize, Serialize};

/// Learning model trained on fleet observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetLearningModel {
    pub robot_id: RobotId,
    pub observations_seen: u64,
    pub accuracy: f32, // 0.0-1.0
}

impl FleetLearningModel {
    /// Create new learning model
    pub fn new(robot_id: RobotId) -> Self {
        FleetLearningModel {
            robot_id,
            observations_seen: 0,
            accuracy: 0.5, // Start neutral
        }
    }
}

/// Stub: Fleet learning engine (to be implemented in Phase 4.3)
pub struct FleetLearningEngine {
    models: Vec<FleetLearningModel>,
}

impl FleetLearningEngine {
    /// Create new learning engine
    pub fn new() -> Self {
        FleetLearningEngine {
            models: Vec::new(),
        }
    }
}

impl Default for FleetLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}
