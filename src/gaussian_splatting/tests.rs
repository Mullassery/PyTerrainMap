#[cfg(test)]
mod gaussian_splatting_integration_tests {
    use crate::gaussian_splatting::*;
    use crate::temporal::DecayFunction;
    use chrono::Utc;

    // ===== Terrain Gaussian Tests =====

    #[test]
    fn test_multi_bot_fusion_agreement() {
        let mut g1 = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        g1.confidence = 0.8;
        let g2 = TerrainGaussian::from_point_observation([40.001, -74.001, 10.05], "bot_02", 0.85);

        let prev_conf = g1.confidence;
        let result = ObservationFuser::fuse(&mut g1, &g2);
        assert!(matches!(
            result.action,
            FusionAction::Fused { .. }
        ));
        assert!(g1.confidence > prev_conf);  // Confidence increased due to agreement
        assert_eq!(g1.observation_count, 2);
        assert!(g1.source_bots.contains(&"bot_02".to_string()));
    }

    #[test]
    fn test_conflict_handling() {
        let mut g1 = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.9);
        let g2 = TerrainGaussian::from_point_observation([40.001, -74.001, 10.05], "bot_02", 0.1);

        let prev_conf = g1.confidence;
        let _result = ObservationFuser::fuse(&mut g1, &g2);
        assert!(g1.confidence < prev_conf);  // Confidence decreased due to conflict
    }

    // ===== Temporal Decay Tests =====

    #[test]
    fn test_temporal_decay_exponential() {
        let mut g = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        g.last_updated = Utc::now().timestamp_micros() - (45 * 24 * 60 * 60 * 1_000_000);  // 45 days ago

        let decay = DecayFunction::Exponential {
            half_life_ms: 45 * 24 * 60 * 60 * 1000,
        };
        let age_ms = ((Utc::now().timestamp_micros() - g.last_updated) / 1000) as f32;
        let decayed = decay.apply(g.confidence, age_ms);

        assert!(decayed < g.confidence);
        assert!(decayed > 0.3);  // Should decay to ~50% after half-life
    }

    // ===== Store Tests =====

    #[test]
    fn test_store_insert_and_query() {
        let mut store = GaussianSplatStore::new();
        let g1 = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        let g2 = TerrainGaussian::from_point_observation([40.05, -74.05, 10.0], "bot_02", 0.85);

        store.insert(g1);
        store.insert(g2);

        let results = store.query_radius([40.025, -74.025, 10.0], 10000.0);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_store_uncertainty() {
        let mut store = GaussianSplatStore::new();
        let g = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.9);
        store.insert(g);

        let uncertainty = store.uncertainty_at([40.0, -74.0, 10.0]);
        assert!(uncertainty < 0.5);  // Should have low uncertainty at observed location
    }

    // ===== Dynamic Object Tests =====

    #[test]
    fn test_one_bot_learns_all_bots_know() {
        let mut fleet = FleetLearningEngine::new();

        // Bot 01 observes pallet
        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let events = fleet.ingest_observation("bot_01", vec![obs]);
        assert_eq!(events.len(), 1);

        // Bot 02 queries the same location without observing directly
        let now = Utc::now().timestamp_micros();
        let nearby = fleet.objects_near([10.001, 20.001, 0.0], 100.0, now);
        assert_eq!(nearby.len(), 1);  // Bot 02 knows about pallet even without observing
    }

    #[test]
    fn test_out_of_sight_inference() {
        let mut fleet = FleetLearningEngine::new();

        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        fleet.ingest_observation("bot_01", vec![obs]);

        // Simulate time passing
        let future_time = Utc::now().timestamp_micros() + (2 * 60 * 60 * 1_000_000);  // 2 hours later
        let nearby = fleet.objects_near([10.001, 20.001, 0.0], 100.0, future_time);

        assert_eq!(nearby.len(), 1);
        assert!(nearby[0].is_out_of_sight);  // Should be marked as out of sight
    }

    #[test]
    fn test_object_movement_detection() {
        let mut fleet = FleetLearningEngine::new();

        let obs1 = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let events1 = fleet.ingest_observation("bot_01", vec![obs1]);
        assert_eq!(events1.len(), 1);
        assert!(matches!(events1[0].event_type, ChangeEventType::ObjectAppeared { .. }));

        // Same pallet, moved 1 meter
        let obs2 = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.6, 20.0, 0.0],  // Moved > 0.5m threshold
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let events2 = fleet.ingest_observation("bot_02", vec![obs2]);
        assert_eq!(fleet.object_store.len(), 1);  // Still just 1 object
        assert!(events2.iter().any(|e| matches!(
            e.event_type,
            ChangeEventType::ObjectMoved { .. }
        )));
    }

    #[test]
    fn test_path_blocked_detection() {
        let mut fleet = FleetLearningEngine::new();

        let obs = ObjectObservation {
            object_class: ObjectClass::Cart,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: Utc::now().timestamp_micros(),
            confidence: 0.8,
            dimensions: Some([1.0, 1.0, 1.5]),
        };

        fleet.ingest_observation("bot_01", vec![obs]);

        let now = Utc::now().timestamp_micros();
        let blocked = fleet.path_blocked([9.0, 19.0, 0.0], [11.0, 21.0, 0.0], 1.0, now);
        assert!(blocked.is_some());  // Path should be blocked by cart
    }

    #[test]
    fn test_fleet_convergence() {
        let mut fleet = FleetLearningEngine::new();

        // 3 bots observe same location with high confidence
        for i in 0..3 {
            let obs = ObjectObservation {
                object_class: ObjectClass::Pallet,
                position: [10.0, 20.0, 0.0],
                covariance: GaussianCovariance::isotropic(0.5),
                timestamp: Utc::now().timestamp_micros(),
                confidence: 0.9,
                dimensions: None,
            };
            fleet.ingest_observation(&format!("bot_{:02}", i), vec![obs]);
        }

        assert_eq!(fleet.object_store.len(), 1);
        let obj_state = fleet.objects_near([10.0, 20.0, 0.0], 100.0, Utc::now().timestamp_micros());
        assert!(obj_state[0].object.confidence > 0.8);
    }

    #[test]
    fn test_dynamic_decay_by_mobility() {
        let person = DynamicObjectSplat::new(ObjectClass::Person, [10.0, 20.0, 0.0], "bot_01");
        let pallet = DynamicObjectSplat::new(ObjectClass::Pallet, [10.0, 20.0, 0.0], "bot_01");

        let future = Utc::now().timestamp_micros() + (45 * 60 * 1_000_000);  // 45 minutes

        let person_decayed = person.decayed_confidence(future);
        let pallet_decayed = pallet.decayed_confidence(future);

        assert!(person_decayed < pallet_decayed);  // Person decays faster
    }

    // ===== Passage Tests =====

    #[test]
    fn test_passage_open_probability() {
        let mut door = PassageSplat::new(
            PassageType::Door,
            [10.0, 20.0, 0.0],
            1.0,
            2.0,
            "room_a",
            "room_b",
        );

        // 7 successful opens, 1 failed
        for _ in 0..7 {
            door.record_traversal("bot_01", true, true);
        }
        door.record_traversal("bot_02", false, false);

        let prob = door.open_probability();
        assert!(prob > 0.7 && prob < 0.9);
    }

    // ===== Prediction Tests =====

    #[test]
    fn test_unknown_region_prediction() {
        let mut store = GaussianSplatStore::new();

        // Create 4 Grass splats around unknown center
        for (x, y) in &[(0.0, 0.0), (0.0, 0.05), (0.05, 0.0), (0.05, 0.05)] {
            let mut g = TerrainGaussian::from_point_observation([40.0 + x, -74.0 + y, 10.0], "bot_01", 0.9);
            g.terrain_type = TerrainType::Grass;
            store.insert(g);
        }

        let predictor = UnknownRegionPredictor::new();
        let prediction = predictor.predict_at([40.025, -74.025, 10.0], &store);

        assert!(prediction.is_some());
        let pred = prediction.unwrap();
        assert_eq!(pred.terrain_gaussian.terrain_type, TerrainType::Grass);
        assert!(pred.prediction_confidence > 0.5);
    }

    // ===== LOD Tests =====

    #[test]
    fn test_lod_split() {
        let lod = HierarchicalLOD::new();
        let mut splat = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        splat.observation_count = 25;

        assert!(lod.should_split(&splat));

        let children = lod.split(&splat);
        assert_eq!(children.len(), 4);
        for child in &children {
            assert!(child.observation_count < splat.observation_count);
        }
    }

    // ===== Exploration Tests =====

    #[test]
    fn test_exploration_prioritization() {
        let mut store = GaussianSplatStore::new();

        // Add some known terrain
        for i in 0..5 {
            let g = TerrainGaussian::from_point_observation(
                [40.0 + (i as f64) * 0.01, -74.0, 10.0],
                "bot_01",
                0.8,
            );
            store.insert(g);
        }

        let strategy = GaussianExplorationStrategy::new();
        let candidates = vec![
            [40.0, -74.0, 10.0],      // Known region
            [40.1, -74.1, 10.0],      // Unknown region
        ];

        let targets = strategy.top_targets(&candidates, &store, 2);
        assert!(targets[0].unknownness > targets[1].unknownness);
    }

    // ===== Semantic Tests =====

    #[test]
    fn test_bot_mission_preferences() {
        let mapper = SemanticGaussianMapper::new_with_defaults();

        let mut road = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        road.terrain_type = TerrainType::Road;

        let mut mud = TerrainGaussian::from_point_observation([40.0, -74.0, 10.0], "bot_01", 0.8);
        mud.terrain_type = TerrainType::Mud;

        let road_cost = mapper.mission_terrain_cost("DeliveryBot", &road);
        let mud_cost = mapper.mission_terrain_cost("DeliveryBot", &mud);

        assert!(road_cost < mud_cost);  // DeliveryBot prefers road
    }
}
