use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

/// Terrain classification enum
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Road,
    Grass,
    Mud,
    Water,
    Sand,
    Corridor,
    Stairs,
    Obstacle,
    ChargingStation,
    RestrictedArea,
    Unknown(u8),  // Use u8 to avoid String in Copy enum, map to string on demand
}

impl TerrainType {
    pub fn as_str(&self) -> &str {
        match self {
            TerrainType::Road => "Road",
            TerrainType::Grass => "Grass",
            TerrainType::Mud => "Mud",
            TerrainType::Water => "Water",
            TerrainType::Sand => "Sand",
            TerrainType::Corridor => "Corridor",
            TerrainType::Stairs => "Stairs",
            TerrainType::Obstacle => "Obstacle",
            TerrainType::ChargingStation => "ChargingStation",
            TerrainType::RestrictedArea => "RestrictedArea",
            TerrainType::Unknown(_) => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Road" => TerrainType::Road,
            "Grass" => TerrainType::Grass,
            "Mud" => TerrainType::Mud,
            "Water" => TerrainType::Water,
            "Sand" => TerrainType::Sand,
            "Corridor" => TerrainType::Corridor,
            "Stairs" => TerrainType::Stairs,
            "Obstacle" => TerrainType::Obstacle,
            "ChargingStation" => TerrainType::ChargingStation,
            "RestrictedArea" => TerrainType::RestrictedArea,
            _ => TerrainType::Unknown(0),
        }
    }
}

/// Splat categorization for filtering and statistics
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum SplatKind {
    Terrain,
    Traversability,
    Semantic,
    Passage,
    Prediction,
    Temporal,
}

/// 3×3 covariance matrix representation with explicit Cramer's-rule inverse
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GaussianCovariance {
    pub matrix: [[f32; 3]; 3],
}

impl GaussianCovariance {
    /// Create isotropic covariance (σ²I)
    pub fn isotropic(std_dev: f32) -> Self {
        let var = std_dev * std_dev;
        GaussianCovariance {
            matrix: [
                [var, 0.0, 0.0],
                [0.0, var, 0.0],
                [0.0, 0.0, var],
            ],
        }
    }

    /// Create diagonal covariance with separate standard deviations
    pub fn diagonal(sx: f32, sy: f32, sz: f32) -> Self {
        GaussianCovariance {
            matrix: [
                [sx * sx, 0.0, 0.0],
                [0.0, sy * sy, 0.0],
                [0.0, 0.0, sz * sz],
            ],
        }
    }

    /// Compute determinant using Rule of Sarrus
    pub fn determinant(&self) -> f32 {
        let m = self.matrix;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Inverse using Cramer's rule (3×3)
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-8 {
            return None;
        }

        let m = self.matrix;
        let inv_det = 1.0 / det;

        let c00 = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det;
        let c01 = -(m[0][1] * m[2][2] - m[0][2] * m[2][1]) * inv_det;
        let c02 = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det;

        let c10 = -(m[1][0] * m[2][2] - m[1][2] * m[2][0]) * inv_det;
        let c11 = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det;
        let c12 = -(m[0][0] * m[1][2] - m[0][2] * m[1][0]) * inv_det;

        let c20 = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det;
        let c21 = -(m[0][0] * m[2][1] - m[0][1] * m[2][0]) * inv_det;
        let c22 = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det;

        Some(GaussianCovariance {
            matrix: [
                [c00, c01, c02],
                [c10, c11, c12],
                [c20, c21, c22],
            ],
        })
    }

    /// Uncertainty volume: (2π)^(3/2) * √det(Σ)
    pub fn uncertainty_volume(&self) -> f32 {
        let det = self.determinant();
        if det <= 0.0 {
            return 0.0;
        }
        let factor = (2.0 * std::f32::consts::PI).powf(1.5);
        factor * det.sqrt()
    }

    /// Squared Mahalanobis distance: δᵀΣ⁻¹δ
    pub fn mahalanobis_sq(&self, delta: [f32; 3]) -> f32 {
        match self.inverse() {
            None => f32::INFINITY,
            Some(inv_cov) => {
                let m = inv_cov.matrix;
                let d = delta;
                (d[0] * (m[0][0] * d[0] + m[0][1] * d[1] + m[0][2] * d[2]))
                    + (d[1] * (m[1][0] * d[0] + m[1][1] * d[1] + m[1][2] * d[2]))
                    + (d[2] * (m[2][0] * d[0] + m[2][1] * d[1] + m[2][2] * d[2]))
            }
        }
    }

    /// Fuse two covariances: (Σ₁⁻¹ + Σ₂⁻¹)⁻¹
    pub fn fuse(&self, other: &Self) -> Self {
        match (self.inverse(), other.inverse()) {
            (Some(inv1), Some(inv2)) => {
                let m1 = inv1.matrix;
                let m2 = inv2.matrix;
                let sum = [
                    [m1[0][0] + m2[0][0], m1[0][1] + m2[0][1], m1[0][2] + m2[0][2]],
                    [m1[1][0] + m2[1][0], m1[1][1] + m2[1][1], m1[1][2] + m2[1][2]],
                    [m1[2][0] + m2[2][0], m1[2][1] + m2[2][1], m1[2][2] + m2[2][2]],
                ];
                let temp = GaussianCovariance { matrix: sum };
                temp.inverse().unwrap_or_else(|| self.clone())
            }
            _ => self.clone(),
        }
    }
}

impl Default for GaussianCovariance {
    fn default() -> Self {
        Self::isotropic(1.0)
    }
}

/// Core terrain Gaussian splat: position + covariance + traversability + type + temporal + source info
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainGaussian {
    pub id: Uuid,
    pub position: [f64; 3],              // [lat, lon, elevation_m]
    pub covariance: GaussianCovariance,
    pub traversability: f32,             // 0.0 = impassable, 1.0 = perfect
    pub terrain_type: TerrainType,
    pub confidence: f32,                 // [0.0, 1.0], decays over time
    pub observation_count: u32,
    pub created_at: i64,                 // microseconds since epoch
    pub last_updated: i64,
    pub source_bots: Vec<String>,
    pub splat_kind: SplatKind,
    pub metadata: HashMap<String, String>,
}

impl TerrainGaussian {
    /// Create a terrain Gaussian from a point observation
    pub fn from_point_observation(
        pos: [f64; 3],
        bot_id: &str,
        traversability: f32,
    ) -> Self {
        let now = Utc::now().timestamp_micros();
        TerrainGaussian {
            id: Uuid::new_v4(),
            position: pos,
            covariance: GaussianCovariance::isotropic(1.0),  // 1m standard deviation
            traversability: traversability.clamp(0.0, 1.0),
            terrain_type: TerrainType::Unknown(0),
            confidence: 0.8,
            observation_count: 1,
            created_at: now,
            last_updated: now,
            source_bots: vec![bot_id.to_string()],
            splat_kind: SplatKind::Terrain,
            metadata: HashMap::new(),
        }
    }

    /// Compute overlap with another splat using Mahalanobis distance
    pub fn overlap_mahalanobis(&self, other: &TerrainGaussian) -> f32 {
        let delta = [
            (other.position[0] - self.position[0]) as f32,
            (other.position[1] - self.position[1]) as f32,
            (other.position[2] - self.position[2]) as f32,
        ];
        self.covariance.mahalanobis_sq(delta)
    }

    /// Check if two splats overlap significantly (within 3σ)
    pub fn overlaps(&self, other: &TerrainGaussian, sigma_threshold: f32) -> bool {
        self.overlap_mahalanobis(other) <= sigma_threshold * sigma_threshold
    }

    /// Clamp confidence to [0.0, 1.0]
    pub fn set_confidence(&mut self, confidence: f32) {
        self.confidence = confidence.clamp(0.0, 1.0);
    }

    /// Increase confidence (used after successful fusion)
    pub fn increase_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence + delta).min(1.0);
    }

    /// Decrease confidence (used after conflicting observations)
    pub fn decrease_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence - delta).max(0.0);
    }

    /// Record a bot as an observer
    pub fn add_source_bot(&mut self, bot_id: &str) {
        if !self.source_bots.contains(&bot_id.to_string()) {
            self.source_bots.push(bot_id.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_covariance_isotropic() {
        let cov = GaussianCovariance::isotropic(2.0);
        let var = 4.0;
        assert!((cov.matrix[0][0] - var).abs() < 0.01);
        assert!((cov.matrix[1][1] - var).abs() < 0.01);
        assert!((cov.matrix[2][2] - var).abs() < 0.01);
        assert_eq!(cov.matrix[0][1], 0.0);
    }

    #[test]
    fn test_gaussian_covariance_diagonal() {
        let cov = GaussianCovariance::diagonal(1.0, 2.0, 3.0);
        assert_eq!(cov.matrix[0][0], 1.0);
        assert_eq!(cov.matrix[1][1], 4.0);
        assert_eq!(cov.matrix[2][2], 9.0);
    }

    #[test]
    fn test_gaussian_covariance_determinant() {
        let cov = GaussianCovariance::isotropic(2.0);
        let det = cov.determinant();
        let expected = 4.0 * 4.0 * 4.0;  // 64
        assert!((det - expected).abs() < 0.01);
    }

    #[test]
    fn test_gaussian_covariance_inverse() {
        let cov = GaussianCovariance::diagonal(2.0, 2.0, 2.0);
        let inv = cov.inverse().unwrap();
        let product = [
            [
                cov.matrix[0][0] * inv.matrix[0][0]
                    + cov.matrix[0][1] * inv.matrix[1][0]
                    + cov.matrix[0][2] * inv.matrix[2][0],
                0.0,
                0.0,
            ],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        assert!((product[0][0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gaussian_covariance_mahalanobis() {
        let cov = GaussianCovariance::isotropic(1.0);
        let delta = [1.0, 0.0, 0.0];
        let dist_sq = cov.mahalanobis_sq(delta);
        assert!((dist_sq - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gaussian_covariance_uncertainty_volume() {
        let cov = GaussianCovariance::isotropic(1.0);
        let vol = cov.uncertainty_volume();
        let expected = (2.0 * std::f32::consts::PI).powf(1.5);
        assert!((vol - expected).abs() < 0.1);
    }

    #[test]
    fn test_terrain_gaussian_from_point() {
        let gaussian = TerrainGaussian::from_point_observation(
            [40.7128, -74.0060, 10.0],
            "bot_01",
            0.85,
        );
        assert_eq!(gaussian.traversability, 0.85);
        assert!(gaussian.confidence > 0.7);
        assert_eq!(gaussian.observation_count, 1);
        assert!(gaussian.source_bots.contains(&"bot_01".to_string()));
    }

    #[test]
    fn test_terrain_gaussian_confidence_clamping() {
        let mut gaussian =
            TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.5);
        gaussian.set_confidence(1.5);
        assert_eq!(gaussian.confidence, 1.0);
        gaussian.set_confidence(-0.5);
        assert_eq!(gaussian.confidence, 0.0);
    }

    #[test]
    fn test_terrain_gaussian_overlaps() {
        let g1 = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.5);
        let g2 = TerrainGaussian::from_point_observation([0.001, 0.001, 0.1], "bot_02", 0.6);
        assert!(g1.overlaps(&g2, 3.0));
    }

    #[test]
    fn test_terrain_type_str() {
        assert_eq!(TerrainType::Road.as_str(), "Road");
        assert_eq!(TerrainType::Grass.as_str(), "Grass");
        assert_eq!(TerrainType::Unknown(0).as_str(), "Unknown");
    }

    #[test]
    fn test_terrain_type_from_str() {
        assert_eq!(TerrainType::from_str("Road"), TerrainType::Road);
        assert_eq!(TerrainType::from_str("Grass"), TerrainType::Grass);
        assert!(matches!(TerrainType::from_str("invalid"), TerrainType::Unknown(_)));
    }
}
