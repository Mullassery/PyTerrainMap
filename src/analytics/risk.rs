//! Risk Scoring & Alerts
//!
//! Phase 6.5: Score locations for risk factors and generate
//! alerts for hazardous terrain conditions.

use serde::{Deserialize, Serialize};

/// Risk level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk factor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: f32, // 0.0-1.0
    pub confidence: f32,
}

/// Risk alert
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskAlert {
    pub location: (f32, f32),
    pub risk_level: RiskLevel,
    pub risk_score: f32,
    pub factors: Vec<RiskFactor>,
}

/// Stub: Risk scorer (to be implemented in Phase 6.5)
pub struct RiskScorer {
    pub threshold: f32,
}

impl RiskScorer {
    /// Create new scorer
    pub fn new(threshold: f32) -> Self {
        RiskScorer { threshold }
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new(0.7)
    }
}
