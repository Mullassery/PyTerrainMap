//! Risk Scoring & Alerts
//!
//! Phase 6.5: Score locations for risk factors and generate
//! alerts for hazardous terrain conditions.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk level
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// Get risk score threshold
    pub fn threshold(&self) -> f32 {
        match self {
            RiskLevel::Low => 0.2,
            RiskLevel::Medium => 0.5,
            RiskLevel::High => 0.7,
            RiskLevel::Critical => 0.9,
        }
    }
}

/// Risk factor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_type: String,
    pub severity: f32, // 0.0-1.0
    pub confidence: f32,
    pub weight: f32, // Impact weight
}

impl RiskFactor {
    /// Create risk factor
    pub fn new(factor_type: &str, severity: f32, confidence: f32) -> Self {
        RiskFactor {
            factor_type: factor_type.to_string(),
            severity: severity.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            weight: 1.0,
        }
    }

    /// Get weighted severity
    pub fn weighted_severity(&self) -> f32 {
        self.severity * self.confidence * self.weight
    }
}

/// Risk alert
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskAlert {
    pub location: (f32, f32),
    pub risk_level: RiskLevel,
    pub risk_score: f32,
    pub factors: Vec<RiskFactor>,
    pub timestamp_us: i64,
}

impl RiskAlert {
    /// Create risk alert
    pub fn new(location: (f32, f32), level: RiskLevel, score: f32, factors: Vec<RiskFactor>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        RiskAlert {
            location,
            risk_level: level,
            risk_score: score.clamp(0.0, 1.0),
            factors,
            timestamp_us: now,
        }
    }
}

/// Risk scorer
pub struct RiskScorer {
    pub threshold: f32,
    pub alerts: Vec<RiskAlert>,
}

impl RiskScorer {
    /// Create new scorer
    pub fn new(threshold: f32) -> Self {
        RiskScorer {
            threshold,
            alerts: Vec::new(),
        }
    }

    /// Score location based on point
    pub fn score_point(&mut self, point: TemporalPoint) -> Option<RiskAlert> {
        let mut factors = Vec::new();

        // Check elevation extremes
        if point.z > 1000.0 {
            factors.push(RiskFactor::new("high_elevation", 0.7, 0.9));
        }
        if point.z < -100.0 {
            factors.push(RiskFactor::new("extreme_depth", 0.8, 0.85));
        }

        // Check quality
        if point.quality < 0.3 {
            factors.push(RiskFactor::new("low_quality", 0.6, 0.8));
        }

        // Calculate risk score
        let risk_score = if factors.is_empty() {
            0.1
        } else {
            factors.iter().map(|f| f.weighted_severity()).sum::<f32>() / factors.len() as f32
        };

        // Determine risk level
        let risk_level = match risk_score {
            s if s >= 0.9 => RiskLevel::Critical,
            s if s >= 0.7 => RiskLevel::High,
            s if s >= 0.5 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        if risk_score >= self.threshold {
            let alert = RiskAlert::new((point.x, point.y), risk_level, risk_score, factors);
            self.alerts.push(alert.clone());
            Some(alert)
        } else {
            None
        }
    }

    /// Add custom risk factor
    pub fn add_factor(&mut self, factor: RiskFactor) {
        // Store for batching
    }

    /// Get risk statistics
    pub fn statistics(&self) -> RiskStatistics {
        let critical_count = self.alerts
            .iter()
            .filter(|a| a.risk_level == RiskLevel::Critical)
            .count() as u32;

        RiskStatistics {
            total_alerts: self.alerts.len() as u32,
            critical_count,
            average_risk_score: if self.alerts.is_empty() {
                0.0
            } else {
                self.alerts.iter().map(|a| a.risk_score).sum::<f32>()
                    / self.alerts.len() as f32
            },
        }
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new(0.7)
    }
}

/// Risk statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RiskStatistics {
    pub total_alerts: u32,
    pub critical_count: u32,
    pub average_risk_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_factor() {
        let factor = RiskFactor::new("test", 0.8, 0.9);
        assert_eq!(factor.factor_type, "test");
        assert!(factor.weighted_severity() > 0.0);
    }

    #[test]
    fn test_risk_alert() {
        let alert = RiskAlert::new((1.0, 2.0), RiskLevel::High, 0.75, vec![]);
        assert_eq!(alert.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_risk_scorer() {
        let scorer = RiskScorer::new(0.7);
        assert_eq!(scorer.threshold, 0.7);
    }

    #[test]
    fn test_score_point() {
        let mut scorer = RiskScorer::new(0.5);
        let point = TemporalPoint::new(1.0, 2.0, 1500.0, 1000, 0.2);

        let alert = scorer.score_point(point);
        assert!(alert.is_some());
    }

    #[test]
    fn test_risk_statistics() {
        let scorer = RiskScorer::new(0.7);
        let stats = scorer.statistics();

        assert_eq!(stats.total_alerts, 0);
    }
}
