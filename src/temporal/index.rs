//! 5D Temporal-Spatial Index Structure
//!
//! Implements R*-tree with temporal dimension treating time as a first-class coordinate.
//! Enables efficient queries in 5D space: (x, y, z, time, quality).

use crate::types::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 5D Point in temporal-spatial space
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemporalPoint {
    /// Spatial coordinates (meters)
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Temporal coordinate (microseconds since epoch)
    pub timestamp: i64,
    /// Quality dimension (0.0-1.0, higher = more confident)
    pub quality: f32,
}

impl TemporalPoint {
    /// Create new temporal point
    pub fn new(x: f32, y: f32, z: f32, timestamp: i64, quality: f32) -> Self {
        TemporalPoint {
            x,
            y,
            z,
            timestamp,
            quality: quality.clamp(0.0, 1.0),
        }
    }

    /// Distance to another point (3D euclidean, ignoring time/quality)
    pub fn spatial_distance(&self, other: &TemporalPoint) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Temporal distance (time difference in seconds)
    pub fn temporal_distance(&self, other: &TemporalPoint) -> f32 {
        ((self.timestamp - other.timestamp).abs() as f32) / 1_000_000.0
    }

    /// Combined 5D distance (normalized)
    pub fn distance_5d(&self, other: &TemporalPoint, temporal_weight: f32) -> f32 {
        let spatial = self.spatial_distance(other);
        let temporal = self.temporal_distance(other);
        (spatial * spatial + (temporal * temporal_weight) * (temporal * temporal_weight)).sqrt()
    }
}

/// Bounding box in 5D space
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BoundingBox5D {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub min_z: f32,
    pub max_z: f32,
    pub min_time: i64,
    pub max_time: i64,
    pub min_quality: f32,
    pub max_quality: f32,
}

impl BoundingBox5D {
    /// Create bounding box from single point
    pub fn from_point(point: &TemporalPoint) -> Self {
        BoundingBox5D {
            min_x: point.x,
            max_x: point.x,
            min_y: point.y,
            max_y: point.y,
            min_z: point.z,
            max_z: point.z,
            min_time: point.timestamp,
            max_time: point.timestamp,
            min_quality: point.quality,
            max_quality: point.quality,
        }
    }

    /// Expand to include another box
    pub fn expand(&mut self, other: &BoundingBox5D) {
        self.min_x = self.min_x.min(other.min_x);
        self.max_x = self.max_x.max(other.max_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_y = self.max_y.max(other.max_y);
        self.min_z = self.min_z.min(other.min_z);
        self.max_z = self.max_z.max(other.max_z);
        self.min_time = self.min_time.min(other.min_time);
        self.max_time = self.max_time.max(other.max_time);
        self.min_quality = self.min_quality.min(other.min_quality);
        self.max_quality = self.max_quality.max(other.max_quality);
    }

    /// Check if point is inside box
    pub fn contains_point(&self, point: &TemporalPoint) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
            && point.z >= self.min_z
            && point.z <= self.max_z
            && point.timestamp >= self.min_time
            && point.timestamp <= self.max_time
            && point.quality >= self.min_quality
            && point.quality <= self.max_quality
    }

    /// Check if box intersects another box
    pub fn intersects(&self, other: &BoundingBox5D) -> bool {
        self.max_x >= other.min_x
            && self.min_x <= other.max_x
            && self.max_y >= other.min_y
            && self.min_y <= other.max_y
            && self.max_z >= other.min_z
            && self.min_z <= other.max_z
            && self.max_time >= other.min_time
            && self.min_time <= other.max_time
            && self.max_quality >= other.min_quality
            && self.min_quality <= other.max_quality
    }

    /// Volume of bounding box (product of all dimensions)
    pub fn volume(&self) -> f32 {
        let spatial_vol = (self.max_x - self.min_x)
            * (self.max_y - self.min_y)
            * (self.max_z - self.min_z);
        let temporal_vol = ((self.max_time - self.min_time) as f32 / 1_000_000.0); // Convert to seconds
        let quality_vol = self.max_quality - self.min_quality;
        spatial_vol * temporal_vol * quality_vol
    }
}

/// Temporal index leaf node (stores points)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexLeaf {
    pub points: Vec<TemporalPoint>,
    pub bbox: BoundingBox5D,
}

impl IndexLeaf {
    /// Create new leaf
    pub fn new() -> Self {
        IndexLeaf {
            points: Vec::new(),
            bbox: BoundingBox5D {
                min_x: f32::INFINITY,
                max_x: f32::NEG_INFINITY,
                min_y: f32::INFINITY,
                max_y: f32::NEG_INFINITY,
                min_z: f32::INFINITY,
                max_z: f32::NEG_INFINITY,
                min_time: i64::MAX,
                max_time: i64::MIN,
                min_quality: 1.0,
                max_quality: 0.0,
            },
        }
    }

    /// Add point to leaf
    pub fn add_point(&mut self, point: TemporalPoint) {
        if self.points.is_empty() {
            self.bbox = BoundingBox5D::from_point(&point);
        } else {
            let point_bbox = BoundingBox5D::from_point(&point);
            self.bbox.expand(&point_bbox);
        }
        self.points.push(point);
    }

    /// Find points in spatial-temporal range
    pub fn range_query(&self, bbox: &BoundingBox5D) -> Vec<TemporalPoint> {
        self.points
            .iter()
            .filter(|p| bbox.contains_point(p))
            .copied()
            .collect()
    }

    /// Get leaf size
    pub fn size(&self) -> usize {
        self.points.len()
    }
}

impl Default for IndexLeaf {
    fn default() -> Self {
        Self::new()
    }
}

/// 5D Temporal-Spatial Index (simplified R*-tree)
pub struct TemporalSpatialIndex {
    /// Leaf nodes (simplified: single level for now)
    leaves: Vec<IndexLeaf>,
    /// Maximum points per leaf
    max_leaf_size: usize,
    /// Index statistics
    pub total_points: u64,
    pub total_inserts: u64,
}

impl TemporalSpatialIndex {
    /// Create new index
    pub fn new(max_leaf_size: usize) -> Self {
        TemporalSpatialIndex {
            leaves: vec![IndexLeaf::new()],
            max_leaf_size,
            total_points: 0,
            total_inserts: 0,
        }
    }

    /// Insert point into index
    pub fn insert(&mut self, point: TemporalPoint) -> Result<()> {
        // Find best leaf (simplified: first with space)
        let mut inserted = false;

        for leaf in &mut self.leaves {
            if leaf.size() < self.max_leaf_size {
                leaf.add_point(point);
                inserted = true;
                break;
            }
        }

        // Create new leaf if needed
        if !inserted {
            let mut new_leaf = IndexLeaf::new();
            new_leaf.add_point(point);
            self.leaves.push(new_leaf);
        }

        self.total_points += 1;
        self.total_inserts += 1;
        Ok(())
    }

    /// Range query in 5D space
    pub fn range_query(&self, bbox: &BoundingBox5D) -> Vec<TemporalPoint> {
        let mut results = Vec::new();
        for leaf in &self.leaves {
            if leaf.bbox.intersects(bbox) {
                results.extend(leaf.range_query(bbox));
            }
        }
        results
    }

    /// Spatial query at specific time
    pub fn query_at_time(
        &self,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
        min_z: f32,
        max_z: f32,
        timestamp: i64,
    ) -> Vec<TemporalPoint> {
        let bbox = BoundingBox5D {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
            min_time: timestamp,
            max_time: timestamp,
            min_quality: 0.0,
            max_quality: 1.0,
        };
        self.range_query(&bbox)
    }

    /// Temporal query (all points in time range)
    pub fn query_time_range(&self, start_time: i64, end_time: i64) -> Vec<TemporalPoint> {
        let bbox = BoundingBox5D {
            min_x: f32::NEG_INFINITY,
            max_x: f32::INFINITY,
            min_y: f32::NEG_INFINITY,
            max_y: f32::INFINITY,
            min_z: f32::NEG_INFINITY,
            max_z: f32::INFINITY,
            min_time: start_time,
            max_time: end_time,
            min_quality: 0.0,
            max_quality: 1.0,
        };
        self.range_query(&bbox)
    }

    /// Nearest neighbor search
    pub fn nearest_neighbor(&self, point: &TemporalPoint, max_distance: f32) -> Option<TemporalPoint> {
        let bbox = BoundingBox5D {
            min_x: point.x - max_distance,
            max_x: point.x + max_distance,
            min_y: point.y - max_distance,
            max_y: point.y + max_distance,
            min_z: point.z - max_distance,
            max_z: point.z + max_distance,
            min_time: point.timestamp - (max_distance as i64 * 1_000_000),
            max_time: point.timestamp + (max_distance as i64 * 1_000_000),
            min_quality: 0.0,
            max_quality: 1.0,
        };

        let candidates = self.range_query(&bbox);
        candidates
            .iter()
            .min_by(|a, b| {
                let dist_a = point.distance_5d(a, 1.0);
                let dist_b = point.distance_5d(b, 1.0);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }

    /// Get index statistics
    pub fn statistics(&self) -> IndexStatistics {
        IndexStatistics {
            total_points: self.total_points,
            total_inserts: self.total_inserts,
            leaf_count: self.leaves.len() as u32,
            avg_points_per_leaf: if self.leaves.is_empty() {
                0.0
            } else {
                self.total_points as f32 / self.leaves.len() as f32
            },
        }
    }

    /// Clear index
    pub fn clear(&mut self) {
        self.leaves.clear();
        self.leaves.push(IndexLeaf::new());
        self.total_points = 0;
    }

    /// Get number of leaves
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }
}

impl Default for TemporalSpatialIndex {
    fn default() -> Self {
        Self::new(1000) // 1000 points per leaf
    }
}

/// Index statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_points: u64,
    pub total_inserts: u64,
    pub leaf_count: u32,
    pub avg_points_per_leaf: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_point_creation() {
        let p = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.timestamp, 1000);
        assert_eq!(p.quality, 0.8);
    }

    #[test]
    fn test_spatial_distance() {
        let p1 = TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.5);
        let p2 = TemporalPoint::new(3.0, 4.0, 0.0, 1000, 0.5);
        assert!((p1.spatial_distance(&p2) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_bounding_box() {
        let p = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.5);
        let bbox = BoundingBox5D::from_point(&p);
        assert!(bbox.contains_point(&p));
    }

    #[test]
    fn test_index_insert() {
        let mut index = TemporalSpatialIndex::new(10);
        let p = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        assert!(index.insert(p).is_ok());
        assert_eq!(index.total_points, 1);
    }

    #[test]
    fn test_range_query() {
        let mut index = TemporalSpatialIndex::new(100);
        for i in 0..10 {
            let p = TemporalPoint::new(i as f32, i as f32, 0.0, 1000 + i as i64, 0.8);
            index.insert(p).unwrap();
        }

        let bbox = BoundingBox5D {
            min_x: 2.0,
            max_x: 5.0,
            min_y: 2.0,
            max_y: 5.0,
            min_z: -1.0,
            max_z: 1.0,
            min_time: 1000,
            max_time: 2000,
            min_quality: 0.0,
            max_quality: 1.0,
        };

        let results = index.range_query(&bbox);
        assert!(results.len() > 0);
    }

    #[test]
    fn test_nearest_neighbor() {
        let mut index = TemporalSpatialIndex::new(100);
        index.insert(TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8)).unwrap();
        index.insert(TemporalPoint::new(10.0, 10.0, 0.0, 1000, 0.8)).unwrap();

        let query = TemporalPoint::new(0.5, 0.5, 0.0, 1000, 0.8);
        let nearest = index.nearest_neighbor(&query, 100.0);
        assert!(nearest.is_some());
    }

    #[test]
    fn test_statistics() {
        let mut index = TemporalSpatialIndex::new(10);
        for i in 0..25 {
            let p = TemporalPoint::new(i as f32, 0.0, 0.0, i as i64, 0.8);
            index.insert(p).unwrap();
        }

        let stats = index.statistics();
        assert_eq!(stats.total_points, 25);
        assert!(stats.leaf_count > 1);
    }
}
