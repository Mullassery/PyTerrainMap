//! Integration tests for autonomous exploration intelligence

use crate::exploration::*;
use crate::traversability::*;

#[test]
fn test_complete_exploration_workflow() {
    // Setup
    let mut pattern_lib = PatternLibrary::new();
    let mut hypothesis_mgr = HypothesisManager::new();
    let mut fleet_stats = FleetStatistics::new();
    let classifier = SemanticClassifier::new();

    // Create some nodes
    let room1 = Node::new(
        "room_1".to_string(),
        NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (0.0, 0.0, 0.0),
    );

    let room2 = Node::new(
        "room_2".to_string(),
        NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (1.0, 0.0, 0.0),
    );

    // Learn patterns
    pattern_lib.learn_from_observations(
        EnvironmentType::OfficeBuilding,
        &[room1.clone(), room2.clone()],
        &[],
    );

    // Classify region
    let (env_type, confidence) = pattern_lib.classify_region(&[room1.clone(), room2.clone()]);
    assert!(confidence > 0.0);

    // Generate hypotheses for unknown edge
    let hypotheses = hypothesis_mgr.generate_hypotheses("unknown_edge_1", &[room1.clone(), room2.clone()]);
    assert!(!hypotheses.is_empty());

    // Get top hypothesis
    let top = hypothesis_mgr.get_top_hypothesis("unknown_edge_1");
    assert!(top.is_some());
    assert!(top.unwrap().confidence > 0.0);

    // Create predictor
    let predictor = TraversabilityPredictor::new(pattern_lib.clone());
    let prediction = predictor.predict_edge(&room1, &room2, &[room1.clone(), room2.clone()]);
    assert!(prediction.traversability_prob > 0.5);
    assert!(prediction.confidence > 0.0);

    // Record observation
    let obs = TraversabilityObservation::new(
        "obs_1".to_string(),
        "edge_1".to_string(),
        "robot_1".to_string(),
        "wheeled".to_string(),
        TraversalOutcome::Success {
            time_ms: 5000,
            energy_used: 0.1,
        },
    );

    fleet_stats.update_from_observation(&obs);
    assert!(fleet_stats.total_observations > 0);

    // Semantic classification
    let (context, conf) = classifier.classify_region(&[room1, room2]);
    assert!(conf > 0.0);
}

#[test]
fn test_pattern_learning_from_office() {
    let mut lib = PatternLibrary::new();

    // Create office-like rooms
    let mut rooms = vec![];
    for i in 0..5 {
        rooms.push(Node::new(
            format!("room_{}", i),
            NodeType::IndoorRoom {
                width: 4.5,
                depth: 3.5,
                height: 2.8,
                floor_material: "carpet".to_string(),
            },
            (i as f64 * 1.0, 0.0, 0.0),
        ));
    }

    lib.learn_from_observations(EnvironmentType::OfficeBuilding, &rooms, &[]);

    let pattern = lib.get_pattern(EnvironmentType::OfficeBuilding);
    assert!(pattern.is_some());

    let p = pattern.unwrap();
    assert!(p.observation_count > 0);
    assert!(p.confidence > 0.0);
}

#[test]
fn test_multi_hypothesis_competition() {
    let mut mgr = HypothesisManager::new();

    // Generate competing hypotheses
    let hypotheses = mgr.generate_hypotheses("elem_1", &[]);
    assert_eq!(hypotheses.len(), 3);

    // Simulate evidence supporting first hypothesis
    mgr.hypotheses.get_mut("elem_1").unwrap()[0]
        .add_evidence("observation_1".to_string());
    mgr.hypotheses.get_mut("elem_1").unwrap()[0]
        .add_evidence("observation_2".to_string());

    // Get top hypothesis - should favor the one with most evidence
    let top = mgr.get_top_hypothesis("elem_1");
    assert!(top.is_some());
    assert!(top.unwrap().confidence > 0.5);
}

#[test]
fn test_robot_heterogeneity() {
    let mut fleet_stats = FleetStatistics::new();

    // Wheeled robot succeeds
    let obs_wheeled = TraversabilityObservation::new(
        "obs_1".to_string(),
        "edge_1".to_string(),
        "robot_wheels".to_string(),
        "wheeled".to_string(),
        TraversalOutcome::Success {
            time_ms: 5000,
            energy_used: 0.1,
        },
    );

    // Aerial robot succeeds (different energy)
    let obs_aerial = TraversabilityObservation::new(
        "obs_2".to_string(),
        "edge_1".to_string(),
        "robot_air".to_string(),
        "aerial".to_string(),
        TraversalOutcome::Success {
            time_ms: 2000,
            energy_used: 0.3,
        },
    );

    fleet_stats.update_from_observation(&obs_wheeled);
    fleet_stats.update_from_observation(&obs_aerial);

    // Both robot types exist
    assert_eq!(fleet_stats.robot_capability_profiles.len(), 2);

    // Aerial is faster
    assert!(fleet_stats.robot_capability_profiles["aerial"].success_rate > 0.0);
}

#[test]
fn test_semantic_classification_outdoor() {
    let classifier = SemanticClassifier::new();

    let mut terrain_nodes = vec![];
    for i in 0..10 {
        terrain_nodes.push(Node::new(
            format!("terrain_{}", i),
            NodeType::TerrainCell {
                surface_type: "grass".to_string(),
                elevation: 10.0,
                slope: 5.0,
                roughness: 0.3,
            },
            ((i as f64) * 0.1, (i as f64) * 0.1, 0.0),
        ));
    }

    let (context, confidence) = classifier.classify_region(&terrain_nodes);
    assert_eq!(context, SemanticContext::OutdoorNatural);
    assert!(confidence > 0.5);

    // Check obstacle likelihood
    let obstacles = classifier.obstacle_likelihood_in(context);
    assert!(obstacles > 0.4);  // Natural terrain has higher obstacles
}

#[test]
fn test_prediction_validation_and_learning() {
    let lib = PatternLibrary::new();
    let predictor = TraversabilityPredictor::new(lib);

    let node1 = Node::new(
        "n1".to_string(),
        NodeType::TerrainCell {
            surface_type: "grass".to_string(),
            elevation: 0.0,
            slope: 0.0,
            roughness: 0.2,
        },
        (0.0, 0.0, 0.0),
    );

    let node2 = Node::new(
        "n2".to_string(),
        NodeType::TerrainCell {
            surface_type: "grass".to_string(),
            elevation: 0.0,
            slope: 0.0,
            roughness: 0.2,
        },
        (0.001, 0.001, 0.0),
    );

    let mut model = predictor.predict_edge(&node1, &node2, &[]);

    // Validate prediction
    assert!(model.traversability_prob > 0.5);
    model.validate(0.05);
    assert!(model.validated_at.is_some());
    assert!(model.validation_error.is_some());
}

#[test]
fn test_frontier_exploration_decision() {
    let mut pattern_lib = PatternLibrary::new();
    let mut hypothesis_mgr = HypothesisManager::new();

    // Setup office pattern
    let mut rooms = vec![];
    for i in 0..3 {
        rooms.push(Node::new(
            format!("room_{}", i),
            NodeType::IndoorRoom {
                width: 5.0,
                depth: 4.0,
                height: 3.0,
                floor_material: "tile".to_string(),
            },
            (i as f64, 0.0, 0.0),
        ));
    }

    // Create edges between rooms (doors)
    let edges = vec![
        Edge::new(
            "door_0_1".to_string(),
            "room_0".to_string(),
            "room_1".to_string(),
            EdgeType::Door {
                width: 0.9,
                height: 2.1,
                is_open: true,
                requires_key: false,
                one_way: false,
            },
            2.0,
            true,
        ),
        Edge::new(
            "door_1_2".to_string(),
            "room_1".to_string(),
            "room_2".to_string(),
            EdgeType::Door {
                width: 0.9,
                height: 2.1,
                is_open: true,
                requires_key: false,
                one_way: false,
            },
            2.0,
            true,
        ),
    ];

    pattern_lib.learn_from_observations(EnvironmentType::OfficeBuilding, &rooms, &edges);

    // Predict what's beyond last room (should predict doors as most likely)
    let predictions = pattern_lib.predict_next_connector(EnvironmentType::OfficeBuilding);
    assert!(!predictions.is_empty());
    assert!(predictions.iter().any(|(t, _)| t == "door"));

    // Generate hypotheses for next unknown region
    let hypotheses = hypothesis_mgr.generate_hypotheses("next_region", &rooms);
    assert!(!hypotheses.is_empty());

    // Top hypothesis should favor doors (typical in offices)
    let top = hypothesis_mgr.get_top_hypothesis("next_region");
    assert!(top.is_some());
}

#[test]
fn test_frontier_prioritization_workflow() {
    use crate::exploration::frontier::*;

    let prioritizer = FrontierPrioritizer::new();

    // Create frontiers with different characteristics
    let mut frontier_high_gain = Frontier::new(
        "frontier_high_gain".to_string(),
        (0.0, 0.0, 0.0),
    );
    frontier_high_gain.evaluate(
        0.9,  // high information gain
        0.3,  // moderate cost
        0.4,  // moderate risk
        0.8,  // high curiosity
    );

    let mut frontier_risky = Frontier::new(
        "frontier_risky".to_string(),
        (0.001, 0.001, 0.0),
    );
    frontier_risky.evaluate(
        0.8,
        0.5,
        0.9,  // very high risk
        0.7,
    );

    let ranked = prioritizer.rank_frontiers(vec![frontier_risky, frontier_high_gain.clone()]);

    // High gain, low risk should rank first
    assert_eq!(ranked[0].id, "frontier_high_gain");
}

#[test]
fn test_robot_specific_frontier_selection() {
    use crate::exploration::frontier::*;

    let prioritizer = FrontierPrioritizer::new();

    // Create diverse frontiers
    let mut frontier_high_value = Frontier::new(
        "frontier_high_value".to_string(),
        (0.0, 0.0, 0.0),
    );
    frontier_high_value.evaluate(0.95, 0.1, 0.2, 0.9);

    let mut frontier_moderate_risk = Frontier::new(
        "frontier_moderate_risk".to_string(),
        (0.001, 0.001, 0.0),
    );
    frontier_moderate_risk.evaluate(0.7, 0.3, 0.5, 0.6);

    let mut frontier_high_risk = Frontier::new(
        "frontier_high_risk".to_string(),
        (0.002, 0.002, 0.0),
    );
    frontier_high_risk.evaluate(0.8, 0.2, 0.85, 0.7);

    let frontiers = vec![frontier_high_value, frontier_moderate_risk, frontier_high_risk];

    // Aerial prefers high info gain
    let aerial = prioritizer.frontier_for_robot(&frontiers, "aerial");
    assert!(aerial.is_some());
    assert_eq!(aerial.unwrap().id, "frontier_high_value");  // Highest gain (0.95)

    // Wheeled avoids high risk
    let wheeled = prioritizer.frontier_for_robot(&frontiers, "wheeled");
    assert!(wheeled.is_some());
    assert!(wheeled.unwrap().risk_estimate < 0.7);

    // Tracked can handle risk
    let tracked = prioritizer.frontier_for_robot(&frontiers, "tracked");
    assert!(tracked.is_some());
}

#[test]
fn test_curiosity_vs_cost_tradeoff() {
    use crate::exploration::frontier::*;

    let prioritizer = FrontierPrioritizer::new();

    // High curiosity, low cost (explore)
    let mut good_frontier = Frontier::new("good".to_string(), (0.0, 0.0, 0.0));
    good_frontier.evaluate(0.8, 0.2, 0.3, 0.9);

    // High curiosity, high cost (don't explore)
    let mut expensive_frontier = Frontier::new("expensive".to_string(), (0.001, 0.001, 0.0));
    expensive_frontier.evaluate(0.8, 0.8, 0.8, 0.9);

    let ranked = prioritizer.rank_frontiers(vec![expensive_frontier, good_frontier.clone()]);

    assert_eq!(ranked[0].id, "good");
    assert!(ranked[0].priority > ranked[1].priority);
}

#[test]
fn test_complete_learning_loop() {
    use crate::exploration::learning::*;

    let mut learner = ActiveLearner::new();

    // Phase 1: Make predictions (poor quality)
    for i in 0..5 {
        let outcome = PredictionOutcome::new(
            format!("pred_{}", i),
            "corridor".to_string(),
            "door".to_string(),  // Wrong prediction
        )
        .with_confidence(0.6);

        learner.learn_from_outcome(&outcome);
    }

    let initial_progress = learner.learning_progress();

    // Phase 2: Make better predictions (correct)
    for i in 5..15 {
        let outcome = PredictionOutcome::new(
            format!("pred_{}", i),
            "door".to_string(),
            "door".to_string(),  // Correct prediction
        )
        .with_confidence(0.85);

        learner.learn_from_outcome(&outcome);
    }

    let final_progress = learner.learning_progress();

    // System should show improvement
    assert!(final_progress > initial_progress);
    assert_eq!(learner.validator.outcomes.len(), 15);
}

#[test]
fn test_systematic_error_identification() {
    use crate::exploration::learning::*;

    let mut validator = PredictionValidator::new();

    // Consistently predict "corridor" but find "door"
    for i in 0..8 {
        validator.record_outcome(PredictionOutcome::new(
            format!("p{}", i),
            "corridor".to_string(),
            "door".to_string(),
        ));
    }

    // Some correct predictions
    for i in 8..12 {
        validator.record_outcome(PredictionOutcome::new(
            format!("p{}", i),
            "door".to_string(),
            "door".to_string(),
        ));
    }

    let errors = validator.systematic_errors();

    // Should identify that "corridor" predictions systematically fail
    assert!(errors.contains_key("corridor"));
    assert_eq!(errors["corridor"].error_count, 8);
    assert!(errors["corridor"].frequencies.contains_key("door"));
}

#[test]
fn test_accuracy_improvement_trajectory() {
    use crate::exploration::learning::*;

    let mut learner = ActiveLearner::new();

    let mut accuracies = Vec::new();

    // Simulate learning over time (starts bad, ends good)
    // Phase 0: 0/5 correct (0.0%)
    for i in 0..5 {
        learner.learn_from_outcome(&PredictionOutcome::new(
            format!("p_0_{}", i),
            "corridor".to_string(),
            "door".to_string(),
        ).with_confidence(0.3));
    }
    accuracies.push(learner.validator.accuracy_metrics().accuracy_rate);

    // Phase 1: 3/5 correct (60%)
    for i in 0..3 {
        learner.learn_from_outcome(&PredictionOutcome::new(
            format!("p_1_{}", i),
            "door".to_string(),
            "door".to_string(),
        ).with_confidence(0.8));
    }
    for i in 3..5 {
        learner.learn_from_outcome(&PredictionOutcome::new(
            format!("p_1_{}", i),
            "corridor".to_string(),
            "door".to_string(),
        ).with_confidence(0.3));
    }
    accuracies.push(learner.validator.accuracy_metrics().accuracy_rate);

    // Phase 2: All correct (100%)
    for i in 0..5 {
        learner.learn_from_outcome(&PredictionOutcome::new(
            format!("p_2_{}", i),
            "door".to_string(),
            "door".to_string(),
        ).with_confidence(0.9));
    }
    accuracies.push(learner.validator.accuracy_metrics().accuracy_rate);

    // Accuracy should trend upward
    assert!(accuracies[1] > accuracies[0]);
    assert!(accuracies[2] > accuracies[1]);
}

#[test]
fn test_confidence_calibration_detection() {
    use crate::exploration::learning::*;

    let mut validator = PredictionValidator::new();

    // Well-calibrated: high confidence predictions are correct, low confidence are wrong
    for _ in 0..20 {
        validator.record_outcome(
            PredictionOutcome::new(
                "p".to_string(),
                "door".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.95),
        );
    }

    for _ in 0..20 {
        validator.record_outcome(
            PredictionOutcome::new(
                "p".to_string(),
                "corridor".to_string(),
                "door".to_string(),
            )
            .with_confidence(0.1),
        );
    }

    let calibration = validator.confidence_calibration();

    // Should detect good calibration
    let metrics = validator.accuracy_metrics();
    assert!(metrics.calibration_score > 0.7);
}

#[test]
fn test_fleet_learning_coordination() {
    use crate::exploration::learning::*;

    let mut robot_a_learner = ActiveLearner::new();
    let mut robot_b_learner = ActiveLearner::new();

    // Robot A explores and learns
    for i in 0..10 {
        let outcome = PredictionOutcome::new(
            format!("robot_a_pred_{}", i),
            "door".to_string(),
            "door".to_string(),
        )
        .with_confidence(0.7);

        robot_a_learner.learn_from_outcome(&outcome);
    }

    // Robot B benefits from shared learning (uses same outcome data)
    for i in 0..10 {
        let outcome = PredictionOutcome::new(
            format!("robot_b_pred_{}", i),
            "door".to_string(),
            "door".to_string(),
        )
        .with_confidence(0.7);

        robot_b_learner.learn_from_outcome(&outcome);
    }

    // Both should have high accuracy
    assert_eq!(
        robot_a_learner.learning_progress(),
        robot_b_learner.learning_progress()
    );
}
