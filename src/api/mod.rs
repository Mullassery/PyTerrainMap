//! HTTP REST API for PyTerrainMap
//!
//! Provides endpoints for observation submission, queries, exports,
//! and fleet coordination.

use serde::{Deserialize, Serialize};
use crate::types::{Observation, GeoPoint, SensorType, SensorValue};

/// API error response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub details: Option<String>,
    pub code: u16,
}

impl ApiError {
    pub fn new(code: u16, error: &str) -> Self {
        ApiError {
            error: error.to_string(),
            details: None,
            code,
        }
    }

    pub fn with_details(code: u16, error: &str, details: &str) -> Self {
        ApiError {
            error: error.to_string(),
            details: Some(details.to_string()),
            code,
        }
    }

    pub fn invalid_request(details: &str) -> Self {
        Self::with_details(400, "Invalid Request", details)
    }

    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized")
    }

    pub fn forbidden() -> Self {
        Self::new(403, "Forbidden")
    }

    pub fn not_found() -> Self {
        Self::new(404, "Not Found")
    }

    pub fn internal_error(details: &str) -> Self {
        Self::with_details(500, "Internal Server Error", details)
    }
}

/// API result type
pub type ApiResult<T> = Result<T, ApiError>;

/// Observation submission request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObservationRequest {
    pub robot_id: String,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f32>,
    pub sensor_type: String,
    pub sensor_value: serde_json::Value,
    pub confidence: f32,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl SubmitObservationRequest {
    /// Convert to typed observation
    pub fn to_observation(self) -> ApiResult<Observation> {
        let location = GeoPoint::new(self.latitude, self.longitude);
        if !location.is_valid() {
            return Err(ApiError::invalid_request("Invalid coordinates"));
        }

        let sensor_type = match self.sensor_type.as_str() {
            "thermal" => SensorType::Thermal,
            "lidar" => SensorType::LiDAR,
            "ultrasonic" => SensorType::Ultrasonic,
            "camera" => SensorType::Camera,
            "movement" => SensorType::Movement,
            _ => return Err(ApiError::invalid_request("Unknown sensor type")),
        };

        let value = Self::parse_sensor_value(sensor_type, &self.sensor_value)?;

        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(ApiError::invalid_request("Confidence must be 0.0-1.0"));
        }

        let mut obs = Observation::new(
            self.robot_id,
            self.timestamp,
            location,
            self.elevation,
            sensor_type,
            value,
            self.confidence,
        );

        for (k, v) in self.metadata {
            obs = obs.with_metadata(k, v);
        }

        Ok(obs)
    }

    fn parse_sensor_value(sensor: SensorType, value: &serde_json::Value) -> ApiResult<SensorValue> {
        match sensor {
            SensorType::Thermal => {
                let celsius = value
                    .get("celsius")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ApiError::invalid_request("Missing celsius value"))?;
                Ok(SensorValue::Temperature {
                    celsius: celsius as f32,
                })
            }
            SensorType::LiDAR => {
                let distances = value
                    .get("distances_cm")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ApiError::invalid_request("Missing distances_cm array"))?
                    .iter()
                    .filter_map(|v| v.as_u64())
                    .map(|v| v as u16)
                    .collect();
                Ok(SensorValue::LiDAR { distances_cm: distances })
            }
            SensorType::Ultrasonic => {
                let distance = value
                    .get("distance_cm")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ApiError::invalid_request("Missing distance_cm"))?
                    as u16;
                Ok(SensorValue::Ultrasonic { distance_cm: distance })
            }
            SensorType::Camera => {
                Ok(SensorValue::Camera { detections: vec![] }) // TODO: parse detections
            }
            SensorType::Movement => {
                let velocity = value
                    .get("velocity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ApiError::invalid_request("Missing velocity"))?;
                let heading = value
                    .get("heading")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| ApiError::invalid_request("Missing heading"))?;
                Ok(SensorValue::Movement {
                    velocity: velocity as f32,
                    heading: heading as f32,
                })
            }
        }
    }
}

/// Observation submission response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitObservationResponse {
    pub id: String,
    pub status: String,
    pub timestamp: i64,
}

/// Spatial query request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpatialQueryRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: f32,
    pub elevation_min: Option<f32>,
    pub elevation_max: Option<f32>,
}

/// Temporal query request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalQueryRequest {
    pub from_timestamp: i64,
    pub to_timestamp: i64,
}

/// Spatial-temporal query request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpatialTemporalQueryRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: f32,
    pub from_timestamp: i64,
    pub to_timestamp: i64,
}

/// Query response (observation with optional decay)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryObservationResponse {
    pub id: String,
    pub robot_id: String,
    pub timestamp: i64,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f32>,
    pub sensor_type: String,
    pub confidence: f32,
    pub decayed_confidence: Option<f32>,
}

/// Export request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub radius_m: Option<f32>,
    pub from_timestamp: Option<i64>,
    pub to_timestamp: Option<i64>,
    pub privacy_level: Option<String>, // "none", "balanced", "maximum"
}

/// Export response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportResponse {
    pub format: String,
    pub size_bytes: usize,
    pub observation_count: usize,
    pub timestamp: i64,
}

/// Health check response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub observations_stored: usize,
}

/// Robot status report
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotStatusReport {
    pub robot_id: String,
    pub last_observation: Option<i64>,
    pub total_observations: usize,
    pub sensor_types: Vec<String>,
    pub status: String, // "active", "idle", "error"
}

/// Fleet status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStatus {
    pub total_robots: usize,
    pub active_robots: usize,
    pub total_observations: usize,
    pub coverage_area: Option<String>, // WKT polygon
    pub anomalies_detected: usize,
    pub last_update: i64,
}

/// API configuration
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Max observations per request
    pub max_observations_per_query: usize,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// API key for authentication (if required)
    pub api_key: Option<String>,
}

impl ApiConfig {
    pub fn default() -> Self {
        ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            enable_cors: true,
            max_observations_per_query: 10000,
            request_timeout_secs: 30,
            api_key: None,
        }
    }

    pub fn public() -> Self {
        ApiConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            enable_cors: true,
            max_observations_per_query: 10000,
            request_timeout_secs: 30,
            api_key: None,
        }
    }

    pub fn secure() -> Self {
        ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8443,
            enable_cors: false,
            max_observations_per_query: 1000,
            request_timeout_secs: 10,
            api_key: Some("change_me_in_production".to_string()),
        }
    }
}

/// API endpoint routes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiRoute {
    // Health
    HealthCheck,
    Version,

    // Observations
    SubmitObservation,
    GetObservation,
    ListObservations,

    // Queries
    SpatialQuery,
    TemporalQuery,
    SpatialTemporalQuery,
    NearbyObservations,

    // Fleet
    FleetStatus,
    RobotStatus,

    // Export
    ExportData,

    // Admin
    Stats,
    Ping,
}

impl ApiRoute {
    pub fn path(&self) -> &'static str {
        match self {
            ApiRoute::HealthCheck => "/health",
            ApiRoute::Version => "/version",
            ApiRoute::SubmitObservation => "/observations",
            ApiRoute::GetObservation => "/observations/:id",
            ApiRoute::ListObservations => "/observations/list",
            ApiRoute::SpatialQuery => "/query/spatial",
            ApiRoute::TemporalQuery => "/query/temporal",
            ApiRoute::SpatialTemporalQuery => "/query/spatial-temporal",
            ApiRoute::NearbyObservations => "/query/nearby",
            ApiRoute::FleetStatus => "/fleet/status",
            ApiRoute::RobotStatus => "/fleet/robots/:robot_id",
            ApiRoute::ExportData => "/export",
            ApiRoute::Stats => "/stats",
            ApiRoute::Ping => "/ping",
        }
    }

    pub fn method(&self) -> &'static str {
        match self {
            ApiRoute::SubmitObservation => "POST",
            ApiRoute::GetObservation => "GET",
            ApiRoute::ListObservations => "GET",
            ApiRoute::SpatialQuery => "POST",
            ApiRoute::TemporalQuery => "POST",
            ApiRoute::SpatialTemporalQuery => "POST",
            ApiRoute::NearbyObservations => "GET",
            ApiRoute::FleetStatus => "GET",
            ApiRoute::RobotStatus => "GET",
            ApiRoute::ExportData => "POST",
            ApiRoute::Stats => "GET",
            _ => "GET",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let err = ApiError::new(400, "Bad Request");
        assert_eq!(err.code, 400);
        assert_eq!(err.error, "Bad Request");
    }

    #[test]
    fn test_api_error_with_details() {
        let err = ApiError::with_details(400, "Invalid", "Missing field");
        assert_eq!(err.details, Some("Missing field".to_string()));
    }

    #[test]
    fn test_api_error_helpers() {
        assert_eq!(ApiError::unauthorized().code, 401);
        assert_eq!(ApiError::forbidden().code, 403);
        assert_eq!(ApiError::not_found().code, 404);
    }

    #[test]
    fn test_submission_request_valid() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: Some(100.0),
            sensor_type: "thermal".to_string(),
            sensor_value: serde_json::json!({ "celsius": 22.5 }),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation();
        assert!(obs.is_ok());
    }

    #[test]
    fn test_submission_request_invalid_coordinates() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 95.0, // Invalid
            longitude: -74.0060,
            elevation: None,
            sensor_type: "thermal".to_string(),
            sensor_value: serde_json::json!({ "celsius": 22.5 }),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation();
        assert!(obs.is_err());
    }

    #[test]
    fn test_submission_request_invalid_sensor_type() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: None,
            sensor_type: "unknown".to_string(),
            sensor_value: serde_json::json!({}),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation();
        assert!(obs.is_err());
    }

    #[test]
    fn test_submission_request_invalid_confidence() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: None,
            sensor_type: "thermal".to_string(),
            sensor_value: serde_json::json!({ "celsius": 22.5 }),
            confidence: 1.5, // Invalid: > 1.0
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation();
        assert!(obs.is_err());
    }

    #[test]
    fn test_thermal_sensor_parsing() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: None,
            sensor_type: "thermal".to_string(),
            sensor_value: serde_json::json!({ "celsius": 22.5 }),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation().unwrap();
        assert_eq!(obs.sensor_type, SensorType::Thermal);
    }

    #[test]
    fn test_lidar_sensor_parsing() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: None,
            sensor_type: "lidar".to_string(),
            sensor_value: serde_json::json!({ "distances_cm": [100, 200, 300] }),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation().unwrap();
        assert_eq!(obs.sensor_type, SensorType::LiDAR);
    }

    #[test]
    fn test_movement_sensor_parsing() {
        let req = SubmitObservationRequest {
            robot_id: "bot_1".to_string(),
            timestamp: 1000000,
            latitude: 40.7128,
            longitude: -74.0060,
            elevation: None,
            sensor_type: "movement".to_string(),
            sensor_value: serde_json::json!({ "velocity": 5.0, "heading": 45.0 }),
            confidence: 0.95,
            metadata: std::collections::HashMap::new(),
        };

        let obs = req.to_observation().unwrap();
        assert_eq!(obs.sensor_type, SensorType::Movement);
    }

    #[test]
    fn test_api_route_paths() {
        assert_eq!(ApiRoute::HealthCheck.path(), "/health");
        assert_eq!(ApiRoute::SubmitObservation.path(), "/observations");
        assert_eq!(ApiRoute::SpatialQuery.path(), "/query/spatial");
        assert_eq!(ApiRoute::FleetStatus.path(), "/fleet/status");
    }

    #[test]
    fn test_api_route_methods() {
        assert_eq!(ApiRoute::SubmitObservation.method(), "POST");
        assert_eq!(ApiRoute::HealthCheck.method(), "GET");
        assert_eq!(ApiRoute::SpatialQuery.method(), "POST");
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_config_public() {
        let config = ApiConfig::public();
        assert_eq!(config.host, "0.0.0.0");
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_config_secure() {
        let config = ApiConfig::secure();
        assert_eq!(config.port, 8443);
        assert!(!config.enable_cors);
        assert!(config.api_key.is_some());
    }

    #[test]
    fn test_query_request_serialization() {
        let req = SpatialQueryRequest {
            latitude: 40.7128,
            longitude: -74.0060,
            radius_m: 100.0,
            elevation_min: None,
            elevation_max: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: SpatialQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.latitude, 40.7128);
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse {
            status: "operational".to_string(),
            version: "0.0.1".to_string(),
            uptime_seconds: 3600,
            observations_stored: 1000,
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("operational"));
    }

    #[test]
    fn test_fleet_status() {
        let fleet = FleetStatus {
            total_robots: 5,
            active_robots: 4,
            total_observations: 50000,
            coverage_area: None,
            anomalies_detected: 10,
            last_update: 1000000,
        };

        assert_eq!(fleet.total_robots, 5);
        assert_eq!(fleet.active_robots, 4);
    }
}
