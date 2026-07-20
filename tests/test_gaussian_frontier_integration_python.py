"""
Pytest tests for Gaussian Splatting + Frontier Detection Integration
Tests intelligent frontier prioritization using Gaussian uncertainty
"""

import pytest
from pyterrain_map import (
    PyGaussianSplatStore,
    PyFrontier,
    PyGaussianFrontierScorer,
)


class TestFrontierCreation:
    """Test PyFrontier wrapper creation"""

    def test_create_frontier(self):
        """Create a frontier"""
        frontier = PyFrontier("test_frontier", 40.0, -74.0, 10.0)
        assert frontier.id == "test_frontier"
        assert abs(frontier.location_lat - 40.0) < 0.0001
        assert abs(frontier.location_lon - -74.0) < 0.0001

    def test_frontier_default_scores(self):
        """Frontier has default scores"""
        frontier = PyFrontier("test", 40.0, -74.0, 10.0)
        assert frontier.priority == 0.5
        assert frontier.confidence == 0.5
        assert frontier.expected_information_gain == 0.5

    def test_frontier_repr(self):
        """Test frontier string representation"""
        frontier = PyFrontier("f1", 40.0, -74.0, 10.0)
        repr_str = repr(frontier)
        assert "Frontier" in repr_str
        assert "f1" in repr_str

    def test_frontier_to_dict(self):
        """Convert frontier to dictionary"""
        frontier = PyFrontier("f1", 40.0, -74.0, 10.0)
        d = frontier.to_dict()
        assert isinstance(d, dict)
        assert "id" in d
        assert "priority" in d


class TestGaussianFrontierScorer:
    """Test PyGaussianFrontierScorer"""

    def test_create_scorer(self):
        """Create Gaussian frontier scorer"""
        scorer = PyGaussianFrontierScorer()
        assert scorer is not None

    def test_score_frontier_unknown_region(self):
        """Score frontier in unknown region (high uncertainty)"""
        scorer = PyGaussianFrontierScorer()
        frontier = PyFrontier("unknown_area", 40.0, -74.0, 10.0)

        store = PyGaussianSplatStore()
        # Empty store = unknown everywhere

        scorer.score_frontier(frontier, store)

        # Unknown region = high information potential (uncertainty = 1.0)
        assert frontier.expected_information_gain >= 0.9
        # Unknown = higher risk (but not extremely high)
        assert frontier.risk_estimate > 0.2
        # Low confidence in unknown areas
        assert frontier.confidence < 0.1

    def test_score_frontier_known_region(self):
        """Score frontier in well-known region (low uncertainty)"""
        scorer = PyGaussianFrontierScorer()
        frontier = PyFrontier("known_area", 40.0, -74.0, 10.0)

        store = PyGaussianSplatStore()
        # Add observations to reduce uncertainty
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.85, "Road")
        store.insert_splat(40.001, -74.001, 10.0, "bot_01", 0.85, "Road")
        store.insert_splat(40.002, -74.002, 10.0, "bot_01", 0.85, "Road")

        scorer.score_frontier(frontier, store)

        # Known region = lower information potential (low uncertainty)
        assert frontier.expected_information_gain < 0.5
        # Known = lower risk
        assert frontier.risk_estimate < 0.2
        # Higher confidence in known areas
        assert frontier.confidence > 0.6

    def test_score_multiple_frontiers_ranking(self):
        """Score multiple frontiers and verify ranking by priority"""
        scorer = PyGaussianFrontierScorer()

        # Create frontiers at different locations
        frontiers = [
            PyFrontier("unknown_frontier", 40.0, -74.0, 10.0),
            PyFrontier("known_frontier", 40.01, -74.01, 10.0),
        ]

        store = PyGaussianSplatStore()
        # Known area: add many observations around known_frontier
        for i in range(10):
            store.insert_splat(
                40.01 + i * 0.001,
                -74.01,
                10.0,
                f"bot_{i:02d}",
                0.85,
                "Road",
            )

        # Score all frontiers
        scored_frontiers = scorer.score_frontiers(frontiers, store)

        # Verify ranking by checking priorities are descending
        assert len(scored_frontiers) == 2
        # Unknown frontier should have highest priority (most to explore)
        assert scored_frontiers[0].id == "unknown_frontier"
        # Known frontier should have lower priority
        assert scored_frontiers[1].id == "known_frontier"
        assert scored_frontiers[0].priority > scored_frontiers[1].priority

    def test_frontier_uncertainty_drives_priority(self):
        """Verify that high uncertainty directly drives frontier priority"""
        scorer = PyGaussianFrontierScorer()

        # Two identical frontiers, different uncertainty regions
        frontier_unknown = PyFrontier("unknown", 40.0, -74.0, 10.0)
        frontier_known = PyFrontier("known", 40.1, -74.1, 10.0)

        store = PyGaussianSplatStore()
        # Make the "known" region well-observed
        for i in range(10):
            store.insert_splat(
                40.1 + i * 0.0001,
                -74.1,
                10.0,
                f"bot_{i:02d}",
                0.95,
                "Road",
            )

        # Score both
        scorer.score_frontier(frontier_unknown, store)
        scorer.score_frontier(frontier_known, store)

        # Unknown should have higher information gain
        assert frontier_unknown.expected_information_gain > frontier_known.expected_information_gain
        # Unknown should have higher priority (more to explore)
        assert frontier_unknown.priority > frontier_known.priority

    def test_scored_frontiers_descending_priority(self):
        """Verify scored frontiers are returned in descending priority order"""
        scorer = PyGaussianFrontierScorer()

        # Create 5 frontiers
        frontiers = [
            PyFrontier(f"frontier_{i}", 40.0 + i * 0.01, -74.0, 10.0)
            for i in range(5)
        ]

        store = PyGaussianSplatStore()

        # Score progressively more at each location
        # frontier_0 = least observed (highest priority)
        # frontier_4 = most observed (lowest priority)
        for j in range(4, -1, -1):  # 4, 3, 2, 1, 0
            for i in range(j + 1):
                store.insert_splat(
                    40.0 + j * 0.01 + i * 0.0001,
                    -74.0,
                    10.0,
                    f"bot_{i:02d}",
                    0.80,
                    "Road",
                )

        scored = scorer.score_frontiers(frontiers, store)

        # Verify descending priority order
        for i in range(len(scored) - 1):
            assert scored[i].priority >= scored[i + 1].priority

    def test_frontier_confidence_inverse_of_uncertainty(self):
        """Verify that frontier confidence is inverse of uncertainty"""
        scorer = PyGaussianFrontierScorer()
        frontier = PyFrontier("test", 40.0, -74.0, 10.0)

        store = PyGaussianSplatStore()
        # Add observations
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.9, "Road")

        scorer.score_frontier(frontier, store)

        # Confidence should be high when region is observed
        assert frontier.confidence > 0.5
        # Information gain should be low when region is known
        assert frontier.expected_information_gain < 0.5

    def test_multiple_bots_reduce_frontier_priority(self):
        """Many bots observing region → lower frontier priority (less to explore)"""
        scorer = PyGaussianFrontierScorer()
        frontier = PyFrontier("test_area", 40.0, -74.0, 10.0)

        store = PyGaussianSplatStore()

        # One bot observes
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.85, "Road")
        scorer.score_frontier(frontier, store)
        priority_1bot = frontier.priority

        # Multiple bots observe same area
        store2 = PyGaussianSplatStore()
        for i in range(5):
            store2.insert_splat(40.0, -74.0, 10.0, f"bot_{i:02d}", 0.85, "Road")
        scorer.score_frontier(frontier, store2)
        priority_5bots = frontier.priority

        # More observations = lower uncertainty = lower priority
        assert priority_5bots <= priority_1bot

    def test_frontier_scoring_consistency(self):
        """Scoring same frontier twice gives consistent results"""
        scorer = PyGaussianFrontierScorer()
        frontier = PyFrontier("consistent_test", 40.0, -74.0, 10.0)

        store = PyGaussianSplatStore()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Score twice
        scorer.score_frontier(frontier, store)
        first_priority = frontier.priority
        first_confidence = frontier.confidence

        scorer.score_frontier(frontier, store)
        second_priority = frontier.priority
        second_confidence = frontier.confidence

        # Should be identical
        assert abs(first_priority - second_priority) < 0.0001
        assert abs(first_confidence - second_confidence) < 0.0001


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
