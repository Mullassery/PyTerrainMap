//! Map Synchronization for Distributed Fleet
//!
//! Phase 4.5: Coordinates updates across robot local maps,
//! ensuring consistent global terrain understanding.

use super::RobotId;
use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Synchronization checkpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncCheckpoint {
    pub checkpoint_id: u64,
    pub robots_synced: Vec<RobotId>,
    pub sync_time_us: i64,
    pub observations_merged: u32,
    pub sync_version: u32,
}

impl SyncCheckpoint {
    /// Create new checkpoint
    pub fn new(checkpoint_id: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        SyncCheckpoint {
            checkpoint_id,
            robots_synced: Vec::new(),
            sync_time_us: now,
            observations_merged: 0,
            sync_version: 1,
        }
    }

    /// Get sync completion percentage
    pub fn completion_percent(&self, total_robots: u32) -> f32 {
        if total_robots == 0 {
            return 100.0;
        }
        (self.robots_synced.len() as f32 / total_robots as f32) * 100.0
    }

    /// Check if sync is complete
    pub fn is_complete(&self, total_robots: u32) -> bool {
        self.robots_synced.len() == total_robots as usize
    }

    /// Add robot to sync
    pub fn add_robot(&mut self, robot_id: RobotId) {
        if !self.robots_synced.contains(&robot_id) {
            self.robots_synced.push(robot_id);
        }
    }
}

/// Map update from a single robot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapUpdate {
    pub robot_id: RobotId,
    pub points: Vec<TemporalPoint>,
    pub timestamp_us: i64,
    pub update_id: u64,
}

impl MapUpdate {
    /// Create new map update
    pub fn new(robot_id: RobotId, points: Vec<TemporalPoint>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        MapUpdate {
            robot_id,
            points,
            timestamp_us: now,
            update_id: now as u64,
        }
    }

    /// Get point count
    pub fn point_count(&self) -> u32 {
        self.points.len() as u32
    }

    /// Get average quality
    pub fn average_quality(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        self.points.iter().map(|p| p.quality).sum::<f32>() / self.points.len() as f32
    }
}

/// Global synchronized map state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalMapState {
    pub version: u32,
    pub last_update_us: i64,
    pub total_points: u64,
    pub contributing_robots: Vec<RobotId>,
    pub last_sync_checkpoint: u64,
}

impl GlobalMapState {
    /// Create new global map state
    pub fn new() -> Self {
        GlobalMapState {
            version: 1,
            last_update_us: 0,
            total_points: 0,
            contributing_robots: Vec::new(),
            last_sync_checkpoint: 0,
        }
    }
}

impl Default for GlobalMapState {
    fn default() -> Self {
        Self::new()
    }
}

/// Map synchronizer for coordinating fleet updates
pub struct MapSynchronizer {
    pub global_state: GlobalMapState,
    pending_checkpoints: VecDeque<SyncCheckpoint>,
    completed_checkpoints: Vec<SyncCheckpoint>,
    pending_updates: HashMap<RobotId, VecDeque<MapUpdate>>,
    sync_interval_us: i64, // Synchronization interval
    last_sync_time_us: i64,
}

impl MapSynchronizer {
    /// Create new synchronizer
    pub fn new() -> Self {
        MapSynchronizer {
            global_state: GlobalMapState::new(),
            pending_checkpoints: VecDeque::new(),
            completed_checkpoints: Vec::new(),
            pending_updates: HashMap::new(),
            sync_interval_us: 5_000_000, // 5 seconds default
            last_sync_time_us: 0,
        }
    }

    /// Set synchronization interval
    pub fn set_sync_interval(&mut self, interval_us: i64) {
        self.sync_interval_us = interval_us;
    }

    /// Create new synchronization checkpoint
    pub fn create_checkpoint(&mut self) -> u64 {
        let checkpoint_id = self.pending_checkpoints.len() as u64;
        let checkpoint = SyncCheckpoint::new(checkpoint_id);
        self.pending_checkpoints.push_back(checkpoint);
        checkpoint_id
    }

    /// Queue map update from robot
    pub fn queue_update(&mut self, update: MapUpdate) {
        let robot_id = update.robot_id;
        self.pending_updates
            .entry(robot_id)
            .or_insert_with(VecDeque::new)
            .push_back(update);
    }

    /// Get pending updates for robot
    pub fn pending_updates(&self, robot_id: RobotId) -> Option<&VecDeque<MapUpdate>> {
        self.pending_updates.get(&robot_id)
    }

    /// Merge all pending updates
    pub fn merge_pending_updates(&mut self) -> SyncMergeResult {
        let mut merged_count = 0;
        let mut merged_points = 0;
        let mut contributing_robots = Vec::new();

        // If no pending checkpoints, create one
        if self.pending_checkpoints.is_empty() {
            self.create_checkpoint();
        }

        let checkpoint = self.pending_checkpoints.front_mut();

        if let Some(checkpoint) = checkpoint {
            for (robot_id, updates) in &self.pending_updates {
                for update in updates {
                    checkpoint.observations_merged += update.point_count();
                    merged_points += update.point_count() as u64;
                    merged_count += 1;

                    if !contributing_robots.contains(robot_id) {
                        contributing_robots.push(*robot_id);
                        checkpoint.add_robot(*robot_id);
                    }
                }
            }

            // Update global state
            self.global_state.version += 1;
            self.global_state.last_update_us = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64;
            self.global_state.total_points += merged_points;
            self.global_state.contributing_robots = contributing_robots.clone();

            SyncMergeResult {
                updates_merged: merged_count,
                points_merged: merged_points,
                robots_involved: contributing_robots,
            }
        } else {
            SyncMergeResult {
                updates_merged: 0,
                points_merged: 0,
                robots_involved: vec![],
            }
        }
    }

    /// Complete current checkpoint
    pub fn complete_checkpoint(&mut self) -> Option<SyncCheckpoint> {
        if let Some(checkpoint) = self.pending_checkpoints.pop_front() {
            self.completed_checkpoints.push(checkpoint.clone());
            self.global_state.last_sync_checkpoint = checkpoint.checkpoint_id;
            Some(checkpoint)
        } else {
            None
        }
    }

    /// Get synchronization progress
    pub fn sync_progress(&self, total_robots: u32) -> f32 {
        if let Some(checkpoint) = self.pending_checkpoints.front() {
            checkpoint.completion_percent(total_robots)
        } else {
            0.0
        }
    }

    /// Check if sync is needed
    pub fn needs_sync(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        now - self.last_sync_time_us > self.sync_interval_us || !self.pending_updates.is_empty()
    }

    /// Get synchronizer statistics
    pub fn statistics(&self) -> SyncStatistics {
        SyncStatistics {
            pending_checkpoints: self.pending_checkpoints.len() as u32,
            completed_checkpoints: self.completed_checkpoints.len() as u32,
            total_updates_queued: self.pending_updates.values().map(|u| u.len() as u32).sum(),
            global_map_version: self.global_state.version,
            total_points_synced: self.global_state.total_points,
        }
    }

    /// Get completed checkpoints
    pub fn completed_checkpoints(&self) -> &[SyncCheckpoint] {
        &self.completed_checkpoints
    }
}

impl Default for MapSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Synchronization merge result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncMergeResult {
    pub updates_merged: u32,
    pub points_merged: u64,
    pub robots_involved: Vec<RobotId>,
}

/// Synchronization statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncStatistics {
    pub pending_checkpoints: u32,
    pub completed_checkpoints: u32,
    pub total_updates_queued: u32,
    pub global_map_version: u32,
    pub total_points_synced: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_checkpoint() {
        let mut checkpoint = SyncCheckpoint::new(1);
        assert_eq!(checkpoint.completion_percent(4), 0.0);

        checkpoint.add_robot(RobotId::new(1));
        assert_eq!(checkpoint.completion_percent(4), 25.0);

        checkpoint.add_robot(RobotId::new(2));
        assert_eq!(checkpoint.completion_percent(4), 50.0);
    }

    #[test]
    fn test_map_update() {
        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let update = MapUpdate::new(robot_id, vec![point]);

        assert_eq!(update.robot_id, robot_id);
        assert_eq!(update.point_count(), 1);
        assert!((update.average_quality() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_map_synchronizer() {
        let mut sync = MapSynchronizer::new();
        assert_eq!(sync.sync_progress(4), 0.0);

        sync.create_checkpoint();
        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let update = MapUpdate::new(robot_id, vec![point]);

        sync.queue_update(update);
        let result = sync.merge_pending_updates();

        assert!(result.updates_merged > 0);
        assert!(result.points_merged > 0);
    }

    #[test]
    fn test_checkpoint_completion() {
        let mut sync = MapSynchronizer::new();
        let robot1 = RobotId::new(1);
        let robot2 = RobotId::new(2);

        let checkpoint_id = sync.create_checkpoint();
        let checkpoint = sync.pending_checkpoints.front_mut().unwrap();
        checkpoint.add_robot(robot1);
        checkpoint.add_robot(robot2);

        let completed = sync.complete_checkpoint();
        assert!(completed.is_some());
        assert_eq!(sync.completed_checkpoints().len(), 1);
    }

    #[test]
    fn test_sync_statistics() {
        let mut sync = MapSynchronizer::new();
        sync.create_checkpoint();

        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let update = MapUpdate::new(robot_id, vec![point]);
        sync.queue_update(update);

        let stats = sync.statistics();
        assert_eq!(stats.pending_checkpoints, 1);
        assert!(stats.total_updates_queued > 0);
    }
}
