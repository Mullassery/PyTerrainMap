//! Predictive Layered Caching
//!
//! System that anticipates where agents will go and pre-caches information.
//! Avoids waiting for requests to occur before retrieving data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::types::GeoPoint;

// ============================================================================
// Decision Graph Types
// ============================================================================

/// Behavioral mode of an agent
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehavioralMode {
    Navigation,
    Exploration,
    Emergency,
    Observation,
    Idle,
}

/// Mission objective
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    pub mission_id: String,
    pub objective: String,
    pub waypoints: Vec<Waypoint>,
    pub completed_waypoints: usize,
}

impl Mission {
    pub fn remaining_waypoints(&self) -> Vec<&Waypoint> {
        self.waypoints[self.completed_waypoints..].iter().collect()
    }
}

/// Single waypoint in a mission
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub location: GeoPoint,
    pub estimated_arrival_s: u32,
    pub confidence: f32,
}


/// Current decision state of an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionState {
    pub agent_id: String,
    pub current_location: GeoPoint,
    pub current_heading: f32,  // 0-360 degrees
    pub velocity: f32,         // m/s
    pub mission_objective: Option<Mission>,
    pub behavioral_mode: BehavioralMode,
    pub confidence: f32,       // 0.0-1.0
    pub timestamp_us: i64,
}

/// Priority level for cache warming
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CachePriority {
    Critical,  // >80% probability
    High,      // 50-80% probability
    Medium,    // 20-50% probability
    Low,       // <20% probability
}

/// Type of knowledge that might be needed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KnowledgeRequirement {
    LocationSummary(GeoPoint),
    TerrainContext(Region),
    ObstacleMap(Region),
    WeatherData(Region),
    AIInferences(String),  // Model name
    RouteHistory(String),
}

/// Geographic region (bounding box)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}

impl Region {
    pub fn center(&self) -> GeoPoint {
        GeoPoint {
            lat: (self.north + self.south) / 2.0,
            lon: (self.east + self.west) / 2.0,
        }
    }

    pub fn contains(&self, point: &GeoPoint) -> bool {
        point.lat >= self.south && point.lat <= self.north
            && point.lon >= self.west && point.lon <= self.east
    }
}

/// Predicted next state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedState {
    pub state: DecisionState,
    pub probability: f32,             // 0.0-1.0
    pub reasoning: String,
    pub confidence: f32,
    pub cache_priority: CachePriority,
    pub required_knowledge: Vec<KnowledgeRequirement>,
}

/// Decision graph with predicted transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionGraph {
    pub current_state: DecisionState,
    pub next_states: Vec<PredictedState>,  // Sorted by probability
}

// ============================================================================
// Trajectory Prediction
// ============================================================================

/// Trajectory model
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrajectoryModel {
    LinearInterpolation,
    BehaviorBased,
    MissionAware,
    EnsembleWeighted,
}

/// Single predicted waypoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaypointPrediction {
    pub location: GeoPoint,
    pub time_from_now_s: u32,
    pub probability: f32,
    pub reasoning: String,
}

/// Complete predicted trajectory
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictedTrajectory {
    pub agent_id: String,
    pub waypoints: Vec<WaypointPrediction>,
    pub confidence: f32,
    pub divergence_points: Vec<GeoPoint>,  // Where course might change
}

/// Behavior history for an agent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorHistory {
    pub agent_id: String,
    pub recent_states: Vec<DecisionState>,
    pub mode_transitions: Vec<(BehavioralMode, i64)>,
    pub successful_trajectories: Vec<PredictedTrajectory>,
}

impl BehaviorHistory {
    pub fn find_similar_states(&self, current: &DecisionState) -> Vec<&DecisionState> {
        self.recent_states
            .iter()
            .filter(|hist| {
                hist.behavioral_mode == current.behavioral_mode
                    && hist.current_location.distance_m(&current.current_location) < 100.0
            })
            .collect()
    }
}

/// Environmental context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalContext {
    pub region: Region,
    pub weather: String,
    pub terrain_type: String,
    pub obstacles_nearby: bool,
}

/// Trajectory predictor
pub struct TrajectoryPredictor {
    model: TrajectoryModel,
    learning_rate: f32,
}

impl TrajectoryPredictor {
    pub fn new(model: TrajectoryModel, learning_rate: f32) -> Self {
        TrajectoryPredictor { model, learning_rate }
    }

    pub fn predict_trajectory(
        &self,
        state: &DecisionState,
        lookahead_seconds: u32,
        history: &BehaviorHistory,
    ) -> PredictedTrajectory {
        let waypoints = match self.model {
            TrajectoryModel::LinearInterpolation => self.predict_linear(state, lookahead_seconds),
            TrajectoryModel::BehaviorBased => self.predict_from_behavior(state, lookahead_seconds, history),
            TrajectoryModel::MissionAware => self.predict_from_mission(state, lookahead_seconds),
            TrajectoryModel::EnsembleWeighted => {
                self.predict_ensemble(state, lookahead_seconds, history)
            }
        };

        let confidence = self.calculate_confidence(&waypoints, &history);

        PredictedTrajectory {
            agent_id: state.agent_id.clone(),
            waypoints,
            confidence,
            divergence_points: self.identify_decision_points(state),
        }
    }

    fn predict_linear(&self, state: &DecisionState, seconds: u32) -> Vec<WaypointPrediction> {
        let distance_m = state.velocity * seconds as f32;
        let dx = (state.current_heading.to_radians().sin()) * distance_m;
        let dy = (state.current_heading.to_radians().cos()) * distance_m;

        vec![WaypointPrediction {
            location: GeoPoint::new(
                state.current_location.lat + (dy as f64) / 111_000.0,
                state.current_location.lon + (dx as f64) / (111_000.0 * state.current_location.lat.to_radians().cos()),
            ),
            time_from_now_s: seconds,
            probability: 0.9,
            reasoning: "Linear extrapolation".to_string(),
        }]
    }

    fn predict_from_behavior(
        &self,
        state: &DecisionState,
        seconds: u32,
        history: &BehaviorHistory,
    ) -> Vec<WaypointPrediction> {
        let similar = history.find_similar_states(state);

        if similar.is_empty() {
            return self.predict_linear(state, seconds);
        }

        similar
            .into_iter()
            .take(3)
            .map(|hist_state| {
                let distance_m = hist_state.velocity * seconds as f32;
                let dx = hist_state.current_heading.to_radians().sin() * distance_m;
                let dy = hist_state.current_heading.to_radians().cos() * distance_m;

                WaypointPrediction {
                    location: GeoPoint::new(
                        hist_state.current_location.lat + (dy as f64) / 111_000.0,
                        hist_state.current_location.lon
                            + (dx as f64) / (111_000.0 * hist_state.current_location.lat.to_radians().cos()),
                    ),
                    time_from_now_s: seconds,
                    probability: 0.7,
                    reasoning: "Behavior pattern".to_string(),
                }
            })
            .collect()
    }

    fn predict_from_mission(&self, state: &DecisionState, _seconds: u32) -> Vec<WaypointPrediction> {
        if let Some(mission) = &state.mission_objective {
            mission
                .remaining_waypoints()
                .iter()
                .map(|wp| WaypointPrediction {
                    location: wp.location,
                    time_from_now_s: wp.estimated_arrival_s,
                    probability: wp.confidence,
                    reasoning: "Mission planning".to_string(),
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn predict_ensemble(
        &self,
        state: &DecisionState,
        seconds: u32,
        history: &BehaviorHistory,
    ) -> Vec<WaypointPrediction> {
        let linear = self.predict_linear(state, seconds);
        let behavior = self.predict_from_behavior(state, seconds, history);
        let mission = self.predict_from_mission(state, seconds);

        self.weight_ensemble(&linear, &behavior, &mission)
    }

    fn weight_ensemble(
        &self,
        linear: &[WaypointPrediction],
        behavior: &[WaypointPrediction],
        mission: &[WaypointPrediction],
    ) -> Vec<WaypointPrediction> {
        let mut result = Vec::new();

        // Weight: mission > behavior > linear
        if !mission.is_empty() {
            result.push(WaypointPrediction {
                probability: 0.6,
                ..mission[0].clone()
            });
        }

        if !behavior.is_empty() {
            result.push(WaypointPrediction {
                probability: 0.3,
                ..behavior[0].clone()
            });
        }

        if !linear.is_empty() {
            result.push(WaypointPrediction {
                probability: 0.1,
                ..linear[0].clone()
            });
        }

        result
    }

    fn calculate_confidence(&self, waypoints: &[WaypointPrediction], _history: &BehaviorHistory) -> f32 {
        if waypoints.is_empty() {
            0.3
        } else {
            waypoints.iter().map(|w| w.probability).sum::<f32>() / waypoints.len() as f32
        }
    }

    fn identify_decision_points(&self, state: &DecisionState) -> Vec<GeoPoint> {
        // Points where agent might change behavior
        // (simplified: future waypoints from mission)
        if let Some(mission) = &state.mission_objective {
            mission
                .remaining_waypoints()
                .iter()
                .map(|w| w.location)
                .collect()
        } else {
            vec![]
        }
    }
}

// ============================================================================
// Intent Inference
// ============================================================================

/// Inferred intent
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferredIntent {
    pub primary_objective: String,  // "reach waypoint", "explore", etc.
    pub immediate_action: String,   // "move forward", "turn left", etc.
    pub next_waypoint: Option<GeoPoint>,
    pub estimated_distance: f32,
    pub confidence: f32,
}

/// Intent inferrer trait
pub trait IntentInferrer {
    fn infer_intent(
        &self,
        state: &DecisionState,
        behavior_history: &BehaviorHistory,
        environmental_context: &EnvironmentalContext,
    ) -> InferredIntent;
}

/// Default intent inferrer
pub struct DefaultIntentInferrer;

impl IntentInferrer for DefaultIntentInferrer {
    fn infer_intent(
        &self,
        state: &DecisionState,
        _behavior_history: &BehaviorHistory,
        _env_context: &EnvironmentalContext,
    ) -> InferredIntent {
        let (primary_objective, next_waypoint) = if let Some(mission) = &state.mission_objective {
            if let Some(wp) = mission.remaining_waypoints().first() {
                (
                    format!("reach waypoint: {}", wp.id),
                    Some(wp.location),
                )
            } else {
                ("explore".to_string(), None)
            }
        } else {
            ("explore".to_string(), None)
        };

        let estimated_distance = next_waypoint
            .map(|wp| state.current_location.distance_m(&wp))
            .unwrap_or(0.0);

        InferredIntent {
            primary_objective,
            immediate_action: "move forward".to_string(),
            next_waypoint,
            estimated_distance,
            confidence: 0.8,
        }
    }
}

// ============================================================================
// Predictive Cache Warming
// ============================================================================

/// Budget for cache warming operations
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CacheWarmingBudget {
    pub max_concurrent_prefetches: u32,
    pub max_bandwidth_mbps: f32,
    pub max_memory_allocation: u64,
    pub max_cpu_usage_percent: f32,
}

impl Default for CacheWarmingBudget {
    fn default() -> Self {
        CacheWarmingBudget {
            max_concurrent_prefetches: 10,
            max_bandwidth_mbps: 100.0,
            max_memory_allocation: 1024 * 1024 * 1024,  // 1GB
            max_cpu_usage_percent: 25.0,
        }
    }
}

/// Prioritized prefetch request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrioritizedPrefetch {
    pub requirement: KnowledgeRequirement,
    pub priority: CachePriority,
    pub probability: f32,
    pub estimated_size_bytes: u64,
}

/// Prediction learning statistics
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AccuracyStatistics {
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub avg_error_distance_m: f32,
    pub accuracy_by_behavioral_mode: [f32; 5],  // One per BehavioralMode
}

impl Default for AccuracyStatistics {
    fn default() -> Self {
        AccuracyStatistics {
            total_predictions: 0,
            correct_predictions: 0,
            avg_error_distance_m: 0.0,
            accuracy_by_behavioral_mode: [0.0; 5],
        }
    }
}

/// Prediction record for learning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PredictionRecord {
    pub predicted_trajectory: PredictedTrajectory,
    pub actual_trajectory: PredictedTrajectory,
    pub prediction_time_us: i64,
}

/// Prediction learner
pub struct PredictionLearner {
    prediction_history: parking_lot::RwLock<Vec<PredictionRecord>>,
    accuracy_stats: parking_lot::RwLock<AccuracyStatistics>,
    learning_rate: f32,
}

impl PredictionLearner {
    pub fn new(learning_rate: f32) -> Self {
        PredictionLearner {
            prediction_history: parking_lot::RwLock::new(Vec::new()),
            accuracy_stats: parking_lot::RwLock::new(AccuracyStatistics::default()),
            learning_rate,
        }
    }

    pub fn record_outcome(
        &self,
        prediction: &PredictedTrajectory,
        actual: &PredictedTrajectory,
    ) {
        let record = PredictionRecord {
            predicted_trajectory: prediction.clone(),
            actual_trajectory: actual.clone(),
            prediction_time_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64,
        };

        let mut history = self.prediction_history.write();
        history.push(record);

        self.update_accuracy_stats(prediction, actual);
    }

    fn update_accuracy_stats(&self, prediction: &PredictedTrajectory, actual: &PredictedTrajectory) {
        let mut stats = self.accuracy_stats.write();

        stats.total_predictions += 1;

        let error = self.calculate_error_distance(prediction, actual);
        stats.avg_error_distance_m =
            (stats.avg_error_distance_m * (stats.total_predictions - 1) as f32 + error)
                / stats.total_predictions as f32;

        if error < 50.0 {
            stats.correct_predictions += 1;
        }
    }

    pub fn accuracy_rate(&self) -> f32 {
        let stats = self.accuracy_stats.read();
        if stats.total_predictions == 0 {
            0.0
        } else {
            stats.correct_predictions as f32 / stats.total_predictions as f32
        }
    }

    pub fn should_expand_cache(&self) -> bool {
        self.accuracy_rate() > 0.7
    }

    pub fn should_limit_cache(&self) -> bool {
        self.accuracy_rate() < 0.5
    }

    fn calculate_error_distance(&self, prediction: &PredictedTrajectory, actual: &PredictedTrajectory) -> f32 {
        if prediction.waypoints.is_empty() || actual.waypoints.is_empty() {
            1000.0  // Max error
        } else {
            prediction.waypoints[0]
                .location
                .distance_m(&actual.waypoints[0].location)
        }
    }

    pub fn statistics(&self) -> AccuracyStatistics {
        *self.accuracy_stats.read()
    }
}

impl Default for PredictionLearner {
    fn default() -> Self {
        Self::new(0.1)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geopoint_distance() {
        let p1 = GeoPoint::new(0.0, 0.0);
        let p2 = GeoPoint::new(0.0, 0.001);  // ~111 meters

        let distance = p1.distance_m(&p2);
        assert!(distance > 100.0 && distance < 150.0);
    }

    #[test]
    fn test_region_contains() {
        let region = Region {
            north: 10.0,
            south: 0.0,
            east: 10.0,
            west: 0.0,
        };

        let inside = GeoPoint::new(5.0, 5.0);
        let outside = GeoPoint::new(15.0, 15.0);

        assert!(region.contains(&inside));
        assert!(!region.contains(&outside));
    }

    #[test]
    fn test_trajectory_predictor_linear() {
        let state = DecisionState {
            agent_id: "agent-1".to_string(),
            current_location: GeoPoint::new(0.0, 0.0),
            current_heading: 0.0,  // North
            velocity: 10.0,        // 10 m/s
            mission_objective: None,
            behavioral_mode: BehavioralMode::Navigation,
            confidence: 0.9,
            timestamp_us: 0,
        };

        let history = BehaviorHistory {
            agent_id: "agent-1".to_string(),
            recent_states: vec![],
            mode_transitions: vec![],
            successful_trajectories: vec![],
        };

        let predictor = TrajectoryPredictor::new(TrajectoryModel::LinearInterpolation, 0.1);
        let trajectory = predictor.predict_trajectory(&state, 10, &history);

        assert!(!trajectory.waypoints.is_empty());
        assert!(trajectory.confidence > 0.0);
    }

    #[test]
    fn test_decision_state_serialization() {
        let state = DecisionState {
            agent_id: "agent-1".to_string(),
            current_location: GeoPoint::new(10.5, 20.5),
            current_heading: 45.0,
            velocity: 5.0,
            mission_objective: None,
            behavioral_mode: BehavioralMode::Exploration,
            confidence: 0.8,
            timestamp_us: 1000000,
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: DecisionState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.agent_id, deserialized.agent_id);
        assert_eq!(state.behavioral_mode, deserialized.behavioral_mode);
    }

    #[test]
    fn test_inferred_intent() {
        let state = DecisionState {
            agent_id: "agent-1".to_string(),
            current_location: GeoPoint::new(0.0, 0.0),
            current_heading: 0.0,
            velocity: 10.0,
            mission_objective: None,
            behavioral_mode: BehavioralMode::Navigation,
            confidence: 0.9,
            timestamp_us: 0,
        };

        let history = BehaviorHistory {
            agent_id: "agent-1".to_string(),
            recent_states: vec![],
            mode_transitions: vec![],
            successful_trajectories: vec![],
        };

        let env = EnvironmentalContext {
            region: Region {
                north: 10.0,
                south: -10.0,
                east: 10.0,
                west: -10.0,
            },
            weather: "clear".to_string(),
            terrain_type: "grass".to_string(),
            obstacles_nearby: false,
        };

        let inferrer = DefaultIntentInferrer;
        let intent = inferrer.infer_intent(&state, &history, &env);

        assert!(intent.confidence > 0.0);
    }

    #[test]
    fn test_prediction_learner_tracking() {
        let learner = PredictionLearner::new(0.1);

        let pred = PredictedTrajectory {
            agent_id: "agent-1".to_string(),
            waypoints: vec![WaypointPrediction {
                location: GeoPoint::new(0.0, 0.0),
                time_from_now_s: 10,
                probability: 0.9,
                reasoning: "test".to_string(),
            }],
            confidence: 0.9,
            divergence_points: vec![],
        };

        let actual = PredictedTrajectory {
            agent_id: "agent-1".to_string(),
            waypoints: vec![WaypointPrediction {
                location: GeoPoint::new(0.00005, 0.00005),  // ~5.5m away
                time_from_now_s: 10,
                probability: 0.95,
                reasoning: "actual".to_string(),
            }],
            confidence: 0.95,
            divergence_points: vec![],
        };

        learner.record_outcome(&pred, &actual);

        assert_eq!(learner.accuracy_rate(), 1.0);
        assert!(learner.should_expand_cache());
    }

    #[test]
    fn test_cache_warming_budget_default() {
        let budget = CacheWarmingBudget::default();
        assert!(budget.max_concurrent_prefetches > 0);
        assert!(budget.max_memory_allocation > 0);
    }

    #[test]
    fn test_behavioral_mode_equality() {
        assert_eq!(BehavioralMode::Navigation, BehavioralMode::Navigation);
        assert_ne!(BehavioralMode::Navigation, BehavioralMode::Exploration);
    }

    #[test]
    fn test_mission_remaining_waypoints() {
        let mission = Mission {
            mission_id: "m1".to_string(),
            objective: "explore".to_string(),
            waypoints: vec![
                Waypoint {
                    id: "w1".to_string(),
                    location: GeoPoint::new(0.0, 0.0),
                    estimated_arrival_s: 10,
                    confidence: 0.9,
                },
                Waypoint {
                    id: "w2".to_string(),
                    location: GeoPoint::new(1.0, 1.0),
                    estimated_arrival_s: 20,
                    confidence: 0.9,
                },
            ],
            completed_waypoints: 1,
        };

        let remaining = mission.remaining_waypoints();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "w2");
    }

    #[test]
    fn test_decision_graph_with_predictions() {
        let graph = DecisionGraph {
            current_state: DecisionState {
                agent_id: "agent-1".to_string(),
                current_location: GeoPoint::new(0.0, 0.0),
                current_heading: 0.0,
                velocity: 10.0,
                mission_objective: None,
                behavioral_mode: BehavioralMode::Navigation,
                confidence: 0.9,
                timestamp_us: 0,
            },
            next_states: vec![
                PredictedState {
                    state: DecisionState {
                        agent_id: "agent-1".to_string(),
                        current_location: GeoPoint::new(0.001, 0.0),
                        current_heading: 0.0,
                        velocity: 10.0,
                        mission_objective: None,
                        behavioral_mode: BehavioralMode::Navigation,
                        confidence: 0.9,
                        timestamp_us: 1000,
                    },
                    probability: 0.9,
                    reasoning: "linear prediction".to_string(),
                    confidence: 0.85,
                    cache_priority: CachePriority::High,
                    required_knowledge: vec![],
                },
            ],
        };

        assert!(!graph.next_states.is_empty());
        assert!(graph.next_states[0].probability > 0.5);
    }
}

use parking_lot;
