# PyTerrainMap Gaussian Splatting Guide

## Overview

Gaussian Splatting is a probabilistic world model that enables multi-bot fleets to collaboratively build and understand shared terrain, obstacles, and environmental changes. Unlike discrete occupancy grids, Gaussian Splats provide continuous, uncertainty-aware representations that naturally support multi-scale queries, temporal decay, and fleet learning.

**Core Philosophy:** PyTerrainMap is the shared map, not the robot brain. Robots feed observations in, query current world state out. The map handles fusion, decay, prediction, and change detection transparently.

---

## Quick Start

### Basic Usage

```python
from pyterrain_map import (
    PyGaussianSplatStore,
    PyFleetCoordinator,
    PyBotObservationMessage,
    PyGaussianFrontierScorer,
    PyFrontier,
)

# 1. Create shared world model
store = PyGaussianSplatStore()

# 2. Register fleet
coordinator = PyFleetCoordinator(store)
coordinator.register_bot("bot_01")
coordinator.register_bot("bot_02")

# 3. Bot 01 observes terrain
observation = PyBotObservationMessage(
    bot_id="bot_01",
    lat=40.0,
    lon=-74.0,
    elev=10.0,
    traversability=0.85,
    confidence=0.90,
    terrain_type="Road",
)

# 4. Broadcast to fleet (all bots now know)
coordinator.broadcast_observation(observation)

# 5. Bot 02 queries without observing directly
uncertainty = store.uncertainty_at(40.0, -74.0, 10.0)
print(f"Uncertainty at location: {uncertainty:.2f}")  # 0.15 (low)

# 6. Identify exploration frontiers
scorer = PyGaussianFrontierScorer()
frontier = PyFrontier("unknown_area", 40.1, -74.1, 10.0)
scorer.score_frontier(frontier, store)
print(f"Frontier priority: {frontier.priority:.2f}")
```

---

## Core Concepts

### Gaussian Splats

A Gaussian Splat represents a probabilistic observation of terrain at a specific location:

```
Position: (lat, lon, elevation)
Covariance: 3×3 uncertainty matrix
Traversability: 0.0 (impassable) to 1.0 (perfect)
Terrain Type: Road, Grass, Mud, Water, Obstacle, etc.
Confidence: 0.0 (unknown) to 1.0 (known)
Temporal: Creation time, last update, decay schedule
```

**Key insight:** Multiple observations of the same location are fused via Bayesian inference, reducing uncertainty and improving confidence.

### Multi-Resolution Queries

The store supports queries at three resolution levels:

- **Fine (10m)**: Detailed local queries for obstacle avoidance
- **Medium (1km)**: Regional planning and frontier detection
- **Coarse (86km)**: Global awareness and multi-region reasoning

```python
# Query all details near position
nearby = store.query_radius(40.0, -74.0, 10.0, radius_m=1000.0)

# Get uncertainty score (0=known, 1=unknown)
uncertainty = store.uncertainty_at(40.0, -74.0, 10.0)

# Compute path cost (5 components)
cost = store.path_cost(
    from_lat=40.0, from_lon=-74.0, from_elev=10.0,
    to_lat=40.1, to_lon=-74.1, to_elev=10.0,
)
print(f"Total cost: {cost.total}, Uncertainty penalty: {cost.uncertainty_cost}")
```

### Temporal Decay

Observations lose confidence over time. Different object types decay at different rates:

- **Terrain** (45-day half-life): Walls, floors, terrain features
- **Movable objects** (8-hour half-life): Boxes, pallets (likely moved)
- **Mobile objects** (2-hour half-life): Carts, forklifts (moving frequently)
- **Dynamic objects** (30-minute half-life): People, robots (rapidly changing)

The store automatically applies decay when queried:

```python
# Apply decay to all observations
import time
now_us = int(time.time() * 1_000_000)
store.apply_temporal_decay(now_us)

# Query at different times sees different confidence
future_us = now_us + (24 * 60 * 60 * 1_000_000)  # 24 hours later
# Confidence will be lower due to decay
```

### Fleet Learning: One Learns, All Know

The core principle enables knowledge sharing:

```
Bot_01 observes obstacle at (40.0, -74.0)
    ↓
Broadcast to coordinator
    ↓
Store updated with observation
    ↓
Bot_02 queries (40.0, -74.0) WITHOUT visiting
    ↓
Sees obstacle in response
```

This is transparent and automatic:

```python
# Bot 01 observes
msg = PyBotObservationMessage(
    bot_id="bot_01",
    lat=40.0, lon=-74.0, elev=0.0,
    traversability=0.0,  # Impassable
    confidence=0.95,
    terrain_type="Obstacle",
)
coordinator.broadcast_observation(msg)

# Bot 02 immediately knows
uncertainty = store.uncertainty_at(40.0, -74.0, 0.0)
# uncertainty is LOW (well-known obstacle)
```

---

## API Reference

### GaussianSplatStore

The central world model.

#### Insertion

```python
# Insert a single terrain splat
splat_id = store.insert_splat(
    lat=40.0,
    lon=-74.0,
    elev=10.0,
    bot_id="bot_01",
    traversability=0.85,
    terrain_type="Road",
)
```

#### Queries

```python
# Radius query: all splats within N meters
results = store.query_radius(lat, lon, elev, radius_m=1000.0)

# Uncertainty at point (0.0=known, 1.0=unknown)
unc = store.uncertainty_at(lat, lon, elev)

# 5-component path cost
cost = store.path_cost(
    from_lat, from_lon, from_elev,
    to_lat, to_lon, to_elev,
)
# cost.distance_cost: physical distance penalty
# cost.terrain_cost: terrain difficulty
# cost.elevation_cost: height change penalty
# cost.passage_cost: door/gate traversal cost
# cost.uncertainty_cost: penalty for unknown regions
# cost.total: weighted sum
```

#### Object Tracking

```python
# Ingest dynamic object observations
from pyterrain_map import PyObjectObservation
obs = PyObjectObservation("Pallet", lat, lon, elev, timestamp_us, confidence)
events = store.ingest_object_observation("bot_01", [obs])

# Query objects near position
objects = store.objects_near(lat, lon, elev, radius_m=1000.0)
```

#### Maintenance

```python
# Apply temporal decay
store.apply_temporal_decay(current_time_us)

# Get statistics
stats = store.stats()
print(f"Total splats: {stats['total_splats']}")
```

### FleetCoordinator

Multi-bot synchronization and coordination.

```python
coordinator = PyFleetCoordinator(store)

# Register bots
coordinator.register_bot("bot_01")

# Broadcast observation
msg = PyBotObservationMessage(...)
broadcast_count = coordinator.broadcast_observation(msg)

# Fleet state
state = coordinator.fleet_state()
# state['active_bots']: number of active robots
# state['total_fused']: observations processed
# state['conflicts_resolved']: disagreements handled

# Bot status
status = coordinator.get_bot_status("bot_01")
# status['is_active']: currently connected
# status['observations_contributed']: count

# Fleet health (0.0-1.0)
health = coordinator.fleet_health()  # 1.0 = all bots active
```

### GaussianFrontierScorer

Prioritize exploration targets using uncertainty.

```python
scorer = PyGaussianFrontierScorer()

# Score single frontier
frontier = PyFrontier("target_1", lat, lon, elev)
scorer.score_frontier(frontier, store)
print(f"Priority: {frontier.priority}")  # 0.0-1.0

# Score and rank multiple frontiers
frontiers = [PyFrontier(...), PyFrontier(...), ...]
ranked = scorer.score_frontiers(frontiers, store)
# Returns frontiers sorted by priority (highest first)
```

### GaussianCacheManager

Multi-layer caching for fast repeated queries.

```python
cache = PyGaussianCacheManager()

# Layer 0 (Summary): terrain distribution, avg traversability
summary = cache.get_summary("warehouse_floor", store)

# Layer 1 (Facts): anomalies, high-uncertainty areas
facts = cache.get_facts("warehouse_floor", store)

# Layer 2 (Context): detailed uncertainty samples
context = cache.get_context("warehouse_floor", store)

# Invalidate on new observations
cache.invalidate_region("warehouse_floor")

# Cache statistics
stats = cache.stats()
print(f"Cache hits: {stats['cache_hits']}")
```

---

## Use Cases

### Warehouse Delivery Coordination

```python
# 3 delivery bots coordinating on floor plan
for i in range(3):
    coordinator.register_bot(f"delivery_bot_{i:02d}")

# Bots observe aisles and shelves
for aisle_id in range(5):
    for shelf_id in range(10):
        obs = PyBotObservationMessage(
            bot_id=f"delivery_bot_{i:02d}",
            lat=40.0 + aisle_id * 0.01,
            lon=-74.0 + shelf_id * 0.001,
            elev=1.5,
            traversability=0.95,
            confidence=0.90,
            terrain_type="Shelf Row",
        )
        coordinator.broadcast_observation(obs)

# Each bot can plan routes avoiding obstacles it never saw
cost = store.path_cost(from_pos, to_pos)
```

### Surveillance Coverage Mapping

```python
# Drones building visibility maps
for drone in ["drone_01", "drone_02", "drone_03"]:
    coordinator.register_bot(drone)

# High-elevation observations improve visibility inference
obs = PyBotObservationMessage(
    bot_id="drone_01",
    lat=40.0, lon=-74.0, elev=50.0,  # 50m altitude
    traversability=0.8,
    confidence=0.85,
    terrain_type="Coverage Map",
)
coordinator.broadcast_observation(obs)

# Later drones know which areas are under surveillance
uncertainty = store.uncertainty_at(40.0, -74.0, 0.0)
```

### Agriculture Field Monitoring

```python
# Rovers monitoring soil conditions
coordinator.register_bot("ag_rover_01")
coordinator.register_bot("ag_rover_02")

# Observations include environmental context
obs = PyBotObservationMessage(
    bot_id="ag_rover_01",
    lat=40.0, lon=-74.0, elev=0.0,
    traversability=0.65,  # Soft soil
    confidence=0.88,
    terrain_type="Soil: Moist",
)
coordinator.broadcast_observation(obs)

# Identify areas needing irrigation
frontier_scorer = PyGaussianFrontierScorer()
# High-uncertainty areas = undermonitored fields
```

### Disaster Response Coordination

```python
# Emergency response team mapping hazards
coordinator.register_bot("rescue_bot_01")
coordinator.register_bot("rescue_bot_02")
coordinator.register_bot("rescue_bot_03")

# Map safe passages through debris
obs = PyBotObservationMessage(
    bot_id="rescue_bot_01",
    lat=40.0, lon=-74.0, elev=0.0,
    traversability=0.0,  # Debris field
    confidence=0.95,
    terrain_type="Debris Field",
)
coordinator.broadcast_observation(obs)

# Other bots route around without visiting
cost = store.path_cost(from_pos, to_pos)
# High uncertainty_cost for unmapped areas
```

---

## Best Practices

### 1. **Update Observations Regularly**
```python
# Bad: Single observation per session
store.insert_splat(40.0, -74.0, 10.0, "bot_01", 0.8, "Road")

# Good: Multiple observations for consensus
for i in range(5):
    obs = PyBotObservationMessage(...)
    coordinator.ingest_observation(obs)
# Confidence will be higher with consensus
```

### 2. **Use Appropriate Confidence Values**
```python
# Direct observation: high confidence
obs_direct = PyBotObservationMessage(..., confidence=0.95)

# Inference/prediction: lower confidence
obs_predicted = PyBotObservationMessage(..., confidence=0.60)
```

### 3. **Leverage Fleet Learning**
```python
# Don't query only locally—broadcast to fleet
msg = PyBotObservationMessage(...)
coordinator.broadcast_observation(msg)  # All bots benefit

# Not: coordinator.ingest_observation(msg)  # Only this bot knows
```

### 4. **Cache Frequently-Queried Regions**
```python
cache = PyGaussianCacheManager()

# First query: cache miss (computes from scratch)
summary = cache.get_summary("warehouse_floor", store)

# Repeated queries: cache hit (180x faster)
summary = cache.get_summary("warehouse_floor", store)

# Invalidate on new observations
coordinator.broadcast_observation(obs)
cache.invalidate_region("warehouse_floor")
```

### 5. **Monitor Fleet Health**
```python
# Track active bots
health = coordinator.fleet_health()
if health < 0.8:
    print("Low fleet health—some bots offline")

# Check bot status
status = coordinator.get_bot_status("bot_01")
if not status['is_active']:
    coordinator.register_bot("bot_01")  # Re-register
```

---

## Performance Characteristics

| Operation | Latency | Throughput | Notes |
|-----------|---------|-----------|-------|
| Insert splat | 0.15ms | ~900k/sec | Linear complexity |
| Radius query (100 splats) | 0.01ms | — | Sub-millisecond |
| Uncertainty query | 0.001ms | — | Constant time |
| Path cost | 0.48ms | — | Traverses path |
| Observation broadcast | 0.002ms | ~800k/sec | Multi-bot |
| Frontier scoring | 0.001ms | ~1M/sec | Very fast |
| Cache hit | 0.001ms | 180x speedup | vs miss |

**Scaling:** Linear performance from 100 to 10,000+ splats. H3 spatial indexing can be enabled for O(log n) queries at scale.

---

## Troubleshooting

### High Uncertainty After Observations
**Problem:** `store.uncertainty_at()` still returns 0.8+ after inserting splats.

**Solution:** Ensure observations are near the query point (within Gaussian covariance, ~10-100m). Multiple observations converge faster.

### Bots Not Sharing Knowledge
**Problem:** Bot_02 doesn't see Bot_01's observations.

**Solution:** Use `coordinator.broadcast_observation()` instead of just `coordinator.ingest_observation()`. Broadcast pushes to all bots; ingest is local-only.

### Stale Observations
**Problem:** Old observations still affect path planning.

**Solution:** Call `store.apply_temporal_decay(current_time_us)` periodically. Older observations have lower confidence.

### Memory Growth
**Problem:** Store size grows unboundedly.

**Solution:** Enable memory pooling (`MemoryPoolManager`) or implement periodic pruning of low-confidence splats.

---

## Advanced Topics

### Custom Terrain Types
```python
# Define domain-specific terrain
terrain_types = [
    "Road", "Grass", "Mud", "Water",
    "Snow", "Sand", "Concrete", "Gravel",
    "Custom: Mining_Pit", "Custom: Factory_Floor",
]

obs = PyBotObservationMessage(..., terrain_type="Custom: Mining_Pit")
```

### Hierarchical Exploration
```python
# Use cache layers for multi-scale planning
cache = PyGaussianCacheManager()

# Coarse level: find regions to explore
summary = cache.get_summary("region", store)

# Medium level: identify zones within region
facts = cache.get_facts("zone", store)

# Fine level: detailed path planning
context = cache.get_context("aisle", store)
```

### Memory Pooling for High Throughput
```python
from pyterrain_map import MemoryPoolManager, PoolConfig

config = PoolConfig(
    splat_pool_size=1000,
    splat_pool_max=10000,
    observation_pool_size=5000,
)
pool_mgr = MemoryPoolManager(config)

stats = pool_mgr.stats()
print(f"Cache hit rate: {stats.combined_hit_rate():.1%}")
```

---

## Integration with Other PyTerrainMap Modules

- **Traversability Module**: Path planning uses both graph edges and Gaussian uncertainty
- **Frontier Detection**: Uses uncertainty scores to prioritize exploration targets
- **Caching**: Multi-layer cache accelerates repeated regional queries
- **Temporal Processing**: Observation timestamps drive decay calculations

---

## See Also

- **API Reference**: `docs/API.md`
- **Architecture**: `docs/ARCHITECTURE.md`
- **Benchmarks**: `tests/test_gaussian_benchmarks.py`
- **Examples**: `examples/warehouse_coordination.py`

---

*Generated for PyTerrainMap v1.2.0*
