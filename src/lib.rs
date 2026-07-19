//! PyPanorama: Collaborative Spatial Intelligence Platform
//!
//! This is the Rust core of PyPanorama. It provides:
//! - 3D spatial indexing (H3 + elevation)
//! - Temporal observation storage and querying
//! - Multi-sensor fusion
//! - Anomaly detection
//! - Change tracking
//!
//! Python bindings are provided via PyO3 in the `python/` directory.

pub mod types;
pub mod spatial;
pub mod temporal;
pub mod storage;
pub mod fusion;
pub mod anomaly;
pub mod query;
pub mod errors;

// Re-exports
pub use types::*;
pub use errors::{Error, Result};

use std::sync::Arc;
use parking_lot::RwLock;

/// PyPanorama core instance
pub struct PanoramaCore {
    // Storage layer
    storage: Arc<storage::Storage>,

    // Fusion engine
    fusion_engine: Arc<fusion::SensorFusionEngine>,

    // Anomaly detector
    anomaly_detector: Arc<anomaly::AnomalyDetector>,
}

impl PanoramaCore {
    /// Create new PyPanorama instance
    pub fn new(config: Option<&str>) -> Result<Self> {
        let storage = Arc::new(storage::Storage::new()?);
        let fusion_engine = Arc::new(fusion::SensorFusionEngine::new());
        let anomaly_detector = Arc::new(anomaly::AnomalyDetector::new());

        Ok(Self {
            storage,
            fusion_engine,
            anomaly_detector,
        })
    }

    /// Push observation to map
    pub async fn push_observation(&self, observation: Observation) -> Result<()> {
        self.storage.store_observation(observation).await?;
        Ok(())
    }

    /// Query for context at location
    pub async fn query(&self, location: GeoPoint, radius_m: f32) -> Result<CompositeContext> {
        todo!("Implement query")
    }

    /// Get image timeline (PyNoramic)
    pub async fn get_image_timeline(&self, location: GeoPoint) -> Result<Vec<ImageObservation>> {
        todo!("Implement image timeline (requires PyNoramic)")
    }
}

// Python FFI
#[cfg(feature = "pyo3")]
pub mod python {
    use pyo3::prelude::*;
    use super::*;

    #[pyclass]
    pub struct PyPanorama {
        core: Arc<PanoramaCore>,
    }

    #[pymethods]
    impl PyPanorama {
        #[new]
        fn new(config: Option<&str>) -> PyResult<Self> {
            let core = PanoramaCore::new(config)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(PyPanorama { core: Arc::new(core) })
        }
    }

    #[pymodule]
    fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<PyPanorama>()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
