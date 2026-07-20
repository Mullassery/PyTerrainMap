//! Active learning and prediction validation
//!
//! Track prediction accuracy, identify errors, and improve models based on
//! real-world outcomes from exploration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::traversability::TraversabilityObservation;
use super::predictions::PredictiveModel;

/// Outcome of a prediction validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionOutcome {
    pub prediction_id: String,
    pub predicted_value: String,
    pub actual_value: String,
    pub error_magnitude: f32,  // 0.0 = perfect, 1.0 = completely wrong
    pub prediction_confidence: f32,
    pub outcome_timestamp: i64,
    pub was_correct: bool,
}

impl PredictionOutcome {
    /// Create a new prediction outcome
    pub fn new(
        prediction_id: String,
        predicted_value: String,
        actual_value: String,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let was_correct = predicted_value == actual_value;
        let error_magnitude = if was_correct { 0.0 } else { 1.0 };

        PredictionOutcome {
            prediction_id,
            predicted_value,
            actual_value,
            error_magnitude,
            prediction_confidence: 0.5,
            outcome_timestamp: now,
            was_correct,
        }
    }

    /// Set continuous error magnitude (for numeric predictions)
    pub fn with_magnitude(mut self, magnitude: f32) -> Self {
        self.error_magnitude = magnitude.max(0.0).min(1.0);
        self
    }

    /// Set prediction confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.prediction_confidence = confidence.max(0.0).min(1.0);
        self
    }
}

/// Prediction validator
#[derive(Clone, Debug)]
pub struct PredictionValidator {
    pub outcomes: Vec<PredictionOutcome>,
}

impl PredictionValidator {
    /// Create a new validator
    pub fn new() -> Self {
        PredictionValidator {
            outcomes: Vec::new(),
        }
    }

    /// Record an outcome
    pub fn record_outcome(&mut self, outcome: PredictionOutcome) {
        self.outcomes.push(outcome);
    }

    /// Get accuracy metrics
    pub fn accuracy_metrics(&self) -> AccuracyMetrics {
        if self.outcomes.is_empty() {
            return AccuracyMetrics::default();
        }

        let total = self.outcomes.len() as f32;
        let correct = self.outcomes.iter().filter(|o| o.was_correct).count() as f32;
        let avg_error: f32 = self.outcomes.iter().map(|o| o.error_magnitude).sum::<f32>() / total;
        let avg_confidence: f32 = self.outcomes.iter().map(|o| o.prediction_confidence).sum::<f32>() / total;

        // Calibration: are high-confidence predictions actually correct?
        let high_conf_correct = self.outcomes
            .iter()
            .filter(|o| o.prediction_confidence > 0.7 && o.was_correct)
            .count() as f32;
        let high_conf_total = self.outcomes
            .iter()
            .filter(|o| o.prediction_confidence > 0.7)
            .count() as f32;
        let calibration = if high_conf_total > 0.0 {
            high_conf_correct / high_conf_total
        } else {
            0.5
        };

        AccuracyMetrics {
            accuracy_rate: correct / total,
            average_error: avg_error,
            average_confidence: avg_confidence,
            calibration_score: calibration,
            total_predictions: self.outcomes.len() as u32,
        }
    }

    /// Get outcomes within time window (seconds)
    pub fn recent_outcomes(&self, window_seconds: i64) -> Vec<&PredictionOutcome> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.outcomes
            .iter()
            .filter(|o| (now - o.outcome_timestamp) < window_seconds)
            .collect()
    }

    /// Identify systematic errors (predictions that are consistently wrong)
    pub fn systematic_errors(&self) -> HashMap<String, ErrorPattern> {
        let mut patterns = HashMap::new();

        for outcome in &self.outcomes {
            if !outcome.was_correct {
                let pattern = patterns
                    .entry(outcome.predicted_value.clone())
                    .or_insert_with(|| ErrorPattern {
                        predicted_value: outcome.predicted_value.clone(),
                        error_count: 0,
                        avg_magnitude: 0.0,
                        frequencies: HashMap::new(),
                    });

                pattern.error_count += 1;
                pattern.avg_magnitude =
                    (pattern.avg_magnitude * (pattern.error_count - 1) as f32
                        + outcome.error_magnitude)
                        / pattern.error_count as f32;

                *pattern
                    .frequencies
                    .entry(outcome.actual_value.clone())
                    .or_insert(0) += 1;
            }
        }

        patterns
    }

    /// Get confidence calibration
    pub fn confidence_calibration(&self) -> ConfidenceCalibration {
        let buckets = 10;
        let mut bucket_size = vec![0; buckets];
        let mut bucket_correct = vec![0; buckets];

        for outcome in &self.outcomes {
            let bucket_idx = ((outcome.prediction_confidence * buckets as f32) as usize).min(buckets - 1);
            bucket_size[bucket_idx] += 1;
            if outcome.was_correct {
                bucket_correct[bucket_idx] += 1;
            }
        }

        let mut calibration_data = Vec::new();
        for i in 0..buckets {
            if bucket_size[i] > 0 {
                let accuracy = bucket_correct[i] as f32 / bucket_size[i] as f32;
                calibration_data.push((
                    (i as f32 + 0.5) / buckets as f32,
                    accuracy,
                ));
            }
        }

        ConfidenceCalibration {
            data: calibration_data,
            is_calibrated: self.accuracy_metrics().calibration_score > 0.8,
        }
    }
}

impl Default for PredictionValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Accuracy metrics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccuracyMetrics {
    pub accuracy_rate: f32,
    pub average_error: f32,
    pub average_confidence: f32,
    pub calibration_score: f32,
    pub total_predictions: u32,
}

impl Default for AccuracyMetrics {
    fn default() -> Self {
        AccuracyMetrics {
            accuracy_rate: 0.5,
            average_error: 0.5,
            average_confidence: 0.5,
            calibration_score: 0.5,
            total_predictions: 0,
        }
    }
}

/// Error pattern (systematic mistakes)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub predicted_value: String,
    pub error_count: u32,
    pub avg_magnitude: f32,
    pub frequencies: HashMap<String, u32>,
}

/// Confidence calibration data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfidenceCalibration {
    pub data: Vec<(f32, f32)>,  // (confidence_bucket, actual_accuracy)
    pub is_calibrated: bool,
}

/// Active learning coordinator
#[derive(Clone, Debug)]
pub struct ActiveLearner {
    pub validator: PredictionValidator,
    pub learning_rate: f32,
}

impl ActiveLearner {
    /// Create a new active learner
    pub fn new() -> Self {
        ActiveLearner {
            validator: PredictionValidator::new(),
            learning_rate: 0.1,
        }
    }

    /// Learn from an outcome (adjust future predictions)
    pub fn learn_from_outcome(&mut self, outcome: &PredictionOutcome) -> LearningUpdate {
        self.validator.record_outcome(outcome.clone());

        let metrics = self.validator.accuracy_metrics();
        let systematic = self.validator.systematic_errors();

        // Calculate what to update
        let mut confidence_adjustment = 0.0;
        if outcome.was_correct {
            confidence_adjustment = self.learning_rate * (1.0 - outcome.prediction_confidence);
        } else {
            confidence_adjustment = -self.learning_rate * outcome.prediction_confidence;
        }

        LearningUpdate {
            outcome_id: outcome.prediction_id.clone(),
            confidence_delta: confidence_adjustment,
            should_increase_model_confidence: metrics.accuracy_rate > 0.7,
            should_decrease_model_confidence: metrics.accuracy_rate < 0.4,
            identified_systematic_error: !systematic.is_empty(),
        }
    }

    /// Get total learning progress
    pub fn learning_progress(&self) -> f32 {
        let metrics = self.validator.accuracy_metrics();
        if metrics.total_predictions == 0 {
            return 0.0;
        }

        // Progress = accuracy + calibration, normalized
        (metrics.accuracy_rate + metrics.calibration_score) / 2.0
    }
}

impl Default for ActiveLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning update recommendation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningUpdate {
    pub outcome_id: String,
    pub confidence_delta: f32,
    pub should_increase_model_confidence: bool,
    pub should_decrease_model_confidence: bool,
    pub identified_systematic_error: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_outcome_creation() {
        let outcome = PredictionOutcome::new(
            "pred_1".to_string(),
            "door".to_string(),
            "door".to_string(),
        );

        assert_eq!(outcome.predicted_value, "door");
        assert_eq!(outcome.actual_value, "door");
        assert!(outcome.was_correct);
        assert_eq!(outcome.error_magnitude, 0.0);
    }

    #[test]
    fn test_prediction_outcome_incorrect() {
        let outcome = PredictionOutcome::new(
            "pred_1".to_string(),
            "corridor".to_string(),
            "door".to_string(),
        );

        assert!(!outcome.was_correct);
        assert_eq!(outcome.error_magnitude, 1.0);
    }

    #[test]
    fn test_outcome_with_magnitude() {
        let outcome = PredictionOutcome::new(
            "pred_1".to_string(),
            "width_1.5".to_string(),
            "width_1.8".to_string(),
        )
        .with_magnitude(0.3)
        .with_confidence(0.75);

        assert_eq!(outcome.error_magnitude, 0.3);
        assert_eq!(outcome.prediction_confidence, 0.75);
    }

    #[test]
    fn test_validator_creation() {
        let validator = PredictionValidator::new();
        assert_eq!(validator.outcomes.len(), 0);
    }

    #[test]
    fn test_record_outcomes() {
        let mut validator = PredictionValidator::new();

        let outcome1 = PredictionOutcome::new(
            "p1".to_string(),
            "door".to_string(),
            "door".to_string(),
        );
        let outcome2 = PredictionOutcome::new(
            "p2".to_string(),
            "corridor".to_string(),
            "door".to_string(),
        );

        validator.record_outcome(outcome1);
        validator.record_outcome(outcome2);

        assert_eq!(validator.outcomes.len(), 2);
    }

    #[test]
    fn test_accuracy_metrics() {
        let mut validator = PredictionValidator::new();

        // 3 correct, 1 wrong
        for _ in 0..3 {
            validator.record_outcome(PredictionOutcome::new(
                "p".to_string(),
                "door".to_string(),
                "door".to_string(),
            ).with_confidence(0.9));
        }

        validator.record_outcome(
            PredictionOutcome::new(
                "p".to_string(),
                "corridor".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.8)
            .with_magnitude(1.0),
        );

        let metrics = validator.accuracy_metrics();
        assert_eq!(metrics.accuracy_rate, 0.75);
        assert!(metrics.total_predictions > 0);
    }

    #[test]
    fn test_calibration_score() {
        let mut validator = PredictionValidator::new();

        // High confidence predictions that are correct
        for _ in 0..8 {
            validator.record_outcome(
                PredictionOutcome::new(
                    "p".to_string(),
                    "door".to_string(),
                    "door".to_string(),
                )
                .with_confidence(0.95),
            );
        }

        // Low confidence predictions
        for _ in 0..2 {
            validator.record_outcome(
                PredictionOutcome::new(
                    "p".to_string(),
                    "corridor".to_string(),
                    "door".to_string(),
                )
                .with_confidence(0.2),
            );
        }

        let metrics = validator.accuracy_metrics();
        assert!(metrics.calibration_score > 0.5);  // High conf preds are mostly correct
    }

    #[test]
    fn test_systematic_errors() {
        let mut validator = PredictionValidator::new();

        // Predict "door" but always find "corridor"
        for _ in 0..5 {
            validator.record_outcome(PredictionOutcome::new(
                "p".to_string(),
                "door".to_string(),
                "corridor".to_string(),
            ));
        }

        let errors = validator.systematic_errors();
        assert!(errors.contains_key("door"));
        assert_eq!(errors["door"].error_count, 5);
    }

    #[test]
    fn test_recent_outcomes() {
        let mut validator = PredictionValidator::new();

        validator.record_outcome(PredictionOutcome::new(
            "p1".to_string(),
            "door".to_string(),
            "door".to_string(),
        ));

        let recent = validator.recent_outcomes(3600);  // Last hour
        assert_eq!(recent.len(), 1);

        let old = validator.recent_outcomes(1);  // Last second (likely empty)
        assert!(old.len() <= 1);
    }

    #[test]
    fn test_active_learner_creation() {
        let learner = ActiveLearner::new();
        assert_eq!(learner.learning_rate, 0.1);
        assert_eq!(learner.learning_progress(), 0.0);
    }

    #[test]
    fn test_learn_from_correct_outcome() {
        let mut learner = ActiveLearner::new();

        let outcome = PredictionOutcome::new(
            "p1".to_string(),
            "door".to_string(),
            "door".to_string(),
        )
        .with_confidence(0.7);

        let update = learner.learn_from_outcome(&outcome);

        assert!(update.confidence_delta > 0.0);  // Increase confidence
        assert_eq!(learner.validator.outcomes.len(), 1);
    }

    #[test]
    fn test_learn_from_incorrect_outcome() {
        let mut learner = ActiveLearner::new();

        let outcome = PredictionOutcome::new(
            "p1".to_string(),
            "corridor".to_string(),
            "door".to_string(),
        )
        .with_confidence(0.8);

        let update = learner.learn_from_outcome(&outcome);

        assert!(update.confidence_delta < 0.0);  // Decrease confidence
    }

    #[test]
    fn test_learning_progress() {
        let mut learner = ActiveLearner::new();

        // Add many correct predictions
        for i in 0..10 {
            let outcome = PredictionOutcome::new(
                format!("p{}", i),
                "door".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.9);

            learner.learn_from_outcome(&outcome);
        }

        let progress = learner.learning_progress();
        assert!(progress > 0.7);  // Should be high with mostly correct predictions
    }

    #[test]
    fn test_confidence_calibration() {
        let mut validator = PredictionValidator::new();

        // High confidence (0.9) predictions - all correct
        for _ in 0..10 {
            validator.record_outcome(
                PredictionOutcome::new(
                    "p".to_string(),
                    "door".to_string(),
                    "door".to_string(),
                )
                .with_confidence(0.9),
            );
        }

        // Low confidence (0.2) predictions - all wrong
        for _ in 0..10 {
            validator.record_outcome(
                PredictionOutcome::new(
                    "p".to_string(),
                    "corridor".to_string(),
                    "door".to_string(),
                )
                .with_confidence(0.2),
            );
        }

        let calibration = validator.confidence_calibration();
        assert!(!calibration.data.is_empty());
    }

    #[test]
    fn test_learning_improvement_over_time() {
        let mut learner = ActiveLearner::new();

        // Start with poor accuracy
        for i in 0..5 {
            let outcome = PredictionOutcome::new(
                format!("p{}", i),
                "corridor".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.5);

            learner.learn_from_outcome(&outcome);
        }

        let early_progress = learner.learning_progress();

        // Then improve
        for i in 5..15 {
            let outcome = PredictionOutcome::new(
                format!("p{}", i),
                "door".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.9);

            learner.learn_from_outcome(&outcome);
        }

        let later_progress = learner.learning_progress();
        assert!(later_progress > early_progress);
    }
}
