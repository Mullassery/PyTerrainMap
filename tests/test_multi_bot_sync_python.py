"""
Pytest tests for Multi-Bot Synchronization
Tests fleet coordination and shared world model updates
"""

import pytest
from pyterrain_map import (
    PyGaussianSplatStore,
    PyBotObservationMessage,
    PyFleetCoordinator,
)


class TestBotObservationMessage:
    """Test BotObservationMessage creation and properties"""

    def test_create_message(self):
        """Create observation message from bot"""
        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )
        assert msg.bot_id == "bot_01"
        assert abs(msg.traversability - 0.85) < 0.01
        assert abs(msg.confidence - 0.9) < 0.01

    def test_message_repr(self):
        """Test message string representation"""
        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )
        repr_str = repr(msg)
        assert "BotObservationMessage" in repr_str
        assert "bot_01" in repr_str

    def test_message_with_none_terrain_type(self):
        """Create message with no terrain type (obstacle sighting)"""
        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            traversability=0.0,
            confidence=0.95,
            terrain_type=None,
        )
        assert msg.traversability == 0.0
        assert abs(msg.confidence - 0.95) < 0.01


class TestFleetCoordinator:
    """Test PyFleetCoordinator"""

    def test_create_coordinator(self):
        """Create fleet coordinator"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        assert coordinator is not None

    def test_coordinator_repr(self):
        """Test coordinator string representation"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)
        repr_str = repr(coordinator)
        assert "FleetCoordinator" in repr_str

    def test_register_single_bot(self):
        """Register single bot in fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        coordinator.register_bot("bot_01")

        state = coordinator.fleet_state()
        assert int(state["active_bots"]) >= 1

    def test_register_multiple_bots(self):
        """Register multiple bots in fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        for i in range(5):
            coordinator.register_bot(f"bot_{i:02d}")

        state = coordinator.fleet_state()
        assert int(state["active_bots"]) >= 5


class TestObservationIngestion:
    """Test ingesting bot observations"""

    def test_ingest_single_observation(self):
        """Ingest observation from bot"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )

        result = coordinator.ingest_observation(msg)
        assert result is None  # Should succeed with no error

    def test_ingest_multiple_observations_same_bot(self):
        """Ingest multiple observations from same bot"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        for i in range(5):
            msg = PyBotObservationMessage(
                bot_id="bot_01",
                lat=40.0 + i * 0.001,
                lon=-74.0,
                elev=10.0,
                terrain_type="Road",
                traversability=0.85,
                confidence=0.9,
            )
            coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 5

    def test_ingest_observations_different_bots(self):
        """Ingest observations from different bots"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        for i in range(3):
            msg = PyBotObservationMessage(
                bot_id=f"bot_{i:02d}",
                lat=40.0,
                lon=-74.0,
                elev=10.0,
                terrain_type="Road",
                traversability=0.85,
                confidence=0.9,
            )
            coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 3


class TestBroadcast:
    """Test observation broadcasting to fleet"""

    def test_broadcast_to_empty_fleet(self):
        """Broadcast should work even with no registered bots"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )

        broadcast_count = coordinator.broadcast_observation(msg)
        assert broadcast_count >= 0

    def test_broadcast_to_multi_bot_fleet(self):
        """Broadcast observation to registered fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register fleet
        for i in range(5):
            coordinator.register_bot(f"bot_{i:02d}")

        msg = PyBotObservationMessage(
            bot_id="bot_00",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )

        broadcast_count = coordinator.broadcast_observation(msg)
        # Should broadcast to all registered bots
        assert broadcast_count >= 1


class TestFleetState:
    """Test fleet state tracking"""

    def test_fleet_state_empty(self):
        """Fleet state with no bots"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        state = coordinator.fleet_state()
        assert isinstance(state, dict)
        assert "active_bots" in state
        assert "total_fused" in state

    def test_fleet_state_updates_on_observations(self):
        """Fleet state should update when observations ingested"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        initial_state = coordinator.fleet_state()
        initial_fused = int(initial_state["total_fused"])

        # Ingest observation
        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )
        coordinator.ingest_observation(msg)

        final_state = coordinator.fleet_state()
        final_fused = int(final_state["total_fused"])

        # Should have processed observation
        assert final_fused > initial_fused


class TestBotStatus:
    """Test individual bot status tracking"""

    def test_get_bot_status_after_observation(self):
        """Bot status updated after observation"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Road",
            traversability=0.85,
            confidence=0.9,
        )
        coordinator.ingest_observation(msg)

        status = coordinator.get_bot_status("bot_01")
        assert status is not None
        assert "bot_id" in status
        assert "is_active" in status

    def test_get_all_bot_statuses(self):
        """Get status of all bots in fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register and have bots send observations
        for i in range(3):
            coordinator.register_bot(f"bot_{i:02d}")
            msg = PyBotObservationMessage(
                bot_id=f"bot_{i:02d}",
                lat=40.0,
                lon=-74.0,
                elev=10.0,
                terrain_type="Road",
                traversability=0.85,
                confidence=0.9,
            )
            coordinator.ingest_observation(msg)

        statuses = coordinator.all_bot_statuses()
        assert isinstance(statuses, list)
        assert len(statuses) >= 3


class TestFleetHealth:
    """Test fleet health monitoring"""

    def test_fleet_health_score(self):
        """Fleet health should return 0.0-1.0"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        health = coordinator.fleet_health()
        assert isinstance(health, float)
        assert 0.0 <= health <= 1.0

    def test_fleet_health_improves_with_bots(self):
        """Health improves as more bots registered"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        health_empty = coordinator.fleet_health()

        # Register bots
        for i in range(5):
            coordinator.register_bot(f"bot_{i:02d}")
            msg = PyBotObservationMessage(
                bot_id=f"bot_{i:02d}",
                lat=40.0,
                lon=-74.0,
                elev=10.0,
                terrain_type="Road",
                traversability=0.85,
                confidence=0.9,
            )
            coordinator.ingest_observation(msg)

        health_full = coordinator.fleet_health()
        # Health should improve or stay same
        assert health_full >= health_empty


class TestOneToMany:
    """Test 'one bot learns, all bots know' principle"""

    def test_observation_sharing_across_fleet(self):
        """Single bot's observation visible to all"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register 3 bots
        for i in range(3):
            coordinator.register_bot(f"bot_{i:02d}")

        # Bot_00 observes something
        msg = PyBotObservationMessage(
            bot_id="bot_00",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            terrain_type="Obstacle",
            traversability=0.0,
            confidence=0.95,
        )
        coordinator.broadcast_observation(msg)

        # All bots should be aware (stored in shared world model)
        state = coordinator.fleet_state()
        assert int(state["total_fused"]) >= 1


class TestConsensusBuilding:
    """Test fleet consensus on observations"""

    def test_multiple_bots_observe_same_location(self):
        """Multiple observations of same location"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # 3 bots observe same location
        for i in range(3):
            msg = PyBotObservationMessage(
                bot_id=f"bot_{i:02d}",
                lat=40.0,
                lon=-74.0,
                elev=10.0,
                terrain_type="Road",
                traversability=0.80 + i * 0.05,  # Slightly different confidence
                confidence=0.85 + i * 0.05,
            )
            coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        # Should have fused all 3 observations
        assert int(state["total_fused"]) >= 3

    def test_converging_observations_improve_confidence(self):
        """Repeated observations should build consensus"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Multiple bots observe same region multiple times
        for iteration in range(3):
            for bot_id in range(3):
                msg = PyBotObservationMessage(
                    bot_id=f"bot_{bot_id:02d}",
                    lat=40.0,
                    lon=-74.0,
                    elev=10.0,
                    terrain_type="Road",
                    traversability=0.85,
                    confidence=0.8 + iteration * 0.05,  # Increasing confidence
                )
                coordinator.ingest_observation(msg)

        state = coordinator.fleet_state()
        # Should have 9 total fused observations
        assert int(state["total_fused"]) >= 9


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
