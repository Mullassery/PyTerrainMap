//! Temporal Query Operators
//!
//! High-level query operators for common temporal-spatial patterns.

use super::index::{TemporalPoint, BoundingBox5D, TemporalSpatialIndex};
use serde::{Deserialize, Serialize};

/// Query result with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalQueryResult {
    pub points: Vec<TemporalPoint>,
    pub query_time_us: u64, // Query execution time in microseconds
    pub result_count: u32,
    pub quality_average: f32,
}

impl TemporalQueryResult {
    /// Create result
    pub fn new(points: Vec<TemporalPoint>, quality_average: f32) -> Self {
        let result_count = points.len() as u32;
        TemporalQueryResult {
            points,
            query_time_us: 0,
            result_count,
            quality_average,
        }
    }

    /// Get average quality of results
    pub fn avg_quality(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        self.points.iter().map(|p| p.quality).sum::<f32>() / self.points.len() as f32
    }

    /// Filter results by quality threshold
    pub fn filter_by_quality(&self, min_quality: f32) -> Vec<TemporalPoint> {
        self.points
            .iter()
            .filter(|p| p.quality >= min_quality)
            .copied()
            .collect()
    }

    /// Get latest point (max timestamp)
    pub fn latest(&self) -> Option<TemporalPoint> {
        self.points
            .iter()
            .max_by_key(|p| p.timestamp)
            .copied()
    }

    /// Get earliest point (min timestamp)
    pub fn earliest(&self) -> Option<TemporalPoint> {
        self.points
            .iter()
            .min_by_key(|p| p.timestamp)
            .copied()
    }
}

/// Query operators
pub struct QueryOperators;

impl QueryOperators {
    /// At-time query: Get data at specific timestamp
    pub fn at_time(
        index: &TemporalSpatialIndex,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        z_min: f32,
        z_max: f32,
        timestamp: i64,
    ) -> TemporalQueryResult {
        let start = std::time::Instant::now();

        let bbox = BoundingBox5D {
            min_x: x_min,
            max_x: x_max,
            min_y: y_min,
            max_y: y_max,
            min_z: z_min,
            max_z: z_max,
            min_time: timestamp,
            max_time: timestamp,
            min_quality: 0.0,
            max_quality: 1.0,
        };

        let points = index.range_query(&bbox);
        let avg_quality = if points.is_empty() {
            0.0
        } else {
            points.iter().map(|p| p.quality).sum::<f32>() / points.len() as f32
        };

        let mut result = TemporalQueryResult::new(points, avg_quality);
        result.query_time_us = start.elapsed().as_micros() as u64;
        result
    }

    /// Time-range query: Get all data in time interval
    pub fn time_range(
        index: &TemporalSpatialIndex,
        start_time: i64,
        end_time: i64,
    ) -> TemporalQueryResult {
        let start = std::time::Instant::now();

        let points = index.query_time_range(start_time, end_time);
        let avg_quality = if points.is_empty() {
            0.0
        } else {
            points.iter().map(|p| p.quality).sum::<f32>() / points.len() as f32
        };

        let mut result = TemporalQueryResult::new(points, avg_quality);
        result.query_time_us = start.elapsed().as_micros() as u64;
        result
    }

    /// Spatio-temporal range: Get data in region and time interval
    pub fn spatio_temporal_range(
        index: &TemporalSpatialIndex,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        z_min: f32,
        z_max: f32,
        start_time: i64,
        end_time: i64,
    ) -> TemporalQueryResult {
        let start = std::time::Instant::now();

        let bbox = BoundingBox5D {
            min_x: x_min,
            max_x: x_max,
            min_y: y_min,
            max_y: y_max,
            min_z: z_min,
            max_z: z_max,
            min_time: start_time,
            max_time: end_time,
            min_quality: 0.0,
            max_quality: 1.0,
        };

        let points = index.range_query(&bbox);
        let avg_quality = if points.is_empty() {
            0.0
        } else {
            points.iter().map(|p| p.quality).sum::<f32>() / points.len() as f32
        };

        let mut result = TemporalQueryResult::new(points, avg_quality);
        result.query_time_us = start.elapsed().as_micros() as u64;
        result
    }

    /// High-quality only: Filter results by quality threshold
    pub fn high_quality(
        index: &TemporalSpatialIndex,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        z_min: f32,
        z_max: f32,
        start_time: i64,
        end_time: i64,
        min_quality: f32,
    ) -> TemporalQueryResult {
        let mut result = Self::spatio_temporal_range(
            index, x_min, x_max, y_min, y_max, z_min, z_max, start_time, end_time,
        );
        result.points.retain(|p| p.quality >= min_quality);
        result.result_count = result.points.len() as u32;
        result
    }

    /// Temporal interpolation: Estimate value between two timestamps
    pub fn interpolate(p1: &TemporalPoint, p2: &TemporalPoint, target_time: i64) -> Option<TemporalPoint> {
        if target_time < p1.timestamp || target_time > p2.timestamp {
            return None;
        }

        let t1 = p1.timestamp as f32;
        let t2 = p2.timestamp as f32;
        let t_target = target_time as f32;

        if (t2 - t1).abs() < 1.0 {
            return Some(*p1);
        }

        let alpha = (t_target - t1) / (t2 - t1);

        Some(TemporalPoint {
            x: p1.x * (1.0 - alpha) + p2.x * alpha,
            y: p1.y * (1.0 - alpha) + p2.y * alpha,
            z: p1.z * (1.0 - alpha) + p2.z * alpha,
            timestamp: target_time,
            quality: (p1.quality * (1.0 - alpha) + p2.quality * alpha).clamp(0.0, 1.0),
        })
    }

    /// Temporal difference: Show how data changed between two times
    pub fn temporal_diff(before: &[TemporalPoint], after: &[TemporalPoint]) -> TemporalDiff {
        let added = after.iter().filter(|a| !before.iter().any(|b| {
            (a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5 && (a.z - b.z).abs() < 1e-5
        })).copied().collect();

        let removed = before.iter().filter(|b| !after.iter().any(|a| {
            (a.x - b.x).abs() < 1e-5 && (a.y - b.y).abs() < 1e-5 && (a.z - b.z).abs() < 1e-5
        })).copied().collect();

        TemporalDiff { added, removed }
    }
}

/// Temporal difference result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalDiff {
    pub added: Vec<TemporalPoint>,
    pub removed: Vec<TemporalPoint>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_avg_quality() {
        let points = vec![
            TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 0.0, 1000, 0.6),
        ];
        let result = TemporalQueryResult::new(points, 0.7);
        assert!((result.avg_quality() - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_filter_by_quality() {
        let points = vec![
            TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 0.0, 1000, 0.4),
        ];
        let result = TemporalQueryResult::new(points, 0.6);
        let filtered = result.filter_by_quality(0.6);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_interpolate() {
        let p1 = TemporalPoint::new(0.0, 0.0, 0.0, 0, 0.8);
        let p2 = TemporalPoint::new(10.0, 0.0, 0.0, 1_000_000, 0.6);
        let interp = QueryOperators::interpolate(&p1, &p2, 500_000).unwrap();

        assert!((interp.x - 5.0).abs() < 1e-5);
        assert!((interp.quality - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_latest_earliest() {
        let points = vec![
            TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8),
            TemporalPoint::new(1.0, 1.0, 0.0, 2000, 0.6),
            TemporalPoint::new(2.0, 2.0, 0.0, 1500, 0.7),
        ];
        let result = TemporalQueryResult::new(points, 0.7);

        let latest = result.latest().unwrap();
        assert_eq!(latest.timestamp, 2000);

        let earliest = result.earliest().unwrap();
        assert_eq!(earliest.timestamp, 1000);
    }
}
