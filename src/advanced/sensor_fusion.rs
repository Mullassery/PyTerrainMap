//! Multi-Sensor Fusion Framework
//!
//! Phase 5.4: Unified framework for fusing observations from
//! diverse sensors (LiDAR, Radar, Camera, etc).

use serde::{Deserialize, Serialize};

/// Sensor type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SensorType {
    LiDAR,
    Radar,
    Camera,
    Sonar,
    IMU,
    Other(String),
}

/// Sensor specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensorSpec {
    pub sensor_type: SensorType,
    pub accuracy_m: f32,
    pub range_m: f32,
    pub frequency_hz: f32,
}

/// Stub: Sensor fusion engine (to be implemented in Phase 5.4)
pub struct SensorFusionEngine {
    pub sensors: Vec<SensorSpec>,
}

impl SensorFusionEngine {
    /// Create new fusion engine
    pub fn new() -> Self {
        SensorFusionEngine {
            sensors: Vec::new(),
        }
    }

    /// Register sensor
    pub fn register_sensor(&mut self, spec: SensorSpec) {
        self.sensors.push(spec);
    }
}

impl Default for SensorFusionEngine {
    fn default() -> Self {
        Self::new()
    }
}
