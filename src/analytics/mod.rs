//! Advanced Analytics & Prediction Layer
//!
//! Phase 6: Machine learning and statistical analytics for terrain prediction,
//! anomaly detection, and intelligent insights.
//!
//! Phase 6 (Advanced Analytics - 10-15 dev-days):
//! - Terrain prediction & extrapolation
//! - Anomaly detection algorithms
//! - Change point detection
//! - Trend analysis
//! - Risk scoring

pub mod prediction;      // Phase 6.1: Terrain prediction & extrapolation
pub mod anomaly;        // Phase 6.2: Anomaly detection algorithms
pub mod changepoint;    // Phase 6.3: Change point detection
pub mod trends;         // Phase 6.4: Trend analysis & forecasting
pub mod risk;           // Phase 6.5: Risk scoring & alerts

use serde::{Deserialize, Serialize};

/// Analytics capabilities
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnalyticsCapabilities {
    pub prediction_enabled: bool,
    pub anomaly_detection: bool,
    pub change_detection: bool,
    pub trend_analysis: bool,
    pub risk_scoring: bool,
}

impl AnalyticsCapabilities {
    /// Create with all features enabled
    pub fn new() -> Self {
        AnalyticsCapabilities {
            prediction_enabled: true,
            anomaly_detection: true,
            change_detection: true,
            trend_analysis: true,
            risk_scoring: true,
        }
    }
}

impl Default for AnalyticsCapabilities {
    fn default() -> Self {
        Self::new()
    }
}
