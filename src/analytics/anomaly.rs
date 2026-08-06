//! Anomaly Detection Algorithms
//!
//! Phase 6.2: Detect unusual terrain observations that deviate
//! from expected patterns using statistical and ML methods.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Anomaly type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    Outlier,             // Deviant value
    SeasonalAnomaly,    // Breaks seasonal pattern
    LevelShift,         // Sudden change
    Trend,              // Unexpected trend
}

/// Detected anomaly
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedAnomaly {
    pub point: TemporalPoint,
    pub anomaly_type: AnomalyType,
    pub z_score: f32,
    pub confidence: f32,
}

impl DetectedAnomaly {
    /// Create new anomaly
    pub fn new(point: TemporalPoint, atype: AnomalyType, z_score: f32) -> Self {
        let confidence = (z_score.abs() / 5.0).min(1.0);
        DetectedAnomaly {
            point,
            anomaly_type: atype,
            z_score,
            confidence,
        }
    }
}

/// Anomaly detector using statistical methods
pub struct AnomalyDetector {
    pub threshold: f32,
    pub window_size: usize,
    pub historical_data: VecDeque<TemporalPoint>,
    pub detected_anomalies: Vec<DetectedAnomaly>,
}

impl AnomalyDetector {
    /// Create new detector
    pub fn new(threshold: f32) -> Self {
        AnomalyDetector {
            threshold,
            window_size: 100,
            historical_data: VecDeque::new(),
            detected_anomalies: Vec::new(),
        }
    }

    /// Add point and check for anomalies
    pub fn process_point(&mut self, point: TemporalPoint) -> Option<DetectedAnomaly> {
        self.historical_data.push_back(point);

        // Keep window size bounded
        if self.historical_data.len() > self.window_size {
            self.historical_data.pop_front();
        }

        // Check for anomaly if we have enough history
        if self.historical_data.len() < 10 {
            return None;
        }

        self.detect_z_score_anomaly(point)
    }

    /// Detect using z-score method
    fn detect_z_score_anomaly(&mut self, point: TemporalPoint) -> Option<DetectedAnomaly> {
        let z_values: Vec<f32> = self.historical_data.iter().map(|p| p.z).collect();

        let mean = z_values.iter().sum::<f32>() / z_values.len() as f32;
        let variance = z_values.iter()
            .map(|z| (z - mean).powi(2))
            .sum::<f32>() / z_values.len() as f32;
        let std_dev = variance.sqrt();

        if std_dev < 1e-6 {
            return None;
        }

        let z_score = (point.z - mean) / std_dev;

        if z_score.abs() > self.threshold {
            let anomaly = DetectedAnomaly::new(point, AnomalyType::Outlier, z_score);
            self.detected_anomalies.push(anomaly.clone());
            Some(anomaly)
        } else {
            None
        }
    }

    /// Detect level shift anomaly
    pub fn detect_level_shift(&mut self, points: &[TemporalPoint]) -> Option<usize> {
        if points.len() < 20 {
            return None;
        }

        let mid = points.len() / 2;
        let first_half: Vec<f32> = points[..mid].iter().map(|p| p.z).collect();
        let second_half: Vec<f32> = points[mid..].iter().map(|p| p.z).collect();

        let mean1 = first_half.iter().sum::<f32>() / first_half.len() as f32;
        let mean2 = second_half.iter().sum::<f32>() / second_half.len() as f32;

        let shift = (mean2 - mean1).abs();
        if shift > self.threshold {
            Some(mid)
        } else {
            None
        }
    }

    /// Get anomaly statistics
    pub fn statistics(&self) -> AnomalyStatistics {
        let outlier_count = self.detected_anomalies
            .iter()
            .filter(|a| a.anomaly_type == AnomalyType::Outlier)
            .count() as u32;

        AnomalyStatistics {
            total_anomalies: self.detected_anomalies.len() as u32,
            outlier_count,
            average_confidence: if self.detected_anomalies.is_empty() {
                0.0
            } else {
                self.detected_anomalies.iter().map(|a| a.confidence).sum::<f32>()
                    / self.detected_anomalies.len() as f32
            },
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new(2.0) // Z-score threshold
    }
}

/// Anomaly statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyStatistics {
    pub total_anomalies: u32,
    pub outlier_count: u32,
    pub average_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_anomaly() {
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let anomaly = DetectedAnomaly::new(point, AnomalyType::Outlier, 3.5);

        assert_eq!(anomaly.anomaly_type, AnomalyType::Outlier);
        assert!(anomaly.confidence > 0.0);
    }

    #[test]
    fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new(2.0);
        assert_eq!(detector.threshold, 2.0);
        assert_eq!(detector.historical_data.len(), 0);
    }

    #[test]
    fn test_anomaly_detector_processing() {
        let mut detector = AnomalyDetector::new(2.0);

        // Add normal points
        for i in 0..15 {
            let point = TemporalPoint::new(1.0, 2.0, 10.0 + i as f32, 1000 + i as i64 * 100, 0.8);
            detector.process_point(point);
        }

        // Add anomalous point
        let anomaly_point = TemporalPoint::new(1.0, 2.0, 50.0, 2500, 0.8);
        let result = detector.process_point(anomaly_point);

        assert!(result.is_some());
    }

    #[test]
    fn test_anomaly_statistics() {
        let detector = AnomalyDetector::new(2.0);
        let stats = detector.statistics();

        assert_eq!(stats.total_anomalies, 0);
    }
}
