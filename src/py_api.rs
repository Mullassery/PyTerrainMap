//! Python wrapper classes for PyTerrainMap public API
//!
//! Provides PyO3 #[pyclass] wrappers for:
//! - TerrainMap: Main engine
//! - Observation: Single sensor reading
//! - QueryResult: Results from spatial queries
//! - Region: Geographic bounds
//! - GeoPoint: Lat/lon coordinate

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::types::GeoPoint;

// ============================================================================
// GeoPoint Wrapper
// ============================================================================

/// Geographic point (latitude, longitude)
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyGeoPoint {
    pub lat: f64,
    pub lon: f64,
}

#[pymethods]
impl PyGeoPoint {
    #[new]
    pub fn new(lat: f64, lon: f64) -> Self {
        PyGeoPoint { lat, lon }
    }

    pub fn __repr__(&self) -> String {
        format!("GeoPoint(lat={}, lon={})", self.lat, self.lon)
    }

    pub fn __eq__(&self, other: &PyGeoPoint) -> bool {
        (self.lat - other.lat).abs() < 1e-9 && (self.lon - other.lon).abs() < 1e-9
    }

    pub fn distance_m(&self, other: &PyGeoPoint) -> f32 {
        GeoPoint::new(self.lat, self.lon).distance_m(&GeoPoint::new(other.lat, other.lon))
    }

    #[getter]
    pub fn lat(&self) -> f64 {
        self.lat
    }

    #[getter]
    pub fn lon(&self) -> f64 {
        self.lon
    }
}

// ============================================================================
// Region Wrapper
// ============================================================================

/// Geographic region (bounding box)
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyRegion {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

#[pymethods]
impl PyRegion {
    #[new]
    pub fn new(north: f64, south: f64, east: f64, west: f64) -> Self {
        PyRegion {
            north,
            south,
            east,
            west,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Region(north={}, south={}, east={}, west={})",
            self.north, self.south, self.east, self.west
        )
    }

    pub fn contains(&self, point: &PyGeoPoint) -> bool {
        point.lat >= self.south && point.lat <= self.north
            && point.lon >= self.west && point.lon <= self.east
    }

    pub fn center(&self) -> PyGeoPoint {
        PyGeoPoint {
            lat: (self.north + self.south) / 2.0,
            lon: (self.east + self.west) / 2.0,
        }
    }

    #[staticmethod]
    pub fn world() -> Self {
        PyRegion {
            north: 90.0,
            south: -90.0,
            east: 180.0,
            west: -180.0,
        }
    }
}

// ============================================================================
// Observation Wrapper
// ============================================================================

/// Single sensor observation
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyObservation {
    pub robot_id: String,
    pub timestamp: i64,
    pub location_lat: f64,
    pub location_lon: f64,
    pub sensor_type: String,
    pub value_json: String,  // JSON-encoded sensor value
    pub confidence: f32,
}

#[pymethods]
impl PyObservation {
    #[new]
    pub fn new(
        robot_id: String,
        timestamp: i64,
        lat: f64,
        lon: f64,
        sensor_type: String,
        value_json: String,
        confidence: f32,
    ) -> Self {
        PyObservation {
            robot_id,
            timestamp,
            location_lat: lat,
            location_lon: lon,
            sensor_type,
            value_json,
            confidence,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Observation(robot={}, sensor={}, confidence={:.2})",
            self.robot_id, self.sensor_type, self.confidence
        )
    }

    #[getter]
    pub fn robot_id(&self) -> String {
        self.robot_id.clone()
    }

    #[getter]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[getter]
    pub fn location(&self) -> PyGeoPoint {
        PyGeoPoint {
            lat: self.location_lat,
            lon: self.location_lon,
        }
    }

    #[getter]
    pub fn sensor_type(&self) -> String {
        self.sensor_type.clone()
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[getter]
    pub fn value(&self) -> String {
        self.value_json.clone()
    }
}

// ============================================================================
// QueryResult Wrapper
// ============================================================================

/// Results from a spatial-temporal query
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyQueryResult {
    pub total_count: u64,
    pub observations: Vec<PyObservation>,
    pub avg_confidence: f32,
    pub time_range_seconds: u64,
}

#[pymethods]
impl PyQueryResult {
    pub fn __repr__(&self) -> String {
        format!(
            "QueryResult(count={}, avg_confidence={:.2}%)",
            self.total_count,
            self.avg_confidence * 100.0
        )
    }

    pub fn __len__(&self) -> usize {
        self.observations.len()
    }

    pub fn __getitem__(&self, idx: usize) -> PyResult<PyObservation> {
        self.observations
            .get(idx)
            .cloned()
            .ok_or_else(|| pyo3::exceptions::PyIndexError::new_err("Index out of range"))
    }

    #[getter]
    pub fn count(&self) -> u64 {
        self.total_count
    }

    #[getter]
    pub fn observations(&self) -> Vec<PyObservation> {
        self.observations.clone()
    }

    #[getter]
    pub fn avg_confidence(&self) -> f32 {
        self.avg_confidence
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyObject {
        let mut dict_items = vec![
            ("count", self.total_count.into_py(py)),
            ("avg_confidence", self.avg_confidence.into_py(py)),
            ("observations", self.observations.len().into_py(py)),
        ];
        dict_items.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// TerrainMap Main Engine
// ============================================================================

/// Main terrain mapping engine
#[pyclass]
pub struct PyTerrainMap {
    observations: Arc<RwLock<Vec<PyObservation>>>,
}

#[pymethods]
impl PyTerrainMap {
    #[new]
    pub fn new() -> Self {
        PyTerrainMap {
            observations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Push a single observation
    pub fn push_observation(&self, obs: &PyObservation) -> PyResult<String> {
        let mut obss = self.observations.write();
        obss.push(obs.clone());
        Ok(format!("{}-{}", obs.robot_id, obs.timestamp))
    }

    /// Push multiple observations
    pub fn push_batch(&self, observations: Vec<PyObservation>) -> PyResult<usize> {
        let mut obss = self.observations.write();
        let count = observations.len();
        obss.extend(observations);
        Ok(count)
    }

    /// Query observations in region
    pub fn query(
        &self,
        location: &PyGeoPoint,
        region_radius_km: f64,
        time_window_seconds: i64,
    ) -> PyResult<PyQueryResult> {
        let obss = self.observations.read();

        // Filter by distance (simplified: use lat/lon delta as approximation)
        let lat_delta = region_radius_km / 111.0;
        let lon_delta = region_radius_km / (111.0 * location.lat.abs().cos());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let filtered: Vec<PyObservation> = obss
            .iter()
            .filter(|o| {
                let lat_match = (o.location_lat - location.lat).abs() < lat_delta;
                let lon_match = (o.location_lon - location.lon).abs() < lon_delta;
                let time_match = (now - o.timestamp).abs() < time_window_seconds;
                lat_match && lon_match && time_match
            })
            .cloned()
            .collect();

        let total_count = filtered.len() as u64;
        let avg_confidence = if filtered.is_empty() {
            0.0
        } else {
            filtered.iter().map(|o| o.confidence).sum::<f32>() / filtered.len() as f32
        };

        Ok(PyQueryResult {
            total_count,
            observations: filtered,
            avg_confidence,
            time_range_seconds: time_window_seconds as u64,
        })
    }

    /// Get statistics for a region
    pub fn region_stats(&self, py: Python, region: &PyRegion) -> PyResult<PyObject> {
        let obss = self.observations.read();

        let in_region: Vec<_> = obss
            .iter()
            .filter(|o| region.contains(&PyGeoPoint { lat: o.location_lat, lon: o.location_lon }))
            .collect();

        let stats_vec: Vec<(&str, PyObject)> = vec![
            ("total_observations", in_region.len().into_py(py)),
            (
                "avg_confidence",
                if in_region.is_empty() {
                    0.0.into_py(py)
                } else {
                    (in_region.iter().map(|o| o.confidence).sum::<f32>() / in_region.len() as f32)
                        .into_py(py)
                },
            ),
            ("unique_robots", {
                let mut robots = std::collections::HashSet::new();
                for obs in &in_region {
                    robots.insert(&obs.robot_id);
                }
                robots.len().into_py(py)
            }),
        ];

        Ok(stats_vec.into_py_dict_bound(py).into())
    }

    /// Get all observations
    pub fn observations(&self) -> Vec<PyObservation> {
        self.observations.read().clone()
    }

    /// Clear all observations
    pub fn clear(&self) -> PyResult<()> {
        self.observations.write().clear();
        Ok(())
    }

    /// Get observation count
    pub fn __len__(&self) -> usize {
        self.observations.read().len()
    }

    pub fn __repr__(&self) -> String {
        format!("TerrainMap(observations={})", self.observations.read().len())
    }
}

impl Default for PyTerrainMap {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geopoint_creation() {
        let point = PyGeoPoint::new(40.7128, -74.0060);
        assert_eq!(point.lat, 40.7128);
        assert_eq!(point.lon, -74.0060);
    }

    #[test]
    fn test_geopoint_distance() {
        let p1 = PyGeoPoint::new(0.0, 0.0);
        let p2 = PyGeoPoint::new(0.0, 0.001);
        let distance = p1.distance_m(&p2);
        assert!(distance > 100.0 && distance < 150.0);
    }

    #[test]
    fn test_region_contains() {
        let region = PyRegion::new(10.0, 0.0, 10.0, 0.0);
        let inside = PyGeoPoint::new(5.0, 5.0);
        let outside = PyGeoPoint::new(15.0, 15.0);

        assert!(region.contains(&inside));
        assert!(!region.contains(&outside));
    }

    #[test]
    fn test_observation_creation() {
        let obs = PyObservation::new(
            "robot-1".to_string(),
            1000,
            40.7128,
            -74.0060,
            "thermal".to_string(),
            r#"{"celsius": 25.0}"#.to_string(),
            0.95,
        );

        assert_eq!(obs.robot_id, "robot-1");
        assert_eq!(obs.confidence, 0.95);
    }

    #[test]
    fn test_terrain_map_push_observation() {
        let map = PyTerrainMap::new();
        let obs = PyObservation::new(
            "robot-1".to_string(),
            1000,
            40.7128,
            -74.0060,
            "thermal".to_string(),
            r#"{"celsius": 25.0}"#.to_string(),
            0.95,
        );

        let result = map.push_observation(&obs);
        assert!(result.is_ok());
        assert_eq!(map.observations.read().len(), 1);
    }

    #[test]
    fn test_terrain_map_query() {
        let map = PyTerrainMap::new();

        // Add observations
        for i in 0..5 {
            let obs = PyObservation::new(
                "robot-1".to_string(),
                1000 + (i as i64),
                40.7128 + (i as f64 * 0.001),
                -74.0060,
                "thermal".to_string(),
                r#"{"celsius": 25.0}"#.to_string(),
                0.95,
            );
            let _ = map.push_observation(&obs);
        }

        // Query
        let location = PyGeoPoint::new(40.7128, -74.0060);
        let result = map.query(&location, 10.0, 10000);

        assert!(result.is_ok());
        let qr = result.unwrap();
        assert!(qr.total_count > 0);
    }

    #[test]
    fn test_query_result() {
        let obs = PyObservation::new(
            "robot-1".to_string(),
            1000,
            40.7128,
            -74.0060,
            "thermal".to_string(),
            r#"{"celsius": 25.0}"#.to_string(),
            0.95,
        );

        let result = PyQueryResult {
            total_count: 1,
            observations: vec![obs],
            avg_confidence: 0.95,
            time_range_seconds: 1000,
        };

        assert_eq!(result.total_count, 1);
        assert_eq!(result.len(), 1);
    }
}
