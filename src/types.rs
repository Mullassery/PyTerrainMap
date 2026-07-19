//! Core data types for PyTerrainMap
//!
//! Defines the fundamental structures for spatial-temporal observation storage:
//! - GeoPoint: Location (lat, lon)
//! - Observation: Single sensor reading with metadata
//! - SensorType & SensorValue: Typed sensor data
//! - Elevation tracking for 3D spatial awareness

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Geographic point in WGS84 coordinates
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    /// Latitude (-90 to 90)
    pub lat: f64,
    /// Longitude (-180 to 180)
    pub lon: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lon: f64) -> Self {
        GeoPoint { lat, lon }
    }

    /// Check if coordinates are valid WGS84
    pub fn is_valid(&self) -> bool {
        self.lat >= -90.0 && self.lat <= 90.0 && self.lon >= -180.0 && self.lon <= 180.0
    }
}

/// Elevation bucket for 3D spatial indexing
/// Groups observations into vertical ranges (e.g., 0-2m, 2-4m, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElevationBucket {
    /// Minimum elevation in meters
    pub min_m: f32,
    /// Maximum elevation in meters
    pub max_m: f32,
}

// Manually implement Eq and Hash for ElevationBucket using bit representation of f32
impl Eq for ElevationBucket {}

impl std::hash::Hash for ElevationBucket {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.min_m.to_bits().hash(state);
        self.max_m.to_bits().hash(state);
    }
}

impl ElevationBucket {
    /// Create elevation bucket with 1m resolution
    pub fn from_elevation_1m(elevation_asl: f32) -> Self {
        let bucket_size = 1.0;
        let min = (elevation_asl / bucket_size).floor() * bucket_size;
        ElevationBucket {
            min_m: min,
            max_m: min + bucket_size,
        }
    }

    /// Create elevation bucket with 2m resolution
    pub fn from_elevation_2m(elevation_asl: f32) -> Self {
        let bucket_size = 2.0;
        let min = (elevation_asl / bucket_size).floor() * bucket_size;
        ElevationBucket {
            min_m: min,
            max_m: min + bucket_size,
        }
    }

    /// Check if elevation falls within bucket
    pub fn contains(&self, elevation: f32) -> bool {
        elevation >= self.min_m && elevation < self.max_m
    }
}

/// Types of sensors that can report observations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorType {
    Thermal,
    LiDAR,
    Ultrasonic,
    Camera,
    Movement,
}

impl std::fmt::Display for SensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorType::Thermal => write!(f, "thermal"),
            SensorType::LiDAR => write!(f, "lidar"),
            SensorType::Ultrasonic => write!(f, "ultrasonic"),
            SensorType::Camera => write!(f, "camera"),
            SensorType::Movement => write!(f, "movement"),
        }
    }
}

/// Sensor-specific value types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SensorValue {
    /// Temperature in Celsius
    Temperature { celsius: f32 },
    /// LiDAR distance readings in centimeters
    LiDAR { distances_cm: Vec<u16> },
    /// Ultrasonic distance in centimeters
    Ultrasonic { distance_cm: u16 },
    /// Camera detections (objects found in image)
    Camera { detections: Vec<ObjectDetection> },
    /// Movement detected (velocity, heading)
    Movement { velocity: f32, heading: f32 },
}

/// Object detected by camera
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectDetection {
    /// Object class label (e.g., "person", "car", "tree")
    pub class_label: String,
    /// Confidence of detection (0.0-1.0)
    pub confidence: f32,
    /// Bounding box: [x, y, width, height] in pixel coordinates
    pub bbox: [f32; 4],
}

/// Single observation: raw sensor data from one robot at one location/time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier
    pub id: Uuid,

    /// Which robot reported this
    pub robot_id: String,

    /// When was this observation recorded (microseconds since epoch)
    pub timestamp: i64,

    /// Where was the observation taken
    pub location: GeoPoint,

    /// Height above sea level (optional, for 3D indexing)
    pub elevation_asl: Option<f32>,

    /// What type of sensor
    pub sensor_type: SensorType,

    /// The actual sensor reading (type-specific)
    pub value: SensorValue,

    /// Confidence in this observation (0.0-1.0)
    /// Represents sensor/device reliability, not affected by temporal decay
    pub confidence: f32,

    /// Additional metadata (battery level, signal strength, etc.)
    pub metadata: HashMap<String, String>,
}

impl Observation {
    /// Create a new observation with UUID
    pub fn new(
        robot_id: String,
        timestamp: i64,
        location: GeoPoint,
        elevation_asl: Option<f32>,
        sensor_type: SensorType,
        value: SensorValue,
        confidence: f32,
    ) -> Self {
        Observation {
            id: Uuid::new_v4(),
            robot_id,
            timestamp,
            location,
            elevation_asl,
            sensor_type,
            value,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if observation is valid
    pub fn is_valid(&self) -> bool {
        self.location.is_valid()
            && self.confidence >= 0.0
            && self.confidence <= 1.0
            && self.timestamp > 0
    }
}

/// Temperature estimate from fused observations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TemperatureEstimate {
    /// Average temperature in Celsius
    pub celsius: f32,
    /// Variance of readings
    pub variance: f32,
    /// Number of observations used
    pub num_readings: u32,
}

/// Occupancy grid cell for obstacle mapping
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct GridCell {
    pub x: i32,
    pub y: i32,
    /// Probability of occupancy (0.0-1.0)
    pub occupancy: f32,
}

/// Object summary from fused detections
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusedDetection {
    /// Object class label
    pub class_label: String,
    /// Average confidence across observations
    pub avg_confidence: f32,
    /// Number of observations that detected this
    pub num_detections: u32,
    /// Bounding box statistics (mean coordinates and size)
    pub bbox_mean: [f32; 4],
}

/// Fused sensor data at a location
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusedData {
    /// Temperature if available
    pub temperature: Option<TemperatureEstimate>,
    /// Obstacle occupancy grid
    pub obstacle_map: Option<Vec<GridCell>>,
    /// Detected objects
    pub object_detections: Vec<FusedDetection>,
    /// Activity level (0.0-1.0)
    pub activity_level: f32,
}

/// Baseline statistics for anomaly detection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineStatistics {
    pub mean: f32,
    pub std: f32,
    pub min: f32,
    pub max: f32,
    pub observation_count: u32,
}

/// Temporal trend indicator
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalTrend {
    Rising,
    Falling,
    Stable,
    Unknown,
}

/// Result type for operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for PyTerrainMap
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Error {
    InvalidLocation,
    InvalidObservation(String),
    QueryError(String),
    StorageError(String),
    TimeError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLocation => write!(f, "Invalid geographic location"),
            Error::InvalidObservation(msg) => write!(f, "Invalid observation: {}", msg),
            Error::QueryError(msg) => write!(f, "Query error: {}", msg),
            Error::StorageError(msg) => write!(f, "Storage error: {}", msg),
            Error::TimeError(msg) => write!(f, "Time error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::{GeoPoint, ElevationBucket, Observation, SensorType, SensorValue, BaselineStatistics};

    #[test]
    fn test_geopoint_creation() {
        let point = GeoPoint::new(40.7128, -74.0060);
        assert_eq!(point.lat, 40.7128);
        assert_eq!(point.lon, -74.0060);
        assert!(point.is_valid());
    }

    #[test]
    fn test_geopoint_invalid() {
        let invalid = GeoPoint::new(95.0, -74.0060);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_elevation_bucket() {
        let bucket = ElevationBucket::from_elevation_1m(42.7);
        assert_eq!(bucket.min_m, 42.0);
        assert_eq!(bucket.max_m, 43.0);
        assert!(bucket.contains(42.5));
        assert!(!bucket.contains(43.5));
    }

    #[test]
    fn test_observation_creation() {
        let obs = Observation::new(
            "bot_1".to_string(),
            1234567890,
            GeoPoint::new(40.7128, -74.0060),
            Some(100.0),
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        );

        assert_eq!(obs.robot_id, "bot_1");
        assert_eq!(obs.confidence, 0.95);
        assert!(obs.is_valid());
    }

    #[test]
    fn test_observation_with_metadata() {
        let obs = Observation::new(
            "bot_1".to_string(),
            1234567890,
            GeoPoint::new(40.7128, -74.0060),
            None,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        )
        .with_metadata("battery".to_string(), "87%".to_string());

        assert_eq!(obs.metadata.get("battery"), Some(&"87%".to_string()));
    }

    #[test]
    fn test_sensor_type_display() {
        assert_eq!(SensorType::Thermal.to_string(), "thermal");
        assert_eq!(SensorType::LiDAR.to_string(), "lidar");
        assert_eq!(SensorType::Camera.to_string(), "camera");
    }

    #[test]
    fn test_serialization() {
        let obs = Observation::new(
            "bot_1".to_string(),
            1234567890,
            GeoPoint::new(40.7128, -74.0060),
            Some(100.0),
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        );

        let json = serde_json::to_string(&obs).unwrap();
        let deserialized: Observation = serde_json::from_str(&json).unwrap();

        assert_eq!(obs.robot_id, deserialized.robot_id);
        assert_eq!(obs.confidence, deserialized.confidence);
    }

    #[test]
    fn test_baseline_statistics() {
        let baseline = BaselineStatistics {
            mean: 22.0,
            std: 0.5,
            min: 21.0,
            max: 23.0,
            observation_count: 100,
        };

        assert_eq!(baseline.mean, 22.0);
        assert_eq!(baseline.observation_count, 100);
    }
}
