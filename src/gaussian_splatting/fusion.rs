use crate::gaussian_splatting::core::{GaussianCovariance, TerrainGaussian, TerrainType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes what happened to an observation during fusion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FusionAction {
    /// New splat created
    Created,
    /// Merged with existing splat
    Fused { with_splat_id: Uuid },
    /// Rejected due to conflict or incompatibility
    Rejected { reason: String },
}

/// Result of a fusion operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusionResult {
    pub action: FusionAction,
    pub splat_id: Uuid,
    pub confidence_delta: f32,
    pub observations_merged: u32,
}

/// Multi-bot Bayesian observation fusion engine
pub struct ObservationFuser;

impl ObservationFuser {
    /// Fuse an incoming observation into an existing splat
    ///
    /// Applies Bayesian updates to position, covariance, traversability, and confidence.
    /// Handles conflicting observations gracefully.
    pub fn fuse(existing: &mut TerrainGaussian, incoming: &TerrainGaussian) -> FusionResult {
        let splat_id = existing.id;
        let prev_confidence = existing.confidence;

        // 1. Position update: weighted mean by inverse covariance
        let new_pos = Self::fuse_positions(&existing.position, &existing.covariance,
                                           &incoming.position, &incoming.covariance);

        // 2. Covariance update: (Σ₁⁻¹ + Σ₂⁻¹)⁻¹
        let new_cov = existing.covariance.fuse(&incoming.covariance);

        // 3. Traversability: confidence-weighted average
        let weighted_trav = (existing.traversability * existing.confidence +
                            incoming.traversability * incoming.confidence) /
                           (existing.confidence + incoming.confidence + 1e-6);

        // 4. Confidence adjustment based on agreement/conflict
        let trav_diff = (existing.traversability - incoming.traversability).abs();
        let confidence_delta = if trav_diff < 0.2 {
            // Agreement: boost confidence
            0.05 * (existing.observation_count.min(10) as f32 / 10.0)
        } else {
            // Conflict: reduce confidence
            -0.03
        };

        // 5. Update existing splat
        existing.position = new_pos;
        existing.covariance = new_cov;
        existing.traversability = weighted_trav.clamp(0.0, 1.0);
        existing.observation_count += 1;
        existing.increase_confidence(confidence_delta);
        existing.last_updated = chrono::Utc::now().timestamp_micros();

        // 6. Merge source attribution
        for bot in &incoming.source_bots {
            existing.add_source_bot(bot);
        }

        // 7. Handle terrain type: majority vote weighted by confidence
        if existing.terrain_type != incoming.terrain_type {
            let existing_weight = existing.confidence;
            let incoming_weight = incoming.confidence;
            if incoming_weight > existing_weight {
                existing.terrain_type = incoming.terrain_type;
            } else if incoming_weight == existing_weight {
                // Tie: mark as conflicting
                existing.terrain_type = TerrainType::Unknown(1);
            }
        }

        let confidence_delta_final = existing.confidence - prev_confidence;

        FusionResult {
            action: FusionAction::Fused { with_splat_id: splat_id },
            splat_id,
            confidence_delta: confidence_delta_final,
            observations_merged: existing.observation_count,
        }
    }

    /// Fuse two positions using inverse-covariance weighting
    /// new_pos = (Σ₁⁻¹·μ₁ + Σ₂⁻¹·μ₂) / (Σ₁⁻¹ + Σ₂⁻¹)
    fn fuse_positions(
        pos1: &[f64; 3],
        cov1: &GaussianCovariance,
        pos2: &[f64; 3],
        cov2: &GaussianCovariance,
    ) -> [f64; 3] {
        match (cov1.inverse(), cov2.inverse()) {
            (Some(inv1), Some(inv2)) => {
                let m1 = inv1.matrix;
                let m2 = inv2.matrix;

                // Compute (Σ₁⁻¹ + Σ₂⁻¹)
                let sum_inv = [
                    [m1[0][0] + m2[0][0], m1[0][1] + m2[0][1], m1[0][2] + m2[0][2]],
                    [m1[1][0] + m2[1][0], m1[1][1] + m2[1][1], m1[1][2] + m2[1][2]],
                    [m1[2][0] + m2[2][0], m1[2][1] + m2[2][1], m1[2][2] + m2[2][2]],
                ];

                // Compute Σ₁⁻¹·μ₁ + Σ₂⁻¹·μ₂
                let p1_f = [pos1[0] as f32, pos1[1] as f32, pos1[2] as f32];
                let p2_f = [pos2[0] as f32, pos2[1] as f32, pos2[2] as f32];

                let term1 = [
                    m1[0][0] * p1_f[0] + m1[0][1] * p1_f[1] + m1[0][2] * p1_f[2],
                    m1[1][0] * p1_f[0] + m1[1][1] * p1_f[1] + m1[1][2] * p1_f[2],
                    m1[2][0] * p1_f[0] + m1[2][1] * p1_f[1] + m1[2][2] * p1_f[2],
                ];

                let term2 = [
                    m2[0][0] * p2_f[0] + m2[0][1] * p2_f[1] + m2[0][2] * p2_f[2],
                    m2[1][0] * p2_f[0] + m2[1][1] * p2_f[1] + m2[1][2] * p2_f[2],
                    m2[2][0] * p2_f[0] + m2[2][1] * p2_f[1] + m2[2][2] * p2_f[2],
                ];

                let numerator = [term1[0] + term2[0], term1[1] + term2[1], term1[2] + term2[2]];

                // (Σ₁⁻¹ + Σ₂⁻¹)⁻¹ · numerator
                let sum_cov = GaussianCovariance { matrix: sum_inv };
                match sum_cov.inverse() {
                    Some(result_cov) => {
                        let m = result_cov.matrix;
                        let x = m[0][0] * numerator[0] + m[0][1] * numerator[1] + m[0][2] * numerator[2];
                        let y = m[1][0] * numerator[0] + m[1][1] * numerator[1] + m[1][2] * numerator[2];
                        let z = m[2][0] * numerator[0] + m[2][1] * numerator[1] + m[2][2] * numerator[2];
                        [x as f64, y as f64, z as f64]
                    }
                    None => [(pos1[0] + pos2[0]) / 2.0, (pos1[1] + pos2[1]) / 2.0, (pos1[2] + pos2[2]) / 2.0],
                }
            }
            _ => {
                // Fallback to simple average if inversion fails
                [(pos1[0] + pos2[0]) / 2.0, (pos1[1] + pos2[1]) / 2.0, (pos1[2] + pos2[2]) / 2.0]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_agreement() {
        let mut g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let g2 = TerrainGaussian::from_point_observation([0.001, 0.001, 0.05], "bot_02", 0.85);

        let result = ObservationFuser::fuse(&mut g1, &g2);
        assert!(matches!(result.action, FusionAction::Fused { .. }));
        assert!(g1.confidence > 0.8);  // Confidence increased due to agreement
        assert_eq!(g1.observation_count, 2);
        assert!(g1.source_bots.contains(&"bot_02".to_string()));
    }

    #[test]
    fn test_fusion_conflict() {
        let mut g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.9);
        let g2 = TerrainGaussian::from_point_observation([0.001, 0.001, 0.05], "bot_02", 0.1);

        let prev_conf = g1.confidence;
        let result = ObservationFuser::fuse(&mut g1, &g2);
        assert!(matches!(result.action, FusionAction::Fused { .. }));
        assert!(g1.confidence < prev_conf);  // Confidence decreased due to conflict
    }

    #[test]
    fn test_fusion_position_update() {
        let mut g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        g1.confidence = 1.0;
        let g2 = TerrainGaussian::from_point_observation([2.0, 0.0, 0.0], "bot_02", 0.5);

        let _result = ObservationFuser::fuse(&mut g1, &g2);
        // Position should have moved toward bot_02's observation (weighted by inverse covariance)
        assert!(g1.position[0] > 0.0 && g1.position[0] < 2.0);
    }

    #[test]
    fn test_fusion_traversability_average() {
        let mut g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        g1.confidence = 0.5;
        let g2 = TerrainGaussian::from_point_observation([0.001, 0.001, 0.05], "bot_02", 0.6);
        g2.confidence = 0.5;

        let _result = ObservationFuser::fuse(&mut g1, &g2);
        assert!((g1.traversability - 0.7).abs() < 0.05);
    }

    #[test]
    fn test_fusion_source_bot_deduplication() {
        let mut g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let mut g2 = TerrainGaussian::from_point_observation([0.001, 0.001, 0.05], "bot_01", 0.8);
        g2.add_source_bot("bot_02");

        let _result = ObservationFuser::fuse(&mut g1, &g2);
        assert_eq!(g1.source_bots.len(), 2);
        assert!(g1.source_bots.contains(&"bot_01".to_string()));
        assert!(g1.source_bots.contains(&"bot_02".to_string()));
    }
}
