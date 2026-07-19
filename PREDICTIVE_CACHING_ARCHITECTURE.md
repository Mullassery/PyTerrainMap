# Predictive Layered Caching Architecture

## Foundational Principle

**Cache for where the agent is likely to go, not only where it is now.**

The system should not wait for requests to occur before retrieving data. Instead, it should continuously anticipate likely next actions and prepare relevant information in advance.

An agent has only a finite set of meaningful next decisions:
- Move Forward
- Move Backward  
- Turn Left
- Turn Right
- Remain Stationary
- Execute Mission Step

The platform should exploit this constraint.

---

## Core Concept: Decision Graphs

Every agent at every moment exists in a **decision state** with probabilistic transitions to **next states**.

```rust
pub struct DecisionState {
    pub current_location: GeoPoint,
    pub current_heading: f32,        // 0-360 degrees
    pub velocity: f32,               // m/s
    pub mission_objective: Option<Mission>,
    pub behavioral_mode: BehavioralMode,
    pub confidence: f32,             // 0.0-1.0 in this state assessment
}

pub struct DecisionGraph {
    pub current_state: DecisionState,
    pub next_states: Vec<PredictedState>,  // Ranked by probability
}

pub struct PredictedState {
    pub state: DecisionState,
    pub probability: f32,            // 0.0-1.0
    pub reasoning: String,           // Why this prediction?
    pub confidence: f32,             // Confidence in prediction
    pub cache_priority: CachePriority,
    pub required_knowledge: Vec<KnowledgeRequirement>,
}

pub enum CachePriority {
    Critical,    // Immediate action likely (>80% probability)
    High,        // Action likely (50-80% probability)
    Medium,      // Possible action (20-50% probability)
    Low,         // Unlikely (<20% probability)
}

pub enum KnowledgeRequirement {
    LocationSummary(GeoPoint),
    TerrainContext(Region),
    ObstacleMap(Region),
    WeatherData(Region),
    AIInferences(String),  // Model-specific outputs
    RouteHistory(String),
}
```

---

## Prediction Engine

### Intent Inference

```rust
pub trait IntentInferrer {
    fn infer_intent(
        &self,
        state: &DecisionState,
        behavior_history: &BehaviorHistory,
        environmental_context: &EnvironmentalContext,
    ) -> InferredIntent {
        // Analyze:
        // - Mission objective (explicit intent)
        // - Current heading (implicit direction)
        // - Velocity and acceleration (urgency)
        // - Historical behavior patterns (learned intent)
        // - Environmental constraints (forced intent)
        // - Time of day and conditions (contextual intent)
        
        InferredIntent {
            primary_objective: self.infer_mission_intent(state),
            immediate_action: self.infer_immediate_action(state),
            next_waypoint: self.predict_next_waypoint(state),
            estimated_distance: 0.0,
            confidence: 0.0,
        }
    }
}

pub struct InferredIntent {
    pub primary_objective: String,    // "reach waypoint", "explore", "return home"
    pub immediate_action: String,     // "move forward", "turn left", "wait"
    pub next_waypoint: Option<GeoPoint>,
    pub estimated_distance: f32,
    pub confidence: f32,
}
```

### Trajectory Prediction

```rust
pub struct TrajectoryPredictor {
    model: TrajectoryModel,
    learning_rate: f32,
}

pub enum TrajectoryModel {
    LinearInterpolation,              // Straight line at current velocity
    BehaviorBased,                    // From learned patterns
    MissionAware,                     // From mission planning
    EnsembleWeighted,                 // Combination of above
}

impl TrajectoryPredictor {
    pub fn predict_trajectory(
        &self,
        state: &DecisionState,
        lookahead_seconds: u32,
        history: &BehaviorHistory,
    ) -> PredictedTrajectory {
        let points = self.generate_trajectory_points(
            state,
            lookahead_seconds,
            history,
        );
        
        PredictedTrajectory {
            waypoints: points,
            confidence: self.calculate_confidence(&points, &history),
            divergence_points: self.identify_decision_points(&points),
        }
    }

    fn generate_trajectory_points(
        &self,
        state: &DecisionState,
        seconds: u32,
        history: &BehaviorHistory,
    ) -> Vec<WaypointPrediction> {
        match self.model {
            TrajectoryModel::LinearInterpolation => {
                self.predict_linear(state, seconds)
            }
            TrajectoryModel::BehaviorBased => {
                self.predict_from_behavior(state, seconds, history)
            }
            TrajectoryModel::MissionAware => {
                self.predict_from_mission(state, seconds)
            }
            TrajectoryModel::EnsembleWeighted => {
                self.predict_ensemble(state, seconds, history)
            }
        }
    }

    fn predict_linear(&self, state: &DecisionState, seconds: u32) -> Vec<WaypointPrediction> {
        // Extrapolate current velocity
        let distance_m = state.velocity * seconds as f32;
        let dx = (state.heading.to_radians().sin()) * distance_m;
        let dy = (state.heading.to_radians().cos()) * distance_m;
        
        vec![WaypointPrediction {
            location: GeoPoint::new(
                state.current_location.lat + dy / 111_000.0,
                state.current_location.lon + dx / (111_000.0 * state.current_location.lat.to_radians().cos()),
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
        // Analyze historical patterns
        // Find similar historical states
        // Replay future trajectories from those states
        history.find_similar_states(state)
            .iter()
            .map(|hist_state| self.extrapolate_from_history(hist_state, seconds))
            .collect()
    }

    fn predict_from_mission(
        &self,
        state: &DecisionState,
        seconds: u32,
    ) -> Vec<WaypointPrediction> {
        // Use mission waypoints as anchor points
        if let Some(mission) = &state.mission_objective {
            mission.remaining_waypoints()
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
        // Combine predictions from all models
        let linear = self.predict_linear(state, seconds);
        let behavior = self.predict_from_behavior(state, seconds, history);
        let mission = self.predict_from_mission(state, seconds);
        
        self.weight_ensemble(&linear, &behavior, &mission, state)
    }
}

pub struct PredictedTrajectory {
    pub waypoints: Vec<WaypointPrediction>,
    pub confidence: f32,
    pub divergence_points: Vec<GeoPoint>,  // Where agent might change course
}

pub struct WaypointPrediction {
    pub location: GeoPoint,
    pub time_from_now_s: u32,
    pub probability: f32,
    pub reasoning: String,
}
```

---

## Predictive Cache Warming

```rust
pub struct PredictiveCacheWarmer {
    cache_manager: Arc<CacheManager>,
    trajectory_predictor: Arc<TrajectoryPredictor>,
    intent_inferrer: Arc<dyn IntentInferrer>,
    warming_budget: CacheWarmingBudget,
}

pub struct CacheWarmingBudget {
    pub max_concurrent_prefetches: u32,
    pub max_bandwidth_mbps: f32,
    pub max_memory_allocation: u64,
    pub max_cpu_usage_percent: f32,
}

impl PredictiveCacheWarmer {
    pub fn warm_cache_for_agent(
        &self,
        state: &DecisionState,
        behavior_history: &BehaviorHistory,
        env_context: &EnvironmentalContext,
    ) {
        // 1. Infer intent
        let intent = self.intent_inferrer.infer_intent(state, behavior_history, env_context);
        
        // 2. Predict trajectory
        let trajectory = self.trajectory_predictor.predict_trajectory(
            state,
            300,  // 5 minutes lookahead
            behavior_history,
        );
        
        // 3. Identify knowledge requirements
        let requirements = self.identify_knowledge_requirements(&trajectory, &intent);
        
        // 4. Prioritize by probability
        let prioritized = self.prioritize_prefetches(&requirements, &trajectory);
        
        // 5. Warm cache within budget
        self.warm_within_budget(&prioritized);
    }

    fn identify_knowledge_requirements(
        &self,
        trajectory: &PredictedTrajectory,
        intent: &InferredIntent,
    ) -> Vec<KnowledgeRequirement> {
        let mut requirements = Vec::new();
        
        // For each predicted waypoint
        for waypoint in &trajectory.waypoints {
            // Cache location summary (Layer 0)
            requirements.push(KnowledgeRequirement::LocationSummary(waypoint.location));
            
            // Cache terrain context (Layer 1-2)
            if waypoint.probability > 0.3 {
                let region = self.location_to_region(&waypoint.location);
                requirements.push(KnowledgeRequirement::TerrainContext(region.clone()));
                requirements.push(KnowledgeRequirement::ObstacleMap(region.clone()));
            }
            
            // Cache weather if important
            if waypoint.probability > 0.5 {
                requirements.push(KnowledgeRequirement::WeatherData(
                    self.location_to_region(&waypoint.location),
                ));
            }
        }
        
        // Add intent-specific requirements
        match intent.primary_objective.as_str() {
            "explore" => {
                // Exploratory missions need full terrain models
                requirements.push(KnowledgeRequirement::TerrainContext(
                    self.location_to_region(&intent.next_waypoint.unwrap_or(state.current_location)),
                ));
            }
            "rescue" | "emergency" => {
                // Emergency operations need real-time AI inferences
                requirements.push(KnowledgeRequirement::AIInferences("object_detection".to_string()));
                requirements.push(KnowledgeRequirement::AIInferences("obstacle_detection".to_string()));
            }
            _ => {}
        }
        
        requirements
    }

    fn prioritize_prefetches(
        &self,
        requirements: &[KnowledgeRequirement],
        trajectory: &PredictedTrajectory,
    ) -> Vec<PrioritizedPrefetch> {
        requirements
            .iter()
            .map(|req| {
                let priority = match req {
                    KnowledgeRequirement::LocationSummary(location) => {
                        // Find probability of visiting this location
                        let prob = trajectory.waypoints
                            .iter()
                            .find(|w| self.locations_match(w.location, *location))
                            .map(|w| w.probability)
                            .unwrap_or(0.0);
                        
                        if prob > 0.8 {
                            CachePriority::Critical
                        } else if prob > 0.5 {
                            CachePriority::High
                        } else if prob > 0.2 {
                            CachePriority::Medium
                        } else {
                            CachePriority::Low
                        }
                    }
                    _ => CachePriority::High,
                };
                
                PrioritizedPrefetch {
                    requirement: req.clone(),
                    priority,
                    probability: 0.0,  // Filled from trajectory
                }
            })
            .collect()
    }

    fn warm_within_budget(&self, prefetches: &[PrioritizedPrefetch]) {
        let mut warmed = 0;
        
        for prefetch in prefetches {
            // Check budget
            if self.warming_budget.max_concurrent_prefetches <= warmed {
                break;
            }
            
            // Prefetch based on type
            match &prefetch.requirement {
                KnowledgeRequirement::LocationSummary(location) => {
                    self.cache_manager.prefetch_location_summary(location);
                }
                KnowledgeRequirement::TerrainContext(region) => {
                    self.cache_manager.prefetch_terrain_context(region);
                }
                KnowledgeRequirement::ObstacleMap(region) => {
                    self.cache_manager.prefetch_obstacle_map(region);
                }
                KnowledgeRequirement::WeatherData(region) => {
                    self.cache_manager.prefetch_weather_data(region);
                }
                KnowledgeRequirement::AIInferences(model_name) => {
                    self.cache_manager.prefetch_ai_inferences(model_name);
                }
                KnowledgeRequirement::RouteHistory(route_id) => {
                    self.cache_manager.prefetch_route_history(route_id);
                }
            }
            
            warmed += 1;
        }
    }
}

pub struct PrioritizedPrefetch {
    pub requirement: KnowledgeRequirement,
    pub priority: CachePriority,
    pub probability: f32,
}
```

---

## Continuous Learning

```rust
pub struct PredictionLearner {
    prediction_history: Arc<Mutex<Vec<PredictionRecord>>>,
    accuracy_stats: Arc<Mutex<AccuracyStatistics>>,
    learning_rate: f32,
}

pub struct PredictionRecord {
    pub predicted_trajectory: PredictedTrajectory,
    pub actual_trajectory: ActualTrajectory,
    pub time_recorded_us: i64,
}

pub struct AccuracyStatistics {
    pub total_predictions: u64,
    pub correct_predictions: u64,
    pub avg_error_distance_m: f32,
    pub by_behavioral_mode: HashMap<BehavioralMode, ModeAccuracy>,
}

impl PredictionLearner {
    pub fn record_outcome(
        &self,
        prediction: &PredictedTrajectory,
        actual: &ActualTrajectory,
    ) {
        let record = PredictionRecord {
            predicted_trajectory: prediction.clone(),
            actual_trajectory: actual.clone(),
            time_recorded_us: now_us(),
        };
        
        // Record for learning
        self.prediction_history.lock().push(record);
        
        // Update accuracy stats
        self.update_accuracy_stats(prediction, actual);
        
        // Adjust model if patterns emerge
        if self.should_retrain_models() {
            self.trigger_model_update();
        }
    }

    fn update_accuracy_stats(&self, prediction: &PredictedTrajectory, actual: &ActualTrajectory) {
        let mut stats = self.accuracy_stats.lock();
        
        stats.total_predictions += 1;
        
        if self.prediction_matches_actual(prediction, actual) {
            stats.correct_predictions += 1;
        }
        
        let error = self.calculate_error_distance(prediction, actual);
        stats.avg_error_distance_m =
            (stats.avg_error_distance_m * (stats.total_predictions - 1) as f32 + error)
            / stats.total_predictions as f32;
    }

    pub fn accuracy_rate(&self) -> f32 {
        let stats = self.accuracy_stats.lock();
        if stats.total_predictions == 0 {
            0.0
        } else {
            stats.correct_predictions as f32 / stats.total_predictions as f32
        }
    }

    pub fn should_trigger_cache_expansion(&self) -> bool {
        // If predictions are accurate (>70%), expand cache depth
        self.accuracy_rate() > 0.7
    }

    pub fn should_limit_cache_expansion(&self) -> bool {
        // If predictions are inaccurate (<50%), limit prefetching
        self.accuracy_rate() < 0.5
    }
}
```

---

## Integration with Layered Cache

```rust
impl CacheManager {
    pub fn integrate_predictive_warming(
        &self,
        predictor: Arc<PredictiveCacheWarmer>,
        learner: Arc<PredictionLearner>,
    ) {
        // Monitor agent state changes
        self.state_watcher.on_state_change(|state, history, context| {
            // Trigger predictive warming
            predictor.warm_cache_for_agent(&state, &history, &context);
            
            // Track what actually happened
            learner.record_outcome(&predicted, &actual);
        });
        
        // Adjust cache depth based on learning
        self.depth_manager.on_metrics_update(|metrics| {
            if learner.should_trigger_cache_expansion() {
                // Expand from Layer 0-1 to Layer 0-2
                self.expand_cache_depth();
            }
            
            if learner.should_limit_cache_expansion() {
                // Contract to conservative Layer 0 only
                self.limit_cache_depth();
            }
        });
    }

    pub fn prefetch_location_summary(&self, location: &GeoPoint) {
        // Pre-warm Layer 0 summary
        if !self.has_summary(location) {
            self.async_load_summary(location);
        }
    }

    pub fn prefetch_terrain_context(&self, region: &Region) {
        // Pre-warm Layer 1-2 context
        if !self.has_context(region) {
            self.async_load_context(region);
        }
    }
}
```

---

## Semantic Prefetching

Instead of caching only locations, cache likely-required knowledge:

```rust
pub enum SemanticPrefetch {
    /// Robot exploring: need detailed terrain
    ExplorationContext {
        region: Region,
        depth: CacheLayer,
    },
    
    /// Emergency mission: need real-time AI inferences
    EmergencyInferences {
        models: Vec<String>,
        region: Region,
    },
    
    /// Navigation to waypoint: need route history
    RouteHistory {
        start: GeoPoint,
        end: GeoPoint,
        recent_paths: usize,
    },
    
    /// Multi-robot coordination: need other agents' positions
    FleetState {
        team_ids: Vec<String>,
        recency_window_s: u32,
    },
}
```

---

## Implementation Phases

### Phase 1 (Week 20): Decision Graph & Prediction
- [ ] DecisionState and DecisionGraph types
- [ ] TrajectoryPredictor (linear + behavior-based models)
- [ ] Intent inference basic engine
- [ ] Trajectory prediction basic

### Phase 2 (Week 21): Cache Warming
- [ ] PredictiveCacheWarmer core
- [ ] Knowledge requirement identification
- [ ] Priority scoring
- [ ] Budget management

### Phase 3 (Week 22): Learning Loop
- [ ] PredictionLearner tracking
- [ ] Accuracy statistics
- [ ] Model retraining trigger
- [ ] Dynamic depth adjustment

### Phase 4 (Week 23): Integration
- [ ] Integration with CacheManager
- [ ] Semantic prefetching patterns
- [ ] State monitoring
- [ ] Feedback loop closure

### Phase 5 (Week 24+): Optimization
- [ ] Model ensemble tuning
- [ ] Latency optimization
- [ ] Memory profiling
- [ ] Large-scale testing

---

## Performance Impact

With predictive caching:
- **Cache hit rate**: 70% → 85%+ (pre-loading future data)
- **Query latency**: 100ms → 10-20ms (data already in cache)
- **Decision speed**: 500ms → 50-100ms (summaries pre-warmed)
- **Bandwidth**: Distributed over time (no spike on access)
- **Memory**: Higher baseline (predictions consume memory), but better utilization

---

## Guiding Philosophy

The goal of predictive caching is to make the **next likely decision appear instantaneous**.

A spatial intelligence system should:
1. **Anticipate** probable future states
2. **Prefetch** knowledge for those states
3. **Learn** from prediction accuracy
4. **Adapt** cache strategy based on learning
5. **Isolate** predictions from facts (marked as speculative)

This allows large-scale world models to remain responsive while avoiding the cost of loading complete datasets before action can begin.
