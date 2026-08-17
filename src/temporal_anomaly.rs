//! Temporal-quality-aware anomaly detection
//!
//! Enhances anomaly detection with temporal quality weighting, recognizing that
//! late arrivals and high-latency observations should have lower confidence scores
//! even if they match the baseline.

use crate::anomaly::{AnomalyDetector, AnomalyType};
use crate::types::{BaselineStatistics, Observation, Result};

/// Anomaly detection with temporal quality weighting
///
/// Combines spatial anomaly detection (z-score, IQR) with temporal quality factors:
/// - Clock synchronization confidence
/// - Latency (event_time to ingestion_time)
/// - Late arrival flag
/// - Temporal continuity
pub struct TemporalAnomalyDetector {
    base_detector: AnomalyDetector,
    /// Min temporal quality to trust anomaly classification (0.3-1.0)
    min_temporal_quality: f32,
}

impl TemporalAnomalyDetector {
    /// Create detector with default thresholds
    pub fn new(detector: AnomalyDetector, min_temporal_quality: f32) -> Self {
        TemporalAnomalyDetector {
            base_detector: detector,
            min_temporal_quality: min_temporal_quality.clamp(0.3, 1.0),
        }
    }

    /// Calculate temporal quality factor from observation metadata
    ///
    /// Quality score (0.0-1.0) based on:
    /// - Clock source sync confidence (0.5-1.0)
    /// - Latency penalty (1.0 fresh, 0.3 stale)
    /// - Late arrival flag (0.7 if late, else 1.0)
    pub fn temporal_quality(&self, obs: &Observation) -> f32 {
        let clock_quality = obs.temporal.sync_confidence;

        // Latency penalty (ingestion - event time)
        let latency_us = obs.temporal.ingestion_time_us - obs.temporal.event_time_us;
        let latency_penalty = self.latency_penalty_factor(latency_us);

        // Late arrival penalty
        let late_arrival_factor = if obs.temporal.is_late_arrival {
            0.7 // Late arrivals get 30% confidence penalty
        } else {
            1.0
        };

        // Combined quality = clock × latency × late_arrival
        (clock_quality * latency_penalty * late_arrival_factor).clamp(0.0, 1.0)
    }

    /// Calculate latency penalty factor
    ///
    /// - <100ms: 1.0 (no penalty)
    /// - 100-5000ms: linear interpolation 1.0→0.3
    /// - >5000ms: 0.3 (high penalty)
    fn latency_penalty_factor(&self, latency_us: i64) -> f32 {
        let latency_ms = latency_us / 1_000;

        if latency_ms <= 100 {
            1.0
        } else if latency_ms >= 5000 {
            0.3
        } else {
            // Linear interpolation: 100ms→1.0, 5000ms→0.3
            let ratio = (latency_ms - 100) as f32 / (5000.0 - 100.0);
            1.0 - (ratio * 0.7) // 1.0 - (ratio × 0.7)
        }
    }

    /// Detect anomaly with temporal quality weighting
    ///
    /// Returns (anomaly_type, confidence, temporal_quality)
    pub fn detect_with_quality(
        &self,
        observation: &Observation,
        baseline: &BaselineStatistics,
    ) -> Result<(AnomalyType, f32, f32)> {
        // Get base anomaly detection
        let anomaly = self.base_detector.detect_zscore(observation, baseline)?;

        // Calculate temporal quality
        let tq = self.temporal_quality(observation);

        // Adjust confidence based on temporal quality
        let adjusted_confidence = observation.confidence * tq;

        Ok((anomaly, adjusted_confidence, tq))
    }

    /// Score anomaly severity considering temporal factors
    ///
    /// Returns a score 0.0-1.0 where:
    /// - 0.0 = definitely normal
    /// - 1.0 = definitely anomalous
    pub fn anomaly_severity(
        &self,
        anomaly: &AnomalyType,
        // Not currently factored into the severity score (severity is driven by
        // the anomaly magnitude + temporal_quality); kept in the signature since
        // callers already have it on hand and a future revision may want it.
        _obs_confidence: f32,
        temporal_quality: f32,
    ) -> f32 {
        match anomaly {
            AnomalyType::ZScoreOutlier { z_score } => {
                // Higher z-score = higher severity, modulated by temporal quality
                let base_severity = (z_score / (self.base_detector.z_threshold * 2.0)).min(1.0);
                base_severity * temporal_quality
            }
            AnomalyType::IQROutlier { iqr_factor } => {
                let base_severity =
                    (iqr_factor / (self.base_detector.iqr_multiplier * 2.0)).min(1.0);
                base_severity * temporal_quality
            }
            AnomalyType::RogueBotSignal { deviation, .. } => {
                let base_severity = (*deviation / self.base_detector.rogue_bot_threshold).min(1.0);
                base_severity * temporal_quality
            }
            AnomalyType::SensorDrift { drift_rate } => {
                // Drift is more suspicious, weight by temporal quality
                let base_severity = (*drift_rate).min(1.0);
                base_severity * temporal_quality
            }
            AnomalyType::SensorSpike { magnitude } => {
                // Spikes on low-quality observations are less trustworthy
                let base_severity = (*magnitude).min(1.0);
                base_severity * temporal_quality
            }
            AnomalyType::Normal => 0.0,
        }
    }

    /// Check if observation passes temporal quality gate
    ///
    /// Returns false if temporal_quality < min_temporal_quality
    pub fn passes_temporal_quality_gate(&self, obs: &Observation) -> bool {
        self.temporal_quality(obs) >= self.min_temporal_quality
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClockSource, GeoPoint, SensorType, SensorValue, TemporalMetadata};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_observation(
        event_time_us: i64,
        ingestion_time_us: i64,
        is_late: bool,
        clock_confidence: f32,
    ) -> Observation {
        Observation {
            id: Uuid::new_v4(),
            robot_id: "robot-1".to_string(),
            timestamp: event_time_us / 1_000_000,
            location: GeoPoint::new(40.7128, -74.0060),
            elevation_asl: None,
            sensor_type: SensorType::Thermal,
            value: SensorValue::Temperature { celsius: 25.0 },
            confidence: 0.9,
            temporal: TemporalMetadata {
                event_time_us,
                capture_time_us: event_time_us + 10_000,
                transmission_time_us: event_time_us + 100_000,
                ingestion_time_us,
                processing_time_us: ingestion_time_us + 100_000,
                clock_source: ClockSource::GPS,
                precision_us: 1_000,
                estimated_latency_us: (ingestion_time_us - event_time_us).max(0) as u32,
                sync_confidence: clock_confidence,
                is_late_arrival: is_late,
                jitter_us: 5_000,
                temporal_confidence: 0.9,
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_latency_penalty_fresh() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        // <100ms latency should have no penalty
        assert!((detector.latency_penalty_factor(50_000) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_latency_penalty_stale() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        // >5s latency should have max penalty
        assert!((detector.latency_penalty_factor(5_001_000) - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_latency_penalty_interpolation() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        // 1s latency should be in middle
        let penalty = detector.latency_penalty_factor(1_000_000);
        assert!(penalty > 0.3 && penalty < 1.0);
    }

    #[test]
    fn test_temporal_quality_fresh_observation() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        let obs = create_test_observation(1000_000, 1050_000, false, 0.95);
        let tq = detector.temporal_quality(&obs);

        // Fresh observation with good clock: quality ~0.95
        assert!(tq > 0.9);
    }

    #[test]
    fn test_temporal_quality_late_arrival() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        let obs = create_test_observation(1000_000, 1050_000, true, 0.95);
        let tq = detector.temporal_quality(&obs);

        // Late arrival: gets 0.7 multiplier
        assert!(tq < 0.9 && tq > 0.6);
    }

    #[test]
    fn test_temporal_quality_gate() {
        let detector = TemporalAnomalyDetector::new(AnomalyDetector::default(), 0.7);

        let good_obs = create_test_observation(1000_000, 1050_000, false, 0.95);
        assert!(detector.passes_temporal_quality_gate(&good_obs));

        let bad_obs = create_test_observation(1000_000, 6000_000, true, 0.5);
        assert!(!detector.passes_temporal_quality_gate(&bad_obs));
    }
}
