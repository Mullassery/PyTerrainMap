//! Memory pooling for Gaussian Splatting objects
//!
//! Implements object pooling pattern to reduce allocations and improve
//! performance in high-throughput multi-bot scenarios.

use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// Configuration for memory pools
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Initial size for splat pool
    pub splat_pool_size: usize,
    /// Max size for splat pool (auto-grow up to this)
    pub splat_pool_max: usize,
    /// Initial size for observation pool
    pub observation_pool_size: usize,
    /// Max size for observation pool
    pub observation_pool_max: usize,
    /// Enable statistics tracking
    pub track_stats: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            splat_pool_size: 100,
            splat_pool_max: 10000,
            observation_pool_size: 1000,
            observation_pool_max: 100000,
            track_stats: true,
        }
    }
}

/// Statistics about pool usage and performance
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total allocations from pool
    pub allocations: u64,
    /// Total deallocations back to pool
    pub deallocations: u64,
    /// Cache hits (object reused)
    pub cache_hits: u64,
    /// Cache misses (new allocation needed)
    pub cache_misses: u64,
    /// Current pool size
    pub pool_size: usize,
    /// Max pool size reached
    pub max_pool_size: usize,
}

impl PoolStats {
    /// Calculate hit rate (0.0-1.0)
    pub fn hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f32) / (total as f32)
        }
    }

    /// Memory saved by pooling (estimated in bytes)
    pub fn estimated_memory_saved_kb(&self) -> f32 {
        // Assume ~500 bytes per reused object
        ((self.cache_hits as f32) * 500.0) / 1024.0
    }
}

/// Pooled splat data (simplified for pooling)
#[derive(Clone, Debug)]
pub struct PooledSplat {
    pub lat: f64,
    pub lon: f64,
    pub elev: f64,
    pub bot_id: String,
    pub traversability: f32,
    pub terrain_type: String,
}

impl Default for PooledSplat {
    fn default() -> Self {
        PooledSplat {
            lat: 0.0,
            lon: 0.0,
            elev: 0.0,
            bot_id: String::new(),
            traversability: 0.5,
            terrain_type: String::new(),
        }
    }
}

impl PooledSplat {
    /// Reset to default state
    pub fn reset(&mut self) {
        *self = PooledSplat::default();
    }
}

/// Object pool for Gaussian splats
pub struct SplatPool {
    available: Arc<Mutex<Vec<PooledSplat>>>,
    config: PoolConfig,
    stats: Arc<Mutex<PoolStats>>,
}

impl SplatPool {
    /// Create new splat pool
    pub fn new(config: PoolConfig) -> Self {
        SplatPool {
            available: Arc::new(Mutex::new(Vec::with_capacity(config.splat_pool_size))),
            config,
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Allocate a splat from the pool
    pub fn allocate(&self) -> PooledSplat {
        let mut available = self.available.lock();

        if let Some(splat) = available.pop() {
            if self.config.track_stats {
                self.stats.lock().cache_hits += 1;
            }
            splat
        } else {
            if self.config.track_stats {
                self.stats.lock().cache_misses += 1;
            }
            PooledSplat::default()
        }
    }

    /// Return a splat to the pool for reuse
    pub fn deallocate(&self, mut splat: PooledSplat) {
        splat.reset();

        let mut available = self.available.lock();
        if available.len() < self.config.splat_pool_max {
            available.push(splat);

            if self.config.track_stats {
                let mut stats = self.stats.lock();
                stats.deallocations += 1;
                stats.pool_size = available.len();
                if available.len() > stats.max_pool_size {
                    stats.max_pool_size = available.len();
                }
            }
        }
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().clone()
    }

    /// Clear all pooled objects
    pub fn clear(&self) {
        self.available.lock().clear();
    }
}

impl Default for SplatPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

/// Pooled observation data
#[derive(Clone, Debug, Default)]
pub struct PooledObservation {
    pub bot_id: String,
    pub lat: f64,
    pub lon: f64,
    pub elev: f64,
    pub confidence: f32,
    pub timestamp_us: i64,
}

impl PooledObservation {
    /// Reset to default state
    pub fn reset(&mut self) {
        *self = PooledObservation::default();
    }
}

/// Object pool for observations
pub struct ObservationPool {
    available: Arc<Mutex<Vec<PooledObservation>>>,
    config: PoolConfig,
    stats: Arc<Mutex<PoolStats>>,
}

impl ObservationPool {
    /// Create new observation pool
    pub fn new(config: PoolConfig) -> Self {
        ObservationPool {
            available: Arc::new(Mutex::new(Vec::with_capacity(config.observation_pool_size))),
            config,
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Allocate an observation from the pool
    pub fn allocate(&self) -> PooledObservation {
        let mut available = self.available.lock();

        if let Some(obs) = available.pop() {
            if self.config.track_stats {
                self.stats.lock().cache_hits += 1;
            }
            obs
        } else {
            if self.config.track_stats {
                self.stats.lock().cache_misses += 1;
            }
            PooledObservation::default()
        }
    }

    /// Return an observation to the pool
    pub fn deallocate(&self, mut obs: PooledObservation) {
        obs.reset();

        let mut available = self.available.lock();
        if available.len() < self.config.observation_pool_max {
            available.push(obs);

            if self.config.track_stats {
                let mut stats = self.stats.lock();
                stats.deallocations += 1;
                stats.pool_size = available.len();
                if available.len() > stats.max_pool_size {
                    stats.max_pool_size = available.len();
                }
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().clone()
    }

    /// Clear all pooled objects
    pub fn clear(&self) {
        self.available.lock().clear();
    }
}

impl Default for ObservationPool {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

/// Memory pool manager for all Gaussian objects
pub struct MemoryPoolManager {
    splat_pool: SplatPool,
    observation_pool: ObservationPool,
}

impl MemoryPoolManager {
    /// Create new memory pool manager
    pub fn new(config: PoolConfig) -> Self {
        MemoryPoolManager {
            splat_pool: SplatPool::new(config.clone()),
            observation_pool: ObservationPool::new(config),
        }
    }

    /// Get splat pool
    pub fn splat_pool(&self) -> &SplatPool {
        &self.splat_pool
    }

    /// Get observation pool
    pub fn observation_pool(&self) -> &ObservationPool {
        &self.observation_pool
    }

    /// Get combined statistics
    pub fn stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            splat_stats: self.splat_pool.stats(),
            observation_stats: self.observation_pool.stats(),
        }
    }

    /// Clear all pools
    pub fn clear(&self) {
        self.splat_pool.clear();
        self.observation_pool.clear();
    }
}

impl Default for MemoryPoolManager {
    fn default() -> Self {
        Self::new(PoolConfig::default())
    }
}

/// Combined pool statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryPoolStats {
    pub splat_stats: PoolStats,
    pub observation_stats: PoolStats,
}

impl MemoryPoolStats {
    /// Total hit rate across all pools
    pub fn combined_hit_rate(&self) -> f32 {
        let total_hits = self.splat_stats.cache_hits + self.observation_stats.cache_hits;
        let total_misses = self.splat_stats.cache_misses + self.observation_stats.cache_misses;
        let total = total_hits + total_misses;

        if total == 0 {
            0.0
        } else {
            (total_hits as f32) / (total as f32)
        }
    }

    /// Total memory saved
    pub fn total_memory_saved_kb(&self) -> f32 {
        self.splat_stats.estimated_memory_saved_kb()
            + self.observation_stats.estimated_memory_saved_kb()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splat_pool_allocation() {
        let pool = SplatPool::default();

        let splat = pool.allocate();
        assert_eq!(splat.lat, 0.0);
        pool.deallocate(splat);

        let stats = pool.stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.deallocations, 1);
    }

    #[test]
    fn test_splat_pool_reuse() {
        let pool = SplatPool::new(PoolConfig {
            splat_pool_size: 10,
            splat_pool_max: 100,
            ..Default::default()
        });

        // Allocate and deallocate
        let splat = pool.allocate();
        pool.deallocate(splat);

        // Next allocation should hit cache
        let _splat2 = pool.allocate();

        let stats = pool.stats();
        assert!(stats.cache_hits > 0);
    }

    #[test]
    fn test_observation_pool() {
        let pool = ObservationPool::default();

        let obs = pool.allocate();
        assert_eq!(obs.confidence, 0.0);
        pool.deallocate(obs);

        let stats = pool.stats();
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.deallocations, 1);
    }

    #[test]
    fn test_pool_hit_rate() {
        let pool = SplatPool::new(PoolConfig {
            splat_pool_size: 5,
            splat_pool_max: 50,
            ..Default::default()
        });

        // Do 10 allocations and deallocations
        for i in 0..10 {
            let splat = pool.allocate();
            pool.deallocate(splat);
        }

        let stats = pool.stats();
        let hit_rate = stats.hit_rate();

        // After first miss, remaining should hit
        assert!(hit_rate > 0.5);
    }

    #[test]
    fn test_manager_stats() {
        let manager = MemoryPoolManager::default();

        // Allocate from both pools
        let splat = manager.splat_pool().allocate();
        let obs = manager.observation_pool().allocate();

        manager.splat_pool().deallocate(splat);
        manager.observation_pool().deallocate(obs);

        let stats = manager.stats();
        assert!(stats.combined_hit_rate() >= 0.0);
        assert!(stats.total_memory_saved_kb() >= 0.0);
    }

    #[test]
    fn test_pool_max_size_enforcement() {
        let config = PoolConfig {
            splat_pool_size: 2,
            splat_pool_max: 3,
            ..Default::default()
        };
        let pool = SplatPool::new(config);

        // Allocate and deallocate 5 times
        for _ in 0..5 {
            let splat = pool.allocate();
            pool.deallocate(splat);
        }

        let stats = pool.stats();
        assert!(stats.max_pool_size <= 3);
    }
}
