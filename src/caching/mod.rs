//! Layered caching and progressive world understanding
//!
//! Provides hierarchical caching system enabling agents to make decisions
//! with progressively richer information, from summaries to raw data.
//!
//! Core principle: Most decisions need summaries. Some need details.
//! Very few need all available historical data.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Information need determines which cache layers to retrieve
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InformationNeed {
    /// Just need basic context (Layer 0)
    BasicContext,
    /// Making planning decision (Layer 0-1)
    PlanningDecision,
    /// Optimizing route (Layer 1-2)
    RouteOptimization,
    /// Avoiding obstacles in real-time (Layer 2)
    ObstacleAvoidance,
    /// Tracking trajectory (Layer 2-3)
    TrajectoryTracking,
    /// Extracting features (Layer 3)
    FeatureExtraction,
    /// Analyzing historical patterns (Layer 3-4)
    HistoricalAnalysis,
    /// Forensic reconstruction (Layer 4)
    ForensicReconstruction,
}

impl InformationNeed {
    /// Get range of cache layers needed for this need
    pub fn layers_needed(&self) -> (u8, u8) {
        match self {
            InformationNeed::BasicContext => (0, 0),
            InformationNeed::PlanningDecision => (0, 1),
            InformationNeed::RouteOptimization => (1, 2),
            InformationNeed::ObstacleAvoidance => (2, 2),
            InformationNeed::TrajectoryTracking => (2, 3),
            InformationNeed::FeatureExtraction => (3, 3),
            InformationNeed::HistoricalAnalysis => (3, 4),
            InformationNeed::ForensicReconstruction => (4, 4),
        }
    }

    /// Typical latency SLA for this need (milliseconds)
    pub fn target_latency_ms(&self) -> u32 {
        match self {
            InformationNeed::BasicContext => 10,
            InformationNeed::PlanningDecision => 100,
            InformationNeed::RouteOptimization => 300,
            InformationNeed::ObstacleAvoidance => 500,
            InformationNeed::TrajectoryTracking => 1000,
            InformationNeed::FeatureExtraction => 2000,
            InformationNeed::HistoricalAnalysis => 10000,
            InformationNeed::ForensicReconstruction => 3600000,  // ~1 hour
        }
    }
}

/// Cache layer levels (0-4)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheLayer(pub u8);

impl CacheLayer {
    pub const SUMMARY: CacheLayer = CacheLayer(0);
    pub const FACTS: CacheLayer = CacheLayer(1);
    pub const CONTEXT: CacheLayer = CacheLayer(2);
    pub const OBSERVATIONS: CacheLayer = CacheLayer(3);
    pub const RAW: CacheLayer = CacheLayer(4);
}

/// Reason why cache became invalid
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvalidationReason {
    TimeExpired,
    NewObservations,
    SignificantChange,
    ManualInvalidation,
    HighErrorRate,
}

/// Cache quality metrics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CacheQuality {
    /// How old is this information (seconds)
    pub age_s: u32,

    /// Confidence in cached information (0.0-1.0)
    pub confidence: f32,

    /// Which layers are currently cached (bit flags)
    pub cached_layers: u8,

    /// Has significant change been detected since last update?
    pub change_detected: bool,

    /// Last time this cache was refreshed
    pub last_refresh_us: i64,

    /// Next scheduled refresh time
    pub next_refresh_us: i64,
}

impl CacheQuality {
    pub fn is_fresh(&self, max_age_s: u32) -> bool {
        self.age_s < max_age_s
    }

    pub fn is_confident(&self, min_confidence: f32) -> bool {
        self.confidence >= min_confidence
    }

    pub fn has_layer(&self, layer: CacheLayer) -> bool {
        (self.cached_layers & (1 << layer.0)) != 0
    }

    pub fn set_layer(&mut self, layer: CacheLayer) {
        self.cached_layers |= 1 << layer.0;
    }
}

/// Summary of a location (Layer 0)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationSummary {
    pub id: Uuid,
    pub location: String,

    /// Primary characteristics
    pub characteristics: Vec<String>,

    /// Area in square kilometers
    pub area_km2: f32,

    /// When was this summary created?
    pub created_at_us: i64,

    /// When was this last updated?
    pub updated_at_us: i64,

    /// How confident are we in this summary?
    pub confidence: f32,

    /// Compressed data size
    pub data_size_bytes: usize,
}

/// Important facts about a location (Layer 1)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationFacts {
    pub id: Uuid,
    pub location: String,

    /// Static features (buildings, water bodies, etc.)
    pub static_features: Vec<(String, usize)>,

    /// Dynamic features (pedestrian count, weather, etc.)
    pub dynamic_features: Vec<(String, String)>,

    /// Known restrictions
    pub restrictions: Vec<String>,

    /// Known hazards
    pub hazards: Vec<String>,

    /// Recent events
    pub recent_events: Vec<Event>,

    pub created_at_us: i64,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub event_type: String,
    pub description: String,
    pub timestamp_us: i64,
}

/// Local context for a region (Layer 2)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionContext {
    pub id: Uuid,
    pub region_name: String,

    /// Terrain characteristics
    pub terrain_zones: Vec<TerrainZone>,

    /// Recent observation statistics
    pub observation_stats: ObservationStats,

    /// Detected obstacles
    pub obstacle_summary: ObstacleSummary,

    /// Behavioral patterns
    pub patterns: Vec<BehavioralPattern>,

    pub created_at_us: i64,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainZone {
    pub id: String,
    pub elevation_m: f32,
    pub slope_pct: f32,
    pub soil_type: String,
    pub vegetation: Vec<String>,
    pub traversability: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationStats {
    pub last_24h: u64,
    pub last_7d: u64,
    pub total: u64,
    pub sensor_types: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObstacleSummary {
    pub total_detected: usize,
    pub types: HashMap<String, usize>,
    pub density_per_km2: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehavioralPattern {
    pub pattern_type: String,
    pub description: String,
    pub frequency: String,
}

/// Cache update configuration
#[derive(Clone, Debug)]
pub struct CacheUpdatePolicy {
    /// Refresh Layer 0 every N seconds
    pub layer0_refresh_interval_s: u32,

    /// Refresh Layer 1 every N seconds
    pub layer1_refresh_interval_s: u32,

    /// Expire Layer 2 after N seconds
    pub layer2_expiry_s: u32,

    /// Archive Layer 3 after N seconds
    pub layer3_archive_s: u32,

    /// Refresh if this many new observations received
    pub observation_threshold: usize,

    /// Refresh if change magnitude exceeds this
    pub change_magnitude_threshold: f32,
}

impl Default for CacheUpdatePolicy {
    fn default() -> Self {
        CacheUpdatePolicy {
            layer0_refresh_interval_s: 300,      // 5 minutes
            layer1_refresh_interval_s: 1800,     // 30 minutes
            layer2_expiry_s: 86400,              // 24 hours
            layer3_archive_s: 2592000,           // 30 days
            observation_threshold: 1000,
            change_magnitude_threshold: 0.2,
        }
    }
}

/// Cache manager: coordinates multi-level caching
pub struct CacheManager {
    /// Layer 0-2 caches in memory
    memory_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,

    /// Cache quality tracking
    quality: Arc<RwLock<HashMap<String, CacheQuality>>>,

    /// Update policy
    policy: CacheUpdatePolicy,

    /// Statistics
    stats: Arc<RwLock<CacheStats>>,
}

struct CacheEntry {
    layer0: Option<LocationSummary>,
    layer1: Option<LocationFacts>,
    layer2: Option<RegionContext>,
}

#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub invalidations: u64,
    pub total_requests: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f32 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.hits as f32 / self.total_requests as f32
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        CacheManager::new()
    }
}

impl CacheManager {
    pub fn new() -> Self {
        CacheManager {
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
            quality: Arc::new(RwLock::new(HashMap::new())),
            policy: CacheUpdatePolicy::default(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get cache for specific information need
    pub fn get_for_need(&self, location: &str, need: InformationNeed) -> CachedResult {
        let (start_layer, end_layer) = need.layers_needed();

        let cache = self.memory_cache.read();
        let quality = self.quality.read();

        if let Some(entry) = cache.get(location) {
            let mut result = CachedResult {
                location: location.to_string(),
                available_layers: 0,
                layers: HashMap::new(),
                quality: quality.get(location).cloned().unwrap_or_default(),
            };

            // Collect available layers
            if start_layer <= 0 && end_layer >= 0 {
                if let Some(summary) = &entry.layer0 {
                    result.layers.insert(CacheLayer(0), CachedLayer::Summary(summary.clone()));
                    result.available_layers |= 1 << 0;
                }
            }

            if start_layer <= 1 && end_layer >= 1 {
                if let Some(facts) = &entry.layer1 {
                    result.layers.insert(CacheLayer(1), CachedLayer::Facts(facts.clone()));
                    result.available_layers |= 1 << 1;
                }
            }

            if start_layer <= 2 && end_layer >= 2 {
                if let Some(context) = &entry.layer2 {
                    result.layers.insert(CacheLayer(2), CachedLayer::Context(context.clone()));
                    result.available_layers |= 1 << 2;
                }
            }

            if !result.layers.is_empty() {
                let mut stats = self.stats.write();
                stats.hits += 1;
                stats.total_requests += 1;
                return result;
            }
        }

        // Cache miss
        let mut stats = self.stats.write();
        stats.misses += 1;
        stats.total_requests += 1;

        CachedResult {
            location: location.to_string(),
            available_layers: 0,
            layers: HashMap::new(),
            quality: CacheQuality {
                age_s: u32::MAX,
                confidence: 0.0,
                cached_layers: 0,
                change_detected: false,
                last_refresh_us: 0,
                next_refresh_us: 0,
            },
        }
    }

    /// Store layer 0 summary
    pub fn put_summary(&self, location: &str, summary: LocationSummary) {
        let mut cache = self.memory_cache.write();
        let entry = cache.entry(location.to_string()).or_insert(CacheEntry {
            layer0: None,
            layer1: None,
            layer2: None,
        });
        entry.layer0 = Some(summary);

        let mut quality = self.quality.write();
        let q = quality.entry(location.to_string()).or_insert(CacheQuality {
            age_s: 0,
            confidence: 0.0,
            cached_layers: 0,
            change_detected: false,
            last_refresh_us: 0,
            next_refresh_us: 0,
        });
        q.set_layer(CacheLayer::SUMMARY);
    }

    /// Store layer 1 facts
    pub fn put_facts(&self, location: &str, facts: LocationFacts) {
        let mut cache = self.memory_cache.write();
        let entry = cache.entry(location.to_string()).or_insert(CacheEntry {
            layer0: None,
            layer1: None,
            layer2: None,
        });
        entry.layer1 = Some(facts);

        let mut quality = self.quality.write();
        let q = quality.entry(location.to_string()).or_insert(CacheQuality {
            age_s: 0,
            confidence: 0.0,
            cached_layers: 0,
            change_detected: false,
            last_refresh_us: 0,
            next_refresh_us: 0,
        });
        q.set_layer(CacheLayer::FACTS);
    }

    /// Store layer 2 context
    pub fn put_context(&self, location: &str, context: RegionContext) {
        let mut cache = self.memory_cache.write();
        let entry = cache.entry(location.to_string()).or_insert(CacheEntry {
            layer0: None,
            layer1: None,
            layer2: None,
        });
        entry.layer2 = Some(context);

        let mut quality = self.quality.write();
        let q = quality.entry(location.to_string()).or_insert(CacheQuality {
            age_s: 0,
            confidence: 0.0,
            cached_layers: 0,
            change_detected: false,
            last_refresh_us: 0,
            next_refresh_us: 0,
        });
        q.set_layer(CacheLayer::CONTEXT);
    }

    /// Invalidate cache for location
    pub fn invalidate(&self, location: &str, _reason: InvalidationReason) {
        let mut cache = self.memory_cache.write();
        cache.remove(location);

        let mut quality = self.quality.write();
        quality.remove(location);

        let mut stats = self.stats.write();
        stats.invalidations += 1;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }
}

/// Result of cache retrieval
#[derive(Clone, Debug)]
pub struct CachedResult {
    pub location: String,
    pub available_layers: u8,
    pub layers: HashMap<CacheLayer, CachedLayer>,
    pub quality: CacheQuality,
}

impl CachedResult {
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn has_layer(&self, layer: CacheLayer) -> bool {
        self.layers.contains_key(&layer)
    }
}

#[derive(Clone, Debug)]
pub enum CachedLayer {
    Summary(LocationSummary),
    Facts(LocationFacts),
    Context(RegionContext),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_information_need_layers() {
        assert_eq!(InformationNeed::BasicContext.layers_needed(), (0, 0));
        assert_eq!(InformationNeed::PlanningDecision.layers_needed(), (0, 1));
        assert_eq!(InformationNeed::ObstacleAvoidance.layers_needed(), (2, 2));
    }

    #[test]
    fn test_cache_quality_tracking() {
        let mut quality = CacheQuality {
            age_s: 60,
            confidence: 0.95,
            cached_layers: 0,
            change_detected: false,
            last_refresh_us: 0,
            next_refresh_us: 0,
        };

        assert!(!quality.has_layer(CacheLayer::SUMMARY));
        quality.set_layer(CacheLayer::SUMMARY);
        assert!(quality.has_layer(CacheLayer::SUMMARY));
    }

    #[test]
    fn test_cache_manager_put_get() {
        let manager = CacheManager::new();
        let summary = LocationSummary {
            id: Uuid::new_v4(),
            location: "Central Park".to_string(),
            characteristics: vec!["Urban park".to_string()],
            area_km2: 3.4,
            created_at_us: 0,
            updated_at_us: 0,
            confidence: 0.95,
            data_size_bytes: 512,
        };

        manager.put_summary("central_park", summary.clone());

        let result = manager.get_for_need("central_park", InformationNeed::BasicContext);
        assert!(!result.is_empty());
        assert!(result.has_layer(CacheLayer::SUMMARY));
    }

    #[test]
    fn test_cache_miss() {
        let manager = CacheManager::new();
        let result = manager.get_for_need("unknown_location", InformationNeed::BasicContext);
        assert!(result.is_empty());

        let stats = manager.stats();
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hits, 0);
    }

    #[test]
    fn test_cache_invalidation() {
        let manager = CacheManager::new();
        let summary = LocationSummary {
            id: Uuid::new_v4(),
            location: "Park".to_string(),
            characteristics: vec![],
            area_km2: 1.0,
            created_at_us: 0,
            updated_at_us: 0,
            confidence: 0.9,
            data_size_bytes: 256,
        };

        manager.put_summary("park", summary);
        manager.invalidate("park", InvalidationReason::TimeExpired);

        let result = manager.get_for_need("park", InformationNeed::BasicContext);
        assert!(result.is_empty());

        let stats = manager.stats();
        assert_eq!(stats.invalidations, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let manager = CacheManager::new();
        let summary = LocationSummary {
            id: Uuid::new_v4(),
            location: "Location".to_string(),
            characteristics: vec![],
            area_km2: 1.0,
            created_at_us: 0,
            updated_at_us: 0,
            confidence: 0.9,
            data_size_bytes: 256,
        };

        manager.put_summary("loc", summary);

        // 3 hits
        for _ in 0..3 {
            manager.get_for_need("loc", InformationNeed::BasicContext);
        }

        // 2 misses
        for _ in 0..2 {
            manager.get_for_need("unknown", InformationNeed::BasicContext);
        }

        let stats = manager.stats();
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.total_requests, 5);
        assert!((stats.hit_rate() - 0.6).abs() < 0.01);  // 60% hit rate
    }
}
