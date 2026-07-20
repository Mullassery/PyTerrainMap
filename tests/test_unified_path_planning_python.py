"""
Pytest tests for Unified Path Planning (Traversability + Gaussian Integration)
Tests the integration of Gaussian Splatting with path planning queries
"""

import pytest
from pyterrain_map import PyGaussianSplatStore, PyUnifiedPathCost


class TestUnifiedPathCostWrapper:
    """Test PyUnifiedPathCost Python wrapper"""

    def test_create_path_cost(self):
        """Create path cost from store"""
        store = PyGaussianSplatStore()

        # Insert splats to create terrain data
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.8, "Road")

        # Query path cost
        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        assert cost is not None
        assert hasattr(cost, 'total')
        assert hasattr(cost, 'distance_cost')
        assert hasattr(cost, 'terrain_cost')
        assert hasattr(cost, 'elevation_cost')
        assert hasattr(cost, 'passage_cost')
        assert hasattr(cost, 'uncertainty_cost')

    def test_cost_components_breakdown(self):
        """Verify path cost component breakdown"""
        store = PyGaussianSplatStore()

        # Create known region (high traversability)
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.9, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.9, "Road")

        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Road should have low terrain cost
        assert cost.terrain_cost < 0.5  # Road cost ≈ 0.1-0.2
        # Total is sum of components
        assert cost.total > 0

    def test_unknown_region_higher_cost(self):
        """Unknown regions should have higher uncertainty cost"""
        store = PyGaussianSplatStore()

        # Empty store: completely unknown
        cost_empty = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Add observations to reduce uncertainty
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.8, "Road")

        cost_known = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Known region should have lower uncertainty cost
        assert cost_known.uncertainty_cost < cost_empty.uncertainty_cost

    def test_difficult_terrain_increases_cost(self):
        """Difficult terrain (mud, obstacles) should increase path cost"""
        store = PyGaussianSplatStore()

        # Road path (low cost)
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.85, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.85, "Road")

        cost_road = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Mud path (higher cost)
        store2 = PyGaussianSplatStore()
        store2.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.4, "Mud")
        store2.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.4, "Mud")

        cost_mud = store2.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Mud path should have higher terrain cost
        assert cost_mud.terrain_cost > cost_road.terrain_cost
        # Mud should have higher total cost
        assert cost_mud.total > cost_road.total

    def test_cost_repr(self):
        """Test cost object representation"""
        store = PyGaussianSplatStore()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        repr_str = repr(cost)
        assert "PathCost" in repr_str
        assert "total" in repr_str.lower()

    def test_cost_components_exist(self):
        """Verify cost has all expected component breakdowns"""
        store = PyGaussianSplatStore()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Verify all components exist
        assert hasattr(cost, 'distance_cost')
        assert hasattr(cost, 'terrain_cost')
        assert hasattr(cost, 'elevation_cost')
        assert hasattr(cost, 'passage_cost')
        assert hasattr(cost, 'uncertainty_cost')
        assert hasattr(cost, 'total')

    def test_multi_observation_consensus_lowers_uncertainty_cost(self):
        """Multiple bot observations should reduce uncertainty cost"""
        store = PyGaussianSplatStore()

        # Single observation
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.7, "Grass")
        cost_single = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Multiple bots observing same region
        store2 = PyGaussianSplatStore()
        for i in range(3):
            store2.insert_splat(40.0, -74.0, 10.0, f"bot_{i:02d}", 0.75, "Grass")

        cost_consensus = store2.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Consensus should have lower uncertainty cost
        assert cost_consensus.uncertainty_cost <= cost_single.uncertainty_cost

    def test_traversability_awareness_in_path_cost(self):
        """Path cost should factor in traversability scores from splats"""
        store = PyGaussianSplatStore()

        # Highly traversable region
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.95, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.95, "Road")

        cost_good = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Poor traversability
        store2 = PyGaussianSplatStore()
        store2.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.2, "Obstacle")
        store2.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.2, "Obstacle")

        cost_poor = store2.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.005,
            to_lon=-74.005,
            to_elev=10.0,
        )

        # Poor traversability should have higher total cost
        assert cost_poor.total > cost_good.total


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
