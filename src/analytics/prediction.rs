//! Terrain Prediction & Extrapolation
//!
//! Phase 6.1: Predict terrain elevation at unobserved locations using
//! spatial interpolation, regression, and machine learning.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prediction model types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PredictionModel {
    LinearRegression,
    PolynomialRegression(u32), // degree
    InverseDistance,           // Inverse Distance Weighting
    KrigingOrdinary,           // Kriging interpolation
    NeuralNetwork,             // NN-based (placeholder)
}

/// Training sample for regression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingSample {
    pub location: (f32, f32),
    pub elevation: f32,
    pub weight: f32, // Importance weight
}

/// Regression coefficients
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegressionCoefficients {
    pub intercept: f32,
    pub x_coeff: f32,
    pub y_coeff: f32,
    pub x2_coeff: f32,  // For polynomial
    pub y2_coeff: f32,  // For polynomial
    pub xy_coeff: f32,  // For polynomial
    pub r_squared: f32, // Model fit quality
}

impl RegressionCoefficients {
    /// Create new coefficients
    pub fn new() -> Self {
        RegressionCoefficients {
            intercept: 0.0,
            x_coeff: 0.0,
            y_coeff: 0.0,
            x2_coeff: 0.0,
            y2_coeff: 0.0,
            xy_coeff: 0.0,
            r_squared: 0.0,
        }
    }

    /// Predict elevation at location
    pub fn predict(&self, x: f32, y: f32) -> f32 {
        self.intercept +
            self.x_coeff * x +
            self.y_coeff * y +
            self.x2_coeff * x * x +
            self.y2_coeff * y * y +
            self.xy_coeff * x * y
    }
}

impl Default for RegressionCoefficients {
    fn default() -> Self {
        Self::new()
    }
}

/// Prediction result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElevationPrediction {
    pub location: (f32, f32),
    pub predicted_elevation: f32,
    pub confidence: f32,     // 0.0-1.0
    pub error_estimate: f32, // Estimated standard error
    pub model_used: String,
}

impl ElevationPrediction {
    /// Create prediction
    pub fn new(
        location: (f32, f32),
        elevation: f32,
        confidence: f32,
        error: f32,
        model: &str,
    ) -> Self {
        ElevationPrediction {
            location,
            predicted_elevation: elevation,
            confidence: confidence.clamp(0.0, 1.0),
            error_estimate: error,
            model_used: model.to_string(),
        }
    }
}

/// Terrain predictor
pub struct TerrainPredictor {
    pub model: PredictionModel,
    pub coefficients: RegressionCoefficients,
    pub training_samples: Vec<TrainingSample>,
    pub predictions_cache: HashMap<String, ElevationPrediction>,
}

impl TerrainPredictor {
    /// Create new predictor
    pub fn new(model: PredictionModel) -> Self {
        TerrainPredictor {
            model,
            coefficients: RegressionCoefficients::new(),
            training_samples: Vec::new(),
            predictions_cache: HashMap::new(),
        }
    }

    /// Add training sample
    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.training_samples.push(sample);
        self.predictions_cache.clear(); // Invalidate cache
    }

    /// Train linear regression model
    pub fn train_linear(&mut self) -> bool {
        if self.training_samples.len() < 3 {
            return false;
        }

        // Simple linear regression: z = a + b*x + c*y
        let n = self.training_samples.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        let mut sum_xz = 0.0;
        let mut sum_yz = 0.0;

        for sample in &self.training_samples {
            let x = sample.location.0;
            let y = sample.location.1;
            let z = sample.elevation;
            let w = sample.weight;

            sum_x += x * w;
            sum_y += y * w;
            sum_z += z * w;
            sum_x2 += x * x * w;
            sum_y2 += y * y * w;
            sum_xz += x * z * w;
            sum_yz += y * z * w;
        }

        let mean_x = sum_x / n;
        let mean_y = sum_y / n;
        let mean_z = sum_z / n;

        // Compute covariances
        let var_x = sum_x2 / n - mean_x * mean_x;
        let var_y = sum_y2 / n - mean_y * mean_y;
        let cov_xz = sum_xz / n - mean_x * mean_z;
        let cov_yz = sum_yz / n - mean_y * mean_z;

        // Solve system (simplified for 2D)
        let det = var_x * var_y;
        if det.abs() < 1e-6 {
            return false;
        }

        self.coefficients.x_coeff = (cov_xz * var_y - cov_yz * var_y) / det;
        self.coefficients.y_coeff = (cov_yz * var_x - cov_xz * var_x) / det;
        self.coefficients.intercept = mean_z - self.coefficients.x_coeff * mean_x - self.coefficients.y_coeff * mean_y;

        // Compute R-squared
        let mut ss_res = 0.0;
        let mut ss_tot = 0.0;
        for sample in &self.training_samples {
            let predicted = self.coefficients.predict(sample.location.0, sample.location.1);
            let residual = sample.elevation - predicted;
            ss_res += residual * residual;
            ss_tot += (sample.elevation - mean_z).powi(2);
        }

        self.coefficients.r_squared = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

        true
    }

    /// Predict using inverse distance weighting
    pub fn predict_idw(&mut self, x: f32, y: f32, power: f32) -> Option<ElevationPrediction> {
        if self.training_samples.is_empty() {
            return None;
        }

        let cache_key = format!("{:.2}_{:.2}", x, y);
        if let Some(cached) = self.predictions_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        let mut weighted_z = 0.0;
        let mut total_weight = 0.0;
        let mut min_distance = f32::INFINITY;

        for sample in &self.training_samples {
            let dx = sample.location.0 - x;
            let dy = sample.location.1 - y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 0.001 {
                // Exact match
                return Some(ElevationPrediction::new(
                    (x, y),
                    sample.elevation,
                    1.0,
                    0.0,
                    "IDW (exact match)",
                ));
            }

            min_distance = min_distance.min(distance);
            let weight = sample.weight / distance.powf(power);
            weighted_z += sample.elevation * weight;
            total_weight += weight;
        }

        let predicted_z = weighted_z / total_weight;
        let confidence = (1.0 / (1.0 + min_distance)).min(1.0);
        let error = min_distance * 0.1; // Rough estimate

        let prediction = ElevationPrediction::new(
            (x, y),
            predicted_z,
            confidence,
            error,
            "IDW",
        );

        self.predictions_cache.insert(cache_key, prediction.clone());
        Some(prediction)
    }

    /// Predict using trained regression model
    pub fn predict_regression(&mut self, x: f32, y: f32) -> Option<ElevationPrediction> {
        if self.training_samples.is_empty() {
            return None;
        }

        let cache_key = format!("{:.2}_{:.2}", x, y);
        if let Some(cached) = self.predictions_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        let predicted_z = self.coefficients.predict(x, y);
        let confidence = self.coefficients.r_squared.max(0.3); // Min 0.3
        let error = (1.0 - confidence) * 10.0; // Scale error inversely to R²

        let prediction = ElevationPrediction::new(
            (x, y),
            predicted_z,
            confidence,
            error,
            "Linear Regression",
        );

        self.predictions_cache.insert(cache_key, prediction.clone());
        Some(prediction)
    }

    /// Get predictor statistics
    pub fn statistics(&self) -> PredictionStatistics {
        PredictionStatistics {
            training_samples: self.training_samples.len() as u32,
            model_r_squared: self.coefficients.r_squared,
            cached_predictions: self.predictions_cache.len() as u32,
        }
    }
}

impl Default for TerrainPredictor {
    fn default() -> Self {
        Self::new(PredictionModel::LinearRegression)
    }
}

/// Prediction statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionStatistics {
    pub training_samples: u32,
    pub model_r_squared: f32,
    pub cached_predictions: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_sample() {
        let sample = TrainingSample {
            location: (1.0, 2.0),
            elevation: 3.0,
            weight: 1.0,
        };

        assert_eq!(sample.location, (1.0, 2.0));
        assert_eq!(sample.elevation, 3.0);
    }

    #[test]
    fn test_regression_coefficients() {
        let coeff = RegressionCoefficients::new();
        let prediction = coeff.predict(1.0, 2.0);
        assert_eq!(prediction, 0.0); // All zeros
    }

    #[test]
    fn test_elevation_prediction() {
        let pred = ElevationPrediction::new((1.0, 2.0), 5.0, 0.9, 0.1, "test");
        assert_eq!(pred.location, (1.0, 2.0));
        assert_eq!(pred.predicted_elevation, 5.0);
    }

    #[test]
    fn test_terrain_predictor() {
        let mut predictor = TerrainPredictor::new(PredictionModel::LinearRegression);
        let sample = TrainingSample {
            location: (1.0, 2.0),
            elevation: 5.0,
            weight: 1.0,
        };

        predictor.add_sample(sample);
        assert_eq!(predictor.training_samples.len(), 1);
    }

    #[test]
    fn test_predictor_idw() {
        let mut predictor = TerrainPredictor::new(PredictionModel::InverseDistance);
        predictor.add_sample(TrainingSample {
            location: (0.0, 0.0),
            elevation: 100.0,
            weight: 1.0,
        });
        predictor.add_sample(TrainingSample {
            location: (10.0, 0.0),
            elevation: 200.0,
            weight: 1.0,
        });

        let prediction = predictor.predict_idw(5.0, 0.0, 2.0);
        assert!(prediction.is_some());
    }

    #[test]
    fn test_predictor_statistics() {
        let mut predictor = TerrainPredictor::default();
        predictor.add_sample(TrainingSample {
            location: (1.0, 2.0),
            elevation: 5.0,
            weight: 1.0,
        });

        let stats = predictor.statistics();
        assert_eq!(stats.training_samples, 1);
    }
}