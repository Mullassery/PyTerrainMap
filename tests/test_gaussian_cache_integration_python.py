"""
Pytest tests for Gaussian Splatting Caching Integration
Tests efficient multi-layer caching for Gaussian world model
"""

import pytest
from pyterrain_map import PyGaussianSplatStore, PyGaussianCacheManager


class TestCacheManagerBasics:
    """Test PyGaussianCacheManager basic functionality"""

    def test_create_cache_manager(self):
        """Create cache manager"""
        manager = PyGaussianCacheManager()
        assert manager is not None

    def test_cache_manager_repr(self):
        """Test cache manager string representation"""
        manager = PyGaussianCacheManager()
        repr_str = repr(manager)
        assert "GaussianCacheManager" in repr_str

    def test_initial_stats_empty(self):
        """Initial stats should show no activity"""
        manager = PyGaussianCacheManager()
        stats = manager.stats()
        assert isinstance(stats, dict)
        assert "cache_hits" in stats
        assert "cache_misses" in stats


class TestCacheLayers:
    """Test different cache layers"""

    def test_get_summary_layer0(self):
        """Get summary (Layer 0 - fast)"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Add some observations
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        store.insert_splat(40.005, -74.005, 10.0, "bot_01", 0.8, "Road")

        summary = manager.get_summary("test_region", store)
        assert isinstance(summary, dict)
        assert "avg_traversability" in summary
        assert "avg_uncertainty" in summary
        assert "confidence" in summary

    def test_get_facts_layer1(self):
        """Get facts (Layer 1 - details)"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        facts = manager.get_facts("test_region", store)
        assert isinstance(facts, dict)
        assert "recent_splats" in facts
        assert "confidence" in facts

    def test_get_context_layer2(self):
        """Get context (Layer 2 - detailed)"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        context = manager.get_context("test_region", store)
        assert isinstance(context, dict)
        assert "coverage_pct" in context
        assert "freshness" in context
        assert "confidence" in context


class TestCacheHitsAndMisses:
    """Test cache hit/miss tracking"""

    def test_cache_miss_on_first_access(self):
        """First access should be cache miss"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        manager.get_summary("region_1", store)

        stats = manager.stats()
        assert int(stats["cache_misses"]) >= 1

    def test_cache_hit_on_second_access(self):
        """Second access within freshness window should be hit"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # First access
        manager.get_summary("region_1", store)
        first_stats = manager.stats()

        # Second access
        manager.get_summary("region_1", store)
        second_stats = manager.stats()

        # Should have at least one hit
        assert int(second_stats["cache_hits"]) > int(first_stats["cache_hits"])

    def test_different_regions_tracked_separately(self):
        """Cache should track different regions separately"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Access different regions
        manager.get_summary("region_1", store)
        manager.get_summary("region_2", store)
        manager.get_summary("region_3", store)

        # Access region_1 again (hit)
        manager.get_summary("region_1", store)

        stats = manager.stats()
        # Should have 3 misses + 1 hit
        assert int(stats["cache_misses"]) >= 3
        assert int(stats["cache_hits"]) >= 1


class TestCacheInvalidation:
    """Test cache invalidation"""

    def test_invalidate_region_clears_cache(self):
        """Invalidating region should force cache miss on next access"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # First access
        manager.get_summary("region_1", store)
        first_misses = int(manager.stats()["cache_misses"])

        # Invalidate
        manager.invalidate_region("region_1")

        # Second access should be miss
        manager.get_summary("region_1", store)
        second_misses = int(manager.stats()["cache_misses"])

        # Should have one more miss
        assert second_misses > first_misses

    def test_invalidation_counter_increments(self):
        """Invalidations should be tracked"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Populate cache
        manager.get_summary("region_1", store)

        # Invalidate
        manager.invalidate_region("region_1")

        stats = manager.stats()
        assert int(stats["invalidations"]) >= 1


class TestMultipleLayers:
    """Test accessing multiple cache layers"""

    def test_all_layers_accessible(self):
        """All three layers should be accessible"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.85, "Road")

        # Access all layers
        summary = manager.get_summary("test", store)
        facts = manager.get_facts("test", store)
        context = manager.get_context("test", store)

        assert summary is not None
        assert facts is not None
        assert context is not None

    def test_layer_consistency_across_calls(self):
        """Same layer should return consistent results"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.85, "Road")

        # Get summary twice
        summary1 = manager.get_summary("region_1", store)
        summary2 = manager.get_summary("region_1", store)

        # Should have same confidence
        assert summary1["confidence"] == summary2["confidence"]


class TestCacheWithVariousObservations:
    """Test caching with different observation patterns"""

    def test_cache_with_empty_store(self):
        """Cache should work with empty store"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Access with empty store
        summary = manager.get_summary("empty_region", store)
        assert summary is not None

    def test_cache_with_dense_observations(self):
        """Cache should handle dense observations"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Add many observations
        for i in range(20):
            store.insert_splat(
                40.0 + i * 0.001,
                -74.0,
                10.0,
                f"bot_{i:02d}",
                0.8 + i * 0.01,
                "Road",
            )

        summary = manager.get_summary("dense_region", store)
        assert summary is not None
        assert "confidence" in summary

    def test_cache_tracks_splat_count(self):
        """Cache statistics should track cached splats"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        # Add observations
        for i in range(5):
            store.insert_splat(40.0, -74.0, 10.0, f"bot_{i:02d}", 0.8, "Road")

        manager.get_summary("region", store)
        stats = manager.stats()

        assert "splats_cached" in stats


class TestCacheStats:
    """Test cache statistics"""

    def test_stats_has_required_fields(self):
        """Stats should have all required fields"""
        manager = PyGaussianCacheManager()
        stats = manager.stats()

        required_fields = ["cache_hits", "cache_misses", "invalidations"]
        for field in required_fields:
            assert field in stats

    def test_stats_update_on_operations(self):
        """Stats should update as operations occur"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        initial_stats = manager.stats()
        initial_misses = int(initial_stats["cache_misses"])

        # Perform cache miss
        manager.get_summary("region", store)

        final_stats = manager.stats()
        final_misses = int(final_stats["cache_misses"])

        assert final_misses > initial_misses


class TestCacheEfficiency:
    """Test cache efficiency and performance characteristics"""

    def test_same_region_hits_increase(self):
        """Repeated access to same region should increase hits"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Access same region multiple times
        for _ in range(5):
            manager.get_summary("same_region", store)

        stats = manager.stats()
        # Should have hits from repeat accesses
        assert int(stats["cache_hits"]) > 0

    def test_large_number_of_regions(self):
        """Cache should handle many regions"""
        manager = PyGaussianCacheManager()
        store = PyGaussianSplatStore()

        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Access many regions
        for i in range(50):
            manager.get_summary(f"region_{i}", store)

        stats = manager.stats()
        assert int(stats["cache_misses"]) >= 50


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
