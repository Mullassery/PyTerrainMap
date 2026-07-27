# Autonomous Exploration Intelligence Engine

**Objective:** Enable robots to make informed "best-guess" navigation decisions in unexplored regions using historical fleet knowledge, environmental patterns, and probabilistic reasoning.

**Timeline:** Phases 9-12 | **Current Focus:** Phase 9 Foundation

---

## Core Philosophy

Move beyond reactive mapping. When a robot encounters an unexplored region, instead of treating it as complete unknown, estimate likely characteristics using:

- Historical fleet observations
- Environmental patterns
- Semantic context
- Probabilistic inference
- Structural similarity matching

**Result:** Robots explore intelligently, maximize discovery, minimize wasted travel.

---

## Phase 9: Predictive Traversability Foundation

**Objective:** Build the probabilistic inference engine that estimates traversability for unexplored areas.

### 9.1 Environmental Pattern Learning

Learn recurring spatial structures from observed environments.

```rust
pub enum EnvironmentType {
    // Indoor
    OfficeBuilding,
    Warehouse,
    Hospital,
    RetailStore,
    Factory,
    School,
    
    // Outdoor
    Road,
    Sidewalk,
    Trail,
    AgriculturalField,
    IndustrialZone,
    Park,
    
    // Hybrid
    CarPark,
    OpenCorridor,
    Unknown,
}

pub struct EnvironmentPattern {
    pub env_type: EnvironmentType,
    pub features: Vec<String>,           // "narrow_corridor", "high_ceiling"
    pub common_sequences: Vec<Vec<String>>, // room->corridor->room patterns
    pub typical_widths: (f32, f32),      // min, max
    pub typical_heights: (f32, f32),
    pub typical_surfaces: Vec<String>,
    pub connector_frequency: HashMap<String, f32>, // door->0.85, stairs->0.2
    pub average_grid_density: f32,       // nodes per 100m²
    pub observation_count: u32,
    pub confidence: f32,
}

pub struct PatternLibrary {
    pub patterns: HashMap<String, EnvironmentPattern>,
    pub similarity_threshold: f32,
}

impl PatternLibrary {
    pub fn learn_from_observations(&mut self, nodes: &[Node], edges: &[Edge]) -> Vec<EnvironmentPattern>
    pub fn classify_region(&self, nodes: &[Node]) -> (EnvironmentType, f32)  // type, confidence
    pub fn predict_next_connector(&self, current_type: EnvironmentType) -> Vec<(String, f32)>  // (type, confidence)
}
```

### 9.2 Predictive Traversability Model

Estimate traversability probability for unexplored edges using Bayesian inference.

```rust
pub struct PredictiveModel {
    pub edge_id: String,
    pub from_node: String,
    pub to_node: String,
    
    // Predictions (probability 0.0-1.0)
    pub traversability_prob: f32,
    pub estimated_width: (f32, f32),     // min, max with confidence
    pub estimated_height: (f32, f32),
    pub estimated_surface: Vec<(String, f32)>,  // (surface_type, confidence)
    pub estimated_connector_type: Vec<(String, f32)>,  // (door/corridor/path, confidence)
    pub estimated_distance: (f32, f32),  // (mean, std_dev)
    pub estimated_traversal_time_ms: (u32, u32),
    pub estimated_energy_cost: (f32, f32),
    
    // Metadata
    pub prediction_basis: Vec<String>,   // "similar_office_18%", "historical_pattern_42%"
    pub confidence: f32,                 // How certain are we?
    pub predicted_at: i64,
    pub validated_at: Option<i64>,
    pub validation_error: Option<f32>,   // Actual vs predicted
}

pub struct TraversabilityPredictor {
    pub pattern_library: PatternLibrary,
    pub fleet_statistics: FleetStatistics,
    pub semantic_classifier: SemanticClassifier,
}

impl TraversabilityPredictor {
    pub fn predict_edge(&self, 
        from_node: &Node, 
        to_node: &Node,
        local_context: &[Node]
    ) -> PredictiveModel
    
    pub fn predict_connector_type(&self, 
        current_env: EnvironmentType,
        historical_data: &[(EnvironmentType, &str, f32)]
    ) -> Vec<(String, f32)>  // (type, confidence)
}
```

### 9.3 Hypothesis Generation Framework

Maintain multiple competing hypotheses for unknown areas.

```rust
pub struct Hypothesis {
    pub id: String,
    pub element_id: String,      // node or edge being predicted
    pub hypothesis_type: HypothesisType,
    pub predictions: HashMap<String, PredictionValue>,
    pub confidence: f32,         // 0.0-1.0
    pub created_at: i64,
    pub supporting_evidence: Vec<String>,  // Why we believe this
    pub contradicting_evidence: Vec<String>,
}

pub enum HypothesisType {
    ConnectorExists(String),     // door, corridor, stairs, etc.
    NodeType(String),            // room, terrain_cell, landmark
    RouteExists { from: String, to: String },
    ObstacleExists { location: String },
}

pub enum PredictionValue {
    Categorical(Vec<(String, f32)>),  // (value, confidence)
    Numerical { mean: f32, std_dev: f32 },
    Boolean(f32),                      // confidence of true
}

pub struct HypothesisManager {
    pub hypotheses: HashMap<String, Vec<Hypothesis>>,  // element_id -> hypotheses
    pub prediction_history: Vec<PredictionOutcome>,
}

pub struct PredictionOutcome {
    pub hypothesis_id: String,
    pub predicted_value: String,
    pub actual_value: String,
    pub accuracy: f32,
    pub prediction_confidence: f32,
    pub outcome_timestamp: i64,
}

impl HypothesisManager {
    pub fn generate_hypotheses(&mut self, 
        element_id: &str,
        context: &[Node],
        pattern_lib: &PatternLibrary
    ) -> Vec<Hypothesis>
    
    pub fn update_hypothesis(&mut self, 
        element_id: &str, 
        observation: &TraversabilityObservation
    ) -> f32  // confidence adjustment
    
    pub fn get_top_hypothesis(&self, element_id: &str) -> Option<&Hypothesis>
    
    pub fn remove_hypothesis(&mut self, element_id: &str, hypothesis_id: &str)
}
```

### 9.4 Fleet Statistics Aggregator

Collect aggregate statistics from all fleet observations.

```rust
pub struct FleetStatistics {
    pub total_observations: u32,
    pub success_rate_by_environment: HashMap<String, f32>,
    pub average_traversal_time_by_connector: HashMap<String, u32>,
    pub energy_cost_by_terrain: HashMap<String, f32>,
    pub failure_reasons: HashMap<String, u32>,
    pub robot_capability_profiles: HashMap<String, RobotProfile>,
}

pub struct RobotProfile {
    pub robot_type: String,
    pub success_rate: f32,
    pub failure_history: Vec<String>,
    pub preferred_environments: Vec<(String, f32)>,
    pub average_energy_consumption: f32,
}

impl FleetStatistics {
    pub fn update_from_observation(&mut self, obs: &TraversabilityObservation)
    pub fn success_probability_for(&self, 
        environment: EnvironmentType,
        robot_type: &str
    ) -> f32
    pub fn expected_cost_for(&self,
        connector_type: &str,
        robot_type: &str
    ) -> (u32, f32)  // (time_ms, energy)
}
```

### 9.5 Semantic Classifier

Assign semantic meaning to unexplored regions.

```rust
pub struct SemanticClassifier {
    pub terrain_classifier: TerrainModel,    // uses PyRoboVision
    pub structure_classifier: StructureModel,
    pub connectivity_analyzer: ConnectivityModel,
}

pub enum SemanticContext {
    IndoorStructured,      // Offices, warehouses (grid-like)
    IndoorOrganic,         // Hospitals, retail (non-grid)
    OutdoorPaved,          // Roads, sidewalks
    OutdoorNatural,        // Fields, trails, forests
    OutdoorMixed,          // Parks, industrial zones
}

impl SemanticClassifier {
    pub fn classify_region(&self, nodes: &[Node]) -> (SemanticContext, f32)
    pub fn predict_structure_from_context(&self, context: SemanticContext) -> StructureTemplate
}

pub struct StructureTemplate {
    pub connectivity_pattern: Vec<Vec<String>>,  // expected sequences
    pub typical_dimensions: (f32, f32, f32),     // (width, depth, height)
    pub obstacle_likelihood: f32,
}
```

---

## Phase 10: Frontier Intelligence & Curiosity

(Coming next week)

### 10.1 Frontier Evaluation

Select which unexplored areas to explore next.

```rust
pub struct Frontier {
    pub id: String,
    pub target_location: (f64, f64, f32),
    pub boundary_edges: Vec<String>,  // edges to cross to reach
    pub expected_information_gain: f32,
    pub exploration_cost: f32,
    pub risk_estimate: f32,
    pub curiosity_score: f32,
    pub priority: u32,
}

pub fn evaluate_frontier(
    frontier: &Frontier,
    current_knowledge: &SpatialGraph,
    robot_state: &RobotState,
    mission: &MissionObjective
) -> FrontierScore
```

### 10.2 Curiosity Scoring

Assign discovery value to unknown regions.

```rust
pub fn curiosity_score(
    region: &UnknownRegion,
    information_scarcity: f32,
    strategic_importance: f32,
    connectivity_potential: f32,
    environmental_uniqueness: f32
) -> f32
```

---

## Phase 11: Active Learning & Outcome Validation

(Coming next week)

Record prediction accuracy and improve future estimates.

```rust
pub struct PredictionValidator {
    pub prediction: PredictiveModel,
    pub actual_observation: TraversabilityObservation,
    pub error_magnitude: f32,
    pub lessons_learned: Vec<String>,
}
```

---

## Phase 12: Robot-Specific Strategies & Discovery Graph

(Coming next week)

Different robots explore differently. Integrate predictions into route planning.

---

## Implementation Roadmap

### Phase 9 (This Week): Foundation
- [x] Design document
- [ ] Environmental pattern learning (learn_from_observations)
- [ ] Pattern classification (classify_region)
- [ ] Predictive model (predict_edge)
- [ ] Hypothesis generation (generate_hypotheses)
- [ ] Fleet statistics aggregation
- [ ] 20-25 comprehensive tests
- [ ] Integration with Phase 8 SpatialGraph

### Phase 10 (Week 2): Exploration Intelligence
- [ ] Frontier detection and evaluation
- [ ] Curiosity scoring engine
- [ ] Risk modeling
- [ ] Frontier prioritization

### Phase 11 (Week 3): Learning Loop
- [ ] Outcome validation
- [ ] Prediction error tracking
- [ ] Model improvement feedback
- [ ] Continuous learning from observations

### Phase 12 (Week 4): Robot Strategies & Integration
- [ ] Robot-type specific exploration
- [ ] Discovery graph (confirmed vs. predicted)
- [ ] Route planner integration
- [ ] End-to-end autonomous exploration demo

---

## Success Criteria

### Phase 9
- ✅ 20-25 passing tests
- ✅ Pattern library learns from 100+ observations
- ✅ Hypothesis generation produces confidence-weighted predictions
- ✅ Fleet statistics accurate within ±15%
- ✅ Semantic classification >85% accuracy

### Phase 10
- ✅ Frontier evaluation produces valid rankings
- ✅ Curiosity scores correlate with discovery value
- ✅ Risk estimates prevent >90% of failures

### Phase 11
- ✅ Prediction accuracy improves over time
- ✅ Error feedback improves model by >10% weekly

### Phase 12
- ✅ Autonomous robots explore efficiently
- ✅ >80% exploration efficiency (discovery/distance ratio)
- ✅ Robot-type strategies reduce failures by >20%

---

## Architecture

```
src/exploration/
├── mod.rs                          (exports)
├── patterns.rs                     (EnvironmentPattern, PatternLibrary)
├── predictions.rs                  (PredictiveModel, TraversabilityPredictor)
├── hypotheses.rs                   (Hypothesis, HypothesisManager)
├── statistics.rs                   (FleetStatistics, RobotProfile)
├── semantics.rs                    (SemanticClassifier, SemanticContext)
├── frontier.rs                     (Phase 10: Frontier, FrontierEvaluator)
├── learning.rs                     (Phase 11: PredictionValidator, OutcomeAnalyzer)
├── strategies.rs                   (Phase 12: RobotExplorationStrategy)
└── tests.rs                        (Integration tests)
```

---

## Key Insights

1. **Probabilistic First:** Never treat predictions as facts. Always express as probability distributions.

2. **Evidence Tracking:** Store why we believe something (similar offices, historical pattern, etc.). Use this to adjust confidence.

3. **Hypothesis Competition:** Multiple theories should coexist. New observations eliminate weak hypotheses.

4. **Continuous Improvement:** Every misprediction is a learning opportunity. Track error patterns to improve future models.

5. **Fleet-Wide Learning:** A discovery by robot A immediately benefits robots B, C, D. Exploration is collaborative, not individual.

6. **Robot Heterogeneity:** Different robots have different capabilities. Exploration strategies adapt to robot type, size, sensors.

7. **Information Geometry:** Prioritize exploration that reduces uncertainty most, not just physical distance.

---

**Next Step:** Implement Phase 9 foundation (patterns, predictions, hypotheses) with 20-25 tests.
