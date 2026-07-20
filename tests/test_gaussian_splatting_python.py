"""
Pytest tests for Gaussian Splatting Python API
Tests all PyGaussianSplatStore methods and wrapper classes
"""

import pytest
import time
from pyterrain_map import (
    PyGaussianSplatStore,
    PyGaussianCovariance,
    PyTerrainGaussian,
    PyDynamicObjectSplat,
    PyObjectObservation,
    PyChangeEvent,
    PyPathCost,
)


class TestGaussianCovariance:
    """Test PyGaussianCovariance wrapper"""

    def test_isotropic_creation(self):
        """Create isotropic covariance"""
        cov = PyGaussianCovariance.isotropic(1.0)
        assert cov is not None
        det = cov.determinant()
        assert det > 0.0

    def test_diagonal_creation(self):
        """Create diagonal covariance"""
        cov = PyGaussianCovariance.diagonal(1.0, 2.0, 3.0)
        assert cov is not None
        det = cov.determinant()
        # det should be 1 * 4 * 9 = 36
        assert abs(det - 36.0) < 0.01

    def test_uncertainty_volume(self):
        """Compute uncertainty volume"""
        cov = PyGaussianCovariance.isotropic(1.0)
        vol = cov.uncertainty_volume()
        assert vol > 0.0
        assert vol > 15.0  # (2π)^(3/2) / √det ≈ 15.75 for det=1

    def test_to_dict(self):
        """Convert covariance to dict"""
        cov = PyGaussianCovariance.isotropic(1.0)
        d = cov.to_dict()
        assert "determinant" in d
        assert "uncertainty_volume" in d


class TestTerrainGaussian:
    """Test PyTerrainGaussian wrapper"""

    def test_from_point_observation(self):
        """Create Gaussian from point observation"""
        g = PyTerrainGaussian.from_point_observation(
            lat=40.7128,
            lon=-74.0060,
            elev=10.0,
            bot_id="bot_01",
            traversability=0.85,
            terrain_type="Road",
        )
        assert g is not None
        assert g.terrain_type == "Road"
        assert abs(g.position_lat - 40.7128) < 0.0001
        assert abs(g.traversability - 0.85) < 0.001  # f32 precision
        assert g.confidence > 0.0

    def test_gaussian_repr(self):
        """Test string representation"""
        g = PyTerrainGaussian.from_point_observation(
            40.0, -74.0, 10.0, "bot_01", 0.85, "Grass"
        )
        repr_str = repr(g)
        assert "TerrainGaussian" in repr_str
        assert "Grass" in repr_str

    def test_gaussian_to_dict(self):
        """Convert Gaussian to dict"""
        g = PyTerrainGaussian.from_point_observation(
            40.0, -74.0, 10.0, "bot_01", 0.85, "Mud"
        )
        d = g.to_dict()
        assert "terrain_type" in d
        assert "traversability" in d
        assert "confidence" in d
        assert d["terrain_type"] == "Mud"


class TestDynamicObjectSplat:
    """Test PyDynamicObjectSplat wrapper"""

    def test_create_object(self):
        """Create dynamic object splat"""
        obj = PyDynamicObjectSplat.new("Pallet", 10.0, 20.0, 0.0, "bot_01")
        assert obj is not None
        assert obj.object_class == "Pallet"
        assert abs(obj.position_lat - 10.0) < 0.0001

    def test_decayed_confidence(self):
        """Test confidence decay over time"""
        obj = PyDynamicObjectSplat.new("Pallet", 10.0, 20.0, 0.0, "bot_01")
        now = int(time.time() * 1_000_000)
        conf_now = obj.decayed_confidence(now)
        # After 2 hours, confidence should decay
        conf_later = obj.decayed_confidence(now + 2 * 60 * 60 * 1_000_000)
        # Movable objects have 2-hour half-life
        assert conf_later < conf_now
        # Should be ~50% after half-life
        assert 0.3 < conf_later < 0.6


class TestObjectObservation:
    """Test PyObjectObservation wrapper"""

    def test_create_observation(self):
        """Create object observation"""
        now = int(time.time() * 1_000_000)
        obs = PyObjectObservation("Cart", 15.0, 25.0, 0.0, now, 0.9)
        assert obs.object_class == "Cart"
        assert abs(obs.confidence - 0.9) < 0.001  # f32 precision


class TestGaussianSplatStore:
    """Test PyGaussianSplatStore wrapper"""

    def test_store_creation(self):
        """Create new store"""
        store = PyGaussianSplatStore()
        assert store is not None

    def test_insert_splat(self):
        """Insert terrain splat"""
        store = PyGaussianSplatStore()
        splat_id = store.insert_splat(
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            bot_id="bot_01",
            traversability=0.8,
            terrain_type="Road",
        )
        assert splat_id is not None
        assert isinstance(splat_id, str)

    def test_query_radius(self):
        """Query splats in radius"""
        store = PyGaussianSplatStore()
        # Insert 3 splats
        for i in range(3):
            store.insert_splat(
                40.0 + i * 0.001,
                -74.0,
                10.0,
                f"bot_{i:02d}",
                0.8,
                "Grass",
            )

        # Query nearby
        results = store.query_radius(40.001, -74.0, 10.0, 1000.0)
        assert len(results) == 3

    def test_uncertainty_at(self):
        """Get uncertainty at position"""
        store = PyGaussianSplatStore()
        # Empty store: high uncertainty
        unc_empty = store.uncertainty_at(40.0, -74.0, 10.0)
        assert unc_empty == 1.0

        # Add observation
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        unc_known = store.uncertainty_at(40.0, -74.0, 10.0)
        assert unc_known < 1.0

    def test_path_cost(self):
        """Compute path cost between two points"""
        store = PyGaussianSplatStore()
        # Add observations along path
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.8, "Road")

        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )
        assert isinstance(cost, PyPathCost)
        assert cost.total > 0.0
        assert cost.distance_cost >= 0.0
        assert cost.terrain_cost >= 0.0

    def test_apply_temporal_decay(self):
        """Apply temporal decay"""
        store = PyGaussianSplatStore()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        now = int(time.time() * 1_000_000)
        store.apply_temporal_decay(now)
        # Should not crash

    def test_ingest_object_observation(self):
        """Ingest object observations"""
        store = PyGaussianSplatStore()
        now = int(time.time() * 1_000_000)

        obs = PyObjectObservation("Pallet", 10.0, 20.0, 0.0, now, 0.9)
        events = store.ingest_object_observation("bot_01", [obs])
        assert isinstance(events, list)
        assert len(events) > 0
        assert isinstance(events[0], PyChangeEvent)

    def test_objects_near(self):
        """Query objects near position"""
        store = PyGaussianSplatStore()
        now = int(time.time() * 1_000_000)

        # Ingest an observation
        obs = PyObjectObservation("Cart", 10.0, 20.0, 0.0, now, 0.9)
        store.ingest_object_observation("bot_01", [obs])

        # Query nearby (use 1000m radius to ensure inclusion; ~111m per 0.001 degrees)
        nearby = store.objects_near(10.0, 20.0, 0.0, 1000.0)
        assert isinstance(nearby, list)
        assert len(nearby) >= 1

    def test_stats(self):
        """Get store statistics"""
        store = PyGaussianSplatStore()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        stats = store.stats()
        assert isinstance(stats, dict)
        assert "total_splats" in stats
        assert int(stats["total_splats"]) > 0


class TestMultiBotCoordination:
    """Test multi-bot coordination through shared store"""

    def test_one_bot_learns_all_know(self):
        """Verify: one bot learns, all bots know"""
        store = PyGaussianSplatStore()
        now = int(time.time() * 1_000_000)

        # Bot 01 observes obstacle
        obs = PyObjectObservation("Cart", 15.0, 25.0, 0.0, now, 0.9)
        events = store.ingest_object_observation("bot_01", [obs])
        assert len(events) > 0
        assert "ObjectAppeared" in events[0].event_type

        # Bot 02 queries WITHOUT observing directly (use 1000m radius)
        nearby = store.objects_near(15.0, 25.0, 0.0, 1000.0)
        assert len(nearby) >= 1  # Bot 02 knows about cart!

    def test_fleet_convergence(self):
        """Test that multiple observations converge on same map"""
        store = PyGaussianSplatStore()

        # 3 different bots observe same region
        for i in range(3):
            store.insert_splat(
                40.0,
                -74.0,
                10.0,
                f"bot_{i:02d}",
                0.8 + i * 0.05,
                "Road",
            )

        results = store.query_radius(40.0, -74.0, 10.0, 1000.0)
        assert len(results) >= 3

        # Uncertainty should be low (many observations)
        unc = store.uncertainty_at(40.0, -74.0, 10.0)
        assert unc < 0.5  # Should be well-known


class TestChangeEvents:
    """Test change event tracking"""

    def test_object_appeared_event(self):
        """Verify ObjectAppeared event"""
        store = PyGaussianSplatStore()
        now = int(time.time() * 1_000_000)

        obs = PyObjectObservation("Pallet", 10.0, 20.0, 0.0, now, 0.9)
        events = store.ingest_object_observation("bot_01", [obs])
        assert len(events) == 1
        assert events[0].event_type == "ObjectAppeared"
        assert "bot_01" in events[0].detected_by

    def test_object_moved_event(self):
        """Verify ObjectMoved event detection"""
        store = PyGaussianSplatStore()
        now = int(time.time() * 1_000_000)

        # First observation
        obs1 = PyObjectObservation("Pallet", 10.0, 20.0, 0.0, now, 0.9)
        events1 = store.ingest_object_observation("bot_01", [obs1])
        assert events1[0].event_type == "ObjectAppeared"

        # Second observation: moved
        obs2 = PyObjectObservation("Pallet", 10.6, 20.0, 0.0, now + 1000, 0.9)
        events2 = store.ingest_object_observation("bot_02", [obs2])
        assert len(events2) > 0
        # Should detect movement (>0.5m threshold for Pallet)
        has_moved = any("ObjectMoved" in e.event_type for e in events2)
        assert has_moved


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
