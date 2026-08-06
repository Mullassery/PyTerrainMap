//! Conflict Resolution for Disagreements in Observations
//!
//! Phase 4.4: Handles situations where robots observe conflicting
//! terrain states and must reconcile differences.

use crate::temporal::TemporalPoint;
use super::RobotId;
use serde::{Deserialize, Serialize};

/// Conflict between robot observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationConflict {
    pub location: (f32, f32), // (x, y)
    pub observations: Vec<(RobotId, TemporalPoint)>,
    pub conflict_type: ConflictType,
}

/// Type of conflict
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    ZValueDifference, // Different z (elevation) values
    QualityDifference, // Different quality scores
    TimingDifference, // Significant time offset
}

/// Conflict resolution strategy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    TakeNewest,     // Use most recent observation
    TakeHighestQuality, // Use highest quality observation
    Average,        // Average all observations
    Weighted,       // Weighted average by robot reliability
}

/// Stub: Conflict resolver (to be implemented in Phase 4.4)
pub struct ConflictResolver {
    strategy: ResolutionStrategy,
}

impl ConflictResolver {
    /// Create new conflict resolver
    pub fn new(strategy: ResolutionStrategy) -> Self {
        ConflictResolver { strategy }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ResolutionStrategy::TakeHighestQuality)
    }
}
