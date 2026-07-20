"""
Performance benchmarks for Gaussian Splatting operations
Measures latency, throughput, and scalability
"""

import time
import pytest
from pyterrain_map import (
    PyGaussianSplatStore,
    PyFleetCoordinator,
    PyBotObservationMessage,
    PyGaussianFrontierScorer,
    PyFrontier,
    PyGaussianCacheManager,
)


class TestInsertBenchmarks:
    """Benchmark splat insertion performance"""

    def test_single_insert_latency(self):
        """Measure latency of single splat insertion"""
        store = PyGaussianSplatStore()

        start = time.perf_counter()
        store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")
        elapsed_ms = (time.perf_counter() - start) * 1000

        # Should be very fast (< 1ms)
        assert elapsed_ms < 10.0
        print(f"Single insert: {elapsed_ms:.3f}ms")

    def test_batch_insert_throughput(self):
        """Measure throughput of batch insertions"""
        store = PyGaussianSplatStore()

        start = time.perf_counter()
        for i in range(100):
            store.insert_splat(
                40.0 + i * 0.001,
                -74.0,
                10.0,
                f"bot_{i % 5:02d}",
                0.8,
                "Road",
            )
        elapsed_ms = (time.perf_counter() - start) * 1000
        throughput = 100 / (elapsed_ms / 1000)

        print(f"100 inserts: {elapsed_ms:.1f}ms ({throughput:.0f} inserts/sec)")
        assert throughput > 100  # > 100 inserts per second


class TestQueryBenchmarks:
    """Benchmark query performance"""

    def test_radius_query_small(self):
        """Measure latency of small radius query"""
        store = PyGaussianSplatStore()

        # Insert 100 splats
        for i in range(100):
            store.insert_splat(40.0 + i * 0.001, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Query
        start = time.perf_counter()
        results = store.query_radius(40.05, -74.0, 10.0, 5000.0)
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(f"Radius query (100 splats): {elapsed_ms:.3f}ms, found {len(results)}")
        assert elapsed_ms < 50.0  # Should be fast even with 100 splats

    def test_uncertainty_query(self):
        """Measure uncertainty_at() latency"""
        store = PyGaussianSplatStore()

        # Insert 50 splats in region
        for i in range(50):
            store.insert_splat(40.0 + i * 0.001, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Query
        start = time.perf_counter()
        uncertainty = store.uncertainty_at(40.025, -74.0, 10.0)
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(f"Uncertainty query: {elapsed_ms:.3f}ms, uncertainty={uncertainty:.3f}")
        assert elapsed_ms < 10.0


class TestPathCostBenchmarks:
    """Benchmark path cost calculation"""

    def test_path_cost_latency(self):
        """Measure latency of path_cost calculation"""
        store = PyGaussianSplatStore()

        # Build dense region
        for i in range(50):
            for j in range(5):
                store.insert_splat(
                    40.0 + i * 0.001,
                    -74.0 + j * 0.001,
                    10.0,
                    f"bot_{(i + j) % 3:02d}",
                    0.8,
                    "Road",
                )

        # Query path cost
        start = time.perf_counter()
        cost = store.path_cost(
            from_lat=40.0,
            from_lon=-74.0,
            from_elev=10.0,
            to_lat=40.05,
            to_lon=-74.01,
            to_elev=10.0,
        )
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(
            f"Path cost (250 splats): {elapsed_ms:.3f}ms, total_cost={cost.total:.2f}"
        )
        assert elapsed_ms < 100.0


class TestFleetCoordinationBenchmarks:
    """Benchmark multi-bot fleet coordination"""

    def test_single_observation_ingestion(self):
        """Measure latency of observation ingestion"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        coordinator.register_bot("bot_01")

        msg = PyBotObservationMessage(
            bot_id="bot_01",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            traversability=0.85,
            confidence=0.9,
            terrain_type="Road",
        )

        start = time.perf_counter()
        coordinator.ingest_observation(msg)
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(f"Observation ingestion: {elapsed_ms:.3f}ms")
        assert elapsed_ms < 10.0

    def test_broadcast_to_fleet(self):
        """Measure broadcast latency to multi-bot fleet"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register 10-bot fleet
        for i in range(10):
            coordinator.register_bot(f"bot_{i:02d}")

        msg = PyBotObservationMessage(
            bot_id="bot_00",
            lat=40.0,
            lon=-74.0,
            elev=10.0,
            traversability=0.85,
            confidence=0.9,
            terrain_type="Road",
        )

        start = time.perf_counter()
        coordinator.broadcast_observation(msg)
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(f"Broadcast to 10 bots: {elapsed_ms:.3f}ms")
        assert elapsed_ms < 20.0

    def test_fleet_state_query(self):
        """Measure fleet_state() latency"""
        store = PyGaussianSplatStore()
        coordinator = PyFleetCoordinator(store)

        # Register and have 20 bots send observations
        for i in range(20):
            coordinator.register_bot(f"bot_{i:02d}")

        # Send 100 observations from fleet
        for i in range(100):
            msg = PyBotObservationMessage(
                bot_id=f"bot_{i % 20:02d}",
                lat=40.0 + (i % 10) * 0.001,
                lon=-74.0,
                elev=10.0,
                traversability=0.8,
                confidence=0.85,
                terrain_type="Road",
            )
            coordinator.ingest_observation(msg)

        # Query state
        start = time.perf_counter()
        state = coordinator.fleet_state()
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(f"Fleet state query (20 bots, 100 obs): {elapsed_ms:.3f}ms")
        assert elapsed_ms < 10.0


class TestFrontierScoringBenchmarks:
    """Benchmark frontier detection and scoring"""

    def test_single_frontier_score(self):
        """Measure frontier scoring latency"""
        store = PyGaussianSplatStore()
        scorer = PyGaussianFrontierScorer()

        # Populate store
        for i in range(100):
            store.insert_splat(40.0 + i * 0.001, -74.0, 10.0, "bot_01", 0.8, "Road")

        frontier = PyFrontier("test_frontier", 40.05, -74.0, 10.0)

        start = time.perf_counter()
        scorer.score_frontier(frontier, store)
        elapsed_ms = (time.perf_counter() - start) * 1000

        print(
            f"Single frontier score (100 splats): {elapsed_ms:.3f}ms, priority={frontier.priority:.2f}"
        )
        assert elapsed_ms < 50.0

    def test_batch_frontier_scoring(self):
        """Measure throughput of batch frontier scoring"""
        store = PyGaussianSplatStore()
        scorer = PyGaussianFrontierScorer()

        # Populate store
        for i in range(100):
            store.insert_splat(40.0 + i * 0.001, -74.0, 10.0, "bot_01", 0.8, "Road")

        # Create 20 frontiers
        frontiers = [
            PyFrontier(f"frontier_{i}", 40.0 + i * 0.01, -74.0, 10.0) for i in range(20)
        ]

        start = time.perf_counter()
        scored = scorer.score_frontiers(frontiers, store)
        elapsed_ms = (time.perf_counter() - start) * 1000
        throughput = 20 / (elapsed_ms / 1000)

        print(
            f"20 frontier scores (100 splats): {elapsed_ms:.1f}ms ({throughput:.0f} frontiers/sec)"
        )
        assert throughput > 50  # > 50 frontiers per second


class TestCachingBenchmarks:
    """Benchmark caching performance"""

    def test_cache_hit_speedup(self):
        """Measure cache hit speedup vs miss"""
        store = PyGaussianSplatStore()
        cache = PyGaussianCacheManager()

        # Populate store
        for i in range(100):
            store.insert_splat(40.0 + i * 0.001, -74.0, 10.0, f"bot_{i % 5:02d}", 0.8, "Road")

        # First access (miss)
        start = time.perf_counter()
        cache.get_summary("region_1", store)
        miss_ms = (time.perf_counter() - start) * 1000

        # Second access (hit)
        start = time.perf_counter()
        cache.get_summary("region_1", store)
        hit_ms = (time.perf_counter() - start) * 1000

        speedup = miss_ms / hit_ms if hit_ms > 0 else float('inf')
        print(f"Cache miss: {miss_ms:.3f}ms, cache hit: {hit_ms:.3f}ms, speedup: {speedup:.1f}x")
        assert hit_ms <= miss_ms  # Hit should be at least as fast


class TestScalabilityBenchmarks:
    """Benchmark scalability with increasing data"""

    def test_scalability_inserts(self):
        """Measure insert performance as data grows"""
        store = PyGaussianSplatStore()

        sizes = [100, 500, 1000]
        for size in sizes:
            start = time.perf_counter()
            for i in range(size):
                store.insert_splat(
                    40.0 + (i % 100) * 0.001,
                    -74.0 + (i // 100) * 0.001,
                    10.0,
                    f"bot_{i % 10:02d}",
                    0.8,
                    "Road",
                )
            elapsed_ms = (time.perf_counter() - start) * 1000
            throughput = size / (elapsed_ms / 1000)
            print(f"Insert {size} splats: {elapsed_ms:.1f}ms ({throughput:.0f} inserts/sec)")

    def test_scalability_queries(self):
        """Measure query performance as data grows"""
        sizes = [100, 500, 1000]

        for size in sizes:
            store = PyGaussianSplatStore()

            # Insert splats
            for i in range(size):
                store.insert_splat(
                    40.0 + (i % 100) * 0.001,
                    -74.0 + (i // 100) * 0.001,
                    10.0,
                    "bot_01",
                    0.8,
                    "Road",
                )

            # Time query
            start = time.perf_counter()
            results = store.query_radius(40.05, -74.005, 10.0, 10000.0)
            elapsed_ms = (time.perf_counter() - start) * 1000

            print(
                f"Query with {size} splats: {elapsed_ms:.3f}ms, found {len(results)} results"
            )

    def test_scalability_fleet(self):
        """Measure fleet coordination as bots increase"""
        bot_counts = [5, 10, 20]

        for bot_count in bot_counts:
            store = PyGaussianSplatStore()
            coordinator = PyFleetCoordinator(store)

            # Register bots
            for i in range(bot_count):
                coordinator.register_bot(f"bot_{i:02d}")

            # Each bot sends observations
            start = time.perf_counter()
            for bot_id in range(bot_count):
                for obs_id in range(10):
                    msg = PyBotObservationMessage(
                        bot_id=f"bot_{bot_id:02d}",
                        lat=40.0 + obs_id * 0.001,
                        lon=-74.0,
                        elev=10.0,
                        traversability=0.8,
                        confidence=0.85,
                        terrain_type="Road",
                    )
                    coordinator.ingest_observation(msg)

            elapsed_ms = (time.perf_counter() - start) * 1000
            obs_per_sec = (bot_count * 10) / (elapsed_ms / 1000)

            print(
                f"Fleet of {bot_count} bots, 100 observations: {elapsed_ms:.1f}ms ({obs_per_sec:.0f} obs/sec)"
            )


class TestMemoryBenchmarks:
    """Benchmark memory usage"""

    def test_store_memory_per_splat(self):
        """Estimate memory per stored splat"""
        import sys

        store1 = PyGaussianSplatStore()
        size1 = sys.getsizeof(store1)

        # Add 1000 splats
        for i in range(1000):
            store1.insert_splat(
                40.0 + (i % 100) * 0.001,
                -74.0 + (i // 100) * 0.001,
                10.0,
                f"bot_{i % 10:02d}",
                0.8,
                "Road",
            )

        size2 = sys.getsizeof(store1)
        per_splat = (size2 - size1) / 1000

        print(f"Estimated memory per splat: {per_splat:.0f} bytes")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "-s"])
