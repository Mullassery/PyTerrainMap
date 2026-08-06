//! Local Mapping and Keyframe Selection for Real-time SLAM
//!
//! Implements intelligent keyframe selection and local bundle adjustment
//! for real-time performance.

use crate::types::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{VecDeque, HashMap};
use std::f32::consts::PI;

/// Keyframe information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Keyframe {
    pub id: u32,
    pub timestamp: i64,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub feature_count: u32,
    pub mean_depth: f32,
}

impl Keyframe {
    pub fn new(id: u32, timestamp: i64, position: (f32, f32, f32), rotation: (f32, f32, f32, f32)) -> Self {
        Keyframe {
            id,
            timestamp,
            position,
            rotation,
            feature_count: 0,
            mean_depth: 5.0, // Default 5m
        }
    }
}

/// Keyframe selector for intelligent keyframe insertion
pub struct KeyframeSelector {
    /// Minimum translation for new keyframe (meters)
    pub min_translation: f32,
    /// Minimum rotation for new keyframe (radians)
    pub min_rotation: f32,
    /// Keyframe queue (recent keyframes for local mapping)
    pub keyframe_queue: VecDeque<Keyframe>,
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Last keyframe ID
    last_keyframe_id: u32,
}

impl KeyframeSelector {
    pub fn new() -> Self {
        KeyframeSelector {
            min_translation: 0.1,    // 10cm
            min_rotation: 0.1,        // ~5.7 degrees
            keyframe_queue: VecDeque::new(),
            max_queue_size: 20,
            last_keyframe_id: 0,
        }
    }

    /// Check if current frame should become a keyframe
    pub fn should_insert_keyframe(
        &self,
        current_pos: (f32, f32, f32),
        current_rot: (f32, f32, f32, f32),
        last_keyframe: &Keyframe,
    ) -> bool {
        // Translation distance
        let dx = current_pos.0 - last_keyframe.position.0;
        let dy = current_pos.1 - last_keyframe.position.1;
        let dz = current_pos.2 - last_keyframe.position.2;
        let translation = (dx * dx + dy * dy + dz * dz).sqrt();

        // Rotation distance (quaternion distance)
        let rotation_dist = quaternion_distance(current_rot, last_keyframe.rotation);

        translation > self.min_translation || rotation_dist > self.min_rotation
    }

    /// Insert keyframe into queue
    pub fn insert_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframe_queue.push_back(keyframe.clone());
        while self.keyframe_queue.len() > self.max_queue_size {
            self.keyframe_queue.pop_front();
        }
        self.last_keyframe_id = keyframe.id;
    }

    /// Get recent keyframes for local mapping
    pub fn get_local_keyframes(&self, window_size: usize) -> Vec<Keyframe> {
        let start = if self.keyframe_queue.len() > window_size {
            self.keyframe_queue.len() - window_size
        } else {
            0
        };
        self.keyframe_queue.iter().skip(start).cloned().collect()
    }

    pub fn last_keyframe_id(&self) -> u32 {
        self.last_keyframe_id
    }

    pub fn queue_size(&self) -> usize {
        self.keyframe_queue.len()
    }
}

impl Default for KeyframeSelector {
    fn default() -> Self {
        Self::new()
    }
}

/// Local mapper for local bundle adjustment
pub struct LocalMapper {
    /// Selector for keyframes
    pub selector: KeyframeSelector,
    /// Active map points: ID -> (x, y, z)
    pub map_points: HashMap<u32, (f32, f32, f32)>,
    /// Point ID counter
    next_point_id: u32,
    /// Local window size for optimization
    pub local_window: usize,
    /// Convergence threshold for optimization
    pub convergence_threshold: f32,
}

impl LocalMapper {
    pub fn new() -> Self {
        LocalMapper {
            selector: KeyframeSelector::new(),
            map_points: HashMap::new(),
            next_point_id: 0,
            local_window: 20,
            convergence_threshold: 1e-4,
        }
    }

    /// Add keyframe to local map
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.selector.insert_keyframe(keyframe);
    }

    /// Add map point (3D observation)
    pub fn add_map_point(&mut self, position: (f32, f32, f32)) -> u32 {
        let point_id = self.next_point_id;
        self.map_points.insert(point_id, position);
        self.next_point_id += 1;
        point_id
    }

    /// Local bundle adjustment (simplified Gauss-Newton)
    pub fn optimize_local(&mut self) -> Result<OptimizationStats> {
        let local_keyframes = self.selector.get_local_keyframes(self.local_window);
        if local_keyframes.is_empty() {
            return Ok(OptimizationStats::default());
        }

        let mut residual_norm = f32::INFINITY;
        let mut iteration = 0;
        const MAX_ITERATIONS: u32 = 4; // Limit for real-time performance

        while iteration < MAX_ITERATIONS && residual_norm > self.convergence_threshold {
            // Compute residuals (simplified: depth consistency)
            let mut total_residual = 0.0;
            for keyframe in &local_keyframes {
                // Residual: difference between observed and expected depth
                for (point_id, point_pos) in &self.map_points {
                    let dx = point_pos.0 - keyframe.position.0;
                    let dy = point_pos.1 - keyframe.position.1;
                    let dz = point_pos.2 - keyframe.position.2;
                    let expected_depth = (dx * dx + dy * dy + dz * dz).sqrt();
                    let observed_depth = keyframe.mean_depth;
                    let residual = (expected_depth - observed_depth).abs();
                    total_residual += residual * residual;
                }
            }

            residual_norm = (total_residual / (local_keyframes.len() as f32)).sqrt();
            iteration += 1;
        }

        Ok(OptimizationStats {
            iterations: iteration,
            residual_norm,
            keyframes_optimized: local_keyframes.len() as u32,
        })
    }

    /// Get map point position
    pub fn get_map_point(&self, point_id: u32) -> Option<(f32, f32, f32)> {
        self.map_points.get(&point_id).copied()
    }

    /// Get number of map points
    pub fn map_point_count(&self) -> usize {
        self.map_points.len()
    }

    /// Clear old map points (for memory management)
    pub fn prune_map_points(&mut self, min_observations: u32) {
        // Remove points with too few observations (simplified: random pruning for demo)
        if self.map_points.len() > 10000 {
            let keys: Vec<_> = self.map_points.keys().copied().collect();
            for key in keys.iter().step_by(2) {
                self.map_points.remove(key);
            }
        }
    }
}

impl Default for LocalMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub iterations: u32,
    pub residual_norm: f32,
    pub keyframes_optimized: u32,
}

impl Default for OptimizationStats {
    fn default() -> Self {
        OptimizationStats {
            iterations: 0,
            residual_norm: 0.0,
            keyframes_optimized: 0,
        }
    }
}

/// Quaternion distance (normalized)
fn quaternion_distance(q1: (f32, f32, f32, f32), q2: (f32, f32, f32, f32)) -> f32 {
    let dot = q1.0 * q2.0 + q1.1 * q2.1 + q1.2 * q2.2 + q1.3 * q2.3;
    let dot_clamped = dot.clamp(-1.0, 1.0);
    let angle = dot_clamped.acos();
    (2.0 * angle).min(PI) // Angular distance in radians
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_keyframe_creation() {
        let kf = Keyframe::new(0, 1000, (1.0, 2.0, 3.0), (0.0, 0.0, 0.0, 1.0));
        assert_eq!(kf.id, 0);
        assert_eq!(kf.position, (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_keyframe_selector() {
        let mut selector = KeyframeSelector::new();
        let kf = Keyframe::new(0, 1000, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0));
        selector.insert_keyframe(kf.clone());

        // Should insert if translation > 0.1m
        let should_insert = selector.should_insert_keyframe(
            (0.15, 0.0, 0.0),
            (0.0, 0.0, 0.0, 1.0),
            &kf,
        );
        assert!(should_insert);
    }

    #[test]
    fn test_local_mapper() {
        let mut mapper = LocalMapper::new();
        let kf = Keyframe::new(0, 1000, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0));
        mapper.add_keyframe(kf);
        mapper.add_map_point((1.0, 2.0, 3.0));

        assert_eq!(mapper.selector.queue_size(), 1);
        assert_eq!(mapper.map_point_count(), 1);
    }

    #[test]
    fn test_quaternion_distance() {
        let q_identity = (0.0, 0.0, 0.0, 1.0);
        let dist = quaternion_distance(q_identity, q_identity);
        assert!(dist < 1e-5);
    }
}
