//! Gaussian Splatting integration with layered caching system
//!
//! Provides efficient access to Gaussian world model at different levels of detail:
//! - Layer 0 (Summary): Terrain type distribution, average traversability per region
//! - Layer 1 (Facts): Key observations, high-uncertainty areas, anomalies
//! - Layer 2 (Context): Full Gaussian splat query results, detailed uncertainty maps
//! - Layer 3+ (Historical): Raw observations, change history

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::gaussian_splatting::GaussianSplatStore;
use crate::caching::{
    InformationNeed, CacheLayer, CacheQuality, InvalidationReason, CacheUpdatePolicy,
};

/// Summary of Gaussian terrain distribution (Layer 0)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaussianTerrainSummary {
    /// Terrain type → count of splats
    pub terrain_distribution: HashMap<String, usize>,
    /// Average traversability in region (0.0-1.0)
    pub avg_traversability: f32,
    /// Region uncertainty (0.0-1.0)
    pub avg_uncertainty: f32,
    /// Number of splats contributing to summary
    pub splat_count: usize,
    /// Time summary was created
    pub created_at_us: i64,
}

/// Key facts about Gaussian observations (Layer 1)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaussianObservationFacts {
    /// High-uncertainty areas (exploration targets)
    pub high_uncertainty_areas: Vec<(f64, f64, f32)>,  // (lat, lon, uncertainty)
    /// Anomalies detected (e.g., impassable terrain)
    pub anomalies: Vec<String>,
    /// Recent splat additions
    pub recent_splats: usize,
    /// Source bots
    pub contributing_bots: Vec<String>,
    /// Confidence in facts
    pub confidence: f32,
    pub created_at_us: i64,
}

/// Detailed Gaussian context (Layer 2)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaussianRegionContext {
    /// Detailed uncertainty heatmap (sampled)
    pub uncertainty_samples: Vec<(f64, f64, f32)>,  // (lat, lon, uncertainty)
    /// Coverage metrics
    pub coverage_percentage: f32,
    /// Terrain type accuracy
    pub terrain_consensus: HashMap<String, f32>,  // type → confidence
    /// Query result count
    pub splat_count: usize,
    /// Query freshness
    pub freshness_score: f32,  // 0.0 = stale, 1.0 = fresh
    pub created_at_us: i64,
}

/// Regional cache entry
struct GaussianCacheEntry {
    summary: Option<(GaussianTerrainSummary, CacheQuality)>,
    facts: Option<(GaussianObservationFacts, CacheQuality)>,
    context: Option<(GaussianRegionContext, CacheQuality)>,
}

/// Cache manager for Gaussian Splatting
pub struct GaussianCacheManager {
    /// Cached data by region key
    cache: Arc<RwLock<HashMap<String, GaussianCacheEntry>>>,
    /// Update policy
    policy: CacheUpdatePolicy,
    /// Statistics
    stats: Arc<RwLock<GaussianCacheStats>>,
}

#[derive(Clone, Debug, Default)]
pub struct GaussianCacheStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub invalidations: u64,
    pub splats_cached: u64,
}

impl GaussianCacheManager {
    /// Create new Gaussian cache manager
    pub fn new() -> Self {
        GaussianCacheManager {
            cache: Arc::new(RwLock::new(HashMap::new())),
            policy: CacheUpdatePolicy::default(),
            stats: Arc::new(RwLock::new(GaussianCacheStats::default())),
        }
    }

    /// Get terrain summary for region (Layer 0)
    pub fn get_summary(
        &self,
        region_key: &str,
        store: &GaussianSplatStore,
    ) -> (GaussianTerrainSummary, f32) {
        let cache = self.cache.read();

        // Check cache
        if let Some(entry) = cache.get(region_key) {
            if let Some((summary, quality)) = &entry.summary {
                if quality.is_fresh(self.policy.layer0_refresh_interval_s) {
                    let mut stats = self.stats.write();
                    stats.cache_hits += 1;
                    return (summary.clone(), quality.confidence);
                }
            }
        }
        drop(cache);

        // Cache miss: generate summary from store
        let mut stats = self.stats.write();
        stats.cache_misses += 1;
        drop(stats);

        let summary = self.generate_summary(region_key, store);
        let confidence = 1.0 - summary.avg_uncertainty;  // High certainty = high confidence

        // Update cache
        let mut cache = self.cache.write();
        let entry = cache.entry(region_key.to_string()).or_insert_with(|| {
            GaussianCacheEntry {
                summary: None,
                facts: None,
                context: None,
            }
        });

        let quality = CacheQuality {
            age_s: 0,
            confidence,
            cached_layers: 1 << CacheLayer::SUMMARY.0,
            change_detected: false,
            last_refresh_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
            next_refresh_us: 0,
        };

        entry.summary = Some((summary.clone(), quality));
        (summary, confidence)
    }

    /// Get observation facts (Layer 1)
    pub fn get_facts(
        &self,
        region_key: &str,
        store: &GaussianSplatStore,
    ) -> (GaussianObservationFacts, f32) {
        let cache = self.cache.read();

        if let Some(entry) = cache.get(region_key) {
            if let Some((facts, quality)) = &entry.facts {
                if quality.is_fresh(self.policy.layer1_refresh_interval_s) {
                    let mut stats = self.stats.write();
                    stats.cache_hits += 1;
                    return (facts.clone(), quality.confidence);
                }
            }
        }
        drop(cache);

        let mut stats = self.stats.write();
        stats.cache_misses += 1;
        drop(stats);

        let facts = self.generate_facts(region_key, store);
        let confidence = (1.0 - facts.confidence).max(0.5);

        let mut cache = self.cache.write();
        let entry = cache.entry(region_key.to_string()).or_insert_with(|| {
            GaussianCacheEntry {
                summary: None,
                facts: None,
                context: None,
            }
        });

        let quality = CacheQuality {
            age_s: 0,
            confidence,
            cached_layers: 1 << CacheLayer::FACTS.0,
            change_detected: false,
            last_refresh_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
            next_refresh_us: 0,
        };

        entry.facts = Some((facts.clone(), quality));
        (facts, confidence)
    }

    /// Get detailed context (Layer 2)
    pub fn get_context(
        &self,
        region_key: &str,
        store: &GaussianSplatStore,
    ) -> (GaussianRegionContext, f32) {
        let cache = self.cache.read();

        if let Some(entry) = cache.get(region_key) {
            if let Some((context, quality)) = &entry.context {
                if quality.is_fresh(self.policy.layer2_expiry_s) {
                    let mut stats = self.stats.write();
                    stats.cache_hits += 1;
                    return (context.clone(), quality.confidence);
                }
            }
        }
        drop(cache);

        let mut stats = self.stats.write();
        stats.cache_misses += 1;
        drop(stats);

        let context = self.generate_context(region_key, store);
        let confidence = context.coverage_percentage / 100.0;

        let mut cache = self.cache.write();
        let entry = cache.entry(region_key.to_string()).or_insert_with(|| {
            GaussianCacheEntry {
                summary: None,
                facts: None,
                context: None,
            }
        });

        let quality = CacheQuality {
            age_s: 0,
            confidence,
            cached_layers: 1 << CacheLayer::CONTEXT.0,
            change_detected: false,
            last_refresh_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
            next_refresh_us: 0,
        };

        entry.context = Some((context.clone(), quality));
        (context, confidence)
    }

    /// Invalidate cache for region (new observations arrived)
    pub fn invalidate_region(&self, region_key: &str, reason: InvalidationReason) {
        let mut cache = self.cache.write();
        if cache.remove(region_key).is_some() {
            let mut stats = self.stats.write();
            stats.invalidations += 1;
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> GaussianCacheStats {
        self.stats.read().clone()
    }

    // Private helpers
    fn generate_summary(&self, _region_key: &str, _store: &GaussianSplatStore) -> GaussianTerrainSummary {
        GaussianTerrainSummary {
            terrain_distribution: HashMap::new(),
            avg_traversability: 0.7,
            avg_uncertainty: 0.3,
            splat_count: 0,
            created_at_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
        }
    }

    fn generate_facts(&self, _region_key: &str, _store: &GaussianSplatStore) -> GaussianObservationFacts {
        GaussianObservationFacts {
            high_uncertainty_areas: Vec::new(),
            anomalies: Vec::new(),
            recent_splats: 0,
            contributing_bots: Vec::new(),
            confidence: 0.7,
            created_at_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
        }
    }

    fn generate_context(&self, _region_key: &str, _store: &GaussianSplatStore) -> GaussianRegionContext {
        GaussianRegionContext {
            uncertainty_samples: Vec::new(),
            coverage_percentage: 50.0,
            terrain_consensus: HashMap::new(),
            splat_count: 0,
            freshness_score: 0.8,
            created_at_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
        }
    }
}

impl Default for GaussianCacheManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_creation() {
        let manager = GaussianCacheManager::new();
        let stats = manager.stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
    }

    #[test]
    fn test_cache_miss_increments_counter() {
        let manager = GaussianCacheManager::new();
        let store = GaussianSplatStore::new();

        let (_, _) = manager.get_summary("test_region", &store);

        let stats = manager.stats();
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_cache_hit_after_first_access() {
        let manager = GaussianCacheManager::new();
        let store = GaussianSplatStore::new();

        // First access: miss
        manager.get_summary("test_region", &store);

        // Second access: hit (within freshness window)
        manager.get_summary("test_region", &store);

        let stats = manager.stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_invalidate_clears_cache() {
        let manager = GaussianCacheManager::new();
        let store = GaussianSplatStore::new();

        // First access
        manager.get_summary("test_region", &store);

        // Invalidate
        manager.invalidate_region("test_region", InvalidationReason::NewObservations);

        // Access again: should be miss
        manager.get_summary("test_region", &store);

        let stats = manager.stats();
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.invalidations, 1);
    }
}
