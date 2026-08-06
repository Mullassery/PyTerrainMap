//! Anomaly Detection Algorithms
//!
//! Phase 6.2: Detect unusual terrain observations that deviate
//! from expected patterns using statistical and ML methods.

use serde::{Deserialize, Serialize};

/// Anomaly type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    Outlier,             // Deviant value
    SeasonalAnomaly,    // Breaks seasonal pattern
    LevelShift,         // Sudden change
    Trend,              // Unexpected trend
}

/// Stub: Anomaly detector (to be implemented in Phase 6.2)
pub struct AnomalyDetector {
    pub threshold: f32,
}

impl AnomalyDetector {
    /// Create new detector
    pub fn new(threshold: f32) -> Self {
        AnomalyDetector { threshold }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(2.0) // Z-score threshold
    }
}
