use crate::gaussian_splatting::core::TerrainGaussian;
use crate::gaussian_splatting::passage::PassageSplat;
use crate::gaussian_splatting::prediction::PredictedSplat;
use crate::gaussian_splatting::store::GaussianSplatStore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Exploration target with priority scoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExplorationTarget {
    pub position: [f64; 3],
    pub priority: f32,
    pub unknownness: f32,
    pub info_gain: f32,
    pub strategic_value: f32,
    pub risk: f32,
    pub predicted: Option<PredictedSplat>,
}

/// Gaussian-aware exploration strategy
pub struct GaussianExplorationStrategy {
    pub unknown_weight: f32,    // 0.4
    pub info_gain_weight: f32,  // 0.3
    pub strategic_weight: f32,  // 0.2
    pub risk_weight: f32,       // 0.1
}

impl GaussianExplorationStrategy {
    pub fn new() -> Self {
        GaussianExplorationStrategy {
            unknown_weight: 0.4,
            info_gain_weight: 0.3,
            strategic_weight: 0.2,
            risk_weight: 0.1,
        }
    }

    /// Score a position for exploration value
    pub fn score_position(&self, pos: [f64; 3], store: &GaussianSplatStore) -> ExplorationTarget {
        let unknownness = store.uncertainty_at(pos);
        let nearby = store.query_radius(pos, 50.0);
        let info_gain = if nearby.is_empty() {
            1.0
        } else {
            1.0 - ((nearby.len() as f32) / 20.0).min(1.0)
        };
        let strategic_value = 0.5;  // Placeholder
        let risk = nearby
            .iter()
            .map(|s| 1.0 - s.traversability)
            .sum::<f32>()
            / (nearby.len().max(1) as f32);

        let priority = self.unknown_weight * unknownness
            + self.info_gain_weight * info_gain
            + self.strategic_weight * strategic_value
            - self.risk_weight * risk;

        ExplorationTarget {
            position: pos,
            priority,
            unknownness,
            info_gain,
            strategic_value,
            risk,
            predicted: None,
        }
    }

    /// Get top exploration targets from candidates
    pub fn top_targets(
        &self,
        candidates: &[[f64; 3]],
        store: &GaussianSplatStore,
        n: usize,
    ) -> Vec<ExplorationTarget> {
        let mut targets: Vec<_> = candidates
            .iter()
            .map(|&pos| self.score_position(pos, store))
            .collect();

        targets.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        targets.into_iter().take(n).collect()
    }
}

impl Default for GaussianExplorationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Patch of map updates from a bot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorePatch {
    pub from_bot: String,
    pub timestamp: i64,
    pub new_splats: Vec<TerrainGaussian>,
    pub passage_updates: Vec<PassageSplat>,
}

/// Result of applying a patch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub fused: u32,
    pub created: u32,
    pub conflicts: u32,
}

/// Multi-splat synchronization across bots
pub struct MultiSplatSynchronizer;

impl MultiSplatSynchronizer {
    /// Apply a patch to the primary store
    pub fn apply_patch(primary: &mut GaussianSplatStore, patch: StorePatch) -> SyncResult {
        let mut result = SyncResult {
            fused: 0,
            created: 0,
            conflicts: 0,
        };

        for splat in patch.new_splats {
            let fusion_result = primary.insert_or_fuse(splat);
            match fusion_result.action {
                crate::gaussian_splatting::fusion::FusionAction::Created => result.created += 1,
                crate::gaussian_splatting::fusion::FusionAction::Fused { .. } => result.fused += 1,
                crate::gaussian_splatting::fusion::FusionAction::Rejected { .. } => result.conflicts += 1,
            }
        }

        result
    }

    /// Merge multiple patches
    pub fn merge_patches(
        primary: &mut GaussianSplatStore,
        patches: Vec<StorePatch>,
    ) -> Vec<SyncResult> {
        patches
            .into_iter()
            .map(|patch| Self::apply_patch(primary, patch))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_exploration_strategy_creation() {
        let strategy = GaussianExplorationStrategy::new();
        assert_eq!(strategy.unknown_weight, 0.4);
    }

    #[test]
    fn test_exploration_target_scoring() {
        let strategy = GaussianExplorationStrategy::new();
        let store = GaussianSplatStore::new();
        let target = strategy.score_position([40.0, -74.0, 10.0], &store);
        assert!(target.priority > 0.0);
        assert_eq!(target.unknownness, 1.0);  // Empty store = maximum unknownness
    }

    #[test]
    fn test_top_targets() {
        let strategy = GaussianExplorationStrategy::new();
        let store = GaussianSplatStore::new();
        let candidates = vec![[40.0, -74.0, 10.0], [40.1, -74.0, 10.0], [40.2, -74.0, 10.0]];
        let targets = strategy.top_targets(&candidates, &store, 2);
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn test_store_patch_creation() {
        let patch = StorePatch {
            from_bot: "bot_01".to_string(),
            timestamp: chrono::Utc::now().timestamp_micros(),
            new_splats: vec![],
            passage_updates: vec![],
        };
        assert_eq!(patch.from_bot, "bot_01");
    }

    #[test]
    fn test_multi_splat_synchronizer() {
        let mut store = GaussianSplatStore::new();
        let patch = StorePatch {
            from_bot: "bot_01".to_string(),
            timestamp: chrono::Utc::now().timestamp_micros(),
            new_splats: vec![TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8)],
            passage_updates: vec![],
        };

        let result = MultiSplatSynchronizer::apply_patch(&mut store, patch);
        assert_eq!(result.created, 1);
    }
}
