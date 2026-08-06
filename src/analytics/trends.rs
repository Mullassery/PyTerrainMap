//! Trend Analysis & Forecasting
//!
//! Phase 6.4: Analyze long-term trends and generate forecasts
//! for future terrain evolution.

use crate::temporal::TemporalPoint;
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
    pub location: (f32, f32),
    pub direction: TrendDirection,
    pub slope: f32,
    pub confidence: f32,
    pub forecast_horizon_us: i64,
    pub forecast_value: f32,
}

impl TrendForecast {
    /// Create forecast
    pub fn new(
        location: (f32, f32),
        direction: TrendDirection,
        slope: f32,
        confidence: f32,
        horizon: i64,
        value: f32,
    ) -> Self {
        TrendForecast {
            location,
            direction,
            slope,
            confidence: confidence.clamp(0.0, 1.0),
            forecast_horizon_us: horizon,
            forecast_value: value,
        }
    }
}

/// Trend analyzer
pub struct TrendAnalyzer {
    pub window_size: u32,
    pub forecasts: Vec<TrendForecast>,
}

impl TrendAnalyzer {
    /// Create new analyzer
    pub fn new(window_size: u32) -> Self {
        TrendAnalyzer {
            window_size,
            forecasts: Vec::new(),
        }
    }

    /// Analyze trend in data
    pub fn analyze(&mut self, points: &[TemporalPoint]) -> Option<TrendForecast> {
        if points.len() < 5 {
            return None;
        }

        let n = points.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, point) in points.iter().enumerate() {
            let x = i as f32;
            let y = point.z;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        let direction = if slope > 0.1 {
            TrendDirection::Increasing
        } else if slope < -0.1 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        let last_point = points.last().unwrap();
        let forecast_value = intercept + slope * n;
        let confidence = (slope.abs() / 5.0).min(1.0);

        let forecast = TrendForecast::new(
            (last_point.x, last_point.y),
            direction,
            slope,
            confidence,
            1_000_000 * 86400, // 1 day forecast
            forecast_value,
        );

        self.forecasts.push(forecast.clone());
        Some(forecast)
    }

    /// Get moving average
    pub fn moving_average(&self, points: &[TemporalPoint], window: usize) -> Vec<f32> {
        let mut ma = Vec::new();

        for i in window..=points.len() {
            let window_data = &points[i - window..i];
            let avg = window_data.iter().map(|p| p.z).sum::<f32>() / window as f32;
            ma.push(avg);
        }

        ma
    }

    /// Get statistics
    pub fn statistics(&self) -> TrendStatistics {
        let increasing = self.forecasts
            .iter()
            .filter(|f| f.direction == TrendDirection::Increasing)
            .count() as u32;

        TrendStatistics {
            total_forecasts: self.forecasts.len() as u32,
            increasing_trends: increasing,
            average_confidence: if self.forecasts.is_empty() {
                0.0
            } else {
                self.forecasts.iter().map(|f| f.confidence).sum::<f32>()
                    / self.forecasts.len() as f32
            },
        }
    }
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Trend statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendStatistics {
    pub total_forecasts: u32,
    pub increasing_trends: u32,
    pub average_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_forecast() {
        let forecast = TrendForecast::new(
            (1.0, 2.0),
            TrendDirection::Increasing,
            0.5,
            0.8,
            1_000_000,
            150.0,
        );

        assert_eq!(forecast.direction, TrendDirection::Increasing);
    }

    #[test]
    fn test_trend_analyzer() {
        let analyzer = TrendAnalyzer::new(100);
        assert_eq!(analyzer.window_size, 100);
    }

    #[test]
    fn test_trend_analyze() {
        let mut analyzer = TrendAnalyzer::new(10);

        let points: Vec<_> = (0..20)
            .map(|i| TemporalPoint::new(1.0, 2.0, 10.0 + i as f32, 1000 + i as i64 * 100, 0.8))
            .collect();

        let forecast = analyzer.analyze(&points);
        assert!(forecast.is_some());
    }
}
