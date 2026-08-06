//! Multi-Resolution Temporal Storage Tiers
//!
//! Implements a tiered storage system with different temporal resolutions:
//! - Tier 1 (High-res): Recent data, 1-second resolution
//! - Tier 2 (Medium-res): 1-week window, 1-minute resolution
//! - Tier 3 (Low-res): Archive, 1-hour resolution

use super::index::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Storage tier type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageTier {
    HighResolution,   // Last 1 hour, 1-second intervals
    MediumResolution, // Last 1 week, 1-minute intervals
    LowResolution,    // Archive, 1-hour intervals
}

impl StorageTier {
    /// Get retention time in microseconds
    pub fn retention_us(&self) -> i64 {
        match self {
            StorageTier::HighResolution => 3600 * 1_000_000, // 1 hour
            StorageTier::MediumResolution => 7 * 24 * 3600 * 1_000_000, // 1 week
            StorageTier::LowResolution => 365 * 24 * 3600 * 1_000_000, // 1 year
        }
    }

    /// Get coarsening interval in microseconds
    pub fn coarsen_interval_us(&self) -> i64 {
        match self {
            StorageTier::HighResolution => 1_000_000, // 1 second
            StorageTier::MediumResolution => 60 * 1_000_000, // 1 minute
            StorageTier::LowResolution => 3600 * 1_000_000, // 1 hour
        }
    }
}

/// Temporal storage tier
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalStorageTier {
    pub tier: StorageTier,
    pub points: VecDeque<TemporalPoint>,
    pub max_points: usize,
    pub insertion_count: u64,
}

impl TemporalStorageTier {
    /// Create new storage tier
    pub fn new(tier: StorageTier) -> Self {
        let max_points = match tier {
            StorageTier::HighResolution => 3600,   // 3600 1-second points
            StorageTier::MediumResolution => 10080, // 10080 1-minute points
            StorageTier::LowResolution => 8760,    // 8760 1-hour points
        };

        TemporalStorageTier {
            tier,
            points: VecDeque::new(),
            max_points,
            insertion_count: 0,
        }
    }

    /// Insert point (with automatic coarsening to tier resolution)
    pub fn insert(&mut self, point: TemporalPoint) {
        self.points.push_back(point);
        self.insertion_count += 1;

        // Remove oldest if over capacity
        while self.points.len() > self.max_points {
            self.points.pop_front();
        }
    }

    /// Coarsen points at this tier's resolution
    pub fn coarsen(&self) -> Vec<TemporalPoint> {
        if self.points.is_empty() {
            return Vec::new();
        }

        let interval = self.tier.coarsen_interval_us();
        let mut coarsened = Vec::new();
        let mut current_bucket_start = self.points[0].timestamp;
        let mut bucket_points = Vec::new();

        for point in &self.points {
            if point.timestamp - current_bucket_start >= interval {
                // Aggregate bucket
                if !bucket_points.is_empty() {
                    let avg_x = bucket_points.iter().map(|p: &TemporalPoint| p.x).sum::<f32>() / bucket_points.len() as f32;
                    let avg_y = bucket_points.iter().map(|p: &TemporalPoint| p.y).sum::<f32>() / bucket_points.len() as f32;
                    let avg_z = bucket_points.iter().map(|p: &TemporalPoint| p.z).sum::<f32>() / bucket_points.len() as f32;
                    let avg_quality = bucket_points.iter().map(|p: &TemporalPoint| p.quality).sum::<f32>() / bucket_points.len() as f32;
                    let mid_time = current_bucket_start + interval / 2;

                    coarsened.push(TemporalPoint::new(avg_x, avg_y, avg_z, mid_time, avg_quality));
                }

                // Start new bucket
                current_bucket_start = point.timestamp;
                bucket_points.clear();
            }

            bucket_points.push(*point);
        }

        // Final bucket
        if !bucket_points.is_empty() {
            let avg_x = bucket_points.iter().map(|p: &TemporalPoint| p.x).sum::<f32>() / bucket_points.len() as f32;
            let avg_y = bucket_points.iter().map(|p: &TemporalPoint| p.y).sum::<f32>() / bucket_points.len() as f32;
            let avg_z = bucket_points.iter().map(|p: &TemporalPoint| p.z).sum::<f32>() / bucket_points.len() as f32;
            let avg_quality = bucket_points.iter().map(|p: &TemporalPoint| p.quality).sum::<f32>() / bucket_points.len() as f32;

            coarsened.push(TemporalPoint::new(avg_x, avg_y, avg_z, current_bucket_start, avg_quality));
        }

        coarsened
    }

    /// Query range in this tier
    pub fn query_range(&self, start_time: i64, end_time: i64) -> Vec<TemporalPoint> {
        self.points
            .iter()
            .filter(|p| p.timestamp >= start_time && p.timestamp <= end_time)
            .copied()
            .collect()
    }

    /// Get storage utilization ratio
    pub fn utilization(&self) -> f32 {
        self.points.len() as f32 / self.max_points as f32
    }

    /// Get tier size (bytes estimate)
    pub fn size_bytes(&self) -> usize {
        self.points.len() * std::mem::size_of::<TemporalPoint>()
    }
}

/// Multi-tier temporal storage pyramid
pub struct TemporalStoragePyramid {
    pub high_res: TemporalStorageTier,
    pub medium_res: TemporalStorageTier,
    pub low_res: TemporalStorageTier,
    pub total_inserts: u64,
}

impl TemporalStoragePyramid {
    /// Create new storage pyramid
    pub fn new() -> Self {
        TemporalStoragePyramid {
            high_res: TemporalStorageTier::new(StorageTier::HighResolution),
            medium_res: TemporalStorageTier::new(StorageTier::MediumResolution),
            low_res: TemporalStorageTier::new(StorageTier::LowResolution),
            total_inserts: 0,
        }
    }

    /// Insert point into pyramid (goes to all tiers)
    pub fn insert(&mut self, point: TemporalPoint) {
        self.high_res.insert(point);
        self.total_inserts += 1;

        // Promote to medium-res after data ages out of high-res
        let high_retention = StorageTier::HighResolution.retention_us();
        if self.high_res.points.is_empty() {
            return;
        }

        let oldest = self.high_res.points[0].timestamp;
        let now = self.high_res.points[self.high_res.points.len() - 1].timestamp;

        if now - oldest > high_retention {
            // Coarsen and promote to medium-res
            for coarse_point in self.high_res.coarsen() {
                self.medium_res.insert(coarse_point);
            }
        }
    }

    /// Query across pyramid (uses best tier)
    pub fn query_range(&self, start_time: i64, end_time: i64) -> Vec<TemporalPoint> {
        // Check which tier has best coverage
        let high_results = self.high_res.query_range(start_time, end_time);
        if !high_results.is_empty() {
            return high_results;
        }

        let medium_results = self.medium_res.query_range(start_time, end_time);
        if !medium_results.is_empty() {
            return medium_results;
        }

        self.low_res.query_range(start_time, end_time)
    }

    /// Promote medium-res to high-res (for recent queries of older data)
    pub fn promote_to_high_res(&mut self, num_points: usize) {
        let to_promote: Vec<_> = self.medium_res.points
            .iter()
            .rev()
            .take(num_points)
            .copied()
            .collect();

        for point in to_promote.iter().rev() {
            self.high_res.insert(*point);
        }
    }

    /// Get pyramid statistics
    pub fn statistics(&self) -> PyramidStatistics {
        PyramidStatistics {
            high_res_count: self.high_res.points.len() as u32,
            medium_res_count: self.medium_res.points.len() as u32,
            low_res_count: self.low_res.points.len() as u32,
            total_points: (self.high_res.points.len() + self.medium_res.points.len() + self.low_res.points.len()) as u32,
            total_inserts: self.total_inserts,
            total_size_bytes: self.high_res.size_bytes() + self.medium_res.size_bytes() + self.low_res.size_bytes(),
        }
    }

    /// Clear pyramid
    pub fn clear(&mut self) {
        self.high_res.points.clear();
        self.medium_res.points.clear();
        self.low_res.points.clear();
        self.total_inserts = 0;
    }
}

impl Default for TemporalStoragePyramid {
    fn default() -> Self {
        Self::new()
    }
}

/// Pyramid statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PyramidStatistics {
    pub high_res_count: u32,
    pub medium_res_count: u32,
    pub low_res_count: u32,
    pub total_points: u32,
    pub total_inserts: u64,
    pub total_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_tier_creation() {
        let tier = TemporalStorageTier::new(StorageTier::HighResolution);
        assert_eq!(tier.tier, StorageTier::HighResolution);
        assert_eq!(tier.points.len(), 0);
    }

    #[test]
    fn test_tier_insert() {
        let mut tier = TemporalStorageTier::new(StorageTier::HighResolution);
        let p = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        tier.insert(p);
        assert_eq!(tier.points.len(), 1);
    }

    #[test]
    fn test_tier_capacity() {
        let mut tier = TemporalStorageTier::new(StorageTier::HighResolution);
        for i in 0..4000 {
            let p = TemporalPoint::new(i as f32, 0.0, 0.0, i as i64 * 1000, 0.8);
            tier.insert(p);
        }
        assert!(tier.points.len() <= tier.max_points);
    }

    #[test]
    fn test_coarsen() {
        let mut tier = TemporalStorageTier::new(StorageTier::HighResolution);
        for i in 0..10 {
            let p = TemporalPoint::new(i as f32, 0.0, 0.0, i as i64 * 500_000, 0.8);
            tier.insert(p);
        }
        let coarsened = tier.coarsen();
        assert!(coarsened.len() > 0);
    }

    #[test]
    fn test_pyramid() {
        let mut pyramid = TemporalStoragePyramid::new();
        for i in 0..100 {
            let p = TemporalPoint::new(i as f32, 0.0, 0.0, i as i64 * 10_000, 0.8);
            pyramid.insert(p);
        }

        let stats = pyramid.statistics();
        assert_eq!(stats.total_inserts, 100);
    }

    #[test]
    fn test_query_range() {
        let mut pyramid = TemporalStoragePyramid::new();
        for i in 0..50 {
            let p = TemporalPoint::new(i as f32, 0.0, 0.0, 1000 + i as i64 * 10_000, 0.8);
            pyramid.insert(p);
        }

        let results = pyramid.query_range(1000, 500_000);
        assert!(!results.is_empty());
    }
}
