//! Change detection for temporal analysis of 3D scenes
//!
//! Compares point clouds over time, detects spatial changes,
//! generates change masks, and tracks temporal evolution.

use serde::{Deserialize, Serialize};

/// Change detection result between two point clouds
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeDetectionResult {
    /// Baseline snapshot ID
    pub baseline_snapshot_id: String,
    /// Current snapshot ID
    pub current_snapshot_id: String,
    /// Timestamp of baseline
    pub baseline_timestamp: i64,
    /// Timestamp of current
    pub current_timestamp: i64,
    /// Time delta (microseconds)
    pub time_delta_us: i64,
    /// Changed points (indices in current cloud)
    pub changed_point_indices: Vec<usize>,
    /// Unchanged points
    pub unchanged_point_indices: Vec<usize>,
    /// Added points (in current, not in baseline)
    pub added_points: Vec<usize>,
    /// Removed points (in baseline, not in current)
    pub removed_points: Vec<usize>,
    /// Change statistics
    pub statistics: ChangeStatistics,
    /// Change mask (spatial grid of changes)
    pub change_mask: Option<ChangeMask>,
}

impl ChangeDetectionResult {
    /// Create change detection result
    pub fn new(
        baseline_id: String,
        current_id: String,
        baseline_ts: i64,
        current_ts: i64,
    ) -> Self {
        ChangeDetectionResult {
            baseline_snapshot_id: baseline_id,
            current_snapshot_id: current_id,
            baseline_timestamp: baseline_ts,
            current_timestamp: current_ts,
            time_delta_us: current_ts - baseline_ts,
            changed_point_indices: Vec::new(),
            unchanged_point_indices: Vec::new(),
            added_points: Vec::new(),
            removed_points: Vec::new(),
            statistics: ChangeStatistics::default(),
            change_mask: None,
        }
    }

    /// Get change percentage
    pub fn change_percentage(&self) -> f32 {
        let total = self.changed_point_indices.len() + self.unchanged_point_indices.len();
        if total == 0 {
            return 0.0;
        }
        (self.changed_point_indices.len() as f32 / total as f32) * 100.0
    }

    /// Get rate of change (percentage per second)
    pub fn change_rate_per_second(&self) -> f32 {
        if self.time_delta_us == 0 {
            return 0.0;
        }
        let seconds = self.time_delta_us as f32 / 1_000_000.0;
        self.change_percentage() / seconds
    }
}

/// Change statistics
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct ChangeStatistics {
    /// Total points in baseline
    pub baseline_point_count: u32,
    /// Total points in current
    pub current_point_count: u32,
    /// Number of changed points
    pub changed_count: u32,
    /// Number of unchanged points
    pub unchanged_count: u32,
    /// Number of added points
    pub added_count: u32,
    /// Number of removed points
    pub removed_count: u32,
    /// Average change magnitude (distance moved)
    pub avg_movement_meters: f32,
    /// Maximum change magnitude
    pub max_movement_meters: f32,
    /// Percentage of scene that changed
    pub change_percentage: f32,
}

/// Spatial change mask (grid-based change representation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeMask {
    /// Grid resolution (cells per dimension)
    pub resolution: u32,
    /// Cell size in meters
    pub cell_size_meters: f32,
    /// Change intensity per cell (0.0-1.0)
    pub cells: Vec<Vec<Vec<f32>>>, // [x][y][z]
    /// Bounding box min
    pub bounds_min: (f32, f32, f32),
    /// Bounding box max
    pub bounds_max: (f32, f32, f32),
}

impl ChangeMask {
    /// Create change mask
    pub fn new(
        resolution: u32,
        cell_size: f32,
        bounds_min: (f32, f32, f32),
        bounds_max: (f32, f32, f32),
    ) -> Self {
        let cells = vec![vec![vec![0.0; resolution as usize]; resolution as usize]; resolution as usize];
        ChangeMask {
            resolution,
            cell_size_meters: cell_size,
            cells,
            bounds_min,
            bounds_max,
        }
    }

    /// Mark cell as changed
    pub fn mark_changed(&mut self, x: usize, y: usize, z: usize, intensity: f32) {
        if x < self.resolution as usize && y < self.resolution as usize && z < self.resolution as usize {
            self.cells[x][y][z] = self.cells[x][y][z].max(intensity);
        }
    }

    /// Get change intensity at position
    pub fn get_change_intensity(&self, x: usize, y: usize, z: usize) -> f32 {
        if x < self.resolution as usize && y < self.resolution as usize && z < self.resolution as usize {
            self.cells[x][y][z]
        } else {
            0.0
        }
    }

    /// Get total change energy
    pub fn total_change_energy(&self) -> f32 {
        self.cells.iter()
            .flat_map(|x| x.iter().flat_map(|y| y.iter()))
            .sum()
    }

    /// Get changed cell count
    pub fn changed_cell_count(&self) -> usize {
        self.cells.iter()
            .flat_map(|x| x.iter().flat_map(|y| y.iter()))
            .filter(|&&intensity| intensity > 0.0)
            .count()
    }
}

/// Change heatmap for visualization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeHeatmap {
    /// Timestamp of snapshot
    pub timestamp: i64,
    /// 2D projection of changes (top-down view)
    pub heatmap_grid: Vec<Vec<f32>>, // [x][y]
    /// Grid resolution
    pub resolution: u32,
    /// Cell size in meters
    pub cell_size_meters: f32,
    /// Total change energy
    pub total_energy: f32,
    /// Peak change intensity
    pub peak_intensity: f32,
}

impl ChangeHeatmap {
    /// Create heatmap
    pub fn new(resolution: u32, cell_size: f32) -> Self {
        let heatmap_grid = vec![vec![0.0; resolution as usize]; resolution as usize];
        ChangeHeatmap {
            timestamp: chrono::Utc::now().timestamp_micros(),
            heatmap_grid,
            resolution,
            cell_size_meters: cell_size,
            total_energy: 0.0,
            peak_intensity: 0.0,
        }
    }

    /// Add change to heatmap
    pub fn add_change(&mut self, x: usize, y: usize, intensity: f32) {
        if x < self.resolution as usize && y < self.resolution as usize {
            self.heatmap_grid[x][y] = (self.heatmap_grid[x][y] + intensity).min(1.0);
            self.peak_intensity = self.peak_intensity.max(intensity);
        }
    }

    /// Normalize heatmap
    pub fn normalize(&mut self) {
        if self.peak_intensity > 0.0 {
            for row in &mut self.heatmap_grid {
                for cell in row {
                    *cell /= self.peak_intensity;
                }
            }
        }
        self.total_energy = self.heatmap_grid.iter()
            .flat_map(|row| row.iter())
            .sum();
    }
}

/// Change detector for 3D point clouds
pub struct ChangeDetector {
    /// Distance threshold for point matching (meters)
    pub match_threshold_meters: f32,
    /// Minimum change magnitude to register as changed (meters)
    pub change_threshold_meters: f32,
    /// Enable change mask generation
    pub generate_mask: bool,
    /// Change mask resolution (cells per dimension)
    pub mask_resolution: u32,
}

impl ChangeDetector {
    /// Create change detector
    pub fn new() -> Self {
        ChangeDetector {
            match_threshold_meters: 0.1,
            change_threshold_meters: 0.05,
            generate_mask: true,
            mask_resolution: 32,
        }
    }

    /// Detect changes between baseline and current point clouds
    pub fn detect_changes(
        &self,
        baseline_id: &str,
        current_id: &str,
        baseline_points: &[(f32, f32, f32, u8, u8, u8)], // (x, y, z, r, g, b)
        current_points: &[(f32, f32, f32, u8, u8, u8)],
        baseline_ts: i64,
        current_ts: i64,
    ) -> ChangeDetectionResult {
        let mut result = ChangeDetectionResult::new(
            baseline_id.to_string(),
            current_id.to_string(),
            baseline_ts,
            current_ts,
        );

        result.statistics.baseline_point_count = baseline_points.len() as u32;
        result.statistics.current_point_count = current_points.len() as u32;

        // Find bounds
        let (bounds_min, bounds_max) = self.find_bounds(baseline_points, current_points);

        // Initialize change mask if needed
        let mut mask = if self.generate_mask {
            Some(ChangeMask::new(
                self.mask_resolution,
                ((bounds_max.0 - bounds_min.0) / self.mask_resolution as f32).abs(),
                bounds_min,
                bounds_max,
            ))
        } else {
            None
        };

        // Match points between clouds
        let mut total_movement: f32 = 0.0;
        let mut max_movement: f32 = 0.0;
        let mut changed_count: usize = 0;

        for (i, &current_point) in current_points.iter().enumerate() {
            let mut closest_distance = f32::MAX;

            // Find closest point in baseline
            for &baseline_point in baseline_points.iter() {
                let dx = current_point.0 - baseline_point.0;
                let dy = current_point.1 - baseline_point.1;
                let dz = current_point.2 - baseline_point.2;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                if distance < closest_distance {
                    closest_distance = distance;
                }
            }

            if closest_distance < self.match_threshold_meters {
                // Point matched
                if closest_distance >= self.change_threshold_meters {
                    // Point moved significantly
                    result.changed_point_indices.push(i);
                    changed_count += 1;
                    total_movement += closest_distance;
                    max_movement = max_movement.max(closest_distance);

                    // Update mask
                    if let Some(ref mut m) = mask {
                        let grid_x = ((current_point.0 - bounds_min.0) / m.cell_size_meters) as usize;
                        let grid_y = ((current_point.1 - bounds_min.1) / m.cell_size_meters) as usize;
                        let grid_z = ((current_point.2 - bounds_min.2) / m.cell_size_meters) as usize;
                        let intensity = (closest_distance / self.match_threshold_meters).min(1.0);
                        m.mark_changed(grid_x, grid_y, grid_z, intensity);
                    }
                } else {
                    result.unchanged_point_indices.push(i);
                }
            } else {
                // New point
                result.added_points.push(i);
            }
        }

        // Find removed points
        for (j, _) in baseline_points.iter().enumerate() {
            let mut found = false;
            for &i in &result.changed_point_indices {
                if i == j {
                    found = true;
                    break;
                }
            }
            for &i in &result.unchanged_point_indices {
                if i == j {
                    found = true;
                    break;
                }
            }
            if !found {
                result.removed_points.push(j);
            }
        }

        // Update statistics
        result.statistics.changed_count = changed_count as u32;
        result.statistics.unchanged_count = result.unchanged_point_indices.len() as u32;
        result.statistics.added_count = result.added_points.len() as u32;
        result.statistics.removed_count = result.removed_points.len() as u32;

        if changed_count > 0 {
            result.statistics.avg_movement_meters = total_movement / changed_count as f32;
        }
        result.statistics.max_movement_meters = max_movement;
        result.statistics.change_percentage = result.change_percentage();

        result.change_mask = mask;
        result
    }

    /// Find bounding box for multiple point clouds
    fn find_bounds(
        &self,
        baseline: &[(f32, f32, f32, u8, u8, u8)],
        current: &[(f32, f32, f32, u8, u8, u8)],
    ) -> ((f32, f32, f32), (f32, f32, f32)) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for &(x, y, z, _, _, _) in baseline.iter().chain(current.iter()) {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            min_z = min_z.min(z);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
            max_z = max_z.max(z);
        }

        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Temporal change series (changes over time)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemporalChangesSeries {
    /// Snapshot ID
    pub snapshot_id: String,
    /// Timestamps of change detections
    pub timestamps: Vec<i64>,
    /// Change metrics at each timestamp
    pub changes: Vec<ChangeStatistics>,
    /// Heatmaps at each timestamp
    pub heatmaps: Vec<ChangeHeatmap>,
}

impl TemporalChangesSeries {
    /// Create new series
    pub fn new(snapshot_id: &str) -> Self {
        TemporalChangesSeries {
            snapshot_id: snapshot_id.to_string(),
            timestamps: Vec::new(),
            changes: Vec::new(),
            heatmaps: Vec::new(),
        }
    }

    /// Add change observation
    pub fn add_change(&mut self, timestamp: i64, stats: ChangeStatistics, heatmap: ChangeHeatmap) {
        self.timestamps.push(timestamp);
        self.changes.push(stats);
        self.heatmaps.push(heatmap);
    }

    /// Get average change rate
    pub fn average_change_rate(&self) -> f32 {
        if self.changes.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.changes.iter().map(|c| c.change_percentage).sum();
        sum / self.changes.len() as f32
    }

    /// Get trend (increasing/decreasing/stable)
    pub fn change_trend(&self) -> ChangeTrend {
        if self.changes.len() < 2 {
            return ChangeTrend::Stable;
        }

        let recent = self.changes[self.changes.len() - 1].change_percentage;
        let older = self.changes[0].change_percentage;
        let delta = recent - older;

        if delta > 5.0 {
            ChangeTrend::Increasing
        } else if delta < -5.0 {
            ChangeTrend::Decreasing
        } else {
            ChangeTrend::Stable
        }
    }
}

/// Change trend direction
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeTrend {
    /// Change rate increasing
    Increasing,
    /// Change rate decreasing
    Decreasing,
    /// Change rate stable
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_detection_result_creation() {
        let result = ChangeDetectionResult::new(
            "baseline".to_string(),
            "current".to_string(),
            1000,
            2000,
        );
        assert_eq!(result.baseline_snapshot_id, "baseline");
        assert_eq!(result.time_delta_us, 1000);
    }

    #[test]
    fn test_change_percentage() {
        let mut result = ChangeDetectionResult::new(
            "baseline".to_string(),
            "current".to_string(),
            1000,
            2000,
        );
        result.changed_point_indices = vec![0, 1];
        result.unchanged_point_indices = vec![2, 3, 4, 5, 6, 7];
        let pct = result.change_percentage();
        assert_eq!(pct, 25.0); // 2 / 8 = 0.25
    }

    #[test]
    fn test_change_rate_per_second() {
        let mut result = ChangeDetectionResult::new(
            "baseline".to_string(),
            "current".to_string(),
            0,
            1_000_000, // 1 second in microseconds
        );
        result.changed_point_indices = vec![0, 1];
        result.unchanged_point_indices = vec![2, 3, 4, 5, 6, 7];
        let rate = result.change_rate_per_second();
        assert_eq!(rate, 25.0); // 25% per second
    }

    #[test]
    fn test_change_mask_creation() {
        let mask = ChangeMask::new(32, 0.5, (-10.0, -10.0, -10.0), (10.0, 10.0, 10.0));
        assert_eq!(mask.resolution, 32);
        assert_eq!(mask.cell_size_meters, 0.5);
    }

    #[test]
    fn test_change_mask_mark_changed() {
        let mut mask = ChangeMask::new(32, 0.5, (-10.0, -10.0, -10.0), (10.0, 10.0, 10.0));
        mask.mark_changed(0, 0, 0, 0.5);
        assert_eq!(mask.get_change_intensity(0, 0, 0), 0.5);
    }

    #[test]
    fn test_change_mask_changed_cell_count() {
        let mut mask = ChangeMask::new(32, 0.5, (-10.0, -10.0, -10.0), (10.0, 10.0, 10.0));
        mask.mark_changed(0, 0, 0, 0.5);
        mask.mark_changed(1, 1, 1, 0.3);
        assert_eq!(mask.changed_cell_count(), 2);
    }

    #[test]
    fn test_change_heatmap_creation() {
        let heatmap = ChangeHeatmap::new(32, 0.5);
        assert_eq!(heatmap.resolution, 32);
        assert_eq!(heatmap.total_energy, 0.0);
    }

    #[test]
    fn test_change_heatmap_add_change() {
        let mut heatmap = ChangeHeatmap::new(32, 0.5);
        heatmap.add_change(0, 0, 0.5);
        assert_eq!(heatmap.heatmap_grid[0][0], 0.5);
        assert_eq!(heatmap.peak_intensity, 0.5);
    }

    #[test]
    fn test_change_heatmap_normalize() {
        let mut heatmap = ChangeHeatmap::new(32, 0.5);
        heatmap.add_change(0, 0, 0.8);
        heatmap.add_change(1, 1, 0.4);
        heatmap.normalize();
        assert_eq!(heatmap.heatmap_grid[0][0], 1.0); // 0.8 / 0.8
        assert!(heatmap.heatmap_grid[1][1] > 0.4); // Normalized
    }

    #[test]
    fn test_change_detector_creation() {
        let detector = ChangeDetector::new();
        assert!(detector.generate_mask);
        assert_eq!(detector.match_threshold_meters, 0.1);
    }

    #[test]
    fn test_change_detector_no_changes() {
        let detector = ChangeDetector::new();
        let points = vec![(0.0, 0.0, 0.0, 255, 0, 0); 10];
        let result = detector.detect_changes("b", "c", &points, &points, 0, 1000);
        // Same points should be mostly unchanged
        assert_eq!(result.statistics.baseline_point_count, 10);
    }

    #[test]
    fn test_change_detector_with_movement() {
        let detector = ChangeDetector::new();
        let baseline = vec![(0.0, 0.0, 0.0, 255, 0, 0), (1.0, 1.0, 1.0, 0, 255, 0)];
        let current = vec![(0.05, 0.05, 0.05, 255, 0, 0), (1.1, 1.1, 1.1, 0, 255, 0)];
        let result = detector.detect_changes("b", "c", &baseline, &current, 0, 1000);
        assert!(result.statistics.changed_count > 0);
    }

    #[test]
    fn test_temporal_changes_series_creation() {
        let series = TemporalChangesSeries::new("snapshot1");
        assert_eq!(series.snapshot_id, "snapshot1");
        assert_eq!(series.changes.len(), 0);
    }

    #[test]
    fn test_temporal_changes_series_add_change() {
        let mut series = TemporalChangesSeries::new("snapshot1");
        let stats = ChangeStatistics {
            baseline_point_count: 100,
            current_point_count: 105,
            changed_count: 10,
            unchanged_count: 90,
            added_count: 5,
            removed_count: 0,
            avg_movement_meters: 0.05,
            max_movement_meters: 0.15,
            change_percentage: 10.0,
        };
        let heatmap = ChangeHeatmap::new(32, 0.5);
        series.add_change(1000, stats, heatmap);
        assert_eq!(series.changes.len(), 1);
        assert_eq!(series.changes[0].change_percentage, 10.0);
    }

    #[test]
    fn test_temporal_changes_average_rate() {
        let mut series = TemporalChangesSeries::new("snapshot1");
        let stats1 = ChangeStatistics {
            change_percentage: 10.0,
            ..Default::default()
        };
        let stats2 = ChangeStatistics {
            change_percentage: 20.0,
            ..Default::default()
        };
        let heatmap = ChangeHeatmap::new(32, 0.5);
        series.add_change(1000, stats1, heatmap.clone());
        series.add_change(2000, stats2, heatmap);
        assert_eq!(series.average_change_rate(), 15.0);
    }

    #[test]
    fn test_temporal_changes_trend() {
        let mut series = TemporalChangesSeries::new("snapshot1");
        let heatmap = ChangeHeatmap::new(32, 0.5);

        let stats1 = ChangeStatistics {
            change_percentage: 5.0,
            ..Default::default()
        };
        series.add_change(1000, stats1, heatmap.clone());

        let stats2 = ChangeStatistics {
            change_percentage: 15.0,
            ..Default::default()
        };
        series.add_change(2000, stats2, heatmap);

        assert_eq!(series.change_trend(), ChangeTrend::Increasing);
    }

    #[test]
    fn test_change_trend_decreasing() {
        let mut series = TemporalChangesSeries::new("snapshot1");
        let heatmap = ChangeHeatmap::new(32, 0.5);

        let stats1 = ChangeStatistics {
            change_percentage: 20.0,
            ..Default::default()
        };
        series.add_change(1000, stats1, heatmap.clone());

        let stats2 = ChangeStatistics {
            change_percentage: 10.0,
            ..Default::default()
        };
        series.add_change(2000, stats2, heatmap);

        assert_eq!(series.change_trend(), ChangeTrend::Decreasing);
    }

    #[test]
    fn test_change_statistics_default() {
        let stats = ChangeStatistics::default();
        assert_eq!(stats.baseline_point_count, 0);
        assert_eq!(stats.changed_count, 0);
    }
}
