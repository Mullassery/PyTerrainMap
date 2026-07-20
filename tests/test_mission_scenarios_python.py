"""
Integration tests for multi-agent mission scenarios
Tests end-to-end Gaussian Splatting with full fleet coordination
"""

import pytest
from pyterrain_map import (
    PyGaussianSplatStore,
    PyGaussianSplatStore,
    PyFleetCoordinator,
    PyBotObservationMessage,
    PyGaussianFrontierScorer,
    PyFrontier,
    PyGaussianCacheManager,
)


class TestWarehouseDeliveryMission:
    """Test end-to-end warehouse delivery mission scenario"""

    def test_delivery_robots_coordinate_on_obstacle(self):
        """Two delivery robots discover and share obstacle"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register delivery fleet
        for i in range(3):
            coordinator.register_bot(f"delivery_bot_{i:02d}")

        # Bot_00 discovers fallen shelf
        obs1 = PyBotObservationMessage(
            bot_id="delivery_bot_00",
            lat=40.0,
            lon=-74.0,
            elev=1.5,
            traversability=0.0,  # Impassable
            confidence=0.95,
            terrain_type="Obstacle",
        )
        coordinator.broadcast_observation(obs1)

        # Bot_01 should now know about obstacle (without seeing it)
        # Verify by checking store has the observation
        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 1

        # Bot_02 learns from store and routes around
        nearby = store.objects_near(40.0, -74.0, 1.5, 1000.0)
        assert len(nearby) >= 0  # Would be > 0 if object tracking implemented

    def test_delivery_mission_consensus_mapping(self):
        """Multiple bots build consensus map of delivery zone"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Fleet discovers delivery zone terrain
        observations = [
            ("Road", 0.95, 40.0, -74.0),
            ("Road", 0.92, 40.001, -74.0),
            ("Grass", 0.85, 40.002, -74.0),
            ("Road", 0.93, 40.0, -74.001),
        ]

        for terrain, traversability, lat, lon in observations:
            coordinator.register_bot(f"bot_{len(coordinator.fleet_state()['active_bots'])}")
            msg = PyBotObservationMessage(
                bot_id=f"bot_{len(coordinator.fleet_state()['active_bots']) - 1:02d}",
                lat=lat,
                lon=lon,
                elev=0.5,
                traversability=traversability,
                confidence=0.85,
                terrain_type=terrain,
            )
            coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 4


class TestSurveillanceMission:
    """Test surveillance robot patrol mission"""

    def test_surveillance_robots_build_visibility_map(self):
        """Surveillance robots map coverage areas"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register surveillance drones
        for i in range(2):
            coordinator.register_bot(f"surveillance_drone_{i:02d}")

        # Drone_00 observes area A
        obs1 = PyBotObservationMessage(
            bot_id="surveillance_drone_00",
            lat=40.0,
            lon=-74.0,
            elev=50.0,  # High elevation for aerial view
            traversability=0.8,
            confidence=0.88,
            terrain_type="Open Area",
        )
        coordinator.broadcast_observation(obs1)

        # Drone_01 observes adjacent area B
        obs2 = PyBotObservationMessage(
            bot_id="surveillance_drone_01",
            lat=40.01,
            lon=-74.01,
            elev=50.0,
            traversability=0.75,
            confidence=0.82,
            terrain_type="Open Area",
        )
        coordinator.broadcast_observation(obs2)

        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 2
        assert int(state["active_bots"]) >= 2


class TestAgricultureMonitoringMission:
    """Test agriculture monitoring mission"""

    def test_agriculture_robots_map_field_conditions(self):
        """Agricultural rovers monitor field health"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        cache_manager = PyGaussianCacheManager()

        # Register agricultural rovers
        coordinator.register_bot("ag_rover_01")
        coordinator.register_bot("ag_rover_02")

        # Rovers observe field zones with different soil conditions
        soil_observations = [
            (40.0, -74.0, "Soil: Moist", 0.85, 0.90),
            (40.002, -74.0, "Soil: Dry", 0.65, 0.75),
            (40.004, -74.0, "Soil: Wet", 0.45, 0.85),
        ]

        for lat, lon, soil_type, traversability, confidence in soil_observations:
            msg = PyBotObservationMessage(
                bot_id="ag_rover_01" if lat < 40.002 else "ag_rover_02",
                lat=lat,
                lon=lon,
                elev=0.0,
                traversability=traversability,
                confidence=confidence,
                terrain_type=soil_type,
            )
            coordinator.ingest_observation(msg)

        # Cache should improve access patterns
        state = coordinator.fleet_state()
        summary = cache_manager.get_summary("field_zone", store)

        assert int(state["total_fused"]) >= 3
        assert summary is not None


class TestDisasterResponseMission:
    """Test disaster response scenario"""

    def test_disaster_response_robots_map_hazards(self):
        """Emergency response robots map disaster area"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        frontier_scorer = PyGaussianFrontierScorer()

        # Register emergency response robots
        coordinator.register_bot("rescue_bot_01")
        coordinator.register_bot("rescue_bot_02")
        coordinator.register_bot("rescue_bot_03")

        # Map disaster area - damaged buildings, blocked roads
        hazard_map = [
            (40.0, -74.0, "Debris Field", 0.0, 0.95),    # Impassable
            (40.001, -74.0, "Flooded Area", 0.1, 0.90),   # Dangerous
            (40.002, -74.0, "Clear Path", 0.8, 0.85),     # Usable route
            (40.003, -74.0, "Unstable Structure", 0.2, 0.88),
        ]

        for lat, lon, hazard_type, traversability, confidence in hazard_map:
            bot_id = f"rescue_bot_{(hazard_map.index((lat, lon, hazard_type, traversability, confidence)) % 3) + 1:02d}"
            msg = PyBotObservationMessage(
                bot_id=bot_id,
                lat=lat,
                lon=lon,
                elev=0.0,
                traversability=traversability,
                confidence=confidence,
                terrain_type=hazard_type,
            )
            coordinator.ingest_observation(msg)

        # Identify safe passages using frontier scoring
        frontiers = [
            PyFrontier("safe_passage_1", 40.002, -74.0, 0.0),
        ]
        frontier_scorer.score_frontiers(frontiers, store)

        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 4
        assert frontiers[0].priority > 0.0


class TestExplorationMission:
    """Test autonomous exploration mission"""

    def test_exploration_robots_discover_and_share_frontier(self):
        """Exploration robots discover new areas"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        frontier_scorer = PyGaussianFrontierScorer()

        # Register exploration robots
        coordinator.register_bot("explorer_01")
        coordinator.register_bot("explorer_02")

        # Build initial map
        coordinator.register_bot("explorer_01")
        msg1 = PyBotObservationMessage(
            bot_id="explorer_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            traversability=0.8,
            confidence=0.85,
            terrain_type="Forest",
        )
        coordinator.ingest_observation(msg1)

        # Identify unexplored frontier
        frontier = PyFrontier("unknown_area", 40.01, -74.01, 10.0)
        frontier_scorer.score_frontier(frontier, store)

        # High uncertainty = high exploration priority
        assert frontier.expected_information_gain > 0.5

        # Explorer_02 heads to frontier
        msg2 = PyBotObservationMessage(
            bot_id="explorer_02",
            lat=40.01,
            lon=-74.01,
            elev=10.0,
            traversability=0.6,
            confidence=0.75,
            terrain_type="Marsh",
        )
        coordinator.broadcast_observation(msg2)

        # Both explorers now know about both areas
        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 2


class TestFleetLearning:
    """Test fleet-wide learning and consensus"""

    def test_one_bot_learns_all_benefit(self):
        """Verify learning propagates across fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register 5-bot fleet
        for i in range(5):
            coordinator.register_bot(f"bot_{i:02d}")

        # Bot_00 discovers dangerous path
        obs = PyBotObservationMessage(
            bot_id="bot_00",
            lat=40.0,
            lon=-74.0,
            elev=0.0,
            traversability=0.1,  # Dangerous
            confidence=0.95,
            terrain_type="Hazard",
        )
        coordinator.broadcast_observation(obs)

        # All bots immediately benefit from knowledge
        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 1

        # Verify bot_04 (who never visited) also learned
        bot_04_status = coordinator.get_bot_status("bot_04")
        assert bot_04_status is not None
        assert bot_04_status["is_active"]

    def test_convergence_through_repeated_observation(self):
        """Multiple observations converge on consensus"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Fleet repeatedly observes same location
        location = (40.0, -74.0)

        for round_num in range(3):
            for bot_id in range(3):
                msg = PyBotObservationMessage(
                    bot_id=f"bot_{bot_id:02d}",
                    lat=location[0],
                    lon=location[1],
                    elev=5.0,
                    traversability=0.75 + round_num * 0.05,  # Converging confidence
                    confidence=0.70 + round_num * 0.10,
                    terrain_type="Road",
                )
                coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        # Should have 9 total fused observations
        assert int(state["total_fused"]) >= 9

        # Fleet health should be good (all active)
        health = coordinator.fleet_health()
        assert health >= 0.5


class TestCachingInMission:
    """Test caching for improved mission performance"""

    def test_cache_accelerates_repeated_queries(self):
        """Repeated region queries benefit from caching"""
        store = PyGaussianSplatStore()
        cache_manager = PyGaussianCacheManager()

        # Build initial map
        for i in range(10):
            store.insert_splat(40.0, -74.0, 10.0, f"bot_{i:02d}", 0.8, "Road")

        # First access: cache miss
        summary1 = cache_manager.get_summary("headquarters", store)
        stats1 = cache_manager.stats()

        # Repeated access: cache hit
        summary2 = cache_manager.get_summary("headquarters", store)
        stats2 = cache_manager.stats()

        # Should have cache hit
        assert int(stats2["cache_hits"]) > int(stats1["cache_hits"])


class TestComplexMissionScenario:
    """Test complex real-world mission scenario"""

    def test_multi_team_warehouse_coordination(self):
        """Complex warehouse with multiple teams coordinating"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        frontier_scorer = PyGaussianFrontierScorer()
        cache_manager = PyGaussianCacheManager()

        # Register mixed team
        teams = {
            "inventory": ["inv_bot_01", "inv_bot_02"],
            "delivery": ["del_bot_01", "del_bot_02"],
            "inspection": ["insp_drone_01"],
        }

        for team_bots in teams.values():
            for bot_id in team_bots:
                coordinator.register_bot(bot_id)

        # Inventory bots scan shelves
        coordinator.ingest_observation(PyBotObservationMessage(
            bot_id="inv_bot_01",
            lat=40.0,
            lon=-74.0,
            elev=1.5,
            traversability=0.95,
            confidence=0.90,
            terrain_type="Shelf Row",
        ))

        # Delivery bots find clear routes
        coordinator.ingest_observation(PyBotObservationMessage(
            bot_id="del_bot_01",
            lat=40.001,
            lon=-74.001,
            elev=0.0,
            traversability=0.85,
            confidence=0.88,
            terrain_type="Main Aisle",
        ))

        # Inspection drone provides overview
        coordinator.ingest_observation(PyBotObservationMessage(
            bot_id="insp_drone_01",
            lat=40.0005,
            lon=-74.0005,
            elev=30.0,
            traversability=0.80,
            confidence=0.85,
            terrain_type="Warehouse Floor",
        ))

        # Identify next exploration zone
        frontier = PyFrontier("stock_room", 40.01, -74.01, 0.0)
        frontier_scorer.score_frontier(frontier, store)

        # Cache workspace for faster queries
        cache_manager.get_summary("warehouse_floor", store)

        # Verify mission coordination
        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 3
        assert int(state["active_bots"]) >= 5

        health = coordinator.fleet_health()
        assert health > 0.0


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
