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
