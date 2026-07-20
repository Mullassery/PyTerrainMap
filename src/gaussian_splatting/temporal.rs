use crate::gaussian_splatting::core::TerrainGaussian;
use crate::gaussian_splatting::store::GaussianSplatStore;
use crate::temporal::DecayFunction;

/// Manages temporal aspects of Gaussian splats: decay, freshness, pruning
#[derive(Clone, Debug)]
pub struct TemporalGaussianManager {
    pub decay: DecayFunction,
    pub revalidation_threshold: f32,  // default 0.4
    pub max_age_ms: u64,              // default 90 days
}

impl TemporalGaussianManager {
    /// Create a manager with exponential decay (45-day half-life for terrain)
    pub fn new() -> Self {
        TemporalGaussianManager {
            decay: DecayFunction::Exponential { half_life_ms: 45 * 24 * 60 * 60 * 1000 },
            revalidation_threshold: 0.4,
            max_age_ms: 90 * 24 * 60 * 60 * 1000,  // 90 days
        }
    }

    /// Get confidence after decay for a splat at current time
    pub fn decayed_confidence(&self, splat: &TerrainGaussian, current_time_us: i64) -> f32 {
        let age_ms = (current_time_us - splat.last_updated) / 1000;
        self.decay.apply(splat.confidence, age_ms)
    }

    /// Compute freshness score (1.0 = fresh, 0.0 = expired)
    pub fn freshness_score(&self, splat: &TerrainGaussian, current_time_us: i64) -> f32 {
        let age_us = current_time_us - splat.last_updated;
        if age_us < 0 {
            return 1.0;
        }

        let age_ms = (age_us / 1000) as f32;
        (1.0 - (age_ms / (self.max_age_ms as f32))).clamp(0.0, 1.0)
    }

    /// Check if splat needs revalidation
    pub fn needs_revalidation(&self, splat: &TerrainGaussian, current_time_us: i64) -> bool {
        let decayed = self.decayed_confidence(splat, current_time_us);
        decayed < self.revalidation_threshold
    }

    /// Apply decay to all splats in store
    pub fn apply_decay_to_store(&self, store: &mut GaussianSplatStore, current_time_us: i64) {
        // Note: In real implementation, would iterate all splats in store
        // For now, this is a placeholder
        let _store = store;
        let _current_time = current_time_us;
    }

    /// Remove expired splats from store
    pub fn prune_expired(&self, store: &mut GaussianSplatStore, current_time_us: i64) -> u32 {
        store.remove_stale(0.0)
    }
}

impl Default for TemporalGaussianManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_manager_creation() {
        let manager = TemporalGaussianManager::new();
        assert_eq!(manager.revalidation_threshold, 0.4);
    }

    #[test]
    fn test_freshness_score_immediate() {
        let manager = TemporalGaussianManager::new();
        let splat = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let now = chrono::Utc::now().timestamp_micros();
        let freshness = manager.freshness_score(&splat, now);
        assert!(freshness > 0.99);
    }

    #[test]
    fn test_needs_revalidation_false_when_fresh() {
        let manager = TemporalGaussianManager::new();
        let splat = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let now = chrono::Utc::now().timestamp_micros();
        assert!(!manager.needs_revalidation(&splat, now));
    }
}
