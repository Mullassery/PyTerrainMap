//! PyRoboVision adapter for vision model registry and terrain-aware model selection
//!
//! Integrates PyRoboVision's model performance metrics with PyTerrainMap fusion,
//! enabling adaptive model selection based on terrain, time of day, weather, etc.

use crate::types::{Observation, Result, Error};
use std::collections::HashMap;

/// Model performance metrics for a specific terrain
#[derive(Clone, Debug)]
pub struct ModelMetrics {
    /// Model identifier (e.g., "YOLOv11-RGB", "thermal_detector_v2")
    pub model_id: String,
    /// Mean average precision on test set (0.0-1.0)
    pub map_score: f32,
    /// Terrain type this metric is for (e.g., "rocky", "grassy", "urban_rubble")
    pub terrain_type: String,
    /// Time-of-day applicability ("day", "night", "any")
    pub time_context: String,
    /// Weather condition ("clear", "rain", "fog", "any")
    pub weather_context: String,
    /// Whether this model is currently enabled
    pub enabled: bool,
}

/// Vision model registry from PyRoboVision
#[derive(Clone, Debug)]
pub struct VisionModelRegistry {
    /// All available models: model_id -> ModelMetrics
    models: HashMap<String, ModelMetrics>,
}

impl VisionModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        VisionModelRegistry {
            models: HashMap::new(),
        }
    }

    /// Register a model with its performance metrics
    pub fn register(&mut self, metrics: ModelMetrics) {
        self.models.insert(metrics.model_id.clone(), metrics);
    }

    /// Get model metrics by ID
    pub fn get_model(&self, model_id: &str) -> Option<&ModelMetrics> {
        self.models.get(model_id)
    }

    /// Get all models for a terrain type
    pub fn get_models_for_terrain(&self, terrain_type: &str) -> Vec<&ModelMetrics> {
        self.models
            .values()
            .filter(|m| m.terrain_type == terrain_type && m.enabled)
            .collect()
    }
}

/// VisionModelAwareAdapter: Weighs sensor fusion by model performance
///
/// Computes: weight = model_mAP × inference_confidence × sensor_quality
/// Enables adaptive model selection and multi-model consensus validation.
pub struct VisionModelAwareAdapter {
    /// Model registry with mAP scores per terrain
    registry: VisionModelRegistry,
    /// Inferred terrain type for current scene
    current_terrain: String,
}

impl VisionModelAwareAdapter {
    /// Create adapter with model registry
    pub fn new(registry: VisionModelRegistry) -> Self {
        VisionModelAwareAdapter {
            registry,
            current_terrain: "unknown".to_string(),
        }
    }

    /// Set inferred terrain for current processing context
    pub fn set_terrain(&mut self, terrain_type: String) {
        self.current_terrain = terrain_type;
    }

    /// Compute fusion weight incorporating model mAP
    ///
    /// weight = model_mAP × inference_confidence × sensor_quality
    pub fn compute_fusion_weight(
        &self,
        model_id: &str,
        inference_confidence: f32,
        sensor_quality: f32,
    ) -> Result<f32> {
        let model = self.registry.get_model(model_id)
            .ok_or_else(|| Error::InvalidObservation(
                format!("Model {} not found in registry", model_id),
            ))?;

        if !model.enabled {
            return Err(Error::InvalidObservation(
                format!("Model {} is disabled", model_id),
            ));
        }

        // Clamp values to valid ranges
        let mAP = model.map_score.clamp(0.0, 1.0);
        let conf = inference_confidence.clamp(0.0, 1.0);
        let quality = sensor_quality.clamp(0.0, 1.0);

        Ok(mAP * conf * quality)
    }

    /// Select best model for current terrain and context
    pub fn select_best_model(&self, time_context: &str, weather: &str) -> Result<ModelMetrics> {
        let candidates = self.registry.get_models_for_terrain(&self.current_terrain);

        if candidates.is_empty() {
            return Err(Error::InvalidObservation(
                format!("No models available for terrain '{}'", self.current_terrain),
            ));
        }

        // Filter by time and weather context
        let mut matching_models: Vec<_> = candidates
            .iter()
            .filter(|m| {
                (m.time_context == "any" || m.time_context == time_context) &&
                (m.weather_context == "any" || m.weather_context == weather)
            })
            .collect();

        // If no exact match, fall back to "any" context models
        if matching_models.is_empty() {
            matching_models = candidates
                .iter()
                .filter(|m| m.time_context == "any" || m.weather_context == "any")
                .collect();
        }

        // Select model with highest mAP
        let best = matching_models
            .iter()
            .max_by(|a, b| a.map_score.partial_cmp(&b.map_score).unwrap())
            .ok_or_else(|| Error::InvalidObservation(
                "No suitable model found".to_string(),
            ))?;

        Ok((**best).clone())
    }

    /// Validate multi-model consensus
    ///
    /// Returns agreement score (0.0-1.0) based on confidence correlation
    pub fn validate_consensus(
        &self,
        model_confidences: &HashMap<String, f32>,
    ) -> Result<f32> {
        if model_confidences.len() < 2 {
            return Ok(1.0); // Single model always agrees with itself
        }

        let values: Vec<f32> = model_confidences.values().copied().collect();

        // Compute mean
        let mean = values.iter().sum::<f32>() / values.len() as f32;

        // Compute standard deviation
        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;

        let std_dev = variance.sqrt();

        // Agreement score: high if std_dev is low
        // score = 1.0 - (std_dev / 0.3).min(1.0)
        // This means: ±0.3 std_dev = agreement score 0.0
        Ok(1.0 - (std_dev / 0.3).min(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> VisionModelRegistry {
        let mut registry = VisionModelRegistry::new();

        // Register some test models
        registry.register(ModelMetrics {
            model_id: "YOLOv11-RGB".to_string(),
            map_score: 0.88,
            terrain_type: "rocky".to_string(),
            time_context: "day".to_string(),
            weather_context: "clear".to_string(),
            enabled: true,
        });

        registry.register(ModelMetrics {
            model_id: "thermal_detector_v2".to_string(),
            map_score: 0.76,
            terrain_type: "rocky".to_string(),
            time_context: "night".to_string(),
            weather_context: "clear".to_string(),
            enabled: true,
        });

        registry.register(ModelMetrics {
            model_id: "LiDAR_clustering".to_string(),
            map_score: 0.82,
            terrain_type: "rocky".to_string(),
            time_context: "any".to_string(),
            weather_context: "any".to_string(),
            enabled: true,
        });

        registry
    }

    #[test]
    fn test_adapter_creation() {
        let registry = create_test_registry();
        let adapter = VisionModelAwareAdapter::new(registry);
        assert_eq!(adapter.current_terrain, "unknown");
    }

    #[test]
    fn test_compute_fusion_weight() {
        let registry = create_test_registry();
        let adapter = VisionModelAwareAdapter::new(registry);

        // weight = 0.88 (mAP) × 0.95 (inference conf) × 0.90 (sensor quality)
        let weight = adapter.compute_fusion_weight("YOLOv11-RGB", 0.95, 0.90).unwrap();
        assert!((weight - 0.7533).abs() < 0.01);
    }

    #[test]
    fn test_select_best_model_rocky_day() {
        let registry = create_test_registry();
        let mut adapter = VisionModelAwareAdapter::new(registry);
        adapter.set_terrain("rocky".to_string());

        let best = adapter.select_best_model("day", "clear").unwrap();
        assert_eq!(best.model_id, "YOLOv11-RGB");
        assert_eq!(best.map_score, 0.88);
    }

    #[test]
    fn test_select_best_model_rocky_night() {
        let registry = create_test_registry();
        let mut adapter = VisionModelAwareAdapter::new(registry);
        adapter.set_terrain("rocky".to_string());

        let best = adapter.select_best_model("night", "clear").unwrap();
        assert_eq!(best.model_id, "thermal_detector_v2");
        assert_eq!(best.map_score, 0.76);
    }

    #[test]
    fn test_multi_model_consensus_perfect_agreement() {
        let registry = create_test_registry();
        let adapter = VisionModelAwareAdapter::new(registry);

        let mut confidences = HashMap::new();
        confidences.insert("model1".to_string(), 0.9);
        confidences.insert("model2".to_string(), 0.9);
        confidences.insert("model3".to_string(), 0.9);

        let agreement = adapter.validate_consensus(&confidences).unwrap();
        assert!((agreement - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_model_consensus_disagreement() {
        let registry = create_test_registry();
        let adapter = VisionModelAwareAdapter::new(registry);

        let mut confidences = HashMap::new();
        confidences.insert("model1".to_string(), 0.9);
        confidences.insert("model2".to_string(), 0.3);

        let agreement = adapter.validate_consensus(&confidences).unwrap();
        assert!(agreement < 0.5); // Models disagree
    }

    #[test]
    fn test_disabled_model_error() {
        let mut registry = create_test_registry();

        // Disable a model
        if let Some(model) = registry.models.get_mut("YOLOv11-RGB") {
            model.enabled = false;
        }

        let adapter = VisionModelAwareAdapter::new(registry);
        let result = adapter.compute_fusion_weight("YOLOv11-RGB", 0.9, 0.9);
        assert!(result.is_err());
    }
}
