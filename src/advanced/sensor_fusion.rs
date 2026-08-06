//! Multi-Sensor Fusion Framework
//!
//! Phase 5.4: Unified framework for fusing observations from
//! diverse sensors (LiDAR, Radar, Camera, etc).

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Sensor type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SensorType {
    LiDAR,
    Radar,
    Camera,
    Sonar,
    IMU,
    Other(String),
}

impl SensorType {
    /// Get sensor-specific accuracy in meters
    pub fn default_accuracy(&self) -> f32 {
        match self {
            SensorType::LiDAR => 0.02,  // 2cm
            SensorType::Radar => 0.1,   // 10cm
            SensorType::Camera => 0.5,  // 50cm
            SensorType::Sonar => 0.05,  // 5cm
            SensorType::IMU => 1.0,     // 1m
            SensorType::Other(_) => 1.0,
        }
    }

    /// Get sensor range in meters
    pub fn default_range(&self) -> f32 {
        match self {
            SensorType::LiDAR => 100.0,
            SensorType::Radar => 200.0,
            SensorType::Camera => 50.0,
            SensorType::Sonar => 30.0,
            SensorType::IMU => 1000.0,
            SensorType::Other(_) => 100.0,
        }
    }

    /// Get sensor frequency in Hz
    pub fn default_frequency(&self) -> f32 {
        match self {
            SensorType::LiDAR => 10.0,
            SensorType::Radar => 20.0,
            SensorType::Camera => 30.0,
            SensorType::Sonar => 10.0,
            SensorType::IMU => 100.0,
            SensorType::Other(_) => 10.0,
        }
    }
}

/// Sensor identifier
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SensorId(pub u32);

impl SensorId {
    /// Create new sensor ID
    pub fn new(id: u32) -> Self {
        SensorId(id)
    }
}

/// Sensor specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorSpec {
    pub sensor_id: SensorId,
    pub sensor_type: SensorType,
    pub accuracy_m: f32,
    pub range_m: f32,
    pub frequency_hz: f32,
    pub reliability: f32, // 0.0-1.0
}

impl SensorSpec {
    /// Create sensor spec with defaults
    pub fn new(sensor_id: SensorId, sensor_type: SensorType) -> Self {
        SensorSpec {
            sensor_id,
            sensor_type: sensor_type.clone(),
            accuracy_m: sensor_type.default_accuracy(),
            range_m: sensor_type.default_range(),
            frequency_hz: sensor_type.default_frequency(),
            reliability: 0.8,
        }
    }

    /// Get quality score based on accuracy and reliability
    pub fn quality_score(&self) -> f32 {
        // Higher accuracy and reliability = higher quality
        let accuracy_factor = 1.0 / (1.0 + self.accuracy_m);
        (accuracy_factor + self.reliability) / 2.0
    }
}

/// Sensor observation with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorObservation {
    pub sensor_id: SensorId,
    pub point: TemporalPoint,
    pub observation_time_us: i64,
    pub signal_strength: f32, // 0.0-1.0
    pub confidence: f32,      // 0.0-1.0
}

impl SensorObservation {
    /// Create new sensor observation
    pub fn new(
        sensor_id: SensorId,
        point: TemporalPoint,
        signal_strength: f32,
    ) -> Self {
        SensorObservation {
            sensor_id,
            point,
            observation_time_us: point.timestamp,
            signal_strength: signal_strength.clamp(0.0, 1.0),
            confidence: point.quality,
        }
    }

    /// Get effective observation quality
    pub fn effective_quality(&self) -> f32 {
        (self.point.quality + self.signal_strength) / 2.0
    }
}

/// Fused observation from multiple sensors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusedObservation {
    pub location: (f32, f32),
    pub fused_point: TemporalPoint,
    pub contributing_sensors: Vec<SensorId>,
    pub fusion_confidence: f32,
    pub fusion_method: FusionMethod,
}

impl FusedObservation {
    /// Create fused observation
    pub fn new(
        location: (f32, f32),
        fused_point: TemporalPoint,
        contributing_sensors: Vec<SensorId>,
        method: FusionMethod,
    ) -> Self {
        let fusion_confidence = (contributing_sensors.len() as f32 / 4.0).min(1.0);

        FusedObservation {
            location,
            fused_point,
            contributing_sensors,
            fusion_confidence,
            fusion_method: method,
        }
    }
}

/// Fusion method
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FusionMethod {
    Average,
    Weighted,
    Kalman,
    Majority,
}

/// Sensor fusion engine
pub struct SensorFusionEngine {
    pub sensors: HashMap<SensorId, SensorSpec>,
    pub observations: Vec<SensorObservation>,
    pub fused_observations: Vec<FusedObservation>,
}

impl SensorFusionEngine {
    /// Create new fusion engine
    pub fn new() -> Self {
        SensorFusionEngine {
            sensors: HashMap::new(),
            observations: Vec::new(),
            fused_observations: Vec::new(),
        }
    }

    /// Register sensor
    pub fn register_sensor(&mut self, spec: SensorSpec) {
        self.sensors.insert(spec.sensor_id, spec);
    }

    /// Get sensor by ID
    pub fn get_sensor(&self, sensor_id: SensorId) -> Option<&SensorSpec> {
        self.sensors.get(&sensor_id)
    }

    /// Add sensor observation
    pub fn add_observation(&mut self, obs: SensorObservation) {
        self.observations.push(obs);
    }

    /// Fuse observations using averaging
    pub fn fuse_average(&mut self, location_key: &str, observations: &[SensorObservation]) -> Option<FusedObservation> {
        if observations.is_empty() {
            return None;
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        let mut sum_quality = 0.0;
        let mut avg_time = 0i64;
        let mut sensor_ids = Vec::new();

        for obs in observations {
            sum_x += obs.point.x;
            sum_y += obs.point.y;
            sum_z += obs.point.z;
            sum_quality += obs.effective_quality();
            avg_time += obs.point.timestamp;
            sensor_ids.push(obs.sensor_id);
        }

        let count = observations.len() as f32;
        let fused_point = TemporalPoint::new(
            sum_x / count,
            sum_y / count,
            sum_z / count,
            avg_time / observations.len() as i64,
            (sum_quality / count).min(1.0),
        );

        let location = (fused_point.x, fused_point.y);
        let fused = FusedObservation::new(location, fused_point, sensor_ids, FusionMethod::Average);
        self.fused_observations.push(fused.clone());

        Some(fused)
    }

    /// Fuse observations using weighting
    pub fn fuse_weighted(&mut self, location_key: &str, observations: &[SensorObservation]) -> Option<FusedObservation> {
        if observations.is_empty() {
            return None;
        }

        let mut weighted_x = 0.0;
        let mut weighted_y = 0.0;
        let mut weighted_z = 0.0;
        let mut weighted_quality = 0.0;
        let mut weighted_time = 0.0;
        let mut total_weight = 0.0;
        let mut sensor_ids = Vec::new();

        for obs in observations {
            let sensor_spec = self.sensors.get(&obs.sensor_id);
            let weight = sensor_spec.map(|s| s.quality_score()).unwrap_or(0.5) * obs.signal_strength;

            weighted_x += obs.point.x * weight;
            weighted_y += obs.point.y * weight;
            weighted_z += obs.point.z * weight;
            weighted_quality += obs.effective_quality() * weight;
            weighted_time += obs.point.timestamp as f32 * weight;
            total_weight += weight;
            sensor_ids.push(obs.sensor_id);
        }

        if total_weight > 0.0 {
            let fused_point = TemporalPoint::new(
                weighted_x / total_weight,
                weighted_y / total_weight,
                weighted_z / total_weight,
                (weighted_time / total_weight) as i64,
                (weighted_quality / total_weight).min(1.0),
            );

            let location = (fused_point.x, fused_point.y);
            let fused = FusedObservation::new(location, fused_point, sensor_ids, FusionMethod::Weighted);
            self.fused_observations.push(fused.clone());

            Some(fused)
        } else {
            None
        }
    }

    /// Get fusion statistics
    pub fn statistics(&self) -> FusionStatistics {
        FusionStatistics {
            registered_sensors: self.sensors.len() as u32,
            observations_received: self.observations.len() as u32,
            observations_fused: self.fused_observations.len() as u32,
            average_fusion_confidence: if self.fused_observations.is_empty() {
                0.0
            } else {
                self.fused_observations.iter().map(|f| f.fusion_confidence).sum::<f32>()
                    / self.fused_observations.len() as f32
            },
        }
    }
}

impl Default for SensorFusionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Fusion statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusionStatistics {
    pub registered_sensors: u32,
    pub observations_received: u32,
    pub observations_fused: u32,
    pub average_fusion_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_type_defaults() {
        assert!(SensorType::LiDAR.default_accuracy() < SensorType::Camera.default_accuracy());
        assert!(SensorType::LiDAR.default_range() > SensorType::Sonar.default_range());
    }

    #[test]
    fn test_sensor_spec() {
        let spec = SensorSpec::new(SensorId::new(1), SensorType::LiDAR);
        assert_eq!(spec.sensor_id, SensorId::new(1));
        assert!(spec.quality_score() > 0.0);
    }

    #[test]
    fn test_sensor_observation() {
        let sensor_id = SensorId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = SensorObservation::new(sensor_id, point, 0.9);

        assert_eq!(obs.sensor_id, sensor_id);
        assert!(obs.effective_quality() > 0.0);
    }

    #[test]
    fn test_fusion_engine() {
        let mut engine = SensorFusionEngine::new();
        let spec = SensorSpec::new(SensorId::new(1), SensorType::LiDAR);
        engine.register_sensor(spec);

        assert_eq!(engine.sensors.len(), 1);
    }

    #[test]
    fn test_fuse_average() {
        let mut engine = SensorFusionEngine::new();
        let spec1 = SensorSpec::new(SensorId::new(1), SensorType::LiDAR);
        let spec2 = SensorSpec::new(SensorId::new(2), SensorType::Radar);
        engine.register_sensor(spec1);
        engine.register_sensor(spec2);

        let point1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let point2 = TemporalPoint::new(1.1, 2.1, 3.1, 1000, 0.9);

        let obs1 = SensorObservation::new(SensorId::new(1), point1, 0.8);
        let obs2 = SensorObservation::new(SensorId::new(2), point2, 0.9);

        let fused = engine.fuse_average("test", &[obs1, obs2]);
        assert!(fused.is_some());
    }

    #[test]
    fn test_fusion_statistics() {
        let engine = SensorFusionEngine::new();
        let stats = engine.statistics();

        assert_eq!(stats.registered_sensors, 0);
        assert_eq!(stats.observations_received, 0);
    }
}
