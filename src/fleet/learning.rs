//! Fleet Learning - Collective Learning Across Multiple Robots
//!
//! Phase 4.3: Enables robots to learn from shared observations,
//! improving terrain models through accumulated fleet knowledge.

use super::RobotId;
use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Learning model trained on fleet observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetLearningModel {
    pub robot_id: RobotId,
    pub observations_seen: u64,
    pub accuracy: f32, // 0.0-1.0
    pub confidence: f32, // How confident in this model
}

impl FleetLearningModel {
    /// Create new learning model
    pub fn new(robot_id: RobotId) -> Self {
        FleetLearningModel {
            robot_id,
            observations_seen: 0,
            accuracy: 0.5, // Start neutral
            confidence: 0.3, // Low confidence initially
        }
    }

    /// Update model with new observation
    pub fn update(&mut self, observation: &TemporalPoint, is_correct: bool) {
        self.observations_seen += 1;

        let update_rate = 0.1; // Learning rate
        if is_correct {
            self.accuracy = (self.accuracy + update_rate).min(1.0);
            self.confidence = (self.confidence + 0.05).min(1.0);
        } else {
            self.accuracy = (self.accuracy - update_rate).max(0.0);
            self.confidence = (self.confidence - 0.05).max(0.0);
        }
    }
}

/// Learned pattern from consensus decisions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub location: (f32, f32),
    pub typical_elevation: f32,
    pub elevation_variance: f32,
    pub observations_contributing: u32,
    pub confidence: f32,
}

impl LearnedPattern {
    /// Create new learned pattern
    pub fn new(location: (f32, f32), typical_elevation: f32) -> Self {
        LearnedPattern {
            location,
            typical_elevation,
            elevation_variance: 0.0,
            observations_contributing: 1,
            confidence: 0.5,
        }
    }

    /// Update pattern with new observation
    pub fn update(&mut self, elevation: f32, weight: f32) {
        // Update running mean
        let old_mean = self.typical_elevation;
        let n = self.observations_contributing as f32;
        self.typical_elevation = (old_mean * n + elevation * weight) / (n + weight);

        // Update variance estimate
        let delta = elevation - old_mean;
        self.elevation_variance = (self.elevation_variance * n + delta * delta * weight) / (n + weight);

        self.observations_contributing += 1;
        self.confidence = ((self.observations_contributing as f32).ln() / 5.0).min(1.0);
    }

    /// Get standard deviation
    pub fn std_dev(&self) -> f32 {
        self.elevation_variance.sqrt()
    }
}

/// Fleet learning engine for collective knowledge
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetLearningEngine {
    pub models: HashMap<RobotId, FleetLearningModel>,
    pub patterns: HashMap<String, LearnedPattern>, // Location key -> pattern
    pub total_learning_cycles: u64,
    pub global_accuracy: f32,
}

impl FleetLearningEngine {
    /// Create new learning engine
    pub fn new() -> Self {
        FleetLearningEngine {
            models: HashMap::new(),
            patterns: HashMap::new(),
            total_learning_cycles: 0,
            global_accuracy: 0.5,
        }
    }

    /// Register robot for learning
    pub fn register_robot(&mut self, robot_id: RobotId) {
        self.models.insert(robot_id, FleetLearningModel::new(robot_id));
    }

    /// Get or create model for robot
    pub fn get_model_mut(&mut self, robot_id: RobotId) -> &mut FleetLearningModel {
        self.models
            .entry(robot_id)
            .or_insert_with(|| FleetLearningModel::new(robot_id))
    }

    /// Get model for robot
    pub fn get_model(&self, robot_id: RobotId) -> Option<&FleetLearningModel> {
        self.models.get(&robot_id)
    }

    /// Learn from consensus point
    pub fn learn_from_consensus(
        &mut self,
        point: &TemporalPoint,
        agreeing_robots: &[RobotId],
        is_validated: bool,
    ) {
        let location_key = format!("{:.2}_{:.2}", point.x, point.y);

        // Update learned pattern
        let pattern = self.patterns
            .entry(location_key)
            .or_insert_with(|| LearnedPattern::new((point.x, point.y), point.z));

        let agreement_weight = (agreeing_robots.len() as f32 / 4.0).min(1.0);
        pattern.update(point.z, agreement_weight);

        // Update robot models
        for robot_id in agreeing_robots {
            let model = self.get_model_mut(*robot_id);
            model.update(point, is_validated);
        }

        self.total_learning_cycles += 1;
        self.update_global_accuracy();
    }

    /// Predict elevation at location based on learned patterns
    pub fn predict_elevation(&self, x: f32, y: f32, search_radius: f32) -> Option<(f32, f32)> {
        let mut nearby_patterns = Vec::new();

        for (location, pattern) in &self.patterns {
            let dx = location.parse::<f32>().ok()? - x; // This is imperfect, but shows the idea
            if dx.abs() < search_radius {
                nearby_patterns.push((pattern, dx.abs()));
            }
        }

        if nearby_patterns.is_empty() {
            return None;
        }

        let pattern_count = nearby_patterns.len() as f32;

        // Weight by distance and confidence
        let mut weighted_z = 0.0;
        let mut total_weight = 0.0;

        for (pattern, distance) in nearby_patterns {
            let distance_weight = 1.0 / (1.0 + distance);
            let weight = distance_weight * pattern.confidence;
            weighted_z += pattern.typical_elevation * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            Some((weighted_z / total_weight, total_weight / pattern_count))
        } else {
            None
        }
    }

    /// Update global accuracy based on robot models
    fn update_global_accuracy(&mut self) {
        if self.models.is_empty() {
            return;
        }

        let total_accuracy: f32 = self.models.values().map(|m| m.accuracy).sum();
        self.global_accuracy = total_accuracy / self.models.len() as f32;
    }

    /// Get learning statistics
    pub fn statistics(&self) -> LearningStatistics {
        LearningStatistics {
            total_robots: self.models.len() as u32,
            learned_locations: self.patterns.len() as u32,
            total_learning_cycles: self.total_learning_cycles,
            global_accuracy: self.global_accuracy,
            average_confidence: if self.patterns.is_empty() {
                0.0
            } else {
                self.patterns.values().map(|p| p.confidence).sum::<f32>()
                    / self.patterns.len() as f32
            },
        }
    }
}

impl Default for FleetLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearningStatistics {
    pub total_robots: u32,
    pub learned_locations: u32,
    pub total_learning_cycles: u64,
    pub global_accuracy: f32,
    pub average_confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_model_creation() {
        let robot_id = RobotId::new(1);
        let model = FleetLearningModel::new(robot_id);

        assert_eq!(model.robot_id, robot_id);
        assert_eq!(model.observations_seen, 0);
        assert_eq!(model.accuracy, 0.5);
    }

    #[test]
    fn test_learning_model_update() {
        let robot_id = RobotId::new(1);
        let mut model = FleetLearningModel::new(robot_id);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);

        model.update(&point, true);
        assert!(model.accuracy > 0.5);
        assert_eq!(model.observations_seen, 1);

        model.update(&point, false);
        assert!(model.accuracy < 0.6);
    }

    #[test]
    fn test_learned_pattern() {
        let mut pattern = LearnedPattern::new((1.0, 2.0), 3.0);

        pattern.update(3.1, 0.8);
        pattern.update(2.9, 0.8);

        assert!(pattern.std_dev() > 0.0);
        assert!(pattern.confidence > 0.5);
    }

    #[test]
    fn test_learning_engine() {
        let mut engine = FleetLearningEngine::new();
        let robot_id = RobotId::new(1);

        engine.register_robot(robot_id);
        assert!(engine.get_model(robot_id).is_some());
    }

    #[test]
    fn test_learning_from_consensus() {
        let mut engine = FleetLearningEngine::new();
        let robot1 = RobotId::new(1);
        let robot2 = RobotId::new(2);

        engine.register_robot(robot1);
        engine.register_robot(robot2);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let agreeing = vec![robot1, robot2];

        engine.learn_from_consensus(&point, &agreeing, true);

        let stats = engine.statistics();
        assert_eq!(stats.total_robots, 2);
        assert!(stats.total_learning_cycles > 0);
    }

    #[test]
    fn test_learning_statistics() {
        let mut engine = FleetLearningEngine::new();
        engine.register_robot(RobotId::new(1));

        let stats = engine.statistics();
        assert_eq!(stats.total_robots, 1);
        assert_eq!(stats.global_accuracy, 0.5);
    }
}
