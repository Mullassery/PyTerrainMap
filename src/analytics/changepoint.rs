//! Change Point Detection
//!
//! Phase 6.3: Identify time points where terrain characteristics
//! fundamentally change, enabling event-based analysis.

use serde::{Deserialize, Serialize};

/// Change point detection result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangePoint {
    pub timestamp_us: i64,
    pub magnitude: f32,
    pub confidence: f32,
}

/// Stub: Change point detector (to be implemented in Phase 6.3)
pub struct ChangePointDetector {
    pub window_size: u32,
}

impl ChangePointDetector {
    /// Create new detector
    pub fn new(window_size: u32) -> Self {
        ChangePointDetector { window_size }
    }
}

impl Default for ChangePointDetector {
    fn default() -> Self {
        Self::new(50)
    }
}
