//! PyO3 Python bindings for PyTerrainMap
//!
//! Exposes Rust core to Python via PyO3 extension module.
//! This module is a stub that documents the API surface.
//! Full implementation will expose Rust types as Python classes.

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

/// PyTerrainMap Python module
///
/// Provides access to spatial intelligence platform from Python.
/// Main classes:
/// - TerrainMap: Analysis engine
/// - Persona: Context for analysis
/// - DataExplanation: Self-documenting fields
/// - SpatialReasoningEngine: Multi-source reasoning
#[pymodule]
fn pyterrain_map(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", "0.0.1")?;
    m.add("__doc__", "PyTerrainMap: Spatial Intelligence Companion for multi-robot terrain mapping")?;

    // Persona constants
    m.add("Persona", py_persona_dict(_py))?;

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
