//! Trend Analysis & Forecasting
//!
//! Phase 6.4: Analyze long-term trends and generate forecasts
//! for future terrain evolution.

use serde::{Deserialize, Serialize};

/// Trend direction
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Trend forecast
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendForecast {
    pub direction: TrendDirection,
    pub slope: f32,
    pub confidence: f32,
    pub forecast_horizon_us: i64,
}

/// Stub: Trend analyzer (to be implemented in Phase 6.4)
pub struct TrendAnalyzer {
    pub window_size: u32,
}

impl TrendAnalyzer {
    /// Create new analyzer
    pub fn new(window_size: u32) -> Self {
        TrendAnalyzer { window_size }
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new(100)
    }
}
