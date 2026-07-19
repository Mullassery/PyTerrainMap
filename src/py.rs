//! PyO3 Python bindings for PyTerrainMap
//!
//! Exposes Rust core to Python via PyO3 extension module.
//! Provides Python classes for spatial intelligence platform:
//! - TerrainMap: Main mapping engine
//! - Observation: Single sensor reading
//! - QueryResult: Results from spatial-temporal queries
//! - GeoPoint: Latitude/longitude coordinate
//! - Region: Geographic bounding box

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use crate::py_api::{PyTerrainMap, PyObservation, PyQueryResult, PyGeoPoint, PyRegion};

/// PyTerrainMap Python module
///
/// Main module for spatial intelligence platform.
/// Core classes: TerrainMap, Observation, QueryResult, GeoPoint, Region
#[pymodule]
fn pyterrain_map(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "0.0.1")?;
    m.add(
        "__doc__",
        "PyTerrainMap: Spatial Intelligence Companion for multi-robot terrain mapping",
    )?;

    // Register main classes
    m.add_class::<PyGeoPoint>()?;
    m.add_class::<PyRegion>()?;
    m.add_class::<PyObservation>()?;
    m.add_class::<PyQueryResult>()?;
    m.add_class::<PyTerrainMap>()?;

    // Persona constants
    m.add("Persona", py_persona_dict(py))?;

    Ok(())
}

/// Create Persona enum as Python dict
fn py_persona_dict(py: Python<'_>) -> PyObject {
    [
        ("MobileRobot", "mobile_robot"),
        ("Drone", "drone"),
        ("Farmer", "farmer"),
        ("DisasterResponse", "disaster_response"),
        ("Vehicle", "vehicle"),
        ("Analyst", "analyst"),
        ("MissionPlanner", "mission_planner"),
    ]
    .into_iter()
    .map(|(k, v)| (k, v))
    .collect::<std::collections::BTreeMap<_, _>>()
    .into_py_dict_bound(py)
    .into()
}
