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

    pub fn len(&self) -> usize {
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

// ============================================================================
// TerrainAnalysis Wrapper (Phase 2)
// ============================================================================

/// Terrain intelligence analysis for a location
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTerrainAnalysis {
    pub location: (f64, f64),
    pub timestamp_us: i64,
    pub summary: String,
    pub observations: Vec<String>,
    pub risks: Vec<PyRisk>,
    pub recommendations: std::collections::HashMap<String, Vec<String>>,
    pub confidence: f32,
}

#[pymethods]
impl PyTerrainAnalysis {
    #[new]
    pub fn new(lat: f64, lon: f64) -> Self {
        PyTerrainAnalysis {
            location: (lat, lon),
            timestamp_us: chrono::Utc::now().timestamp_micros(),
            summary: String::new(),
            observations: Vec::new(),
            risks: Vec::new(),
            recommendations: std::collections::HashMap::new(),
            confidence: 0.7,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "TerrainAnalysis(location={:?}, confidence={:.2}, risks={}, obs={})",
            self.location,
            self.confidence,
            self.risks.len(),
            self.observations.len()
        )
    }

    #[getter]
    pub fn location(&self) -> (f64, f64) {
        self.location
    }

    #[getter]
    pub fn summary(&self) -> String {
        self.summary.clone()
    }

    #[setter]
    pub fn set_summary(&mut self, summary: String) {
        self.summary = summary;
    }

    #[getter]
    pub fn observations(&self) -> Vec<String> {
        self.observations.clone()
    }

    #[getter]
    pub fn risks(&self) -> Vec<PyRisk> {
        self.risks.clone()
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[setter]
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    pub fn add_observation(&mut self, obs: String) {
        self.observations.push(obs);
    }

    pub fn add_risk(&mut self, risk: PyRisk) {
        self.risks.push(risk);
    }

    pub fn add_recommendation(&mut self, persona: String, recommendation: String) {
        self.recommendations
            .entry(persona)
            .or_insert_with(Vec::new)
            .push(recommendation);
    }

    pub fn advice_for(&self, persona: &str) -> Vec<String> {
        self.recommendations
            .get(persona)
            .cloned()
            .unwrap_or_default()
    }
}

// ============================================================================
// Risk Wrapper (Phase 2)
// ============================================================================

/// Risk assessment for terrain analysis
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyRisk {
    pub risk_type: String,
    pub severity: f32,
    pub description: String,
    pub affected_personas: Vec<String>,
    pub mitigations: Vec<String>,
}

#[pymethods]
impl PyRisk {
    #[new]
    pub fn new(risk_type: String, severity: f32, description: String) -> Self {
        PyRisk {
            risk_type,
            severity: severity.clamp(0.0, 1.0),
            description,
            affected_personas: Vec::new(),
            mitigations: Vec::new(),
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Risk(type={}, severity={:.2}, label={})",
            self.risk_type,
            self.severity,
            self.severity_label()
        )
    }

    #[getter]
    pub fn risk_type(&self) -> String {
        self.risk_type.clone()
    }

    #[getter]
    pub fn severity(&self) -> f32 {
        self.severity
    }

    #[setter]
    pub fn set_severity(&mut self, severity: f32) {
        self.severity = severity.clamp(0.0, 1.0);
    }

    #[getter]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[getter]
    pub fn affected_personas(&self) -> Vec<String> {
        self.affected_personas.clone()
    }

    #[getter]
    pub fn mitigations(&self) -> Vec<String> {
        self.mitigations.clone()
    }

    pub fn severity_label(&self) -> String {
        match self.severity {
            s if s > 0.8 => "Critical".to_string(),
            s if s > 0.6 => "High".to_string(),
            s if s > 0.4 => "Medium".to_string(),
            _ => "Low".to_string(),
        }
    }

    pub fn affects(mut slf: PyRefMut<'_, Self>, persona: String) -> PyRefMut<'_, Self> {
        slf.affected_personas.push(persona);
        slf
    }

    pub fn with_mitigation(mut slf: PyRefMut<'_, Self>, mitigation: String) -> PyRefMut<'_, Self> {
        slf.mitigations.push(mitigation);
        slf
    }
}

// ============================================================================
// MobilityAssessment Wrapper (Phase 2)
// ============================================================================

/// Robot mobility assessment for terrain
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyMobilityAssessment {
    pub traversable: bool,
    pub difficulty: f32,
    pub hazards: Vec<String>,
    pub recommended_speed_ms: f32,
    pub battery_impact: f32,
    pub time_to_cross_100m_seconds: f32,
}

#[pymethods]
impl PyMobilityAssessment {
    #[new]
    pub fn new() -> Self {
        PyMobilityAssessment {
            traversable: true,
            difficulty: 0.3,
            hazards: Vec::new(),
            recommended_speed_ms: 0.5,
            battery_impact: 1.0,
            time_to_cross_100m_seconds: 200.0,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "MobilityAssessment(difficulty={:.2}, label={}, traversable={})",
            self.difficulty,
            self.difficulty_label(),
            self.traversable
        )
    }

    #[getter]
    pub fn traversable(&self) -> bool {
        self.traversable
    }

    #[setter]
    pub fn set_traversable(&mut self, traversable: bool) {
        self.traversable = traversable;
    }

    #[getter]
    pub fn difficulty(&self) -> f32 {
        self.difficulty
    }

    #[setter]
    pub fn set_difficulty(&mut self, difficulty: f32) {
        self.difficulty = difficulty.clamp(0.0, 1.0);
    }

    #[getter]
    pub fn hazards(&self) -> Vec<String> {
        self.hazards.clone()
    }

    #[getter]
    pub fn recommended_speed_ms(&self) -> f32 {
        self.recommended_speed_ms
    }

    #[setter]
    pub fn set_recommended_speed_ms(&mut self, speed: f32) {
        self.recommended_speed_ms = speed;
    }

    #[getter]
    pub fn battery_impact(&self) -> f32 {
        self.battery_impact
    }

    #[setter]
    pub fn set_battery_impact(&mut self, impact: f32) {
        self.battery_impact = impact;
    }

    #[getter]
    pub fn time_to_cross_100m_seconds(&self) -> f32 {
        self.time_to_cross_100m_seconds
    }

    #[setter]
    pub fn set_time_to_cross_100m_seconds(&mut self, time: f32) {
        self.time_to_cross_100m_seconds = time;
    }

    pub fn difficulty_label(&self) -> String {
        match self.difficulty {
            d if d > 0.8 => "Extremely difficult".to_string(),
            d if d > 0.6 => "Very difficult".to_string(),
            d if d > 0.4 => "Moderately difficult".to_string(),
            d if d > 0.2 => "Slightly difficult".to_string(),
            _ => "Easy".to_string(),
        }
    }

    pub fn add_hazard(&mut self, hazard: String) {
        self.hazards.push(hazard);
    }
}

// ============================================================================
// EnvironmentalConditions Wrapper (Phase 2)
// ============================================================================

/// Environmental conditions (weather + soil)
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyEnvironmentalConditions {
    pub location: (f64, f64),
    pub timestamp_us: i64,
    pub mission_suitability: f32,
}

#[pymethods]
impl PyEnvironmentalConditions {
    #[new]
    pub fn new(lat: f64, lon: f64) -> Self {
        PyEnvironmentalConditions {
            location: (lat, lon),
            timestamp_us: chrono::Utc::now().timestamp_micros(),
            mission_suitability: 0.5,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "EnvironmentalConditions(location={:?}, suitability={:.2})",
            self.location, self.mission_suitability
        )
    }

    #[getter]
    pub fn location(&self) -> (f64, f64) {
        self.location
    }

    #[getter]
    pub fn mission_suitability(&self) -> f32 {
        self.mission_suitability
    }

    #[setter]
    pub fn set_mission_suitability(&mut self, suitability: f32) {
        self.mission_suitability = suitability.clamp(0.0, 1.0);
    }

    pub fn update_suitability(&mut self, score: f32) {
        self.mission_suitability = score.clamp(0.0, 1.0);
    }
}

// ============================================================================
// DataExplanation Wrapper (Phase 2)
// ============================================================================

/// Explanation of a data field for agent introspection
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyDataExplanation {
    pub field: String,
    pub description: String,
    pub applications: Vec<String>,
    pub confidence: f32,
    pub source: String,
    pub units: String,
    pub normal_range: String,
}

#[pymethods]
impl PyDataExplanation {
    #[new]
    pub fn new(
        field: String,
        description: String,
        confidence: f32,
        source: String,
        units: String,
        normal_range: String,
    ) -> Self {
        PyDataExplanation {
            field,
            description,
            applications: Vec::new(),
            confidence: confidence.clamp(0.0, 1.0),
            source,
            units,
            normal_range,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DataExplanation(field={}, confidence={:.2}, source={})",
            self.field, self.confidence, self.source
        )
    }

    #[getter]
    pub fn field(&self) -> String {
        self.field.clone()
    }

    #[getter]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[getter]
    pub fn applications(&self) -> Vec<String> {
        self.applications.clone()
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[getter]
    pub fn source(&self) -> String {
        self.source.clone()
    }

    #[getter]
    pub fn units(&self) -> String {
        self.units.clone()
    }

    #[getter]
    pub fn normal_range(&self) -> String {
        self.normal_range.clone()
    }

    pub fn add_application(&mut self, app: String) {
        self.applications.push(app);
    }

    #[staticmethod]
    pub fn soil_moisture() -> Self {
        PyDataExplanation {
            field: "soil_moisture".to_string(),
            description: "Amount of water retained in the upper soil layer (volumetric percentage)".to_string(),
            applications: vec![
                "Agricultural planning".to_string(),
                "Robot mobility prediction".to_string(),
                "Flood risk assessment".to_string(),
                "Crop health monitoring".to_string(),
            ],
            confidence: 0.75,
            source: "SoilGrids / USDA NRCS".to_string(),
            units: "Volumetric % (0-100)".to_string(),
            normal_range: "20-40% for most crops".to_string(),
        }
    }

    #[staticmethod]
    pub fn temperature() -> Self {
        PyDataExplanation {
            field: "temperature".to_string(),
            description: "Current air temperature at location".to_string(),
            applications: vec![
                "Robot battery performance".to_string(),
                "Sensor calibration".to_string(),
                "Mission feasibility".to_string(),
                "Weather forecasting".to_string(),
            ],
            confidence: 0.95,
            source: "Open-Meteo / NOAA / OpenWeather".to_string(),
            units: "Celsius (°C)".to_string(),
            normal_range: "-20 to +50°C typical".to_string(),
        }
    }

    #[staticmethod]
    pub fn visibility() -> Self {
        PyDataExplanation {
            field: "visibility".to_string(),
            description: "How far visual and LiDAR sensors can effectively see".to_string(),
            applications: vec![
                "Camera/LiDAR range planning".to_string(),
                "Obstacle detection capability".to_string(),
                "Mission safety assessment".to_string(),
                "Sensor confidence adjustment".to_string(),
            ],
            confidence: 0.80,
            source: "Weather station data".to_string(),
            units: "Meters".to_string(),
            normal_range: ">5000m in clear weather, <1000m in fog/rain".to_string(),
        }
    }

    #[staticmethod]
    pub fn slope() -> Self {
        PyDataExplanation {
            field: "slope".to_string(),
            description: "Terrain gradient (steepness)".to_string(),
            applications: vec![
                "Robot traversability assessment".to_string(),
                "Vehicle capability planning".to_string(),
                "Erosion risk".to_string(),
                "Agricultural suitability".to_string(),
            ],
            confidence: 0.9,
            source: "SRTM / USGS DEM".to_string(),
            units: "Degrees from horizontal (0-90)".to_string(),
            normal_range: "0-10° passable, >30° challenging for most robots".to_string(),
        }
    }
}

impl Default for PyTerrainMap {
    fn default() -> Self {
        Self::new()
    }
}
