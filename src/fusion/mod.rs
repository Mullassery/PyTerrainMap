//! Multi-sensor fusion for observation consensus
//!
//! Combines observations from multiple robots/sensors at same location
//! to produce fused estimates with aggregated confidence.

use crate::types::{
    FusedData, FusedDetection, GridCell, ObjectDetection, Observation, Result, SensorType,
    SensorValue, TemperatureEstimate, Error, BaselineStatistics,
};
use std::collections::HashMap;
use std::f32::consts::TAU;

/// Default occupancy grid cell size in meters.
const DEFAULT_GRID_CELL_SIZE_M: f32 = 0.5;
/// LiDAR beams reporting more than this (or zero, meaning "no return") carry
/// no obstacle evidence either way and are skipped.
const MAX_LIDAR_RANGE_CM: u16 = 3000;
/// Log-odds increments for a Bayesian binary occupancy grid (Thrun et al.),
/// corresponding to p(occupied)=0.7 / p(free)=0.3 per ray-cell update.
const LOG_ODDS_OCCUPIED: f32 = 0.847_298; // ln(0.7 / 0.3)
const LOG_ODDS_FREE: f32 = -0.847_298; // ln(0.3 / 0.7)
/// Clamp accumulated log-odds so a cell hammered by many observations
/// doesn't diverge to +/-infinity; keeps probabilities within (~0.0007, ~0.9993).
const LOG_ODDS_CLAMP: f32 = 7.0;

/// Weights for different sensor types in fusion
#[derive(Clone, Debug)]
pub struct SensorWeights {
    /// Weight for thermal sensors (0.0-1.0)
    pub thermal: f32,
    /// Weight for LiDAR sensors
    pub lidar: f32,
    /// Weight for ultrasonic sensors
    pub ultrasonic: f32,
    /// Weight for camera/vision sensors
    pub camera: f32,
    /// Weight for movement sensors
    pub movement: f32,
}

impl SensorWeights {
    /// Default weights (equal for all sensors)
    pub fn default_equal() -> Self {
        SensorWeights {
            thermal: 1.0,
            lidar: 1.0,
            ultrasonic: 1.0,
            camera: 1.0,
            movement: 1.0,
        }
    }

    /// Get weight for a sensor type
    pub fn for_sensor(&self, sensor: SensorType) -> f32 {
        match sensor {
            SensorType::Thermal => self.thermal,
            SensorType::LiDAR => self.lidar,
            SensorType::Ultrasonic => self.ultrasonic,
            SensorType::Camera => self.camera,
            SensorType::Movement => self.movement,
        }
    }
}

/// Sensor fusion engine
pub struct SensorFusion {
    weights: SensorWeights,
    /// Apply temporal quality weighting (default: true)
    temporal_quality_enabled: bool,
    /// Occupancy grid cell size in meters (default: 0.5m)
    grid_cell_size_m: f32,
}

impl SensorFusion {
    /// Create fusion engine with custom sensor weights
    pub fn new(weights: SensorWeights) -> Self {
        SensorFusion {
            weights,
            temporal_quality_enabled: true,
            grid_cell_size_m: DEFAULT_GRID_CELL_SIZE_M,
        }
    }

    /// Create fusion with default equal weights
    pub fn default() -> Self {
        SensorFusion {
            weights: SensorWeights::default_equal(),
            temporal_quality_enabled: true,
            grid_cell_size_m: DEFAULT_GRID_CELL_SIZE_M,
        }
    }

    /// Enable/disable temporal quality weighting
    pub fn set_temporal_quality_enabled(&mut self, enabled: bool) {
        self.temporal_quality_enabled = enabled;
    }

    /// Set the occupancy grid resolution (meters per cell)
    pub fn set_grid_cell_size_m(&mut self, cell_size_m: f32) {
        self.grid_cell_size_m = cell_size_m;
    }

    /// Extract temporal quality from observation
    ///
    /// Quality based on latency (ingestion_time - event_time):
    /// - <100ms: quality 1.0
    /// - >5s: quality 0.3
    /// - Linear interpolation in between
    fn extract_temporal_quality(&self, obs: &Observation) -> f32 {
        let latency_us = obs.temporal.ingestion_time_us.saturating_sub(obs.temporal.event_time_us);
        let latency_ms = (latency_us / 1_000) as i64;

        if latency_ms <= 100 {
            1.0
        } else if latency_ms >= 5000 {
            0.3
        } else {
            // Linear interpolation between 100ms (1.0) and 5000ms (0.3)
            let ratio = (latency_ms - 100) as f32 / (5000.0 - 100.0);
            1.0 - (ratio * 0.7)
        }
    }

    /// Calculate sensor weight with optional temporal quality factor
    ///
    /// Weight = sensor_weight × confidence × temporal_quality
    fn calculate_weight(
        &self,
        sensor_weight: f32,
        confidence: f32,
        temporal_quality: Option<f32>,
    ) -> f32 {
        let mut weight = sensor_weight * confidence;

        if self.temporal_quality_enabled {
            if let Some(tq) = temporal_quality {
                weight *= tq.clamp(0.0, 1.0);
            }
        }

        weight
    }

    /// Fuse multiple observations into single estimate
    pub fn fuse(&self, observations: &[&Observation]) -> Result<FusedData> {
        if observations.is_empty() {
            return Err(Error::InvalidObservation(
                "Cannot fuse empty observation set".to_string(),
            ));
        }

        let temperature = self.fuse_temperature(observations);
        let object_detections = self.fuse_detections(observations);
        let activity_level = self.compute_activity_level(observations);

        let obstacle_map = self.fuse_occupancy_grid(observations);

        Ok(FusedData {
            temperature,
            obstacle_map,
            object_detections,
            activity_level,
        })
    }

    /// Fuse LiDAR and ultrasonic range readings into a local occupancy grid.
    ///
    /// Uses standard Bayesian binary occupancy grid mapping (log-odds
    /// accumulation, Thrun et al.): each range reading ray-traces from the
    /// sensor origin, marking cells it passes through as (weakly) free and
    /// the terminal cell as occupied. Multiple observations reinforce or
    /// contradict each other in the same cells, so noisy single readings
    /// don't dominate.
    ///
    /// `distances_cm` beams are assumed evenly distributed over a full
    /// 360-degree sweep (index 0 at angle 0, wrapping around) — the richest
    /// assumption supportable by `SensorValue::LiDAR`, which doesn't carry
    /// per-beam angle metadata. Ultrasonic sensors are treated as a single
    /// forward-facing (angle 0) beam.
    ///
    /// Returns `None` when no observation carries usable range data (no
    /// LiDAR/Ultrasonic present, or all beams were zero/out-of-range) —
    /// distinct from an empty grid, which would incorrectly claim "surveyed
    /// and found nothing."
    fn fuse_occupancy_grid(&self, observations: &[&Observation]) -> Option<Vec<GridCell>> {
        let mut log_odds: HashMap<(i32, i32), f32> = HashMap::new();
        let mut have_range_data = false;

        for obs in observations {
            let sensor_weight = self.weights.for_sensor(obs.sensor_type);
            let temporal_quality = self.extract_temporal_quality(obs);
            let weight = self
                .calculate_weight(sensor_weight, obs.confidence, Some(temporal_quality))
                .max(0.0);
            if weight == 0.0 {
                continue;
            }

            match &obs.value {
                SensorValue::LiDAR { distances_cm } if !distances_cm.is_empty() => {
                    let angle_step = TAU / distances_cm.len() as f32;
                    for (i, &distance_cm) in distances_cm.iter().enumerate() {
                        if distance_cm == 0 || distance_cm > MAX_LIDAR_RANGE_CM {
                            continue;
                        }
                        have_range_data = true;
                        let angle = i as f32 * angle_step;
                        self.trace_ray(&mut log_odds, angle, distance_cm as f32 / 100.0, weight);
                    }
                }
                SensorValue::Ultrasonic { distance_cm } if *distance_cm > 0 => {
                    have_range_data = true;
                    self.trace_ray(&mut log_odds, 0.0, *distance_cm as f32 / 100.0, weight);
                }
                _ => {}
            }
        }

        if !have_range_data {
            return None;
        }

        let mut grid: Vec<GridCell> = log_odds
            .into_iter()
            .map(|((x, y), l)| GridCell {
                x,
                y,
                occupancy: 1.0 / (1.0 + (-l).exp()),
            })
            .collect();
        grid.sort_by_key(|c| (c.x, c.y));
        Some(grid)
    }

    /// Ray-traces from the sensor origin (grid cell (0,0)) out to a hit at
    /// `distance_m` along `angle_rad`, applying a log-odds "free" update to
    /// every cell passed through and a log-odds "occupied" update to the
    /// terminal cell. `weight` scales this ray's influence relative to
    /// others (sensor weight x confidence x temporal quality).
    fn trace_ray(
        &self,
        log_odds: &mut HashMap<(i32, i32), f32>,
        angle_rad: f32,
        distance_m: f32,
        weight: f32,
    ) {
        let cell_size = self.grid_cell_size_m.max(0.01);
        let steps = (distance_m / cell_size).floor() as i32;

        for step in 0..steps {
            let d = step as f32 * cell_size;
            let cell = Self::to_cell(angle_rad, d, cell_size);
            let entry = log_odds.entry(cell).or_insert(0.0);
            *entry = (*entry + LOG_ODDS_FREE * weight).clamp(-LOG_ODDS_CLAMP, LOG_ODDS_CLAMP);
        }

        let hit_cell = Self::to_cell(angle_rad, distance_m, cell_size);
        let entry = log_odds.entry(hit_cell).or_insert(0.0);
        *entry = (*entry + LOG_ODDS_OCCUPIED * weight).clamp(-LOG_ODDS_CLAMP, LOG_ODDS_CLAMP);
    }

    fn to_cell(angle_rad: f32, distance_m: f32, cell_size: f32) -> (i32, i32) {
        let x = distance_m * angle_rad.cos();
        let y = distance_m * angle_rad.sin();
        ((x / cell_size).round() as i32, (y / cell_size).round() as i32)
    }

    /// Fuse temperature observations with temporal quality weighting
    fn fuse_temperature(&self, observations: &[&Observation]) -> Option<TemperatureEstimate> {
        let mut temps = Vec::new();
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for obs in observations {
            if let SensorValue::Temperature { celsius } = &obs.value {
                let sensor_weight = self.weights.for_sensor(obs.sensor_type);
                let temporal_quality = self.extract_temporal_quality(obs);
                let weight = self.calculate_weight(sensor_weight, obs.confidence, Some(temporal_quality));

                weighted_sum += celsius * weight;
                total_weight += weight;
                temps.push(*celsius);
            }
        }

        if temps.is_empty() {
            return None;
        }

        // Weighted average
        let mean = weighted_sum / total_weight;

        // Variance (unweighted for now - represents observation spread)
        let variance = temps
            .iter()
            .map(|t| (t - mean).powi(2))
            .sum::<f32>()
            / temps.len() as f32;

        Some(TemperatureEstimate {
            celsius: mean,
            variance,
            num_readings: temps.len() as u32,
        })
    }

    /// Fuse object detections with consensus and temporal quality weighting
    fn fuse_detections(&self, observations: &[&Observation]) -> Vec<FusedDetection> {
        // Group detections by class label
        let mut detection_groups: HashMap<String, Vec<(f32, [f32; 4])>> = HashMap::new();
        let mut total_weight_by_class: HashMap<String, f32> = HashMap::new();

        for obs in observations {
            if let SensorValue::Camera { detections } = &obs.value {
                let sensor_weight = self.weights.for_sensor(obs.sensor_type);
                let temporal_quality = self.extract_temporal_quality(obs);

                for detection in detections {
                    let base_weight = sensor_weight * obs.confidence * detection.confidence;
                    let weight = if self.temporal_quality_enabled {
                        base_weight * temporal_quality
                    } else {
                        base_weight
                    };

                    detection_groups
                        .entry(detection.class_label.clone())
                        .or_insert_with(Vec::new)
                        .push((weight, detection.bbox));

                    *total_weight_by_class
                        .entry(detection.class_label.clone())
                        .or_insert(0.0) += weight;
                }
            }
        }

        // Compute fused detections
        let mut fused = Vec::new();
        for (class_label, detections) in detection_groups.into_iter() {
            if detections.is_empty() {
                continue;
            }

            let total_weight = total_weight_by_class[&class_label];
            let num_detections = detections.len() as u32;

            // Weighted average confidence
            let avg_confidence = detections.iter().map(|(w, _)| w).sum::<f32>() / total_weight;

            // Weighted average bbox
            let bbox_mean = {
                let mut bbox = [0.0; 4];
                for (weight, bounding_box) in &detections {
                    let w_norm = weight / total_weight;
                    for i in 0..4 {
                        bbox[i] += bounding_box[i] * w_norm;
                    }
                }
                bbox
            };

            fused.push(FusedDetection {
                class_label,
                avg_confidence,
                num_detections,
                bbox_mean,
            });
        }

        fused
    }

    /// Compute activity level as weighted movement detection rate with temporal quality
    fn compute_activity_level(&self, observations: &[&Observation]) -> f32 {
        let mut total_weight = 0.0;
        let mut weighted_activity = 0.0;

        for obs in observations {
            let sensor_weight = self.weights.for_sensor(obs.sensor_type);
            let temporal_quality = self.extract_temporal_quality(obs);
            let weight = self.calculate_weight(sensor_weight, obs.confidence, Some(temporal_quality));

            total_weight += weight;

            // Movement sensor contributes directly
            if let SensorValue::Movement { velocity, .. } = &obs.value {
                let motion_factor = (velocity / 10.0).min(1.0); // Normalize to 0-1
                weighted_activity += motion_factor * weight;
            }

            // Other sensors contribute based on presence of data
            weighted_activity += 0.1 * sensor_weight * obs.confidence;
        }

        if total_weight == 0.0 {
            0.0
        } else {
            (weighted_activity / total_weight).min(1.0)
        }
    }

    /// Compute baseline statistics from observations
    pub fn baseline_statistics(&self, observations: &[&Observation]) -> Result<Option<BaselineStatistics>> {
        if observations.is_empty() {
            return Ok(None);
        }

        // Collect numeric values (temperatures for now)
        let mut values = Vec::new();
        for obs in observations {
            if let SensorValue::Temperature { celsius } = &obs.value {
                values.push(*celsius);
            }
        }

        if values.is_empty() {
            return Ok(None);
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>()
            / values.len() as f32;
        let std = variance.sqrt();
        let min = values[0];
        let max = values[values.len() - 1];

        Ok(Some(BaselineStatistics {
            mean,
            std,
            min,
            max,
            observation_count: values.len() as u32,
        }))
    }

    /// Check if observation is anomalous using z-score
    pub fn is_anomalous(
        &self,
        observation: &Observation,
        baseline: &BaselineStatistics,
        z_threshold: f32,
    ) -> bool {
        if let SensorValue::Temperature { celsius } = observation.value {
            if baseline.std == 0.0 {
                return false; // Can't compute z-score with zero variance
            }
            let z_score = (celsius - baseline.mean).abs() / baseline.std;
            z_score > z_threshold
        } else {
            false
        }
    }
}

impl Default for SensorFusion {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GeoPoint;

    fn create_obs(
        robot_id: &str,
        sensor: SensorType,
        value: SensorValue,
        confidence: f32,
    ) -> Observation {
        Observation::new(
            robot_id.to_string(),
            1000,
            GeoPoint::new(40.7128, -74.0060),
            None,
            sensor,
            value,
            confidence,
        )
    }

    #[test]
    fn test_sensor_weights() {
        let weights = SensorWeights::default_equal();
        assert_eq!(weights.for_sensor(SensorType::Thermal), 1.0);
        assert_eq!(weights.for_sensor(SensorType::LiDAR), 1.0);
    }

    #[test]
    fn test_fusion_empty() {
        let fusion = SensorFusion::default();
        let result = fusion.fuse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_fuse_single_temperature() {
        let fusion = SensorFusion::default();
        let obs = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        );

        let result = fusion.fuse(&[&obs]).unwrap();
        assert!(result.temperature.is_some());

        let temp = result.temperature.unwrap();
        assert!((temp.celsius - 22.5).abs() < 0.001);
        assert_eq!(temp.num_readings, 1);
    }

    #[test]
    fn test_fuse_multiple_temperatures() {
        let fusion = SensorFusion::default();
        let obs1 = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 20.0 },
            1.0,
        );
        let obs2 = create_obs(
            "bot_2",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 24.0 },
            1.0,
        );

        let result = fusion.fuse(&[&obs1, &obs2]).unwrap();
        let temp = result.temperature.unwrap();

        // Average of 20 and 24
        assert!((temp.celsius - 22.0).abs() < 0.001);
        assert_eq!(temp.num_readings, 2);
    }

    #[test]
    fn test_fuse_weighted_temperatures() {
        let fusion = SensorFusion::default();
        let obs1 = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 20.0 },
            1.0,
        );
        let obs2 = create_obs(
            "bot_2",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 30.0 },
            0.5, // Lower confidence
        );

        let result = fusion.fuse(&[&obs1, &obs2]).unwrap();
        let temp = result.temperature.unwrap();

        // Weighted: (20*1 + 30*0.5) / (1 + 0.5) = 50/1.5 = 33.33...
        let expected = (20.0 * 1.0 + 30.0 * 0.5) / (1.0 + 0.5);
        assert!((temp.celsius - expected).abs() < 0.001);
    }

    #[test]
    fn test_fuse_detections() {
        let fusion = SensorFusion::default();
        let detections = vec![
            ObjectDetection {
                class_label: "person".to_string(),
                confidence: 0.9,
                bbox: [10.0, 20.0, 50.0, 100.0],
            },
            ObjectDetection {
                class_label: "car".to_string(),
                confidence: 0.8,
                bbox: [0.0, 0.0, 100.0, 200.0],
            },
        ];

        let obs = create_obs(
            "bot_1",
            SensorType::Camera,
            SensorValue::Camera { detections },
            0.95,
        );

        let result = fusion.fuse(&[&obs]).unwrap();
        assert_eq!(result.object_detections.len(), 2);

        // Check person detection
        let person = result
            .object_detections
            .iter()
            .find(|d| d.class_label == "person")
            .unwrap();
        assert!(person.avg_confidence > 0.8);
    }

    #[test]
    fn test_fuse_detection_consensus() {
        let fusion = SensorFusion::default();

        // Both robots see a person
        let obs1_detections = vec![ObjectDetection {
            class_label: "person".to_string(),
            confidence: 0.9,
            bbox: [10.0, 20.0, 50.0, 100.0],
        }];

        let obs2_detections = vec![ObjectDetection {
            class_label: "person".to_string(),
            confidence: 0.85,
            bbox: [12.0, 22.0, 48.0, 98.0],
        }];

        let obs1 = create_obs(
            "bot_1",
            SensorType::Camera,
            SensorValue::Camera {
                detections: obs1_detections,
            },
            1.0,
        );

        let obs2 = create_obs(
            "bot_2",
            SensorType::Camera,
            SensorValue::Camera {
                detections: obs2_detections,
            },
            1.0,
        );

        let result = fusion.fuse(&[&obs1, &obs2]).unwrap();
        let person = result
            .object_detections
            .iter()
            .find(|d| d.class_label == "person")
            .unwrap();

        // Multiple detections boost confidence
        assert_eq!(person.num_detections, 2);
        assert!(person.avg_confidence > 0.85);
    }

    #[test]
    fn test_activity_level() {
        let fusion = SensorFusion::default();

        let obs_static = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        );

        let obs_moving = create_obs(
            "bot_2",
            SensorType::Movement,
            SensorValue::Movement {
                velocity: 5.0,
                heading: 0.0,
            },
            0.9,
        );

        let result_static = fusion.fuse(&[&obs_static]).unwrap();
        let result_moving = fusion.fuse(&[&obs_moving]).unwrap();

        // Moving should have higher activity
        assert!(result_moving.activity_level > result_static.activity_level);
    }

    #[test]
    fn test_baseline_statistics() {
        let fusion = SensorFusion::default();
        let obs1 = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 20.0 },
            1.0,
        );
        let obs2 = create_obs(
            "bot_2",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 24.0 },
            1.0,
        );
        let obs3 = create_obs(
            "bot_3",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.0 },
            1.0,
        );

        let baseline = fusion
            .baseline_statistics(&[&obs1, &obs2, &obs3])
            .unwrap()
            .unwrap();

        assert!((baseline.mean - 22.0).abs() < 0.001);
        assert_eq!(baseline.min, 20.0);
        assert_eq!(baseline.max, 24.0);
        assert_eq!(baseline.observation_count, 3);
    }

    #[test]
    fn test_anomaly_detection() {
        let fusion = SensorFusion::default();

        // Create baseline from normal observations
        let obs1 = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.0 },
            1.0,
        );
        let obs2 = create_obs(
            "bot_2",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 23.0 },
            1.0,
        );
        let obs3 = create_obs(
            "bot_3",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 21.0 },
            1.0,
        );

        let baseline = fusion
            .baseline_statistics(&[&obs1, &obs2, &obs3])
            .unwrap()
            .unwrap();

        // Normal observation (within 1 std)
        let normal_obs = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            1.0,
        );
        assert!(!fusion.is_anomalous(&normal_obs, &baseline, 2.0));

        // Anomalous observation (far from mean)
        let anomalous_obs = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 50.0 },
            1.0,
        );
        assert!(fusion.is_anomalous(&anomalous_obs, &baseline, 2.0));
    }

    #[test]
    fn test_mixed_sensor_fusion() {
        let fusion = SensorFusion::default();

        let thermal_obs = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        );

        let camera_obs = create_obs(
            "bot_2",
            SensorType::Camera,
            SensorValue::Camera {
                detections: vec![ObjectDetection {
                    class_label: "person".to_string(),
                    confidence: 0.9,
                    bbox: [0.0, 0.0, 100.0, 200.0],
                }],
            },
            0.9,
        );

        let result = fusion.fuse(&[&thermal_obs, &camera_obs]).unwrap();

        // Should have both temperature and detections
        assert!(result.temperature.is_some());
        assert_eq!(result.object_detections.len(), 1);
        assert!(result.activity_level > 0.0);
    }

    #[test]
    fn test_baseline_empty_observations() {
        let fusion = SensorFusion::default();
        let result = fusion.baseline_statistics(&[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_baseline_non_temperature_observations() {
        let fusion = SensorFusion::default();
        let obs = create_obs(
            "bot_1",
            SensorType::Movement,
            SensorValue::Movement {
                velocity: 5.0,
                heading: 0.0,
            },
            0.9,
        );

        let result = fusion.baseline_statistics(&[&obs]).unwrap();
        assert!(result.is_none()); // No temperature readings
    }

    #[test]
    fn test_occupancy_grid_absent_without_range_sensors() {
        let fusion = SensorFusion::default();
        let obs = create_obs(
            "bot_1",
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 20.0 },
            0.9,
        );
        let result = fusion.fuse(&[&obs]).unwrap();
        assert!(result.obstacle_map.is_none());
    }

    #[test]
    fn test_occupancy_grid_marks_hit_cell_as_occupied() {
        let fusion = SensorFusion::default();
        // 4 beams evenly spaced around 360 degrees; only beam 0 (angle 0,
        // pointing along +x) returns a hit, at 1.0m. The others are "no
        // return" (0 = out of range, ignored).
        let obs = create_obs(
            "bot_1",
            SensorType::LiDAR,
            SensorValue::LiDAR {
                distances_cm: vec![100, 0, 0, 0],
            },
            0.9,
        );

        let result = fusion.fuse(&[&obs]).unwrap();
        let grid = result.obstacle_map.expect("expected an occupancy grid");

        // Cell size defaults to 0.5m, so a 1.0m hit along +x lands at grid cell (2, 0).
        let hit = grid.iter().find(|c| c.x == 2 && c.y == 0);
        assert!(hit.is_some(), "expected a marked cell at the hit location");
        assert!(hit.unwrap().occupancy > 0.5, "hit cell should read as occupied");

        // A cell along the ray before the hit should read as free (< 0.5).
        let along_ray = grid.iter().find(|c| c.x == 1 && c.y == 0);
        assert!(along_ray.is_some(), "expected a marked cell along the ray");
        assert!(along_ray.unwrap().occupancy < 0.5, "cell before the hit should read as free");
    }

    #[test]
    fn test_occupancy_grid_reinforces_from_repeated_observations() {
        let fusion = SensorFusion::default();
        let single = create_obs(
            "bot_1",
            SensorType::LiDAR,
            SensorValue::LiDAR { distances_cm: vec![100] },
            0.9,
        );
        let single_result = fusion.fuse(&[&single]).unwrap();
        let single_grid = single_result.obstacle_map.unwrap();
        let single_occupancy = single_grid.iter().find(|c| c.x == 2 && c.y == 0).unwrap().occupancy;

        let obs_a = create_obs(
            "bot_1",
            SensorType::LiDAR,
            SensorValue::LiDAR { distances_cm: vec![100] },
            0.9,
        );
        let obs_b = create_obs(
            "bot_2",
            SensorType::LiDAR,
            SensorValue::LiDAR { distances_cm: vec![100] },
            0.9,
        );
        let repeated_result = fusion.fuse(&[&obs_a, &obs_b]).unwrap();
        let repeated_grid = repeated_result.obstacle_map.unwrap();
        let repeated_occupancy = repeated_grid.iter().find(|c| c.x == 2 && c.y == 0).unwrap().occupancy;

        // Two independent robots agreeing on the same obstacle should push
        // confidence higher than a single observation.
        assert!(repeated_occupancy > single_occupancy);
    }

    #[test]
    fn test_occupancy_grid_ultrasonic_is_forward_facing_single_beam() {
        let fusion = SensorFusion::default();
        let obs = create_obs(
            "bot_1",
            SensorType::Ultrasonic,
            SensorValue::Ultrasonic { distance_cm: 50 },
            0.9,
        );

        let result = fusion.fuse(&[&obs]).unwrap();
        let grid = result.obstacle_map.expect("expected an occupancy grid");

        // 0.5m forward (+x) at 0.5m cell size lands exactly on cell (1, 0).
        let hit = grid.iter().find(|c| c.x == 1 && c.y == 0);
        assert!(hit.is_some());
        assert!(hit.unwrap().occupancy > 0.5);
    }

    #[test]
    fn test_occupancy_grid_ignores_zero_and_out_of_range_lidar_beams() {
        let fusion = SensorFusion::default();
        let obs = create_obs(
            "bot_1",
            SensorType::LiDAR,
            // All beams are "no return" (0) or beyond MAX_LIDAR_RANGE_CM.
            SensorValue::LiDAR {
                distances_cm: vec![0, 0, 5000, 0],
            },
            0.9,
        );

        let result = fusion.fuse(&[&obs]).unwrap();
        assert!(result.obstacle_map.is_none());
    }
}
