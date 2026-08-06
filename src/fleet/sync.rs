//! Map Synchronization for Distributed Fleet
//!
//! Phase 4.5: Coordinates updates across robot local maps,
//! ensuring consistent global terrain understanding.

use super::RobotId;
use serde::{Deserialize, Serialize};

/// Synchronization checkpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncCheckpoint {
    pub checkpoint_id: u64,
    pub robots_synced: Vec<RobotId>,
    pub sync_time_us: i64,
    pub observations_merged: u32,
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
        }
    }

    /// Get sync completion percentage
    pub fn completion_percent(&self, total_robots: u32) -> f32 {
        if total_robots == 0 {
            return 100.0;
        }
        (self.robots_synced.len() as f32 / total_robots as f32) * 100.0
    }
}

/// Stub: Map synchronizer (to be implemented in Phase 4.5)
pub struct MapSynchronizer {
    pending_checkpoints: Vec<SyncCheckpoint>,
}

impl MapSynchronizer {
    /// Create new synchronizer
    pub fn new() -> Self {
        MapSynchronizer {
            pending_checkpoints: Vec::new(),
        }
    }
}

impl Default for MapSynchronizer {
    fn default() -> Self {
        Self::new()
    }
}
