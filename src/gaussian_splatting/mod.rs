//! Gaussian Splatting Probabilistic Mapping Layer
//!
//! This module provides a continuous, uncertainty-aware probabilistic world model
//! for multi-robot fleet coordination. It replaces rigid occupancy grids with
//! Gaussian representations that enable robots to collectively understand terrain,
//! predict unknown regions, track dynamic objects, and detect environmental changes.
//!
//! # Core Components
//!
//! - **core.rs**: TerrainGaussian, GaussianCovariance, terrain classification
//! - **store.rs**: Global GaussianSplatStore (shared by all robots)
//! - **fusion.rs**: Multi-bot Bayesian observation fusion
//! - **distance.rs**: 5-component traversability-aware path cost
//! - **passage.rs**: Dynamic passage modeling (doors, hallways, gates)
//! - **prediction.rs**: Unknown region prediction via neighbor inference
//! - **temporal.rs**: Confidence decay and freshness scoring
//! - **semantic.rs**: Semantic terrain mapping and bot mission profiles
//! - **lod.rs**: Hierarchical level-of-detail auto-splitting/merging
//! - **exploration.rs**: Gaussian-aware exploration strategy
//! - **objects.rs**: Dynamic object splats (pallets, people, carts, etc.)
//! - **change_events.rs**: Environment change detection and logging
//! - **fleet_learning.rs**: Collective intelligence engine ("one bot learns, all bots know")
//!
//! # Philosophy
//!
//! PyTerrainMap is the shared map, not the robot brain. Robots feed observations in,
//! query current world state out. The map handles fusion, decay, prediction, and
//! change detection transparently.

pub mod core;
pub mod store;
pub mod fusion;
pub mod distance;
pub mod passage;
pub mod prediction;
pub mod temporal;
pub mod semantic;
pub mod lod;
pub mod exploration;
pub mod objects;
pub mod change_events;
pub mod fleet_learning;
pub mod h3_optimization;
pub mod memory_pool;

// Re-export main public types for ergonomic access
pub use core::{
    GaussianCovariance, TerrainGaussian, TerrainType, SplatKind,
};
pub use store::{
    GaussianSplatStore, H3SplatKey, StoreStats,
};
pub use fusion::{
    FusionAction, FusionResult, ObservationFuser,
};
pub use distance::{
    PathCost, TerrainCostMap, TraversabilityDistanceEngine,
};
pub use passage::{
    PassageType, PassageSplat, PassageTraversal,
};
pub use prediction::{
    PredictedSplat, UnknownRegionPredictor, VerificationResult,
};
pub use temporal::{
    TemporalGaussianManager,
};
pub use semantic::{
    BotMissionProfile, SemanticGaussianMapper,
};
pub use lod::{
    HierarchicalLOD, LODLevel,
};
pub use exploration::{
    ExplorationTarget, GaussianExplorationStrategy, StorePatch, SyncResult, MultiSplatSynchronizer,
};
pub use objects::{
    ObjectClass, ObjectMobility, DynamicObjectSplat, PositionSnapshot,
};
pub use change_events::{
    ChangeEvent, ChangeEventType, ChangeEventLog,
};
pub use fleet_learning::{
    FleetLearningEngine, ObjectObservation, ObjectState, ObjectPrediction, AreaDynamicsProfile,
};
pub use h3_optimization::{
    H3SpatialIndex, H3Resolution, H3IndexStats,
};
pub use memory_pool::{
    MemoryPoolManager, PoolConfig, PoolStats, MemoryPoolStats,
    SplatPool, ObservationPool, PooledSplat, PooledObservation,
};
