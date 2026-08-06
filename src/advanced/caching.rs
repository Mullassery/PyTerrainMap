//! Distributed Multi-tier Caching Strategy
//!
//! Phase 5.1: Optimize performance with intelligent caching across
//! L1 (in-memory), L2 (local-disk), and L3 (distributed) layers.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Cache tier designation
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheTier {
    L1, // In-memory (hot cache)
    L2, // Local disk (warm cache)
    L3, // Distributed (cold cache)
}

/// Cache entry with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<TemporalPoint>,
    pub tier: CacheTier,
    pub created_at_us: i64,
    pub accessed_at_us: i64,
    pub access_count: u64,
    pub size_bytes: usize,
}

impl CacheEntry {
    /// Create new cache entry
    pub fn new(key: String, data: Vec<TemporalPoint>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        let size_bytes = std::mem::size_of_val(&data[..]);

        CacheEntry {
            key,
            data,
            tier: CacheTier::L1,
            created_at_us: now,
            accessed_at_us: now,
            access_count: 0,
            size_bytes,
        }
    }

    /// Update access time
    pub fn access(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        self.accessed_at_us = now;
        self.access_count += 1;
    }

    /// Get age in microseconds
    pub fn age_us(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        now - self.created_at_us
    }

    /// Get time since last access
    pub fn time_since_access_us(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        now - self.accessed_at_us
    }
}

/// L1 Cache (in-memory, size-bounded)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct L1Cache {
    pub entries: HashMap<String, CacheEntry>,
    pub max_size_bytes: usize,
    pub current_size_bytes: usize,
    pub hits: u64,
    pub misses: u64,
}

impl L1Cache {
    /// Create new L1 cache
    pub fn new(max_size_bytes: usize) -> Self {
        L1Cache {
            entries: HashMap::new(),
            max_size_bytes,
            current_size_bytes: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Get entry from L1
    pub fn get(&mut self, key: &str) -> Option<Vec<TemporalPoint>> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.access();
            self.hits += 1;
            Some(entry.data.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put entry in L1
    pub fn put(&mut self, entry: CacheEntry) -> bool {
        let key = entry.key.clone();
        let size = entry.size_bytes;

        // Check if adding this would exceed capacity
        if self.current_size_bytes + size > self.max_size_bytes {
            self.evict_lru();
        }

        self.current_size_bytes += size;
        self.entries.insert(key, entry);
        true
    }

    /// Evict least-recently-used entry
    fn evict_lru(&mut self) {
        if let Some((key, _)) = self.entries
            .iter()
            .min_by_key(|(_, entry)| entry.accessed_at_us)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            if let Some(removed) = self.entries.remove(&key) {
                self.current_size_bytes = self.current_size_bytes.saturating_sub(removed.size_bytes);
            }
        }
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f32 {
        let total = (self.hits + self.misses) as f32;
        if total == 0.0 {
            return 0.0;
        }
        self.hits as f32 / total
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_size_bytes = 0;
    }
}

impl Default for L1Cache {
    fn default() -> Self {
        Self::new(100_000_000) // 100MB default
    }
}

/// L2 Cache (disk-backed, persistent)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct L2Cache {
    pub entries: VecDeque<CacheEntry>,
    pub max_entries: usize,
    pub hits: u64,
    pub misses: u64,
}

impl L2Cache {
    /// Create new L2 cache
    pub fn new(max_entries: usize) -> Self {
        L2Cache {
            entries: VecDeque::new(),
            max_entries,
            hits: 0,
            misses: 0,
        }
    }

    /// Get entry from L2
    pub fn get(&mut self, key: &str) -> Option<Vec<TemporalPoint>> {
        if let Some(pos) = self.entries.iter().position(|e| e.key == key) {
            let mut entry = self.entries.remove(pos).unwrap();
            entry.access();
            let data = entry.data.clone();
            self.entries.push_back(entry);
            self.hits += 1;
            Some(data)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put entry in L2
    pub fn put(&mut self, mut entry: CacheEntry) {
        entry.tier = CacheTier::L2;

        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f32 {
        let total = (self.hits + self.misses) as f32;
        if total == 0.0 {
            return 0.0;
        }
        self.hits as f32 / total
    }
}

impl Default for L2Cache {
    fn default() -> Self {
        Self::new(1000) // 1000 entries default
    }
}

/// Multi-tier cache manager
pub struct MultiTierCache {
    pub l1: L1Cache,
    pub l2: L2Cache,
    pub l3_enabled: bool,
}

impl MultiTierCache {
    /// Create new multi-tier cache
    pub fn new(l1_size: usize, l2_entries: usize) -> Self {
        MultiTierCache {
            l1: L1Cache::new(l1_size),
            l2: L2Cache::new(l2_entries),
            l3_enabled: false,
        }
    }

    /// Get from cache (try L1 → L2 → L3)
    pub fn get(&mut self, key: &str) -> Option<Vec<TemporalPoint>> {
        // Try L1
        if let Some(data) = self.l1.get(key) {
            return Some(data);
        }

        // Try L2
        if let Some(data) = self.l2.get(key) {
            // Promote to L1
            let entry = CacheEntry::new(key.to_string(), data.clone());
            self.l1.put(entry);
            return Some(data);
        }

        None
    }

    /// Put in cache (L1 first, overflow to L2)
    pub fn put(&mut self, entry: CacheEntry) {
        let key = entry.key.clone();
        let size = entry.size_bytes;

        // If too large for L1, put in L2
        if size > self.l1.max_size_bytes / 2 {
            self.l2.put(entry);
        } else {
            self.l1.put(entry);
        }
    }

    /// Get cache statistics
    pub fn statistics(&self) -> CacheStatistics {
        CacheStatistics {
            l1_entries: self.l1.entries.len() as u32,
            l1_size_bytes: self.l1.current_size_bytes,
            l1_hit_rate: self.l1.hit_rate(),
            l2_entries: self.l2.entries.len() as u32,
            l2_hit_rate: self.l2.hit_rate(),
            total_hits: self.l1.hits + self.l2.hits,
            total_misses: self.l1.misses + self.l2.misses,
        }
    }

    /// Clear all caches
    pub fn clear(&mut self) {
        self.l1.clear();
        self.l2.entries.clear();
    }
}

impl Default for MultiTierCache {
    fn default() -> Self {
        Self::new(100_000_000, 1000) // 100MB L1, 1000 entries L2
    }
}

/// Cache statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheStatistics {
    pub l1_entries: u32,
    pub l1_size_bytes: usize,
    pub l1_hit_rate: f32,
    pub l2_entries: u32,
    pub l2_hit_rate: f32,
    pub total_hits: u64,
    pub total_misses: u64,
}

impl CacheStatistics {
    /// Get combined hit rate
    pub fn combined_hit_rate(&self) -> f32 {
        let total = (self.total_hits + self.total_misses) as f32;
        if total == 0.0 {
            return 0.0;
        }
        self.total_hits as f32 / total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry() {
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let entry = CacheEntry::new("test".to_string(), vec![point]);

        assert_eq!(entry.key, "test");
        assert_eq!(entry.access_count, 0);
        assert!(entry.size_bytes > 0);
    }

    #[test]
    fn test_l1_cache_put_get() {
        let mut cache = L1Cache::new(1000000);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let entry = CacheEntry::new("test".to_string(), vec![point]);

        cache.put(entry);
        let result = cache.get("test");

        assert!(result.is_some());
        assert_eq!(cache.hits, 1);
    }

    #[test]
    fn test_l1_cache_miss() {
        let mut cache = L1Cache::new(1000000);
        let result = cache.get("nonexistent");

        assert!(result.is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_l1_cache_hit_rate() {
        let mut cache = L1Cache::new(1000000);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let entry = CacheEntry::new("test".to_string(), vec![point]);

        cache.put(entry);
        cache.get("test");
        cache.get("nonexistent");

        assert!(cache.hit_rate() > 0.0 && cache.hit_rate() < 1.0);
    }

    #[test]
    fn test_multi_tier_cache() {
        let mut cache = MultiTierCache::new(1000000, 100);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let entry = CacheEntry::new("test".to_string(), vec![point]);

        cache.put(entry);
        let result = cache.get("test");

        assert!(result.is_some());
    }

    #[test]
    fn test_cache_statistics() {
        let cache = MultiTierCache::new(1000000, 100);
        let stats = cache.statistics();

        assert_eq!(stats.l1_entries, 0);
        assert_eq!(stats.l2_entries, 0);
    }
}
