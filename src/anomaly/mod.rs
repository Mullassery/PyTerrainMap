//! Anomaly detection for sensor malfunction and rogue bot identification
//!
//! Detects deviations from baseline, multi-robot conflicts, and malfunction patterns.
//! Distinguishes between environmental changes and sensor errors.

use crate::types::{
    Observation, Result, SensorValue, BaselineStatistics,
};

/// Anomaly detection result
#[derive(Clone, Debug, PartialEq)]
pub enum AnomalyType {
    /// Observation differs >z_threshold standard deviations from baseline
    ZScoreOutlier { z_score: f32 },
    /// Observation outside interquartile range (IQR method)
    IQROutlier { iqr_factor: f32 },
    /// Robot consistently reports values different from others
    RogueBotSignal { deviation: f32, consensus_count: usize },
    /// Sensor shows drift pattern (gradual deviation over time)
    SensorDrift { drift_rate: f32 },
    /// Single extreme value, likely sensor spike
    SensorSpike { magnitude: f32 },
    /// Normal observation
    Normal,
}

/// Anomaly detector with configurable thresholds
pub struct AnomalyDetector {
    /// Z-score threshold for outlier detection
    pub z_threshold: f32,
    /// IQR multiplier (typically 1.5 for outliers, 3.0 for extreme)
    pub iqr_multiplier: f32,
    /// Minimum observations to establish baseline
    pub min_baseline_size: usize,
    /// Maximum allowed deviation for rogue bot detection
    pub rogue_bot_threshold: f32,
}

impl AnomalyDetector {
    /// Create detector with custom thresholds
    pub fn new(z_threshold: f32, iqr_multiplier: f32, min_baseline_size: usize) -> Self {
        AnomalyDetector {
            z_threshold,
            iqr_multiplier,
            min_baseline_size,
            rogue_bot_threshold: 3.0,
        }
    }

    /// Create detector with default thresholds (conservative)
    pub fn default() -> Self {
        AnomalyDetector {
            z_threshold: 2.5,      // 2.5 sigma = ~98% normal
            iqr_multiplier: 1.5,   // Standard IQR multiplier
            min_baseline_size: 5,
            rogue_bot_threshold: 2.5,
        }
    }

    /// Create aggressive detector (fewer false negatives, more false positives)
    pub fn aggressive() -> Self {
        AnomalyDetector {
            z_threshold: 2.0,      // 2.0 sigma = ~95% normal
            iqr_multiplier: 1.0,
            min_baseline_size: 3,
            rogue_bot_threshold: 1.5,
        }
    }

    /// Detect anomaly using z-score method
    pub fn detect_zscore(
        &self,
        observation: &Observation,
        baseline: &BaselineStatistics,
    ) -> Result<AnomalyType> {
        if let SensorValue::Temperature { celsius } = observation.value {
            if baseline.std == 0.0 {
                return Ok(AnomalyType::Normal);
            }

            let z_score = (celsius - baseline.mean).abs() / baseline.std;
            if z_score > self.z_threshold {
                Ok(AnomalyType::ZScoreOutlier { z_score })
            } else {
                Ok(AnomalyType::Normal)
            }
        } else {
            Ok(AnomalyType::Normal) // Non-temperature data returns normal
        }
    }

    /// Detect anomaly using IQR method
    pub fn detect_iqr(
        &self,
        observation: &Observation,
        sorted_values: &[f32],
    ) -> Result<AnomalyType> {
        if let SensorValue::Temperature { celsius } = observation.value {
            if sorted_values.len() < 4 {
                return Ok(AnomalyType::Normal); // Need at least 4 points for quartiles
            }

            let q1_idx = sorted_values.len() / 4;
            let q3_idx = (3 * sorted_values.len()) / 4;
            let q1 = sorted_values[q1_idx];
            let q3 = sorted_values[q3_idx];
            let iqr = q3 - q1;

            let lower_bound = q1 - self.iqr_multiplier * iqr;
            let upper_bound = q3 + self.iqr_multiplier * iqr;

            if celsius < lower_bound || celsius > upper_bound {
                let iqr_factor = if celsius < lower_bound {
                    (lower_bound - celsius) / iqr.max(0.1)
                } else {
                    (celsius - upper_bound) / iqr.max(0.1)
                };
                Ok(AnomalyType::IQROutlier { iqr_factor })
            } else {
                Ok(AnomalyType::Normal)
            }
        } else {
            Ok(AnomalyType::Normal)
        }
    }

    /// Detect rogue bot: observation conflicts with consensus
    pub fn detect_rogue_bot(
        &self,
        observation: &Observation,
        other_observations: &[&Observation],
    ) -> Result<AnomalyType> {
        if let SensorValue::Temperature { celsius } = observation.value {
            if other_observations.is_empty() {
                return Ok(AnomalyType::Normal);
            }

            // Get consensus (mean of other observations)
            let other_temps: Vec<f32> = other_observations
                .iter()
                .filter_map(|obs| {
                    if let SensorValue::Temperature { celsius } = obs.value {
                        Some(celsius)
                    } else {
                        None
                    }
                })
                .collect();

            if other_temps.is_empty() {
                return Ok(AnomalyType::Normal);
            }

            let consensus = other_temps.iter().sum::<f32>() / other_temps.len() as f32;
            let deviation = (celsius - consensus).abs();

            if deviation > self.rogue_bot_threshold {
                Ok(AnomalyType::RogueBotSignal {
                    deviation,
                    consensus_count: other_temps.len(),
                })
            } else {
                Ok(AnomalyType::Normal)
            }
        } else {
            Ok(AnomalyType::Normal)
        }
    }

    /// Detect sensor spike: single extreme value
    pub fn detect_spike(
        &self,
        observation: &Observation,
        baseline: &BaselineStatistics,
    ) -> Result<AnomalyType> {
        if let SensorValue::Temperature { celsius } = observation.value {
            if baseline.std == 0.0 {
                return Ok(AnomalyType::Normal);
            }

            let deviation = (celsius - baseline.mean).abs();
            let magnitude = deviation / baseline.std;

            // Spike if >4 sigma AND observation is isolated (high z-score)
            if magnitude > 4.0 {
                Ok(AnomalyType::SensorSpike { magnitude })
            } else {
                Ok(AnomalyType::Normal)
            }
        } else {
            Ok(AnomalyType::Normal)
        }
    }

    /// Detect sensor drift: consistent deviation over sequence
    pub fn detect_drift(&self, observations: &[&Observation]) -> Result<AnomalyType> {
        if observations.len() < 3 {
            return Ok(AnomalyType::Normal);
        }

        let temps: Vec<f32> = observations
            .iter()
            .filter_map(|obs| {
                if let SensorValue::Temperature { celsius } = obs.value {
                    Some(celsius)
                } else {
                    None
                }
            })
            .collect();

        if temps.len() < 3 {
            return Ok(AnomalyType::Normal);
        }

        // Simple linear drift detection: compare first third vs last third
        let split_point = temps.len() / 3;
        let first_third = temps[0..split_point].iter().sum::<f32>() / split_point as f32;
        let last_third_start = 2 * split_point;
        let last_third =
            temps[last_third_start..].iter().sum::<f32>() / (temps.len() - last_third_start) as f32;

        let drift = last_third - first_third;
        let drift_rate = drift / split_point as f32; // Drift per observation

        if drift_rate.abs() > 0.5 {
            Ok(AnomalyType::SensorDrift {
                drift_rate: drift_rate.abs(),
            })
        } else {
            Ok(AnomalyType::Normal)
        }
    }

    /// Combined anomaly detection
    pub fn detect(
        &self,
        observation: &Observation,
        baseline: Option<&BaselineStatistics>,
        historical_data: &[f32],
        other_observations: &[&Observation],
    ) -> Result<AnomalyType> {
        // Priority order: Rogue Bot > Spike > Z-Score > IQR > Normal

        // Check rogue bot first (strongest signal)
        if !other_observations.is_empty() {
            let rogue = self.detect_rogue_bot(observation, other_observations)?;
            if rogue != AnomalyType::Normal {
                return Ok(rogue);
            }
        }

        // Check for sensor spike
        if let Some(baseline) = baseline {
            let spike = self.detect_spike(observation, baseline)?;
            if spike != AnomalyType::Normal {
                return Ok(spike);
            }

            // Z-score check
            let zscore = self.detect_zscore(observation, baseline)?;
            if zscore != AnomalyType::Normal {
                return Ok(zscore);
            }
        }

        // IQR check (doesn't need baseline stats)
        if !historical_data.is_empty() {
            let mut sorted = historical_data.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let iqr = self.detect_iqr(observation, &sorted)?;
            if iqr != AnomalyType::Normal {
                return Ok(iqr);
            }
        }

        Ok(AnomalyType::Normal)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::default()
    }
}

/// Statistics for anomaly rate tracking
#[derive(Clone, Debug)]
pub struct AnomalyStats {
    /// Total observations processed
    pub total: u32,
    /// Anomalies detected
    pub anomaly_count: u32,
    /// Z-score outliers
    pub zscore_count: u32,
    /// IQR outliers
    pub iqr_count: u32,
    /// Rogue bot signals
    pub rogue_bot_count: u32,
    /// Sensor spikes
    pub spike_count: u32,
    /// Sensor drifts
    pub drift_count: u32,
}

impl AnomalyStats {
    /// Create empty stats
    pub fn new() -> Self {
        AnomalyStats {
            total: 0,
            anomaly_count: 0,
            zscore_count: 0,
            iqr_count: 0,
            rogue_bot_count: 0,
            spike_count: 0,
            drift_count: 0,
        }
    }

    /// Update stats with detection result
    pub fn update(&mut self, anomaly: &AnomalyType) {
        self.total += 1;
        match anomaly {
            AnomalyType::Normal => {}
            AnomalyType::ZScoreOutlier { .. } => {
                self.anomaly_count += 1;
                self.zscore_count += 1;
            }
            AnomalyType::IQROutlier { .. } => {
                self.anomaly_count += 1;
                self.iqr_count += 1;
            }
            AnomalyType::RogueBotSignal { .. } => {
                self.anomaly_count += 1;
                self.rogue_bot_count += 1;
            }
            AnomalyType::SensorSpike { .. } => {
                self.anomaly_count += 1;
                self.spike_count += 1;
            }
            AnomalyType::SensorDrift { .. } => {
                self.anomaly_count += 1;
                self.drift_count += 1;
            }
        }
    }

    /// Get anomaly rate (0.0-1.0)
    pub fn anomaly_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.anomaly_count as f32 / self.total as f32
        }
    }
}

impl Default for AnomalyStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType};

    fn create_obs(celsius: f32, robot: &str) -> Observation {
        Observation::new(
            robot.to_string(),
            1000,
            GeoPoint::new(40.7128, -74.0060),
            None,
            SensorType::Thermal,
            SensorValue::Temperature { celsius },
            0.95,
        )
    }

    #[test]
    fn test_detector_creation() {
        let detector = AnomalyDetector::default();
        assert_eq!(detector.z_threshold, 2.5);

        let aggressive = AnomalyDetector::aggressive();
        assert_eq!(aggressive.z_threshold, 2.0);
    }

    #[test]
    fn test_zscore_normal() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 2.0,
            min: 20.0,
            max: 24.0,
            observation_count: 10,
        };

        let obs = create_obs(22.0, "bot_1");
        let result = detector.detect_zscore(&obs, &baseline).unwrap();
        assert_eq!(result, AnomalyType::Normal);
    }

    #[test]
    fn test_zscore_outlier() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 2.0,
            min: 20.0,
            max: 24.0,
            observation_count: 10,
        };

        // 35°C is (35-22)/2 = 6.5 sigma away
        let obs = create_obs(35.0, "bot_1");
        let result = detector.detect_zscore(&obs, &baseline).unwrap();
        assert!(matches!(result, AnomalyType::ZScoreOutlier { .. }));
    }

    #[test]
    fn test_iqr_detection() {
        let detector = AnomalyDetector::default();
        let values = vec![20.0, 21.0, 22.0, 23.0, 24.0];

        let obs = create_obs(50.0, "bot_1"); // Way outside range
        let result = detector.detect_iqr(&obs, &values).unwrap();
        assert!(matches!(result, AnomalyType::IQROutlier { .. }));
    }

    #[test]
    fn test_rogue_bot_detection() {
        let detector = AnomalyDetector::default();

        let suspect = create_obs(10.0, "bot_rogue");
        let consensus_obs1 = create_obs(22.0, "bot_1");
        let consensus_obs2 = create_obs(23.0, "bot_2");
        let consensus_obs3 = create_obs(21.0, "bot_3");

        let result = detector
            .detect_rogue_bot(&suspect, &[&consensus_obs1, &consensus_obs2, &consensus_obs3])
            .unwrap();

        assert!(matches!(result, AnomalyType::RogueBotSignal { .. }));
    }

    #[test]
    fn test_rogue_bot_normal() {
        let detector = AnomalyDetector::default();

        let obs1 = create_obs(22.0, "bot_1");
        let obs2 = create_obs(23.0, "bot_2");
        let obs3 = create_obs(21.5, "bot_3");

        let result = detector.detect_rogue_bot(&obs1, &[&obs2, &obs3]).unwrap();
        assert_eq!(result, AnomalyType::Normal);
    }

    #[test]
    fn test_spike_detection() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 1.0,
            min: 20.0,
            max: 24.0,
            observation_count: 10,
        };

        // 50°C is (50-22)/1 = 28 sigma away
        let obs = create_obs(50.0, "bot_1");
        let result = detector.detect_spike(&obs, &baseline).unwrap();
        assert!(matches!(result, AnomalyType::SensorSpike { .. }));
    }

    #[test]
    fn test_drift_detection() {
        let detector = AnomalyDetector::default();

        // Observations showing upward trend: 20, 21, 22, 23, 24, 25
        let obs_list: Vec<Observation> = vec![20.0, 21.0, 22.0, 23.0, 24.0, 25.0]
            .into_iter()
            .enumerate()
            .map(|(i, temp)| create_obs(temp, &format!("bot_{}", i)))
            .collect();

        let obs_refs: Vec<&Observation> = obs_list.iter().collect();
        let result = detector.detect_drift(&obs_refs).unwrap();
        assert!(matches!(result, AnomalyType::SensorDrift { .. }));
    }

    #[test]
    fn test_anomaly_stats() {
        let mut stats = AnomalyStats::new();

        stats.update(&AnomalyType::Normal);
        stats.update(&AnomalyType::Normal);
        stats.update(&AnomalyType::ZScoreOutlier { z_score: 3.0 });
        stats.update(&AnomalyType::RogueBotSignal {
            deviation: 5.0,
            consensus_count: 3,
        });

        assert_eq!(stats.total, 4);
        assert_eq!(stats.anomaly_count, 2);
        assert_eq!(stats.zscore_count, 1);
        assert_eq!(stats.rogue_bot_count, 1);
        assert!((stats.anomaly_rate() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_non_temperature_observations() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 2.0,
            min: 20.0,
            max: 24.0,
            observation_count: 10,
        };

        let obs = Observation::new(
            "bot_1".to_string(),
            1000,
            GeoPoint::new(40.7128, -74.0060),
            None,
            SensorType::Movement,
            SensorValue::Movement {
                velocity: 5.0,
                heading: 0.0,
            },
            0.95,
        );

        let result = detector.detect_zscore(&obs, &baseline).unwrap();
        assert_eq!(result, AnomalyType::Normal);
    }

    #[test]
    fn test_combined_detection() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 2.0,
            min: 20.0,
            max: 24.0,
            observation_count: 10,
        };

        let obs = create_obs(35.0, "bot_suspect");
        let consensus = create_obs(22.0, "bot_1");

        let result = detector
            .detect(&obs, Some(&baseline), &[20.0, 21.0, 22.0, 23.0, 24.0], &[&consensus])
            .unwrap();

        // Should detect as rogue bot or zscore outlier
        assert!(result != AnomalyType::Normal);
    }

    #[test]
    fn test_zero_variance_baseline() {
        let detector = AnomalyDetector::default();
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 0.0, // Zero variance
            min: 22.0,
            max: 22.0,
            observation_count: 10,
        };

        let obs = create_obs(22.5, "bot_1");
        let result = detector.detect_zscore(&obs, &baseline).unwrap();
        assert_eq!(result, AnomalyType::Normal); // Can't compute z-score
    }
}
