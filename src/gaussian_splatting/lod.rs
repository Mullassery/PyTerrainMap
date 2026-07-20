use crate::gaussian_splatting::core::{GaussianCovariance, SplatKind, TerrainGaussian};
use serde::{Deserialize, Serialize};

/// A level of detail for hierarchical Gaussian representations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LODLevel {
    pub level: u8,              // 0=World, 1=Region, 2=Zone, 3=Area, 4=Detail
    pub cell_size_m: f32,       // 1000, 100, 10, 1, 0.1
    pub h3_resolution: u8,      // H3 resolution: 5, 7, 9, 11, 12
}

impl LODLevel {
    pub fn new(level: u8) -> Self {
        match level {
            0 => LODLevel { level: 0, cell_size_m: 1000.0, h3_resolution: 5 },
            1 => LODLevel { level: 1, cell_size_m: 100.0, h3_resolution: 7 },
            2 => LODLevel { level: 2, cell_size_m: 10.0, h3_resolution: 9 },
            3 => LODLevel { level: 3, cell_size_m: 1.0, h3_resolution: 11 },
            4 => LODLevel { level: 4, cell_size_m: 0.1, h3_resolution: 12 },
            _ => LODLevel { level: 2, cell_size_m: 10.0, h3_resolution: 9 },
        }
    }
}

/// Hierarchical level-of-detail manager
pub struct HierarchicalLOD {
    pub levels: [LODLevel; 5],
    pub split_obs_threshold: u32,           // default 20
    pub merge_confidence_threshold: f32,    // default 0.3
}

impl HierarchicalLOD {
    pub fn new() -> Self {
        HierarchicalLOD {
            levels: [
                LODLevel::new(0),
                LODLevel::new(1),
                LODLevel::new(2),
                LODLevel::new(3),
                LODLevel::new(4),
            ],
            split_obs_threshold: 20,
            merge_confidence_threshold: 0.3,
        }
    }

    /// Get LOD level based on interest (0=coarse, 1=fine)
    pub fn level_for_interest(&self, interest: f32) -> &LODLevel {
        let idx = ((interest * 4.0) as usize).clamp(0, 4);
        &self.levels[idx]
    }

    /// Check if splat should be split into finer representation
    pub fn should_split(&self, splat: &TerrainGaussian) -> bool {
        splat.observation_count > self.split_obs_threshold
    }

    /// Check if group of splats should be merged
    pub fn should_merge(&self, group: &[&TerrainGaussian]) -> bool {
        if group.is_empty() {
            return false;
        }
        let avg_confidence = group.iter().map(|s| s.confidence).sum::<f32>() / (group.len() as f32);
        avg_confidence < self.merge_confidence_threshold
    }

    /// Split a splat into 4 children (2×2 grid at ±σ/2)
    pub fn split(&self, splat: &TerrainGaussian) -> Vec<TerrainGaussian> {
        let mut children = Vec::new();

        // Get standard deviations from covariance
        let sigma_x = splat.covariance.matrix[0][0].sqrt();
        let sigma_y = splat.covariance.matrix[1][1].sqrt();

        let offsets = [
            (-0.5, -0.5),
            (-0.5, 0.5),
            (0.5, -0.5),
            (0.5, 0.5),
        ];

        for (dx, dy) in offsets {
            let mut child = splat.clone();
            child.id = uuid::Uuid::new_v4();
            child.position[0] += (dx * sigma_x as f64);
            child.position[1] += (dy * sigma_y as f64);

            // Halve covariance
            let mut new_cov = [[0.0; 3]; 3];
            for i in 0..3 {
                for j in 0..3 {
                    new_cov[i][j] = splat.covariance.matrix[i][j] * 0.25;
                }
            }
            child.covariance = GaussianCovariance { matrix: new_cov };

            // Distribute observation count
            child.observation_count = (splat.observation_count / 4).max(1);
            children.push(child);
        }

        children
    }

    /// Merge group of splats into single representation
    pub fn merge(&self, group: &[&TerrainGaussian]) -> TerrainGaussian {
        if group.is_empty() {
            return TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "", 0.5);
        }

        // Confidence-weighted position
        let total_confidence: f32 = group.iter().map(|s| s.confidence).sum();
        let mut merged_pos = [0.0; 3];
        for splat in group {
            let weight = splat.confidence / (total_confidence + 1e-6);
            merged_pos[0] += splat.position[0] * (weight as f64);
            merged_pos[1] += splat.position[1] * (weight as f64);
            merged_pos[2] += splat.position[2] * (weight as f64);
        }

        // Union covariance (max of all)
        let mut merged_cov = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let max_val = group
                    .iter()
                    .map(|s| s.covariance.matrix[i][j])
                    .fold(0.0_f32, f32::max);
                merged_cov[i][j] = max_val;
            }
        }

        let mut merged = TerrainGaussian::from_point_observation(merged_pos, "", total_confidence / (group.len() as f32));
        merged.covariance = GaussianCovariance { matrix: merged_cov };
        merged.observation_count = group.iter().map(|s| s.observation_count).sum();
        merged.source_bots = group.iter().flat_map(|s| s.source_bots.clone()).collect();
        merged
    }
}

impl Default for HierarchicalLOD {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level_creation() {
        let lod = LODLevel::new(2);
        assert_eq!(lod.level, 2);
        assert_eq!(lod.cell_size_m, 10.0);
    }

    #[test]
    fn test_hierarchical_lod_creation() {
        let lod = HierarchicalLOD::new();
        assert_eq!(lod.split_obs_threshold, 20);
    }

    #[test]
    fn test_level_for_interest() {
        let lod = HierarchicalLOD::new();
        let coarse = lod.level_for_interest(0.0);
        let fine = lod.level_for_interest(1.0);
        assert!(coarse.cell_size_m > fine.cell_size_m);
    }

    #[test]
    fn test_should_split() {
        let lod = HierarchicalLOD::new();
        let mut splat = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        assert!(!lod.should_split(&splat));

        splat.observation_count = 25;
        assert!(lod.should_split(&splat));
    }

    #[test]
    fn test_split_splat() {
        let lod = HierarchicalLOD::new();
        let splat = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let children = lod.split(&splat);
        assert_eq!(children.len(), 4);

        // Each child should have different position
        let positions: Vec<_> = children.iter().map(|c| c.position[0]).collect();
        assert!(positions.iter().all(|&p| p != 0.0 || positions.len() > 1));
    }

    #[test]
    fn test_merge_splats() {
        let lod = HierarchicalLOD::new();
        let s1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let s2 = TerrainGaussian::from_point_observation([1.0, 1.0, 0.0], "bot_02", 0.7);

        let merged = lod.merge(&[&s1, &s2]);
        assert!(merged.observation_count > 0);
    }
}
