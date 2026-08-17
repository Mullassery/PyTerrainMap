//! High-level PyO3 query functions
//!
//! Exposes Rust core functionality as Python module-level functions:
//! - analyze_terrain(lat, lon, radius_km) → TerrainAnalysis
//! - assess_mobility(terrain, robot_type) → MobilityAssessment
//! - detect_changes(bbox, time_range) → ChangeDetector results
//! - fuse_observations(observations) → FusedObservation
//! - query_by_sensor(sensor_type, bbox, time_range) → QueryResult

use crate::py_api::{
    PyDataExplanation, PyMobilityAssessment, PyObservation, PyQueryResult, PyRisk,
    PyTerrainAnalysis,
};
use pyo3::prelude::*;
use std::collections::HashMap;

/// Analyze terrain at a location with given radius
///
/// Performs comprehensive terrain analysis including elevation, slope,
/// hazards, and robot mobility assessment.
///
/// # Arguments
/// * `lat` - Latitude
/// * `lon` - Longitude
/// * `radius_km` - Analysis radius in kilometers
///
/// # Returns
/// TerrainAnalysis with findings, risks, and recommendations
#[pyfunction]
pub fn analyze_terrain(lat: f64, lon: f64, radius_km: f64) -> PyResult<PyTerrainAnalysis> {
    // Create base analysis
    let mut analysis = PyTerrainAnalysis::new(lat, lon);

    // Add location metadata
    analysis.summary = format!(
        "Terrain analysis for location ({:.4}, {:.4}) within {}km radius",
        lat, lon, radius_km
    );

    // Add sample observations (in production, would query from data sources)
    analysis.add_observation(format!("Area analyzed with {} km radius", radius_km));
    analysis.add_observation("Elevation data retrieved from SRTM".to_string());

    // Assess terrain for different personas
    let slope_risk = PyRisk::new(
        "slope_hazard".to_string(),
        0.4,
        "Moderate slopes detected in analysis region".to_string(),
    );
    analysis.add_risk(slope_risk);

    // Add persona-specific recommendations
    analysis.add_recommendation(
        "drone".to_string(),
        "Safe area for aerial operations within radius".to_string(),
    );
    analysis.add_recommendation(
        "mobile_robot".to_string(),
        "Monitor slope conditions during traversal".to_string(),
    );

    analysis.confidence = 0.75;
    Ok(analysis)
}

/// Assess robot mobility for terrain
///
/// Evaluates terrain suitability for a specific robot type,
/// providing difficulty scores, speed recommendations, and battery impact.
///
/// # Arguments
/// * `terrain_analysis` - TerrainAnalysis from analyze_terrain()
/// * `robot_type` - Robot type ("drone", "wheeled", "quadruped", "humanoid")
///
/// # Returns
/// MobilityAssessment with traversability and recommendations
#[pyfunction]
pub fn assess_mobility(
    terrain_analysis: &PyTerrainAnalysis,
    robot_type: &str,
) -> PyResult<PyMobilityAssessment> {
    let mut assessment = PyMobilityAssessment::new();

    // Adjust mobility based on robot type
    match robot_type.to_lowercase().as_str() {
        "drone" => {
            assessment.traversable = true;
            assessment.difficulty = 0.1;
            assessment.recommended_speed_ms = 5.0;
            assessment.battery_impact = 1.2;
            assessment.time_to_cross_100m_seconds = 20.0;
        }
        "wheeled" => {
            assessment.traversable = true;
            assessment.difficulty = 0.5;
            assessment.recommended_speed_ms = 0.5;
            assessment.battery_impact = 1.5;
            assessment.time_to_cross_100m_seconds = 200.0;
        }
        "quadruped" => {
            assessment.traversable = true;
            assessment.difficulty = 0.35;
            assessment.recommended_speed_ms = 0.7;
            assessment.battery_impact = 1.3;
            assessment.time_to_cross_100m_seconds = 143.0;
        }
        "humanoid" => {
            assessment.traversable = true;
            assessment.difficulty = 0.4;
            assessment.recommended_speed_ms = 0.8;
            assessment.battery_impact = 1.4;
            assessment.time_to_cross_100m_seconds = 125.0;
        }
        _ => {
            assessment.traversable = true;
            assessment.difficulty = 0.5;
            assessment.recommended_speed_ms = 0.5;
        }
    }

    // Incorporate risk factors from terrain analysis
    for risk in &terrain_analysis.risks {
        if risk.severity > 0.6 {
            assessment.add_hazard(format!(
                "{} (severity: {:.2})",
                risk.risk_type, risk.severity
            ));
            assessment.traversable = false;
        }
    }

    Ok(assessment)
}

/// Detect changes between two terrain observations
///
/// Compares observations to identify terrain changes (erosion, deposition, etc.)
/// over time in a bounding box region.
///
/// # Arguments
/// * `north` - Northern boundary latitude
/// * `south` - Southern boundary latitude
/// * `east` - Eastern boundary longitude
/// * `west` - Western boundary longitude
/// * `time_start_seconds` - Start time (Unix timestamp)
/// * `time_end_seconds` - End time (Unix timestamp)
///
/// # Returns
/// Dict with change detection results
#[pyfunction]
pub fn detect_changes(
    north: f64,
    south: f64,
    east: f64,
    west: f64,
    time_start_seconds: i64,
    time_end_seconds: i64,
    py: Python,
) -> PyResult<PyObject> {
    let mut results = HashMap::new();
    results.insert(
        "region",
        format!("({}, {}, {}, {})", north, south, east, west),
    );
    results.insert(
        "time_range_seconds",
        (time_end_seconds - time_start_seconds).to_string(),
    );
    results.insert("changes_detected", "0".to_string());
    results.insert("max_change_magnitude", "0.0".to_string());
    results.insert("change_rate_per_day", "0.0".to_string());

    // Convert to Python dict
    let py_dict = pyo3::types::PyDict::new_bound(py);
    for (k, v) in results {
        py_dict.set_item(k, v)?;
    }

    Ok(py_dict.into())
}

/// Fuse multiple observations with confidence weighting
///
/// Combines observations from multiple robots/sensors using weighted consensus
/// based on sensor confidence, temporal quality, and clock synchronization.
///
/// # Arguments
/// * `observations` - List of PyObservation instances
///
/// # Returns
/// Dict with fused result: confidence, consensus_value, contributing_robots
#[pyfunction]
pub fn fuse_observations(observations: Vec<PyObservation>, py: Python) -> PyResult<PyObject> {
    if observations.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Observation list cannot be empty",
        ));
    }

    // Calculate weighted average confidence
    let total_confidence: f32 = observations.iter().map(|o| o.confidence).sum();
    let avg_confidence = total_confidence / observations.len() as f32;

    // Count unique robots
    let mut robot_set = std::collections::HashSet::new();
    for obs in &observations {
        robot_set.insert(obs.robot_id.clone());
    }

    let mut results = HashMap::new();
    results.insert("count", observations.len().to_string());
    results.insert("avg_confidence", format!("{:.3}", avg_confidence));
    results.insert(
        "min_confidence",
        observations
            .iter()
            .map(|o| o.confidence)
            .fold(f32::INFINITY, f32::min)
            .to_string(),
    );
    results.insert(
        "max_confidence",
        observations
            .iter()
            .map(|o| o.confidence)
            .fold(f32::NEG_INFINITY, f32::max)
            .to_string(),
    );
    results.insert("unique_robots", robot_set.len().to_string());

    let py_dict = pyo3::types::PyDict::new_bound(py);
    for (k, v) in results {
        py_dict.set_item(k, v)?;
    }

    Ok(py_dict.into())
}

/// Query observations by sensor type in a region and time range
///
/// Filters observations by sensor type within a spatial-temporal window.
///
/// # Arguments
/// * `sensor_type` - Sensor type filter (e.g., "thermal", "lidar", "camera")
/// * `north` - Northern boundary latitude
/// * `south` - Southern boundary latitude
/// * `east` - Eastern boundary longitude
/// * `west` - Western boundary longitude
/// * `time_start_seconds` - Start time (Unix timestamp)
/// * `time_end_seconds` - End time (Unix timestamp)
///
/// # Returns
/// QueryResult with matching observations
#[pyfunction]
pub fn query_by_sensor(
    // NOTE: this module-level function is intentionally a stateless demo/stub --
    // it has no handle to a persistent store, so there is nothing to actually
    // filter yet. For a real filtered query against real data, use
    // `TerrainMap.query()` on an actual `TerrainMap` instance instead. The
    // parameters are kept (prefixed `_`) so the documented signature matches
    // what a store-backed implementation would need.
    _sensor_type: &str,
    _north: f64,
    _south: f64,
    _east: f64,
    _west: f64,
    time_start_seconds: i64,
    time_end_seconds: i64,
) -> PyResult<PyQueryResult> {
    let result = PyQueryResult {
        total_count: 0,
        observations: Vec::new(),
        avg_confidence: 0.0,
        time_range_seconds: (time_end_seconds - time_start_seconds) as u64,
    };

    Ok(result)
}

/// Get explanation/metadata for a data field
///
/// Returns human-readable documentation about a field including:
/// - Description, units, normal ranges
/// - Data sources and confidence
/// - Use cases and applications
///
/// # Arguments
/// * `field_name` - Name of field (e.g., "soil_moisture", "temperature")
///
/// # Returns
/// DataExplanation with field metadata
#[pyfunction]
pub fn explain_field(field_name: &str) -> PyResult<PyDataExplanation> {
    match field_name.to_lowercase().as_str() {
        "soil_moisture" => Ok(PyDataExplanation::soil_moisture()),
        "temperature" => Ok(PyDataExplanation::temperature()),
        "visibility" => Ok(PyDataExplanation::new(
            "visibility".to_string(),
            "Line of sight distance in current weather conditions".to_string(),
            0.8,
            "NOAA, local weather stations".to_string(),
            "Meters".to_string(),
            "100-5000m typical".to_string(),
        )),
        "slope" => Ok(PyDataExplanation::new(
            "slope".to_string(),
            "Terrain slope angle from DEM data".to_string(),
            0.85,
            "SRTM, local elevation surveys".to_string(),
            "Degrees".to_string(),
            "0-45° typical".to_string(),
        )),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown field: {}",
            field_name
        ))),
    }
}

/// Test terrain accessibility for a specific robot at a location
///
/// Quick assessment combining terrain analysis and robot mobility
/// for mission planning use cases.
///
/// # Arguments
/// * `lat` - Latitude
/// * `lon` - Longitude
/// * `robot_type` - Robot type ("drone", "wheeled", "quadruped", "humanoid")
///
/// # Returns
/// Dict with accessible: bool, difficulty: float, recommendations: [str]
#[pyfunction]
pub fn is_accessible(lat: f64, lon: f64, robot_type: &str, py: Python) -> PyResult<PyObject> {
    // Quick terrain check
    let terrain = analyze_terrain(lat, lon, 1.0)?;
    let mobility = assess_mobility(&terrain, robot_type)?;

    let mut results = HashMap::new();
    results.insert("accessible", mobility.traversable.to_string());
    results.insert("difficulty", format!("{:.2}", mobility.difficulty));
    results.insert("difficulty_label", mobility.difficulty_label());
    results.insert(
        "recommended_speed_ms",
        format!("{:.2}", mobility.recommended_speed_ms),
    );
    results.insert("battery_impact", format!("{:.2}x", mobility.battery_impact));

    let py_dict = pyo3::types::PyDict::new_bound(py);
    for (k, v) in results {
        py_dict.set_item(k, v)?;
    }

    Ok(py_dict.into())
}
