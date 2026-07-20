//! Python wrapper classes for Gaussian Splatting API
//!
//! Provides PyO3 #[pyclass] wrappers for:
//! - PyTerrainGaussian: Core Gaussian splat
//! - PyGaussianCovariance: 3×3 covariance matrix
//! - PyGaussianSplatStore: Global shared world model
//! - PyDynamicObjectSplat: Tracked dynamic objects
//! - PyChangeEvent: Environment change events
//! - PyPathCost: Multi-component path cost
//! - PyObjectState: Object state with position + confidence decay
//! - PyObjectObservation: Bot observation of an object

use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::gaussian_splatting::{
    TerrainGaussian, GaussianCovariance, TerrainType,
    GaussianSplatStore, DynamicObjectSplat, ObjectClass,
    ChangeEvent, ChangeEventType, PathCost,
    ObjectObservation as RsObjectObservation,
};
use crate::exploration::gaussian_frontier_integration::GaussianFrontierScorer as RsGaussianFrontierScorer;
use crate::exploration::gaussian_cache_integration::{
    GaussianCacheManager as RsGaussianCacheManager,
    GaussianTerrainSummary as RsGaussianTerrainSummary,
};
use crate::exploration::multi_bot_sync::{
    FleetCoordinator as RsFleetCoordinator,
    BotObservationMessage as RsBotObservationMessage,
    BotStatus as RsBotStatus,
};
use crate::exploration::frontier::Frontier;
use crate::caching::InvalidationReason;

// ============================================================================
// PyGaussianCovariance Wrapper
// ============================================================================

/// 3×3 covariance matrix for Gaussian splats
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyGaussianCovariance {
    pub matrix: [[f32; 3]; 3],
}

#[pymethods]
impl PyGaussianCovariance {
    /// Create isotropic covariance (σ²I)
    #[staticmethod]
    pub fn isotropic(std_dev: f32) -> Self {
        let cov = GaussianCovariance::isotropic(std_dev);
        PyGaussianCovariance {
            matrix: cov.matrix,
        }
    }

    /// Create diagonal covariance with separate standard deviations
    #[staticmethod]
    pub fn diagonal(sx: f32, sy: f32, sz: f32) -> Self {
        let cov = GaussianCovariance::diagonal(sx, sy, sz);
        PyGaussianCovariance {
            matrix: cov.matrix,
        }
    }

    /// Compute determinant
    pub fn determinant(&self) -> f32 {
        GaussianCovariance { matrix: self.matrix }.determinant()
    }

    /// Compute uncertainty volume: (2π)^(3/2) * √det(Σ)
    pub fn uncertainty_volume(&self) -> f32 {
        GaussianCovariance { matrix: self.matrix }.uncertainty_volume()
    }

    pub fn __repr__(&self) -> String {
        format!("GaussianCovariance(determinant={})", self.determinant())
    }

    /// Convert to dictionary representation
    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("determinant", self.determinant().to_string());
        d.insert("uncertainty_volume", self.uncertainty_volume().to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyTerrainGaussian Wrapper
// ============================================================================

/// Core Gaussian splat: position + covariance + traversability + terrain type
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyTerrainGaussian {
    pub id: String,
    pub position_lat: f64,
    pub position_lon: f64,
    pub position_elev: f64,
    pub traversability: f32,
    pub terrain_type: String,
    pub confidence: f32,
    pub observation_count: u32,
    pub created_at: i64,
    pub last_updated: i64,
    pub source_bots: Vec<String>,
    pub covariance: PyGaussianCovariance,
}

#[pymethods]
impl PyTerrainGaussian {
    /// Create from point observation
    #[staticmethod]
    pub fn from_point_observation(
        lat: f64,
        lon: f64,
        elev: f64,
        bot_id: &str,
        traversability: f32,
        terrain_type: &str,
    ) -> Self {
        let g = TerrainGaussian::from_point_observation([lat, lon, elev], bot_id, traversability);
        PyTerrainGaussian {
            id: g.id.to_string(),
            position_lat: lat,
            position_lon: lon,
            position_elev: elev,
            traversability,
            terrain_type: terrain_type.to_string(),
            confidence: g.confidence,
            observation_count: g.observation_count,
            created_at: g.created_at,
            last_updated: g.last_updated,
            source_bots: g.source_bots,
            covariance: PyGaussianCovariance { matrix: g.covariance.matrix },
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "TerrainGaussian(pos=({:.4},{:.4},{:.1}), terrain={}, conf={:.2})",
            self.position_lat, self.position_lon, self.position_elev,
            self.terrain_type, self.confidence
        )
    }

    #[getter]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[getter]
    pub fn position_lat(&self) -> f64 {
        self.position_lat
    }

    #[getter]
    pub fn position_lon(&self) -> f64 {
        self.position_lon
    }

    #[getter]
    pub fn position_elev(&self) -> f64 {
        self.position_elev
    }

    #[getter]
    pub fn terrain_type(&self) -> String {
        self.terrain_type.clone()
    }

    #[getter]
    pub fn traversability(&self) -> f32 {
        self.traversability
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[getter]
    pub fn source_bots(&self) -> Vec<String> {
        self.source_bots.clone()
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("id", self.id.clone());
        d.insert("position", format!("({:.4}, {:.4}, {:.1})",
            self.position_lat, self.position_lon, self.position_elev));
        d.insert("terrain_type", self.terrain_type.clone());
        d.insert("traversability", self.traversability.to_string());
        d.insert("confidence", self.confidence.to_string());
        d.insert("observation_count", self.observation_count.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyDynamicObjectSplat Wrapper
// ============================================================================

/// Tracked dynamic object (pallet, person, cart, etc.)
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyDynamicObjectSplat {
    pub id: String,
    pub object_class: String,
    pub position_lat: f64,
    pub position_lon: f64,
    pub position_elev: f64,
    pub confidence: f32,
    pub first_seen: i64,
    pub last_seen: i64,
    pub source_bots: Vec<String>,
}

#[pymethods]
impl PyDynamicObjectSplat {
    /// Create new dynamic object splat
    #[staticmethod]
    pub fn new(object_class: &str, lat: f64, lon: f64, elev: f64, bot_id: &str) -> Self {
        let class = ObjectClass::from_str(object_class);
        let obj = DynamicObjectSplat::new(class, [lat, lon, elev], bot_id);
        PyDynamicObjectSplat {
            id: obj.id.to_string(),
            object_class: object_class.to_string(),
            position_lat: lat,
            position_lon: lon,
            position_elev: elev,
            confidence: obj.confidence,
            first_seen: obj.first_seen,
            last_seen: obj.last_seen,
            source_bots: obj.source_bots,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "DynamicObjectSplat(class={}, pos=({:.4},{:.4}), conf={:.2})",
            self.object_class, self.position_lat, self.position_lon, self.confidence
        )
    }

    #[getter]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[getter]
    pub fn position_lat(&self) -> f64 {
        self.position_lat
    }

    #[getter]
    pub fn position_lon(&self) -> f64 {
        self.position_lon
    }

    #[getter]
    pub fn position_elev(&self) -> f64 {
        self.position_elev
    }

    #[getter]
    pub fn object_class(&self) -> String {
        self.object_class.clone()
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[getter]
    pub fn source_bots(&self) -> Vec<String> {
        self.source_bots.clone()
    }

    /// Decay confidence based on time elapsed
    pub fn decayed_confidence(&self, current_time_us: i64) -> f32 {
        let elapsed_ms = (current_time_us - self.last_seen) / 1000;
        if elapsed_ms < 0 {
            return self.confidence;
        }
        // Exponential decay with 2-hour half-life for Movable objects
        let half_life_ms = 2 * 60 * 60 * 1000;
        let decay = 0.5_f32.powf((elapsed_ms as f32) / (half_life_ms as f32));
        (self.confidence * decay).max(0.0)
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("id", self.id.clone());
        d.insert("class", self.object_class.clone());
        d.insert("position", format!("({:.4}, {:.4}, {:.1})",
            self.position_lat, self.position_lon, self.position_elev));
        d.insert("confidence", self.confidence.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyChangeEvent Wrapper
// ============================================================================

/// Environment change event
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyChangeEvent {
    pub id: String,
    pub event_type: String,
    pub detected_by: Vec<String>,
    pub timestamp: i64,
    pub confidence: f32,
}

#[pymethods]
impl PyChangeEvent {
    pub fn __repr__(&self) -> String {
        format!(
            "ChangeEvent(type={}, detected_by={}, conf={:.2})",
            self.event_type,
            self.detected_by.join(","),
            self.confidence
        )
    }

    #[getter]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[getter]
    pub fn event_type(&self) -> String {
        self.event_type.clone()
    }

    #[getter]
    pub fn detected_by(&self) -> Vec<String> {
        self.detected_by.clone()
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("id", self.id.clone());
        d.insert("event_type", self.event_type.clone());
        d.insert("detected_by", self.detected_by.join(","));
        d.insert("timestamp", self.timestamp.to_string());
        d.insert("confidence", self.confidence.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyPathCost Wrapper
// ============================================================================

/// 5-component path cost breakdown
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyPathCost {
    pub distance_cost: f32,
    pub terrain_cost: f32,
    pub elevation_cost: f32,
    pub passage_cost: f32,
    pub uncertainty_cost: f32,
    pub total: f32,
}

#[pymethods]
impl PyPathCost {
    pub fn __repr__(&self) -> String {
        format!(
            "PathCost(total={:.2}, distance={:.2}, terrain={:.2}, elev={:.2}, passage={:.2}, unc={:.2})",
            self.total, self.distance_cost, self.terrain_cost,
            self.elevation_cost, self.passage_cost, self.uncertainty_cost
        )
    }

    #[getter]
    pub fn total(&self) -> f32 {
        self.total
    }

    #[getter]
    pub fn distance_cost(&self) -> f32 {
        self.distance_cost
    }

    #[getter]
    pub fn terrain_cost(&self) -> f32 {
        self.terrain_cost
    }

    #[getter]
    pub fn elevation_cost(&self) -> f32 {
        self.elevation_cost
    }

    #[getter]
    pub fn passage_cost(&self) -> f32 {
        self.passage_cost
    }

    #[getter]
    pub fn uncertainty_cost(&self) -> f32 {
        self.uncertainty_cost
    }

    pub fn breakdown(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("distance", self.distance_cost);
        d.insert("terrain", self.terrain_cost);
        d.insert("elevation", self.elevation_cost);
        d.insert("passage", self.passage_cost);
        d.insert("uncertainty", self.uncertainty_cost);
        d.insert("total", self.total);
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyObjectObservation Wrapper
// ============================================================================

/// Observation of an object from a bot
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyObjectObservation {
    pub object_class: String,
    pub position_lat: f64,
    pub position_lon: f64,
    pub position_elev: f64,
    pub timestamp: i64,
    pub confidence: f32,
}

#[pymethods]
impl PyObjectObservation {
    #[new]
    pub fn new(
        object_class: &str,
        lat: f64,
        lon: f64,
        elev: f64,
        timestamp: i64,
        confidence: f32,
    ) -> Self {
        PyObjectObservation {
            object_class: object_class.to_string(),
            position_lat: lat,
            position_lon: lon,
            position_elev: elev,
            timestamp,
            confidence,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "ObjectObservation(class={}, pos=({:.4},{:.4}), conf={:.2})",
            self.object_class, self.position_lat, self.position_lon, self.confidence
        )
    }

    #[getter]
    pub fn position_lat(&self) -> f64 {
        self.position_lat
    }

    #[getter]
    pub fn position_lon(&self) -> f64 {
        self.position_lon
    }

    #[getter]
    pub fn position_elev(&self) -> f64 {
        self.position_elev
    }

    #[getter]
    pub fn object_class(&self) -> String {
        self.object_class.clone()
    }

    #[getter]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

// ============================================================================
// PyObjectState Wrapper
// ============================================================================

/// Current state of an object from the shared map
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyObjectState {
    pub id: String,
    pub object_class: String,
    pub position_lat: f64,
    pub position_lon: f64,
    pub position_elev: f64,
    pub position_confidence: f32,
    pub is_out_of_sight: bool,
}

#[pymethods]
impl PyObjectState {
    pub fn __repr__(&self) -> String {
        format!(
            "ObjectState(class={}, pos=({:.4},{:.4}), conf={:.2}, oos={})",
            self.object_class, self.position_lat, self.position_lon,
            self.position_confidence, self.is_out_of_sight
        )
    }

    #[getter]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[getter]
    pub fn object_class(&self) -> String {
        self.object_class.clone()
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("id", self.id.clone());
        d.insert("class", self.object_class.clone());
        d.insert("position", format!("({:.4}, {:.4}, {:.1})",
            self.position_lat, self.position_lon, self.position_elev));
        d.insert("confidence", self.position_confidence.to_string());
        d.insert("out_of_sight", self.is_out_of_sight.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyGaussianSplatStore Wrapper
// ============================================================================

/// Global shared probabilistic world model (thread-safe)
#[pyclass]
pub struct PyGaussianSplatStore {
    inner: Arc<RwLock<GaussianSplatStore>>,
}

#[pymethods]
impl PyGaussianSplatStore {
    /// Create new store
    #[new]
    pub fn new() -> Self {
        PyGaussianSplatStore {
            inner: Arc::new(RwLock::new(GaussianSplatStore::new())),
        }
    }

    /// Insert terrain observation
    pub fn insert_splat(
        &self,
        lat: f64,
        lon: f64,
        elev: f64,
        bot_id: &str,
        traversability: f32,
        terrain_type: &str,
    ) -> String {
        let mut store = self.inner.write();
        let mut splat = TerrainGaussian::from_point_observation(
            [lat, lon, elev],
            bot_id,
            traversability,
        );
        splat.terrain_type = TerrainType::from_str(terrain_type);
        let id = splat.id;
        store.insert(splat);
        id.to_string()
    }

    /// Query splats in radius
    pub fn query_radius(&self, lat: f64, lon: f64, elev: f64, radius_m: f64) -> Vec<PyTerrainGaussian> {
        let store = self.inner.read();
        let results = store.query_radius([lat, lon, elev], radius_m);
        results.into_iter().map(|g| PyTerrainGaussian {
            id: g.id.to_string(),
            position_lat: g.position[0],
            position_lon: g.position[1],
            position_elev: g.position[2],
            traversability: g.traversability,
            terrain_type: g.terrain_type.as_str().to_string(),
            confidence: g.confidence,
            observation_count: g.observation_count,
            created_at: g.created_at,
            last_updated: g.last_updated,
            source_bots: g.source_bots.clone(),
            covariance: PyGaussianCovariance { matrix: g.covariance.matrix },
        }).collect()
    }

    /// Get uncertainty at position (0=known, 1=unknown)
    pub fn uncertainty_at(&self, lat: f64, lon: f64, elev: f64) -> f32 {
        let store = self.inner.read();
        store.uncertainty_at([lat, lon, elev])
    }

    /// Ingest bot observation of objects
    pub fn ingest_object_observation(
        &self,
        bot_id: &str,
        objects: Vec<PyObjectObservation>,
    ) -> Vec<PyChangeEvent> {
        let mut store = self.inner.write();
        let rs_obs: Vec<RsObjectObservation> = objects.into_iter().map(|o| {
            RsObjectObservation {
                object_class: ObjectClass::from_str(&o.object_class),
                position: [o.position_lat, o.position_lon, o.position_elev],
                covariance: GaussianCovariance::isotropic(0.5),
                timestamp: o.timestamp,
                confidence: o.confidence,
                dimensions: None,
            }
        }).collect();

        let events = store.ingest_bot_observation(bot_id, rs_obs);
        events.into_iter().map(|e| {
            let event_type = match e.event_type {
                ChangeEventType::ObjectAppeared { .. } => "ObjectAppeared".to_string(),
                ChangeEventType::ObjectMoved { .. } => "ObjectMoved".to_string(),
                ChangeEventType::ObjectDisappeared { .. } => "ObjectDisappeared".to_string(),
                ChangeEventType::PathBlocked { .. } => "PathBlocked".to_string(),
                ChangeEventType::PathCleared { .. } => "PathCleared".to_string(),
                ChangeEventType::AreaCleared { .. } => "AreaCleared".to_string(),
            };
            PyChangeEvent {
                id: e.id.to_string(),
                event_type,
                detected_by: e.detected_by.clone(),
                timestamp: e.timestamp,
                confidence: e.confidence,
            }
        }).collect()
    }

    /// Get objects near position
    pub fn objects_near(&self, lat: f64, lon: f64, elev: f64, radius_m: f64) -> Vec<PyObjectState> {
        let store = self.inner.read();
        let now = chrono::Utc::now().timestamp_micros();
        let objects = store.objects_near([lat, lon, elev], radius_m, now);
        objects.into_iter().map(|obj| {
            PyObjectState {
                id: obj.object.id.to_string(),
                object_class: obj.object.object_class.as_str().to_string(),
                position_lat: obj.object.position[0],
                position_lon: obj.object.position[1],
                position_elev: obj.object.position[2],
                position_confidence: obj.position_confidence,
                is_out_of_sight: obj.is_out_of_sight,
            }
        }).collect()
    }

    /// Compute path cost between two positions
    pub fn path_cost(
        &self,
        from_lat: f64,
        from_lon: f64,
        from_elev: f64,
        to_lat: f64,
        to_lon: f64,
        to_elev: f64,
    ) -> PyPathCost {
        let store = self.inner.read();
        let engine = crate::gaussian_splatting::TraversabilityDistanceEngine::new();
        let cost = engine.path_cost(
            [from_lat, from_lon, from_elev],
            [to_lat, to_lon, to_elev],
            &store,
        );
        PyPathCost {
            distance_cost: cost.distance_cost,
            terrain_cost: cost.terrain_cost,
            elevation_cost: cost.elevation_cost,
            passage_cost: cost.passage_cost,
            uncertainty_cost: cost.uncertainty_cost,
            total: cost.total,
        }
    }

    /// Apply temporal decay to all splats
    pub fn apply_temporal_decay(&self, current_time_us: i64) {
        let mut store = self.inner.write();
        let decay = crate::temporal::DecayFunction::Exponential {
            half_life_ms: 45 * 24 * 60 * 60 * 1000,  // 45-day half-life
        };
        store.apply_temporal_decay(current_time_us, &decay);
    }

    /// Get statistics
    pub fn stats(&self, py: Python) -> PyObject {
        let store = self.inner.read();
        let stats = store.stats();
        let mut d = HashMap::new();
        d.insert("total_splats", stats.total_splats.to_string());
        d.insert("terrain_splats", stats.terrain_splats.to_string());
        d.insert("object_splats", stats.object_splats.to_string());
        d.into_py_dict_bound(py).into()
    }
}

impl Clone for PyGaussianSplatStore {
    fn clone(&self) -> Self {
        PyGaussianSplatStore {
            inner: Arc::clone(&self.inner),
        }
    }
}

// ============================================================================
// PyUnifiedPathCost Wrapper (Traversability + Gaussian Integration)
// ============================================================================

/// Unified path cost integrating traversability graphs and Gaussian uncertainty
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyUnifiedPathCost {
    pub graph_cost: f32,
    pub gaussian_cost: f32,
    pub passage_cost: f32,
    pub total_cost: f32,
    pub confidence: f32,
}

#[pymethods]
impl PyUnifiedPathCost {
    pub fn __repr__(&self) -> String {
        format!(
            "UnifiedPathCost(total={:.2}, graph={:.2}, gaussian={:.2}, passage={:.2}, conf={:.2})",
            self.total_cost, self.graph_cost, self.gaussian_cost, self.passage_cost, self.confidence
        )
    }

    #[getter]
    pub fn graph_cost(&self) -> f32 {
        self.graph_cost
    }

    #[getter]
    pub fn gaussian_cost(&self) -> f32 {
        self.gaussian_cost
    }

    #[getter]
    pub fn passage_cost(&self) -> f32 {
        self.passage_cost
    }

    #[getter]
    pub fn total_cost(&self) -> f32 {
        self.total_cost
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("graph_cost", self.graph_cost.to_string());
        d.insert("gaussian_cost", self.gaussian_cost.to_string());
        d.insert("passage_cost", self.passage_cost.to_string());
        d.insert("total_cost", self.total_cost.to_string());
        d.insert("confidence", self.confidence.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyFrontier Wrapper (from exploration::frontier module)
// ============================================================================

/// Exploration frontier with Gaussian-aware scoring
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyFrontier {
    pub id: String,
    pub location_lat: f64,
    pub location_lon: f64,
    pub location_elev: f32,
    pub expected_information_gain: f32,
    pub exploration_cost: f32,
    pub risk_estimate: f32,
    pub curiosity_score: f32,
    pub priority: f32,
    pub confidence: f32,
}

#[pymethods]
impl PyFrontier {
    #[new]
    pub fn new(id: String, lat: f64, lon: f64, elev: f32) -> Self {
        PyFrontier {
            id,
            location_lat: lat,
            location_lon: lon,
            location_elev: elev,
            expected_information_gain: 0.5,
            exploration_cost: 0.5,
            risk_estimate: 0.5,
            curiosity_score: 0.5,
            priority: 0.5,
            confidence: 0.5,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "Frontier(id={}, pos=({:.4},{:.4}), info_gain={:.2}, priority={:.2}, conf={:.2})",
            self.id, self.location_lat, self.location_lon,
            self.expected_information_gain, self.priority, self.confidence
        )
    }

    #[getter]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[getter]
    pub fn location_lat(&self) -> f64 {
        self.location_lat
    }

    #[getter]
    pub fn location_lon(&self) -> f64 {
        self.location_lon
    }

    #[getter]
    pub fn priority(&self) -> f32 {
        self.priority
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    #[getter]
    pub fn expected_information_gain(&self) -> f32 {
        self.expected_information_gain
    }

    #[getter]
    pub fn curiosity_score(&self) -> f32 {
        self.curiosity_score
    }

    #[getter]
    pub fn exploration_cost(&self) -> f32 {
        self.exploration_cost
    }

    #[getter]
    pub fn risk_estimate(&self) -> f32 {
        self.risk_estimate
    }

    pub fn to_dict(&self, py: Python) -> PyObject {
        let mut d = HashMap::new();
        d.insert("id", self.id.clone());
        d.insert("location", format!("({:.4}, {:.4}, {:.1})",
            self.location_lat, self.location_lon, self.location_elev));
        d.insert("priority", self.priority.to_string());
        d.insert("confidence", self.confidence.to_string());
        d.insert("info_gain", self.expected_information_gain.to_string());
        d.into_py_dict_bound(py).into()
    }
}

// ============================================================================
// PyGaussianFrontierScorer Wrapper
// ============================================================================

/// Frontier scorer using Gaussian uncertainty for prioritization
#[pyclass]
pub struct PyGaussianFrontierScorer {
    inner: RsGaussianFrontierScorer,
}

#[pymethods]
impl PyGaussianFrontierScorer {
    #[new]
    pub fn new() -> Self {
        PyGaussianFrontierScorer {
            inner: RsGaussianFrontierScorer::new(),
        }
    }

    /// Score a single frontier using Gaussian uncertainty
    pub fn score_frontier(
        &self,
        frontier: &mut PyFrontier,
        store: &PyGaussianSplatStore,
    ) {
        // Convert PyFrontier to Frontier
        let mut rs_frontier = Frontier::new(
            frontier.id.clone(),
            (frontier.location_lat, frontier.location_lon, frontier.location_elev),
        );
        rs_frontier.expected_information_gain = frontier.expected_information_gain;
        rs_frontier.exploration_cost = frontier.exploration_cost;
        rs_frontier.risk_estimate = frontier.risk_estimate;
        rs_frontier.curiosity_score = frontier.curiosity_score;
        rs_frontier.confidence = frontier.confidence;

        // Score using Gaussian store
        let store_ref = store.inner.read();
        self.inner.score_frontier_with_gaussian(&mut rs_frontier, &store_ref);
        drop(store_ref);

        // Update PyFrontier with results
        frontier.expected_information_gain = rs_frontier.expected_information_gain;
        frontier.exploration_cost = rs_frontier.exploration_cost;
        frontier.risk_estimate = rs_frontier.risk_estimate;
        frontier.curiosity_score = rs_frontier.curiosity_score;
        frontier.priority = rs_frontier.priority;
        frontier.confidence = rs_frontier.confidence;
    }

    /// Score multiple frontiers and return ranked by priority
    pub fn score_frontiers(
        &self,
        frontiers: Vec<PyFrontier>,
        store: &PyGaussianSplatStore,
    ) -> Vec<PyFrontier> {
        let mut rs_frontiers = Vec::new();

        // Convert all PyFrontier to Frontier
        for py_f in frontiers {
            let mut rs_f = Frontier::new(
                py_f.id.clone(),
                (py_f.location_lat, py_f.location_lon, py_f.location_elev),
            );
            rs_f.expected_information_gain = py_f.expected_information_gain;
            rs_f.exploration_cost = py_f.exploration_cost;
            rs_f.risk_estimate = py_f.risk_estimate;
            rs_f.curiosity_score = py_f.curiosity_score;
            rs_f.confidence = py_f.confidence;
            rs_frontiers.push(rs_f);
        }

        // Score all using Gaussian store
        let store_ref = store.inner.read();
        self.inner.score_frontiers_batch(&mut rs_frontiers, &store_ref);
        drop(store_ref);

        // Rank by priority
        rs_frontiers.sort_by(|a, b| {
            b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Convert back to PyFrontier
        rs_frontiers.into_iter().map(|rs_f| {
            PyFrontier {
                id: rs_f.id,
                location_lat: rs_f.location.0,
                location_lon: rs_f.location.1,
                location_elev: rs_f.location.2,
                expected_information_gain: rs_f.expected_information_gain,
                exploration_cost: rs_f.exploration_cost,
                risk_estimate: rs_f.risk_estimate,
                curiosity_score: rs_f.curiosity_score,
                priority: rs_f.priority,
                confidence: rs_f.confidence,
            }
        }).collect()
    }

    pub fn __repr__(&self) -> String {
        "GaussianFrontierScorer(uncertainty_weight=0.7)".to_string()
    }
}

// ============================================================================
// PyGaussianCacheManager Wrapper (Caching Integration)
// ============================================================================

/// Cache manager for Gaussian Splatting world model
#[pyclass]
pub struct PyGaussianCacheManager {
    inner: RsGaussianCacheManager,
}

#[pymethods]
impl PyGaussianCacheManager {
    #[new]
    pub fn new() -> Self {
        PyGaussianCacheManager {
            inner: RsGaussianCacheManager::new(),
        }
    }

    /// Get terrain summary (Layer 0 - fast, summaries only)
    pub fn get_summary(&self, region_key: &str, store: &PyGaussianSplatStore) -> PyObject {
        let py_store_read = store.inner.read();
        let (summary, confidence) = self.inner.get_summary(region_key, &py_store_read);
        drop(py_store_read);

        Python::with_gil(|py| {
            let mut d = HashMap::new();
            d.insert("avg_traversability", summary.avg_traversability.to_string());
            d.insert("avg_uncertainty", summary.avg_uncertainty.to_string());
            d.insert("splat_count", summary.splat_count.to_string());
            d.insert("confidence", confidence.to_string());
            d.into_py_dict_bound(py).into()
        })
    }

    /// Get observation facts (Layer 1 - details on observations)
    pub fn get_facts(&self, region_key: &str, store: &PyGaussianSplatStore) -> PyObject {
        let py_store_read = store.inner.read();
        let (facts, confidence) = self.inner.get_facts(region_key, &py_store_read);
        drop(py_store_read);

        Python::with_gil(|py| {
            let mut d = HashMap::new();
            d.insert("anomalies", facts.anomalies.len().to_string());
            d.insert("recent_splats", facts.recent_splats.to_string());
            d.insert("confidence", confidence.to_string());
            d.into_py_dict_bound(py).into()
        })
    }

    /// Get detailed context (Layer 2 - full query results)
    pub fn get_context(&self, region_key: &str, store: &PyGaussianSplatStore) -> PyObject {
        let py_store_read = store.inner.read();
        let (context, confidence) = self.inner.get_context(region_key, &py_store_read);
        drop(py_store_read);

        Python::with_gil(|py| {
            let mut d = HashMap::new();
            d.insert("coverage_pct", context.coverage_percentage.to_string());
            d.insert("splat_count", context.splat_count.to_string());
            d.insert("freshness", context.freshness_score.to_string());
            d.insert("confidence", confidence.to_string());
            d.into_py_dict_bound(py).into()
        })
    }

    /// Invalidate cache for region (call after new observations)
    pub fn invalidate_region(&self, region_key: &str) {
        self.inner.invalidate_region(region_key, InvalidationReason::NewObservations);
    }

    /// Get cache statistics
    pub fn stats(&self, py: Python) -> PyObject {
        let stats = self.inner.stats();
        let mut d = HashMap::new();
        d.insert("cache_hits", stats.cache_hits.to_string());
        d.insert("cache_misses", stats.cache_misses.to_string());
        d.insert("invalidations", stats.invalidations.to_string());
        d.insert("splats_cached", stats.splats_cached.to_string());
        d.into_py_dict_bound(py).into()
    }

    pub fn __repr__(&self) -> String {
        "GaussianCacheManager(layers=0-2)".to_string()
    }
}

// ============================================================================
// PyFleetCoordinator Wrapper (Multi-Bot Synchronization)
// ============================================================================

/// Multi-bot observation message
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyBotObservationMessage {
    pub bot_id: String,
    pub timestamp_us: i64,
    pub location_lat: f64,
    pub location_lon: f64,
    pub location_elev: f32,
    pub observation_type: String,
    pub terrain_type: Option<String>,
    pub traversability: f32,
    pub confidence: f32,
}

#[pymethods]
impl PyBotObservationMessage {
    #[new]
    pub fn new(
        bot_id: String,
        lat: f64,
        lon: f64,
        elev: f32,
        traversability: f32,
        confidence: f32,
        terrain_type: Option<String>,
    ) -> Self {
        PyBotObservationMessage {
            bot_id,
            timestamp_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
            location_lat: lat,
            location_lon: lon,
            location_elev: elev,
            observation_type: "terrain".to_string(),
            terrain_type,
            traversability,
            confidence,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "BotObservationMessage(bot={}, pos=({:.4},{:.4}), terrain={:?}, trav={:.2})",
            self.bot_id, self.location_lat, self.location_lon, self.terrain_type, self.traversability
        )
    }

    #[getter]
    pub fn bot_id(&self) -> String {
        self.bot_id.clone()
    }

    #[getter]
    pub fn traversability(&self) -> f32 {
        self.traversability
    }

    #[getter]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

/// Bot status in fleet
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyBotStatus {
    pub bot_id: String,
    pub is_active: bool,
    pub observations_contributed: u64,
    pub location_lat: Option<f64>,
    pub location_lon: Option<f64>,
}

#[pymethods]
impl PyBotStatus {
    pub fn __repr__(&self) -> String {
        format!(
            "BotStatus(bot={}, active={}, observations={})",
            self.bot_id, self.is_active, self.observations_contributed
        )
    }

    #[getter]
    pub fn bot_id(&self) -> String {
        self.bot_id.clone()
    }

    #[getter]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    #[getter]
    pub fn observations_contributed(&self) -> u64 {
        self.observations_contributed
    }
}

/// Fleet coordinator for multi-bot missions
#[pyclass]
pub struct PyFleetCoordinator {
    inner: Arc<RwLock<RsFleetCoordinator>>,
}

#[pymethods]
impl PyFleetCoordinator {
    #[new]
    pub fn new(store: &PyGaussianSplatStore) -> Self {
        let coordinator = RsFleetCoordinator::new(Arc::clone(&store.inner));
        PyFleetCoordinator {
            inner: Arc::new(RwLock::new(coordinator)),
        }
    }

    /// Register bot in fleet
    pub fn register_bot(&self, bot_id: &str) {
        self.inner.read().register_bot(bot_id);
    }

    /// Ingest observation from bot
    pub fn ingest_observation(&self, message: PyBotObservationMessage) -> PyResult<()> {
        let rs_msg = RsBotObservationMessage {
            bot_id: message.bot_id,
            timestamp_us: message.timestamp_us,
            location: (message.location_lat, message.location_lon, message.location_elev),
            observation_type: message.observation_type,
            terrain_type: message.terrain_type,
            traversability: message.traversability,
            confidence: message.confidence,
        };

        self.inner.read().ingest_bot_observation(rs_msg)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    }

    /// Broadcast observation to fleet
    pub fn broadcast_observation(&self, message: PyBotObservationMessage) -> PyResult<u32> {
        let rs_msg = RsBotObservationMessage {
            bot_id: message.bot_id,
            timestamp_us: message.timestamp_us,
            location: (message.location_lat, message.location_lon, message.location_elev),
            observation_type: message.observation_type,
            terrain_type: message.terrain_type,
            traversability: message.traversability,
            confidence: message.confidence,
        };

        self.inner.read().broadcast_observation(rs_msg)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    }

    /// Get fleet synchronization state
    pub fn fleet_state(&self, py: Python) -> PyObject {
        let state = self.inner.read().fleet_state();
        let mut d = HashMap::new();
        d.insert("active_bots", state.active_bots.len().to_string());
        d.insert("pending_observations", state.pending_observations.to_string());
        d.insert("conflicts_resolved", state.conflicts_resolved.to_string());
        d.insert("total_fused", state.total_fused.to_string());
        d.into_py_dict_bound(py).into()
    }

    /// Get bot status
    pub fn get_bot_status(&self, bot_id: &str, py: Python) -> Option<PyObject> {
        let status = self.inner.read().get_bot_status(bot_id)?;
        let mut d = HashMap::new();
        d.insert("bot_id", status.bot_id.clone());
        d.insert("is_active", status.is_active.to_string());
        d.insert("observations", status.observations_contributed.to_string());
        Some(d.into_py_dict_bound(py).into())
    }

    /// Get all bot statuses
    pub fn all_bot_statuses(&self, py: Python) -> PyObject {
        let statuses = self.inner.read().all_bot_statuses();
        let list: Vec<PyObject> = statuses
            .into_iter()
            .map(|status| {
                let mut d = HashMap::new();
                d.insert("bot_id", status.bot_id.clone());
                d.insert("is_active", status.is_active.to_string());
                d.insert("observations", status.observations_contributed.to_string());
                d.into_py_dict_bound(py).into()
            })
            .collect();

        Python::with_gil(|py| {
            let py_list = PyList::new_bound(py, list);
            py_list.into()
        })
    }

    /// Check fleet health (0.0-1.0)
    pub fn fleet_health(&self) -> f32 {
        self.inner.read().fleet_health()
    }

    pub fn __repr__(&self) -> String {
        "FleetCoordinator(multi-bot sync)".to_string()
    }
}

use pyo3::types::PyList;

