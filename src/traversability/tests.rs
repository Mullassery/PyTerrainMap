//! Integration tests for spatial knowledge graph
//!
//! High-level tests combining multiple components.

use crate::traversability::{
    SpatialGraph, Node, NodeType, Edge, EdgeType,
    TraversabilityObservation, TraversalOutcome,
};

#[test]
fn test_complete_workflow() {
    // Create graph
    let mut graph = SpatialGraph::new("farm_map".to_string(), (37.7749, -122.4194, 10.0));

    // Add rooms
    let room1 = Node::new(
        "barn".to_string(),
        NodeType::IndoorRoom {
            width: 20.0,
            depth: 15.0,
            height: 4.0,
            floor_material: "concrete".to_string(),
        },
        (37.7749, -122.4194, 10.0),
    );

    let room2 = Node::new(
        "storage".to_string(),
        NodeType::IndoorRoom {
            width: 10.0,
            depth: 8.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (37.7750, -122.4193, 10.0),
    );

    graph.add_node(room1).unwrap();
    graph.add_node(room2).unwrap();

    assert_eq!(graph.node_count(), 2);

    // Connect with door
    let door = Edge::new(
        "barn_to_storage".to_string(),
        "barn".to_string(),
        "storage".to_string(),
        EdgeType::Door {
            width: 1.2,
            height: 2.2,
            is_open: true,
            requires_key: false,
            one_way: false,
        },
        3.5,
        true,
    );

    graph.add_edge(door).unwrap();
    assert_eq!(graph.edge_count(), 1);

    // Record successful traversals
    for i in 0..3 {
        let obs = TraversabilityObservation::new(
            format!("obs_{}", i),
            "barn_to_storage".to_string(),
            format!("robot_{}", i),
            "wheeled".to_string(),
            TraversalOutcome::Success {
                time_ms: 5000 + (i as u32 * 500),
                energy_used: 0.1 + (i as f32 * 0.02),
            },
        );
        graph.record_observation(obs).unwrap();
    }

    // Compute consensus
    let consensus = graph.compute_consensus("barn_to_storage", "wheeled").unwrap();
    assert_eq!(consensus.success_count, 3);
    assert_eq!(consensus.failure_count, 0);
    assert!(consensus.traversability_score() > 0.9);
}

#[test]
fn test_multi_robot_learning() {
    let mut graph = SpatialGraph::new("terrain_map".to_string(), (0.0, 0.0, 0.0));

    // Create terrain cells
    let cell1 = Node::new(
        "muddy_field".to_string(),
        NodeType::TerrainCell {
            surface_type: "mud".to_string(),
            elevation: 0.0,
            slope: 5.0,
            roughness: 0.8,
        },
        (0.0, 0.0, 0.0),
    );

    let cell2 = Node::new(
        "dry_field".to_string(),
        NodeType::TerrainCell {
            surface_type: "grass".to_string(),
            elevation: 0.0,
            slope: 2.0,
            roughness: 0.3,
        },
        (0.001, 0.001, 0.0),
    );

    graph.add_node(cell1).unwrap();
    graph.add_node(cell2).unwrap();

    let path = Edge::new(
        "cross_muddy".to_string(),
        "dry_field".to_string(),
        "muddy_field".to_string(),
        EdgeType::Path {
            surface_type: "mud".to_string(),
            distance: 100.0,
            clearance_height: None,
        },
        100.0,
        true,
    );

    graph.add_edge(path).unwrap();

    // Wheeled robot fails (too much slip)
    let wheeled_fail = TraversabilityObservation::new(
        "obs_wheel_1".to_string(),
        "cross_muddy".to_string(),
        "robot_wheels".to_string(),
        "wheeled".to_string(),
        TraversalOutcome::Failure {
            reason: "high_slip".to_string(),
        },
    );

    // Tracked robot succeeds
    let tracked_ok = TraversabilityObservation::new(
        "obs_track_1".to_string(),
        "cross_muddy".to_string(),
        "robot_tracks".to_string(),
        "tracked".to_string(),
        TraversalOutcome::Success {
            time_ms: 8000,
            energy_used: 0.25,
        },
    );

    graph.record_observation(wheeled_fail).unwrap();
    graph.record_observation(tracked_ok).unwrap();

    // Wheeled consensus: low traversability
    let wheeled_consensus = graph.compute_consensus("cross_muddy", "wheeled").unwrap();
    assert_eq!(wheeled_consensus.failure_count, 1);
    assert!(wheeled_consensus.traversability_score() < 0.5);

    // Tracked consensus: high traversability
    let tracked_consensus = graph.compute_consensus("cross_muddy", "tracked").unwrap();
    assert_eq!(tracked_consensus.success_count, 1);
    assert!(tracked_consensus.traversability_score() > 0.5);
}

#[test]
fn test_connected_components() {
    let mut graph = SpatialGraph::new("building".to_string(), (0.0, 0.0, 0.0));

    // Create chain of rooms: R1 - R2 - R3
    let r1 = Node::new(
        "room_1".to_string(),
        NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (0.0, 0.0, 0.0),
    );

    let r2 = Node::new(
        "room_2".to_string(),
        NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (1.0, 0.0, 0.0),
    );

    let r3 = Node::new(
        "room_3".to_string(),
        NodeType::IndoorRoom {
            width: 5.0,
            depth: 4.0,
            height: 3.0,
            floor_material: "tile".to_string(),
        },
        (2.0, 0.0, 0.0),
    );

    graph.add_node(r1).unwrap();
    graph.add_node(r2).unwrap();
    graph.add_node(r3).unwrap();

    let e1 = Edge::new(
        "edge_1_2".to_string(),
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
    );

    let e2 = Edge::new(
        "edge_2_3".to_string(),
        "room_2".to_string(),
        "room_3".to_string(),
        EdgeType::Door {
            width: 0.9,
            height: 2.1,
            is_open: true,
            requires_key: false,
            one_way: false,
        },
        2.0,
        true,
    );

    graph.add_edge(e1).unwrap();
    graph.add_edge(e2).unwrap();

    // All three rooms should be connected
    let component = graph.connected_component("room_1");
    assert_eq!(component.len(), 3);
    assert!(component.contains(&"room_1".to_string()));
    assert!(component.contains(&"room_2".to_string()));
    assert!(component.contains(&"room_3".to_string()));
}
