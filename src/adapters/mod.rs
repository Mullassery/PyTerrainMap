//! Ecosystem adapters for PyRoboFrames and PyRoboVision integration
//!
//! Enables PyTerrainMap to consume data from upstream systems:
//! - PyRoboFrames: MCAP/ROS2 dataframe composition with multi-rate sensor alignment
//! - PyRoboVision: Vision model registry with terrain-aware model selection
//!
//! Data flow: PyRoboFrames → PyRoboVision → PyTerrainMap

pub mod pyroboframes_adapter;
pub mod pyrobovision_adapter;
pub mod data_contracts;

// Re-export main types
pub use pyroboframes_adapter::RoboticsDataFrameAdapter;
pub use pyrobovision_adapter::VisionModelAwareAdapter;
pub use data_contracts::{UnifiedObservationSchema, LineageTracker};
