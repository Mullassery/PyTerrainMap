//! Autonomous Exploration Intelligence Engine
//!
//! Enables robots to make informed navigation decisions in unexplored regions using
//! historical fleet knowledge, environmental patterns, and probabilistic reasoning.

pub mod patterns;
pub mod predictions;
pub mod hypotheses;
pub mod statistics;
pub mod semantics;
pub mod frontier;
pub mod learning;
pub mod path_planning;
pub mod gaussian_frontier_integration;
pub mod gaussian_cache_integration;
pub mod multi_bot_sync;

// Re-exports for convenience
pub use patterns::{EnvironmentType, EnvironmentPattern, PatternLibrary};
pub use predictions::{PredictiveModel, TraversabilityPredictor};
pub use hypotheses::{Hypothesis, HypothesisType, HypothesisManager, PredictionValue};
pub use statistics::{FleetStatistics, RobotProfile};
pub use semantics::{SemanticContext, SemanticClassifier, StructureTemplate};
pub use frontier::{
    Frontier, FrontierDetector, CuriosityScorer, RiskEvaluator, FrontierPrioritizer,
};
pub use learning::{
    PredictionOutcome, PredictionValidator, AccuracyMetrics, ErrorPattern,
    ConfidenceCalibration, ActiveLearner, LearningUpdate,
};
pub use path_planning::{UnifiedPathPlanner, UnifiedPathCost};
pub use gaussian_frontier_integration::GaussianFrontierScorer;
pub use gaussian_cache_integration::{
    GaussianCacheManager, GaussianTerrainSummary, GaussianObservationFacts, GaussianRegionContext,
    GaussianCacheStats,
};
pub use multi_bot_sync::{
    FleetCoordinator, BotObservationMessage, BotStatus, FleetSyncState, ConflictResolution,
};

#[cfg(test)]
mod tests;
