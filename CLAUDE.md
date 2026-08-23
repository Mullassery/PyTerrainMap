# PyTerrainMap: 3-Layer Spatial-Temporal Intelligence Engine

**Status:** v1.5.0 Released | **Architecture:** Rust Core + Python Bindings (PyO3)  
**Distribution:** Wheels-only via PyPI | **Tests:** 780+ Rust, 200+ Python | **License:** Proprietary

---

## Product Vision

**PyTerrainMap** is a three-layer collaborative spatial intelligence system that serves as the ground-truth spatial context layer for autonomous robotics fleets. It combines real-time 3D reconstruction (SLAM), traversability analysis, and temporal normalization into a unified knowledge graph.

### Why It Exists

Robots need to understand not just "what is the map right now" but:
- **Where can we traverse?** (Traversability + dynamic obstacles)
- **What changed since last mission?** (Temporal evolution)
- **How do sensor observations relate to space + time?** (Spatial-temporal fusion)
- **Can multiple robots share terrain knowledge?** (Fleet learning)

PyTerrainMap answers all four—synchronously.

### Core Differentiators

1. **Temporal Normalization** (5D + clock + quality): Time is a first-class coordinate alongside X, Y, Z, and quality score
2. **Multi-GPU Scheduling** (3 planes): Spatial, temporal, and sensor fusion can run in parallel
3. **Pluggable Storage**: SQLite → PostgreSQL → BigQuery seamlessly
4. **Fleet Learning**: Robots pool observations; terrain knowledge improves with each mission

---

## Architecture: 3-Layer Stack

```
┌─────────────────────────────────────────┐
│   Layer 1: Python API (PyO3 abi3)       │  Import: from pyterrainmap import TerrainMap
│   • Bindings for Python 3.10+           │  Wheels-only distribution
│   • Async-native (tokio bridges)        │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│   Layer 2: Rust Core (Spatial Engine)   │  High-performance computation
│   • H3 hierarchical spatial indexing    │  (LLA → H3Cell → observations)
│   • Elevation bucketing (1-2m per bin)  │
│   • SLAM integration (real-time fusion) │
│   • Traversability scoring              │
│   • Temporal decay functions            │
│   • Multi-GPU task scheduling           │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│   Layer 3: Storage Abstraction Layer     │  Pluggable backends:
│   • In-memory (fast, transient)         │  • SQLite (single-robot)
│   • SQLite (development)                │  • PostgreSQL (fleet)
│   • PostgreSQL (team scale)             │  • BigQuery (warehouse-scale)
│   • BigQuery (data warehouse)           │
│   • S3/Cloud Storage (archive)          │
└─────────────────────────────────────────┘
```

---

## Core Data Model: Observations → Spatial Layers → Composite Context

### Observation (Atomic Unit)

Every sensor reading normalizes to:
```python
class Observation:
    id: UUID                        # Unique identifier
    robot_id: str                   # Which robot made this observation
    timestamp: int                  # Microseconds since epoch (microseconds!!)
    
    # Location in 3D space (absolute)
    latitude: float                 # Degrees
    longitude: float                # Degrees
    elevation_asl: Optional[float]  # Above sea level (meters)
    elevation_agl: Optional[float]  # Above ground level (meters)
    
    # Sensor data (union type)
    sensor_type: SensorType         # Thermal, LiDAR, Ultrasonic, Camera, Movement, Custom
    value: SensorValue              # Type-specific (Thermal → float°C, LiDAR → Vec<u16> cm, etc.)
    
    # Quality metadata
    confidence: float               # Sensor confidence (0.0-1.0)
    robot_context: RobotContext     # Position, velocity, heading when observation made
    metadata: Dict[str, str]        # Custom key-value pairs
```

### Spatial Layer (per Sensor Type)

Observations pool into spatial layers:
```python
class SpatialLayer:
    sensor_type: SensorType                         # What kind of data
    key: (H3Cell, ElevationBucket)                  # Where (lat/lon bucket + elevation range)
    
    observations: VecDeque[Observation]             # All observations at this cell
    fused_view: FusedData                           # Aggregated understanding
    
    # Temporal tracking
    temporal_trend: TemporalTrend                   # Rising / stable / falling / unknown
    temporal_validity: float                        # 0.0-1.0 freshness (decays over time)
    baseline_stats: Optional[BaselineStatistics]    # What "normal" looks like here
    change_score: float                             # 0.0-1.0 deviation from normal
    
    last_updated: int                               # Microseconds since epoch
```

### Composite Context (Query Response)

When you ask "what's happening at (40.123, -74.567) right now?":
```python
class CompositeContext:
    location: (float, float)                        # Query location (lat, lon)
    elevation_range: (float, float)                 # Min/max elevation at location
    timestamp_query: int                            # When you asked (microseconds)
    
    # Multi-modal fusion
    thermal_summary: Optional[TemperatureEstimate]  # Temperature + confidence
    obstacle_map: Optional[ObstacleMap]             # Traversability + hazards
    detected_objects: Vec[FusedDetection]           # Fused detections from cameras
    activity_level: ActivityLevel                   # Movement intensity
    
    # Temporal insights
    temporal_trends: Vec<str>                       # ["Temperature rising 2°C/hour", "New obstacle", ...]
    time_since_observation: int                     # Freshness (microseconds)
    observation_count: int                          # How many observations here
    
    # Guidance
    suggested_focus_areas: Vec<(GeoPoint, str)>     # Where to explore next
    missing_sensor_layers: Vec<SensorType>          # What data we lack
    confidence: float                               # Overall confidence (0.0-1.0)
```

---

## Spatial Indexing: H3 Hierarchical Grid

**Why H3?**
- Hierarchical: Resolution 0 = Earth, resolution 15 = ~1m²
- Consistent cell size: No axis-aligned grid artifacts
- Ring queries: Efficiently find neighbors
- Native to geographic coordinates (lat/lon)

**How it works:**
1. Observation arrives: (40.123°N, -74.567°W, 1.5m elevation)
2. Map to H3 cell at resolution 9 (typical neighborhood: ~175m²)
3. Bucket elevation: 1.5m → ElevationBucket(1.0m–2.0m)
4. Key = (H3Cell(40.123, -74.567, res=9), ElevationBucket(1.0, 2.0))
5. Retrieve or create SpatialLayer at that key
6. Aggregate new observation with existing ones

```python
# Example: Query "what's within 500m of robot's current position?"
current_pos = (40.123, -74.567)
radius_meters = 500

# H3 ring query (returns ~7 cells at resolution 9)
neighboring_cells = h3.grid_ring_unsafe(h3_cell, ring_size=2)

# For each neighboring cell, check all elevation buckets
results = []
for cell in neighboring_cells:
    for bucket in elevation_buckets:
        key = (cell, bucket)
        if layer := spatial_layers.get(key):
            results.append(layer.fused_view)  # Aggregated data
```

---

## Temporal Normalization: 5D + Clock + Quality

**The Problem:** Different sensors report at different rates, with clock skew.
- LiDAR: 10 Hz, IMU: 100 Hz, camera: 30 Hz, thermal: 1 Hz
- Robot 1's clock is 0.5s ahead of robot 2's

**The Solution:** Temporal normalization treats time as a first-class coordinate.

### Temporal Decay

Observations become "stale" over time:
```python
def temporal_decay(age_seconds: int, half_life: int = 3600) -> float:
    """
    Exponential decay: after `half_life` seconds, confidence = 0.5.
    Typical half-lives:
    - 3600 (1 hour): Dynamic obstacles, pedestrians
    - 86400 (1 day): Structural changes
    - 604800 (1 week): Long-term maps
    """
    decay_rate = 0.693 / half_life
    return math.exp(-decay_rate * age_seconds)

# Example: Obstacle detected 30 minutes ago (half_life = 1 hour)
confidence = temporal_decay(age_seconds=1800, half_life=3600)
# confidence ≈ 0.707 (70.7% confident it's still there)
```

### Clock Synchronization

```python
class TemporalContext:
    primary_timestamp: int              # Microseconds since UNIX epoch
    robot_local_time: int               # Robot's clock (may be skewed)
    clock_offset: int                   # Estimated skew (microseconds)
    clock_confidence: float              # How sure are we of the offset? (0-1)
    
    # Query respects both absolute and relative time
    def temporal_validity(self, age_seconds: int) -> float:
        """How fresh is data this old?"""
        return temporal_decay(age_seconds, half_life=3600)
```

### Temporal Planes (GPU-Accelerated)

With multiple GPUs, you can parallelize:
1. **Spatial plane:** Voxel grid updates (SLAM)
2. **Temporal plane:** Decay + freshness recomputation
3. **Sensor plane:** Per-sensor-type fusion (thermal, LiDAR, camera)

---

## Python API: From Simple to Powerful

### Beginner: "I have a map, show me what's changed"

```python
from pyterrainmap import TerrainMap

# Create in-memory map
terrain = TerrainMap(backend="memory")

# Add observations from ROS 2 bag
for msg in rosbag2_reader:
    observation = Observation(
        robot_id="robot1",
        timestamp=msg.header.stamp,
        latitude=gps.latitude,
        longitude=gps.longitude,
        sensor_type="lidar",
        value=msg.ranges,
        confidence=0.95
    )
    terrain.add_observation(observation)

# Query: "What's at my current location?"
context = terrain.query(
    latitude=40.123,
    longitude=-74.567,
    timestamp_micros=now_micros()
)

print(f"Obstacles: {context.obstacle_map}")
print(f"Temperature: {context.thermal_summary}")
print(f"Activity: {context.activity_level}")
```

### Advanced: "Fleet learning, persistent storage, GPU fusion"

```python
from pyterrainmap import TerrainMap, StorageBackend

# PostgreSQL backend for team-scale fleet learning
terrain = TerrainMap(
    backend=StorageBackend.PostgreSQL(
        host="db.company.com",
        port=5432,
        database="robot_terrain"
    ),
    enable_gpu_fusion=True,  # Multi-GPU scheduling
    fusion_planes=3,         # Spatial, temporal, sensor planes
)

# Ingest from multiple robots simultaneously
for robot_id in ["robot1", "robot2", "robot3"]:
    rosbag_path = f"s3://data/{robot_id}/mission_001.bag"
    terrain.ingest_rosbag(rosbag_path, robot_id=robot_id)

# Fleet-level query: "What do all robots agree on?"
consensus_map = terrain.fleet_consensus(
    latitude=40.123,
    longitude=-74.567,
    confidence_threshold=0.85,  # Only very confident observations
    time_window_hours=24        # Last 24 hours
)

# Traversability: "Can robot2 safely traverse this path?"
path_safety = terrain.traversability_analysis(
    waypoints=[
        (40.123, -74.567, 0.5),  # x, y, z
        (40.125, -74.569, 0.5),
        (40.127, -74.571, 0.5),
    ],
    robot_type="wheeled_medium",  # Robot footprint
    time_query=now_micros(),
    confidence_threshold=0.8,
)

for segment, safety_score, hazards in path_safety:
    print(f"Segment {segment}: {safety_score:.1%} safe")
    for hazard in hazards:
        print(f"  ⚠️ {hazard}")
```

---

## Integration Points (aspirational — none of this is implemented)

Verified by grepping this repo, `pyroboreplay`, `StatGuardian`, and
`PyStreamMCP` for any cross-reference in either direction: **zero code in
any of the four repos implements any of the integrations below.** No
Cargo/pip dependency exists between PyTerrainMap and any of them.
`pyroboreplay` does have a `pyterrain_bridge.rs` module referencing
PyTerrainMap by name in comments, but it's self-contained internal
terrain-modeling logic with a schema that doesn't match this repo's real
`TerrainGaussian`/observation types — not real data interop (see
`pyroboreplay`'s own README for the corrected framing). Treat everything
below as unimplemented design intent, not a current capability:

### PyRoboReplay (Spatial Context for Causality) — not implemented
- PyTerrainMap provides spatial grounding for event analysis
- Temporal normalization aligns out-of-order sensor events
- Fleet learning: "Did obstacle cause this robot's failure?"

### StatGuardian (Quality Scoring) — not implemented
- StatGuardian flags anomalous observations
- PyTerrainMap confidence scores: observation quality →  spatial layer quality
- Anomalies don't pollute fleet consensus

### PyStreamMCP (Orchestration) — not implemented
- Selective caching of spatial queries (high-value, stable)
- Context prioritization: "Recent obstacles > static map"

---

## Key Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Add observation | <1ms | In-memory layer |
| Query at point | <100ms | H3 lookup + aggregation |
| Fleet consensus | <500ms | Multi-robot merge |
| Traversability (10 waypoints) | <200ms | Vectorized Rust |
| GPU fusion update | <50ms | Multi-GPU parallel |

**Memory:** ~1GB per 1M observations (in-memory)  
**Disk (PostgreSQL):** ~10MB per 1M observations

---

## Common Patterns

### Pattern 1: Incremental Mission Mapping
```python
# Robot explores, continuously updates terrain
terrain = TerrainMap(backend="sqlite", db_path="local_terrain.db")

for sensor_msg in ros_stream:
    obs = sensor_msg.to_observation(robot_id="robot1")
    terrain.add_observation(obs)
    
    # Check if traversable
    if terrain.is_traversable((lat, lon), robot_type="wheeled"):
        publish_safe_zone()
    else:
        publish_hazard()
```

### Pattern 2: Fleet Consensus at Fleet Handoff
```python
# Robot1 completes mission, Robot2 starts
robot1_terrain = TerrainMap.load("robot1_mission.sqlite")
robot2_terrain = TerrainMap.load("robot2_mission.sqlite")

# Merge into fleet knowledge
fleet_terrain = TerrainMap(backend="postgresql")
fleet_terrain.merge(robot1_terrain)
fleet_terrain.merge(robot2_terrain)

# Robot2 plans path based on pooled knowledge
safe_waypoints = fleet_terrain.plan_traversable_path(
    start=(40.123, -74.567),
    goal=(40.200, -74.600),
    robot_type="wheeled",
    confidence_threshold=0.8,
)
```

### Pattern 3: Time-Travel Replay
```python
# When did this area become unsafe?
query_times = list(range(mission_start, mission_end, 60_000_000))  # Every minute

for t in query_times:
    context = terrain.query(
        latitude=40.123,
        longitude=-74.567,
        timestamp_micros=t
    )
    print(f"t={t}: obstacles={len(context.detected_objects)}")
```

---

## Critical Design Decisions

1. **Microsecond timestamps** — Sensor fusion needs precision; milliseconds cause de-sync
2. **H3 spatial indexing** — Hierarchical, geographic-native; no axis-aligned grid artifacts
3. **Elevation bucketing** — Robots operate in 3D; need vertical stratification
4. **Pluggable storage** — SQLite for dev, PostgreSQL for team, BigQuery for warehouse
5. **Temporal decay (exponential)** — Observations become stale realistically; linear decay is wrong
6. **GPU acceleration (optional)** — Fleet learning and multi-plane fusion parallelize naturally

---

## Development Guidelines

### Code Quality
- Rust core: Type-safe, no unsafe outside FFI
- Python bindings: Full type hints (PyO3 + `py.typed`)
- All public APIs tested; >90% coverage

### Performance Profiling
- H3 lookup must be <1ms (index it, don't recompute)
- Temporal decay should be vectorized (not per-observation)
- GPU fusion: Measure actual throughput before committing

### Storage Abstraction
- Implement `StorageBackend` trait for new backends
- In-memory → SQLite → PostgreSQL should be transparent swaps
- No business logic in storage layer

---

## Resources

- **Architecture Deep Dive:** `ARCHITECTURE.md`
- **Temporal Normalization:** `TEMPORAL_NORMALIZATION.md`
- **Data Integrity:** `DATA_INTEGRITY.md`
- **ROS Integration:** `ROS_BRIDGE_ARCHITECTURE.md`

---

**Last Updated:** 2026-08-17 | **Version:** 1.5.0
