//! Data Sharding & Partitioning
//!
//! Phase 7.1: Distribute data across nodes using geographic sharding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShardKey {
    pub shard_id: u32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

pub struct Sharding {
    pub shard_count: u32,
    pub shards: HashMap<u32, ShardKey>,
}

impl Sharding {
    pub fn new(shard_count: u32) -> Self {
        let mut shards = HashMap::new();
        let grid_size = (shard_count as f32).sqrt() as u32;
        let shard_size = 100.0 / (grid_size as f32);

        for i in 0..shard_count {
            let x_idx = i % grid_size;
            let y_idx = i / grid_size;

            shards.insert(
                i,
                ShardKey {
                    shard_id: i,
                    x_min: (x_idx as f32) * shard_size,
                    x_max: (x_idx as f32 + 1.0) * shard_size,
                    y_min: (y_idx as f32) * shard_size,
                    y_max: (y_idx as f32 + 1.0) * shard_size,
                },
            );
        }

        Sharding { shard_count, shards }
    }

    pub fn get_shard(&self, x: f32, y: f32) -> Option<u32> {
        self.shards.values().find(|s| {
            x >= s.x_min && x < s.x_max && y >= s.y_min && y < s.y_max
        }).map(|s| s.shard_id)
    }
}

impl Default for Sharding {
    fn default() -> Self {
        Self::new(4)
    }
}
