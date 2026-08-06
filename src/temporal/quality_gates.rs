//! Temporal Quality Gates & Consistency Validation
//!
//! Validate temporal data quality, detect anomalies, and enforce consistency constraints.

use super::index::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Quality gate for temporal data validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub min_quality: f32,
    pub max_temporal_gap_us: i64, // Maximum time gap between consecutive points
    pub max_spatial_jump_m: f32, // Maximum spatial distance between consecutive points
    pub enabled: bool,
}

impl QualityGate {
    /// Create new quality gate
    pub fn new(
        name: &str,
        min_quality: f32,
        max_temporal_gap_us: i64,
        max_spatial_jump_m: f32,
    ) -> Self {
        QualityGate {
            name: name.to_string(),
            min_quality,
            max_temporal_gap_us,
            max_spatial_jump_m,
            enabled: true,
        }
    }

    /// Check if point passes quality gate
    pub fn validate_point(&self, point: &TemporalPoint) -> bool {
        if !self.enabled {
            return true;
        }
        point.quality >= self.min_quality
    }

    /// Check if point pair passes consistency gates
    pub fn validate_pair(&self, before: &TemporalPoint, after: &TemporalPoint) -> bool {
        if !self.enabled {
            return true;
        }

        // Check temporal gap
        let temporal_gap = (after.timestamp - before.timestamp).abs();
        if temporal_gap > self.max_temporal_gap_us && temporal_gap > 0 {
            return false;
        }

        // Check spatial jump
        let dx = after.x - before.x;
        let dy = after.y - before.y;
        let dz = after.z - before.z;
        let spatial_distance = (dx * dx + dy * dy + dz * dz).sqrt();

        spatial_distance <= self.max_spatial_jump_m
    }
}

impl Default for QualityGate {
    fn default() -> Self {
        QualityGate {
            name: "default".to_string(),
            min_quality: 0.5,
            max_temporal_gap_us: 1_000_000, // 1 second
            max_spatial_jump_m: 100.0,
            enabled: true,
        }
    }
}

/// Temporal consistency validator
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalConsistencyValidator {
    pub quality_gates: Vec<QualityGate>,
    pub enable_interpolation: bool,
}

impl TemporalConsistencyValidator {
    /// Create new validator
    pub fn new() -> Self {
        TemporalConsistencyValidator {
            quality_gates: vec![QualityGate::default()],
            enable_interpolation: true,
        }
    }

    /// Add quality gate
    pub fn add_gate(&mut self, gate: QualityGate) {
        self.quality_gates.push(gate);
    }

    /// Validate sequence of points
    pub fn validate_sequence(&self, points: &[TemporalPoint]) -> SequenceValidationResult {
        let mut violations = Vec::new();
        let mut quality_issues = Vec::new();
        let mut continuity_issues = Vec::new();

        for point in points {
            for gate in &self.quality_gates {
                if !gate.validate_point(point) {
                    quality_issues.push(QualityViolation {
                        point: *point,
                        gate_name: gate.name.clone(),
                        quality: point.quality,
                        min_required: gate.min_quality,
                    });
                }
            }
        }

        // Check consecutive pairs
        for window in points.windows(2) {
            let before = &window[0];
            let after = &window[1];

            for gate in &self.quality_gates {
                if !gate.validate_pair(before, after) {
                    continuity_issues.push(ContinuityViolation {
                        from: *before,
                        to: *after,
                        gate_name: gate.name.clone(),
                        temporal_gap_us: after.timestamp - before.timestamp,
                        spatial_distance: (
                            (after.x - before.x).powi(2)
                                + (after.y - before.y).powi(2)
                                + (after.z - before.z).powi(2)
                        ).sqrt(),
                    });
                }
            }
        }

        violations.extend(
            quality_issues
                .iter()
                .map(|v| ValidationViolation::Quality(v.clone())),
        );
        violations.extend(
            continuity_issues
                .iter()
                .map(|v| ValidationViolation::Continuity(v.clone())),
        );

        SequenceValidationResult {
            is_valid: violations.is_empty(),
            violation_count: violations.len() as u32,
            violations,
        }
    }

    /// Detect temporal anomalies
    pub fn detect_anomalies(&self, points: &[TemporalPoint]) -> AnomalyDetectionResult {
        let mut anomalies = Vec::new();

        if points.len() < 3 {
            return AnomalyDetectionResult {
                anomaly_count: 0,
                anomalies,
            };
        }

        // Compute moving average of z-values for trend detection
        let window_size = 3;
        for i in window_size..points.len() {
            let window = &points[i - window_size..=i];
            let z_values: Vec<f32> = window.iter().map(|p| p.z).collect();
            let mean_z = z_values.iter().sum::<f32>() / z_values.len() as f32;

            // Detect sudden spikes/drops
            let current_z = points[i].z;
            let deviation = (current_z - mean_z).abs();
            let std_dev = (z_values
                .iter()
                .map(|z| (z - mean_z).powi(2))
                .sum::<f32>()
                / z_values.len() as f32)
                .sqrt();

            if deviation > 3.0 * std_dev && std_dev > 0.01 {
                anomalies.push(TemporalAnomaly {
                    point: points[i],
                    anomaly_type: AnomalyType::ZValueSpike,
                    severity: deviation / std_dev, // Z-score
                    context: format!("Z-value {} deviates {} from mean {}", current_z, deviation, mean_z),
                });
            }
        }

        // Detect quality degradation patterns
        for window in points.windows(3) {
            let quality_trend = window[2].quality - window[0].quality;
            if quality_trend < -0.3 {
                anomalies.push(TemporalAnomaly {
                    point: window[2],
                    anomaly_type: AnomalyType::QualityDegradation,
                    severity: quality_trend.abs(),
                    context: format!("Quality degraded from {} to {}", window[0].quality, window[2].quality),
                });
            }
        }

        AnomalyDetectionResult {
            anomaly_count: anomalies.len() as u32,
            anomalies,
        }
    }

    /// Correct quality values based on neighboring points
    pub fn correct_quality_values(&self, points: &mut [TemporalPoint]) {
        if points.len() < 3 {
            return;
        }

        for i in 1..points.len() - 1 {
            let before = points[i - 1];
            let after = points[i + 1];

            // If quality is too low, try to estimate from neighbors
            if points[i].quality < 0.4 {
                let neighbor_quality = (before.quality + after.quality) / 2.0;
                if neighbor_quality > 0.6 {
                    points[i].quality = neighbor_quality * 0.9; // Slightly penalize interpolated quality
                }
            }
        }
    }

    /// Validate and correct sequence
    pub fn validate_and_correct(&self, points: &mut [TemporalPoint]) -> ValidationResult {
        let validation_before = self.validate_sequence(points);

        // Apply corrections
        self.correct_quality_values(points);

        let validation_after = self.validate_sequence(points);

        ValidationResult {
            violations_before: validation_before.violation_count,
            violations_after: validation_after.violation_count,
            violations_fixed: validation_before.violation_count.saturating_sub(validation_after.violation_count),
            is_valid: validation_after.is_valid,
        }
    }
}

impl Default for TemporalConsistencyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quality violation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QualityViolation {
    pub point: TemporalPoint,
    pub gate_name: String,
    pub quality: f32,
    pub min_required: f32,
}

/// Continuity violation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContinuityViolation {
    pub from: TemporalPoint,
    pub to: TemporalPoint,
    pub gate_name: String,
    pub temporal_gap_us: i64,
    pub spatial_distance: f32,
}

/// Validation violation (enum of possible violations)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValidationViolation {
    Quality(QualityViolation),
    Continuity(ContinuityViolation),
}

/// Validation result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequenceValidationResult {
    pub is_valid: bool,
    pub violation_count: u32,
    pub violations: Vec<ValidationViolation>,
}

/// Temporal anomaly
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalAnomaly {
    pub point: TemporalPoint,
    pub anomaly_type: AnomalyType,
    pub severity: f32, // 0.0-1.0+ scale
    pub context: String,
}

/// Anomaly type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    ZValueSpike,
    QualityDegradation,
    TemporalOutlier,
    SpatialJump,
}

/// Anomaly detection result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    pub anomaly_count: u32,
    pub anomalies: Vec<TemporalAnomaly>,
}

/// Validation and correction result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub violations_before: u32,
    pub violations_after: u32,
    pub violations_fixed: u32,
    pub is_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_creation() {
        let gate = QualityGate::new("strict", 0.8, 1_000_000, 50.0);
        assert_eq!(gate.min_quality, 0.8);
        assert!(gate.enabled);
    }

    #[test]
    fn test_point_quality_validation() {
        let gate = QualityGate::new("strict", 0.8, 1_000_000, 50.0);
        let high_quality = TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.9);
        let low_quality = TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.5);

        assert!(gate.validate_point(&high_quality));
        assert!(!gate.validate_point(&low_quality));
    }

    #[test]
    fn test_continuity_validation() {
        let gate = QualityGate::new("normal", 0.5, 1_000_000, 50.0);
        let p1 = TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8);
        let p2 = TemporalPoint::new(1.0, 1.0, 1.0, 2000, 0.8); // 1.73m away, 1s gap
        let p3 = TemporalPoint::new(100.0, 100.0, 0.0, 3000, 0.8); // 141m away, huge jump

        assert!(gate.validate_pair(&p1, &p2));
        assert!(!gate.validate_pair(&p1, &p3));
    }

    #[test]
    fn test_sequence_validation() {
        let validator = TemporalConsistencyValidator::new();
        let points = vec![
            TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 1.0, 2000, 0.8),
            TemporalPoint::new(2.0, 2.0, 2.0, 3000, 0.8),
        ];

        let result = validator.validate_sequence(&points);
        assert!(result.is_valid);
        assert_eq!(result.violation_count, 0);
    }

    #[test]
    fn test_anomaly_detection_spike() {
        let validator = TemporalConsistencyValidator::new();
        let mut points = vec![
            TemporalPoint::new(0.0, 0.0, 1.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 1.0, 2000, 0.8),
            TemporalPoint::new(2.0, 2.0, 1.0, 3000, 0.8),
            TemporalPoint::new(3.0, 3.0, 10.0, 4000, 0.8), // Spike
            TemporalPoint::new(4.0, 4.0, 1.0, 5000, 0.8),
        ];

        let result = validator.detect_anomalies(&points);
        assert!(result.anomaly_count > 0);
    }

    #[test]
    fn test_quality_correction() {
        let validator = TemporalConsistencyValidator::new();
        let mut points = vec![
            TemporalPoint::new(0.0, 0.0, 1.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 1.0, 2000, 0.2), // Low quality
            TemporalPoint::new(2.0, 2.0, 1.0, 3000, 0.8),
        ];

        validator.correct_quality_values(&mut points);
        // Middle point quality should be corrected
        assert!(points[1].quality > 0.2);
    }

    #[test]
    fn test_disabled_gate() {
        let mut gate = QualityGate::new("test", 0.8, 1_000_000, 50.0);
        gate.enabled = false;
        let low_quality = TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.5);

        // Should pass because gate is disabled
        assert!(gate.validate_point(&low_quality));
    }
}
