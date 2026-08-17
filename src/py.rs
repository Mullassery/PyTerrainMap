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
use crate::py_api::{
    PyTerrainMap, PyObservation, PyQueryResult, PyGeoPoint, PyRegion,
    PyTerrainAnalysis, PyRisk, PyMobilityAssessment, PyEnvironmentalConditions, PyDataExplanation,
};
use crate::py_gaussian_splatting::{
    PyGaussianCovariance, PyTerrainGaussian, PyDynamicObjectSplat, PyChangeEvent,
    PyPathCost, PyObjectObservation, PyObjectState, PyGaussianSplatStore,
    PyUnifiedPathCost, PyFrontier, PyGaussianFrontierScorer, PyGaussianCacheManager,
    PyBotObservationMessage, PyBotStatus, PyFleetCoordinator,
};
use crate::py_query_functions::{
    analyze_terrain, assess_mobility, detect_changes, explain_field, fuse_observations,
    is_accessible, query_by_sensor,
};
use crate::py_server::{start_server, PyServerHandle};

/// PyTerrainMap Python module
///
/// Main module for spatial intelligence platform.
/// Core classes: TerrainMap, Observation, QueryResult, GeoPoint, Region
#[pymodule]
fn pyterrain_map(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
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

    // Phase 2: Intelligence & Analysis classes
    m.add_class::<PyTerrainAnalysis>()?;
    m.add_class::<PyRisk>()?;
    m.add_class::<PyMobilityAssessment>()?;
    m.add_class::<PyEnvironmentalConditions>()?;
    m.add_class::<PyDataExplanation>()?;

    // Phase 3: Gaussian Splatting probabilistic mapping layer
    m.add_class::<PyGaussianCovariance>()?;
    m.add_class::<PyTerrainGaussian>()?;
    m.add_class::<PyDynamicObjectSplat>()?;
    m.add_class::<PyChangeEvent>()?;
    m.add_class::<PyPathCost>()?;
    m.add_class::<PyObjectObservation>()?;
    m.add_class::<PyObjectState>()?;
    m.add_class::<PyGaussianSplatStore>()?;

    // Phase 4: Unified path planning (Traversability + Gaussian integration)
    m.add_class::<PyUnifiedPathCost>()?;

    // Phase 5: Frontier detection with Gaussian uncertainty
    m.add_class::<PyFrontier>()?;
    m.add_class::<PyGaussianFrontierScorer>()?;

    // Phase 6: Caching integration with Gaussian world model
    m.add_class::<PyGaussianCacheManager>()?;

    // Phase 7: Multi-bot synchronization for fleet coordination
    m.add_class::<PyBotObservationMessage>()?;
    m.add_class::<PyBotStatus>()?;
    m.add_class::<PyFleetCoordinator>()?;

    // High-level query/intelligence functions (analyze_terrain, assess_mobility, etc.)
    m.add_function(wrap_pyfunction!(analyze_terrain, m)?)?;
    m.add_function(wrap_pyfunction!(assess_mobility, m)?)?;
    m.add_function(wrap_pyfunction!(detect_changes, m)?)?;
    m.add_function(wrap_pyfunction!(fuse_observations, m)?)?;
    m.add_function(wrap_pyfunction!(query_by_sensor, m)?)?;
    m.add_function(wrap_pyfunction!(explain_field, m)?)?;
    m.add_function(wrap_pyfunction!(is_accessible, m)?)?;

    // Real HTTP/HTTPS API server (previously type-only, now actually bindable)
    m.add_class::<PyServerHandle>()?;
    m.add_function(wrap_pyfunction!(start_server, m)?)?;

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
