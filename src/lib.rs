//! PyTerrainMap: Collaborative Spatial Intelligence Platform
//!
//! This is the Rust core of PyTerrainMap. It provides:
//! - 3D spatial indexing (H3 + elevation)
//! - Temporal observation storage and querying
//! - Multi-sensor fusion
//! - Anomaly detection
//! - Change tracking
//!
//! Python bindings are provided via PyO3 in the `python/` directory.

pub mod types;

// Re-export all public types
pub use types::{
    BaselineStatistics, ElevationBucket, Error, FusedData, FusedDetection, GeoPoint,
    GridCell, ObjectDetection, Observation, Result, SensorType, SensorValue, TemperatureEstimate,
    TemporalTrend,
};

// TODO: Implement in future weeks
// pub mod spatial;     // Week 2-3: H3 spatial indexing
// pub mod temporal;    // Week 3-4: Time-series indexing
// pub mod storage;     // Week 3-4: In-memory storage
// pub mod fusion;      // Week 4-5: Sensor fusion
// pub mod anomaly;     // Week 7-8: Anomaly detection
// pub mod query;       // Week 4-5: Query API
// pub mod python;      // Week 5-6: PyO3 bindings

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
