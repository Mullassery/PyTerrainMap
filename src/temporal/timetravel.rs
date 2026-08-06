//! Time-Travel Queries & Historical Analysis
//!
//! Enable querying terrain state at any historical timestamp with interpolation,
//! change tracking, and temporal statistics.

use super::index::TemporalPoint;
use super::query::QueryOperators;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Snapshot at a specific point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalSnapshot {
    pub timestamp: i64,
    pub points: Vec<TemporalPoint>,
    pub avg_quality: f32,
    pub point_count: u32,
}

impl TemporalSnapshot {
    /// Create snapshot from points
    pub fn new(timestamp: i64, points: Vec<TemporalPoint>) -> Self {
        let avg_quality = if points.is_empty() {
            0.0
        } else {
            points.iter().map(|p| p.quality).sum::<f32>() / points.len() as f32
        };

        let point_count = points.len() as u32;

        TemporalSnapshot {
            timestamp,
            points,
            avg_quality,
            point_count,
        }
    }

    /// Get bounding region of snapshot
    pub fn bounding_region(&self) -> Option<(f32, f32, f32, f32, f32, f32)> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for p in &self.points {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
            min_z = min_z.min(p.z);
            max_z = max_z.max(p.z);
        }

        Some((min_x, max_x, min_y, max_y, min_z, max_z))
    }
}

/// Change between two snapshots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalChange {
    pub before_time: i64,
    pub after_time: i64,
    pub added_points: Vec<TemporalPoint>,
    pub removed_points: Vec<TemporalPoint>,
    pub modified_points: Vec<(TemporalPoint, TemporalPoint)>, // (before, after)
    pub total_change_magnitude: f32,
}

impl TemporalChange {
    /// Create change from two snapshots
    pub fn between_snapshots(before: &TemporalSnapshot, after: &TemporalSnapshot) -> Self {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();
        let mut total_magnitude = 0.0;

        // Find added and modified points
        for after_point in &after.points {
            if let Some(before_point) = before.points.iter().find(|p| {
                (p.x - after_point.x).abs() < 1e-5 && (p.y - after_point.y).abs() < 1e-5
            }) {
                if (before_point.z - after_point.z).abs() > 1e-5 || (before_point.quality - after_point.quality).abs() > 0.01 {
                    let delta_z = (after_point.z - before_point.z).abs();
                    total_magnitude += delta_z;
                    modified.push((*before_point, *after_point));
                }
            } else {
                added.push(*after_point);
            }
        }

        // Find removed points
        for before_point in &before.points {
            if !after.points.iter().any(|p| {
                (p.x - before_point.x).abs() < 1e-5 && (p.y - before_point.y).abs() < 1e-5
            }) {
                removed.push(*before_point);
            }
        }

        TemporalChange {
            before_time: before.timestamp,
            after_time: after.timestamp,
            added_points: added,
            removed_points: removed,
            modified_points: modified,
            total_change_magnitude,
        }
    }

    /// Get change statistics
    pub fn statistics(&self) -> ChangeStatistics {
        ChangeStatistics {
            added_count: self.added_points.len() as u32,
            removed_count: self.removed_points.len() as u32,
            modified_count: self.modified_points.len() as u32,
            total_changes: (self.added_points.len() + self.removed_points.len() + self.modified_points.len()) as u32,
            magnitude: self.total_change_magnitude,
            time_delta_us: self.after_time - self.before_time,
        }
    }
}

/// Change statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeStatistics {
    pub added_count: u32,
    pub removed_count: u32,
    pub modified_count: u32,
    pub total_changes: u32,
    pub magnitude: f32,
    pub time_delta_us: i64,
}

/// Timeline of changes over time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalTimeline {
    pub snapshots: BTreeMap<i64, TemporalSnapshot>,
    pub changes: Vec<TemporalChange>,
}

impl TemporalTimeline {
    /// Create new timeline
    pub fn new() -> Self {
        TemporalTimeline {
            snapshots: BTreeMap::new(),
            changes: Vec::new(),
        }
    }

    /// Add snapshot to timeline
    pub fn add_snapshot(&mut self, snapshot: TemporalSnapshot) {
        // Calculate change from previous snapshot
        if let Some((_, prev_snapshot)) = self.snapshots.iter().next_back() {
            let change = TemporalChange::between_snapshots(prev_snapshot, &snapshot);
            self.changes.push(change);
        }

        self.snapshots.insert(snapshot.timestamp, snapshot);
    }

    /// Get snapshot at exact time (or nearest before)
    pub fn snapshot_at_time(&self, timestamp: i64) -> Option<TemporalSnapshot> {
        // Try exact match first
        if let Some(snapshot) = self.snapshots.get(&timestamp) {
            return Some(snapshot.clone());
        }

        // Find nearest before
        self.snapshots
            .range(..=timestamp)
            .next_back()
            .map(|(_, snapshot)| snapshot.clone())
    }

    /// Get snapshots in time range
    pub fn snapshots_in_range(&self, start_time: i64, end_time: i64) -> Vec<TemporalSnapshot> {
        self.snapshots
            .range(start_time..=end_time)
            .map(|(_, snapshot)| snapshot.clone())
            .collect()
    }

    /// Get all changes affecting a region
    pub fn changes_in_region(
        &self,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        start_time: i64,
        end_time: i64,
    ) -> Vec<TemporalChange> {
        self.changes
            .iter()
            .filter(|change| {
                change.before_time >= start_time
                    && change.after_time <= end_time
                    && (change.added_points.iter().any(|p| {
                        p.x >= x_min && p.x <= x_max && p.y >= y_min && p.y <= y_max
                    }) || change.removed_points.iter().any(|p| {
                        p.x >= x_min && p.x <= x_max && p.y >= y_min && p.y <= y_max
                    }) || change.modified_points.iter().any(|(_, after)| {
                        after.x >= x_min && after.x <= x_max && after.y >= y_min && after.y <= y_max
                    }))
            })
            .cloned()
            .collect()
    }

    /// Get evolution of a specific location
    pub fn location_evolution(&self, x: f32, y: f32) -> Vec<(i64, f32)> {
        // (timestamp, z value) pairs
        let mut evolution = Vec::new();

        for (timestamp, snapshot) in &self.snapshots {
            if let Some(point) = snapshot.points.iter().find(|p| {
                (p.x - x).abs() < 1e-5 && (p.y - y).abs() < 1e-5
            }) {
                evolution.push((*timestamp, point.z));
            }
        }

        evolution
    }

    /// Get temporal statistics
    pub fn statistics(&self) -> TimelineStatistics {
        TimelineStatistics {
            snapshot_count: self.snapshots.len() as u32,
            change_count: self.changes.len() as u32,
            total_points: self.snapshots.values().map(|s| s.point_count).sum(),
            time_span_us: if let (Some(first), Some(last)) = (self.snapshots.keys().next(), self.snapshots.keys().next_back()) {
                last - first
            } else {
                0
            },
        }
    }

    /// Reconstruct state at arbitrary time via interpolation
    pub fn reconstruct_at_time(&self, target_time: i64) -> Option<TemporalSnapshot> {
        // Find bracketing snapshots
        let before = self.snapshots.range(..=target_time).next_back();
        let after = self.snapshots.range((std::ops::Bound::Excluded(target_time), std::ops::Bound::Unbounded)).next();

        match (before, after) {
            (Some((_, before_snap)), Some((_, after_snap))) => {
                // Interpolate between snapshots
                let alpha = (target_time - before_snap.timestamp) as f32
                    / (after_snap.timestamp - before_snap.timestamp) as f32;

                let mut interpolated = Vec::new();

                for before_point in &before_snap.points {
                    if let Some(after_point) = after_snap.points.iter().find(|p| {
                        (p.x - before_point.x).abs() < 1e-5 && (p.y - before_point.y).abs() < 1e-5
                    }) {
                        let interp_point = TemporalPoint::new(
                            before_point.x,
                            before_point.y,
                            before_point.z * (1.0 - alpha) + after_point.z * alpha,
                            target_time,
                            before_point.quality * (1.0 - alpha) + after_point.quality * alpha,
                        );
                        interpolated.push(interp_point);
                    }
                }

                Some(TemporalSnapshot::new(target_time, interpolated))
            }
            (Some((_, snapshot)), None) | (None, Some((_, snapshot))) => Some(snapshot.clone()),
            _ => None,
        }
    }
}

impl Default for TemporalTimeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Timeline statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineStatistics {
    pub snapshot_count: u32,
    pub change_count: u32,
    pub total_points: u32,
    pub time_span_us: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let points = vec![TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8)];
        let snapshot = TemporalSnapshot::new(1000, points);
        assert_eq!(snapshot.point_count, 1);
        assert!((snapshot.avg_quality - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_bounding_region() {
        let points = vec![
            TemporalPoint::new(0.0, 0.0, 0.0, 1000, 0.8),
            TemporalPoint::new(10.0, 10.0, 5.0, 1000, 0.8),
        ];
        let snapshot = TemporalSnapshot::new(1000, points);
        let bbox = snapshot.bounding_region().unwrap();
        assert_eq!(bbox, (0.0, 10.0, 0.0, 10.0, 0.0, 5.0));
    }

    #[test]
    fn test_temporal_change() {
        let before = TemporalSnapshot::new(
            1000,
            vec![TemporalPoint::new(0.0, 0.0, 1.0, 1000, 0.8)],
        );
        let after = TemporalSnapshot::new(
            2000,
            vec![TemporalPoint::new(0.0, 0.0, 2.0, 2000, 0.9)],
        );

        let change = TemporalChange::between_snapshots(&before, &after);
        assert_eq!(change.modified_points.len(), 1);
    }

    #[test]
    fn test_timeline() {
        let mut timeline = TemporalTimeline::new();
        timeline.add_snapshot(TemporalSnapshot::new(1000, vec![TemporalPoint::new(0.0, 0.0, 1.0, 1000, 0.8)]));
        timeline.add_snapshot(TemporalSnapshot::new(2000, vec![TemporalPoint::new(0.0, 0.0, 2.0, 2000, 0.8)]));

        let stats = timeline.statistics();
        assert_eq!(stats.snapshot_count, 2);
        assert_eq!(stats.change_count, 1);
    }

    #[test]
    fn test_location_evolution() {
        let mut timeline = TemporalTimeline::new();
        timeline.add_snapshot(TemporalSnapshot::new(1000, vec![TemporalPoint::new(5.0, 5.0, 1.0, 1000, 0.8)]));
        timeline.add_snapshot(TemporalSnapshot::new(2000, vec![TemporalPoint::new(5.0, 5.0, 2.0, 2000, 0.8)]));
        timeline.add_snapshot(TemporalSnapshot::new(3000, vec![TemporalPoint::new(5.0, 5.0, 3.0, 3000, 0.8)]));

        let evolution = timeline.location_evolution(5.0, 5.0);
        assert_eq!(evolution.len(), 3);
        assert_eq!(evolution[0].1, 1.0); // First z value
        assert_eq!(evolution[2].1, 3.0); // Last z value
    }

    #[test]
    fn test_reconstruct_interpolation() {
        let mut timeline = TemporalTimeline::new();
        timeline.add_snapshot(TemporalSnapshot::new(
            0,
            vec![TemporalPoint::new(0.0, 0.0, 0.0, 0, 1.0)],
        ));
        timeline.add_snapshot(TemporalSnapshot::new(
            2_000_000,
            vec![TemporalPoint::new(0.0, 0.0, 10.0, 2_000_000, 1.0)],
        ));

        let reconstructed = timeline.reconstruct_at_time(1_000_000).unwrap();
        assert_eq!(reconstructed.point_count, 1);
        // Should interpolate to ~5.0
        assert!((reconstructed.points[0].z - 5.0).abs() < 0.1);
    }
}
