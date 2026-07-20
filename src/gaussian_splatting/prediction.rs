use crate::gaussian_splatting::core::{GaussianCovariance, SplatKind, TerrainGaussian};
use crate::gaussian_splatting::store::GaussianSplatStore;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Result of predicting an unknown region
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub prediction_id: Uuid,
    pub was_correct: bool,
    pub error_magnitude: f32,
}

/// A predicted splat for an unknown region
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedSplat {
    pub terrain_gaussian: TerrainGaussian,
    pub prediction_confidence: f32,
    pub basis_splat_ids: Vec<Uuid>,
    pub is_verified: bool,
    pub verified_at: Option<i64>,
}

/// Predicts terrain characteristics in unknown regions via neighbor inference
pub struct UnknownRegionPredictor {
    pub neighbor_radius_m: f64,  // default 25.0m
    pub min_neighbors: usize,    // default 3
}

impl UnknownRegionPredictor {
    pub fn new() -> Self {
        UnknownRegionPredictor {
            neighbor_radius_m: 25.0,
            min_neighbors: 3,
        }
    }

    /// Predict terrain at a position based on nearby observations
    pub fn predict_at(
        &self,
        pos: [f64; 3],
        store: &GaussianSplatStore,
    ) -> Option<PredictedSplat> {
        let nearby = store.query_radius(pos, self.neighbor_radius_m);

        if nearby.len() < self.min_neighbors {
            return None;
        }

        // Majority vote for terrain type (weighted by confidence)
        let mut terrain_votes: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        let mut total_confidence = 0.0;

        for splat in &nearby {
            let vote = terrain_votes
                .entry(splat.terrain_type.as_str().to_string())
                .or_insert(0.0);
            *vote += splat.confidence;
            total_confidence += splat.confidence;
        }

        let terrain_type = terrain_votes
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| crate::gaussian_splatting::core::TerrainType::from_str(k))
            .unwrap_or(crate::gaussian_splatting::core::TerrainType::Unknown(0));

        // Average traversability
        let avg_traversability =
            nearby.iter().map(|s| s.traversability).sum::<f32>() / (nearby.len() as f32);

        // Average covariance (but doubled for extra uncertainty)
        let mut avg_cov = [[0.0; 3]; 3];
        for splat in &nearby {
            for i in 0..3 {
                for j in 0..3 {
                    avg_cov[i][j] += splat.covariance.matrix[i][j];
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                avg_cov[i][j] /= nearby.len() as f32;
                avg_cov[i][j] *= 2.0;  // Double uncertainty for predictions
            }
        }

        let avg_confidence = (nearby.len() as f32 / 10.0).min(1.0)
            * (total_confidence / (nearby.len() as f32))
            * 0.8;

        let basis_ids = nearby.iter().map(|s| s.id).collect();

        let mut predicted = TerrainGaussian {
            id: Uuid::new_v4(),
            position: pos,
            covariance: GaussianCovariance {
                matrix: avg_cov,
            },
            traversability: avg_traversability.clamp(0.0, 1.0),
            terrain_type,
            confidence: avg_confidence,
            observation_count: 0,
            created_at: chrono::Utc::now().timestamp_micros(),
            last_updated: chrono::Utc::now().timestamp_micros(),
            source_bots: vec![],
            splat_kind: SplatKind::Prediction,
            metadata: std::collections::HashMap::new(),
        };
        predicted.metadata.insert("predicted".to_string(), "true".to_string());

        Some(PredictedSplat {
            terrain_gaussian: predicted,
            prediction_confidence: avg_confidence,
            basis_splat_ids: basis_ids,
            is_verified: false,
            verified_at: None,
        })
    }

    /// Fill frontier locations with predictions
    pub fn fill_frontier_predictions(
        &self,
        frontiers: &[[f64; 3]],
        store: &GaussianSplatStore,
    ) -> Vec<PredictedSplat> {
        frontiers
            .iter()
            .filter_map(|pos| self.predict_at(*pos, store))
            .collect()
    }

    /// Verify a prediction with actual observation
    pub fn verify_prediction(
        &self,
        pred_id: Uuid,
        actual: TerrainGaussian,
        pred_splat: &mut PredictedSplat,
    ) -> VerificationResult {
        let error = (actual.traversability - pred_splat.terrain_gaussian.traversability).abs();
        let was_correct = error < 0.2;

        pred_splat.is_verified = true;
        pred_splat.verified_at = Some(chrono::Utc::now().timestamp_micros());

        VerificationResult {
            prediction_id: pred_id,
            was_correct,
            error_magnitude: error,
        }
    }
}

impl Default for UnknownRegionPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_region_predictor_creation() {
        let predictor = UnknownRegionPredictor::new();
        assert_eq!(predictor.neighbor_radius_m, 25.0);
        assert_eq!(predictor.min_neighbors, 3);
    }

    #[test]
    fn test_predict_at_insufficient_neighbors() {
        let predictor = UnknownRegionPredictor::new();
        let store = GaussianSplatStore::new();
        let result = predictor.predict_at([10.0, 20.0, 0.0], &store);
        assert!(result.is_none());
    }
}
