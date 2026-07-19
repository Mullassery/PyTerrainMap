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
pub mod spatial;
pub mod temporal;
pub mod storage;
pub mod query;
pub mod fusion;
pub mod anomaly;
pub mod export;
pub mod export_security;
pub mod api;
pub mod api_tls;
pub mod data_sources;
pub mod reference_images;
pub mod reconstruction_3d;
pub mod slam;
pub mod photogrammetry;

// Re-export all public types
pub use types::{
    BaselineStatistics, ElevationBucket, Error, FusedData, FusedDetection, GeoPoint,
    GridCell, ObjectDetection, Observation, Result, SensorType, SensorValue, TemperatureEstimate,
    TemporalTrend,
};

// Re-export spatial types
pub use spatial::{SpatialIndex, SpatialKey, H3Cell};

// Re-export temporal types
pub use temporal::{TemporalIndex, DecayFunction};

// Re-export storage types
pub use storage::ObservationStore;

// Re-export query types
pub use query::{Query, QueryResult};

// Re-export fusion types
pub use fusion::{SensorFusion, SensorWeights};

// Re-export anomaly types
pub use anomaly::{AnomalyDetector, AnomalyType, AnomalyStats};

// Re-export export types
pub use export::{ExportFormat, SpatialExporter, GeoJSONExporter, KMLExporter};

// Re-export security types
pub use export_security::{
    DataClassification, UserRole, ExportPrivacy, ExportPolicy, AuditLogger, AuditLogEntry,
};

// Re-export API types
pub use api::{
    ApiError, ApiResult, SubmitObservationRequest, SpatialQueryRequest,
    TemporalQueryRequest, ExportRequest, HealthResponse, FleetStatus, ApiConfig, ApiRoute,
};

// Re-export TLS/HTTPS types
pub use api_tls::{
    HttpsMode, TlsConfig, TlsVersion, CertificateInfo, CertificateValidator, SecurityHeaders,
};

// Re-export data source types
pub use data_sources::{
    DataSourceType, ExternalFeature, DataSourceConfig, DataSourceRegistry, GeometryType,
    ContextEnrichment,
};

// Re-export reference image types
pub use reference_images::{
    ReferenceImage, ReferenceImageStore, VisualDescriptor, ImageMatch, GeoreferenceStatus,
    ImageOrientation,
};

// Re-export 3D reconstruction types
pub use reconstruction_3d::{
    CameraIntrinsics, CameraPose, Point3D, ReconstructionFrame, PointCloud, ReconstructionEngine,
    PointCloudStats, ReconstructionStats,
};

// Re-export SLAM types
pub use slam::{
    SLAMTracker, SLAMPose, PoseGraph, CameraFrame, VisualFeature, IMUReading, DepthMeasurement,
    DepthSensorType, PoseEdge, LoopClosure, SLAMStats,
};

// Re-export Photogrammetry types
pub use photogrammetry::{
    StructureFromMotion, PhotogrammetryProcessor, DensePointCloud, TriangulatedPoint,
    CameraPoseEstimate, Neural3DRepresentation, GaussianSplat, PhotogrammetryStats,
};

// TODO: Implement in future weeks
// pub mod storage;     // Week 3-4: In-memory storage
// pub mod fusion;      // Week 4-5: Sensor fusion
// pub mod anomaly;     // Week 7-8: Anomaly detection
// pub mod query;       // Week 4-5: Query API
// pub mod python;      // Week 5-6: PyO3 bindings

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
