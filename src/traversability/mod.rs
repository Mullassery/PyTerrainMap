//! Spatial Knowledge Graph for Traversability Intelligence
//!
//! Represents the environment as a graph of spatial regions (nodes) and connections (edges),
//! with robot-specific traversability observations and fleet learning.

pub mod nodes;
pub mod edges;
pub mod observations;
pub mod metadata;
pub mod spatial_graph;

// Re-exports for convenience
pub use nodes::{Node, NodeType};
pub use edges::{Edge, EdgeType};
pub use observations::{TraversalOutcome, TraversabilityObservation, ConsensusResult};
pub use metadata::{SpatialMetadata, EnvironmentVersion};
pub use spatial_graph::SpatialGraph;

#[cfg(test)]
mod tests;
