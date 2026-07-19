# PyTerrain Ecosystem: Collaborative Terrain Intelligence

## Overview

The **PyTerrain** ecosystem is a two-layer collaborative platform for multi-robot terrain intelligence:

1. **PyTerrainMap** — Spatial indexing, observation storage, real-time terrain mapping
2. **PyTerrainAI** — Intelligence layer: anomaly detection, image stitching, context synthesis

Together they enable heterogeneous robot fleets to build shared terrain understanding. Independently, they solve focused problems with clear architectural boundaries.

## Executive Summary (PyTerrainMap)

**PyTerrainMap** is an MIT-licensed, self-hosted collaborative terrain mapping platform that enables heterogeneous robot fleets to build shared terrain understanding in real-time. It serves as the foundation layer that ANY autonomous system (DimOS, ROS, custom) can query for context and contribute observations to.

Unlike monolithic robot autonomy platforms, PyTerrainMap solves a different problem: **"How do we collectively map this space?"** rather than **"How do we make each robot autonomous?"**

---

## The Problem

### Current State
- Roboticists rebuild collaborative mapping for every deployment
- No standard protocol for multi-bot sensor fusion
- Temporal data validity ignored (old observations treated same as fresh)
- 3D/elevation ignored in most systems (ground vs multi-floor)
- Image timelines unexploited (progressive reconstruction lost)
- Each robot type requires custom integration
- Knowledge is siloed per robot, not shared

### Real-World Impact
- Police surveillance fleets can't share discovered threats between units
- Construction sites rebuild site knowledge for every inspection
- Agricultural drones can't coordinate with ground rovers
- Factory inspections duplicate effort across bot visits
- Security systems can't track temporal changes

---

## The Vision

**PyTerrainMap is the shared knowledge layer for collaborative robot inspection, surveillance, and monitoring.**

Users download, deploy locally, provide their terrain, deploy their bot fleet, and PyTerrainMap handles:
- Multi-perspective terrain reconstruction
- Temporal knowledge decay and freshness scoring
- Context synthesis ("what should this bot know before exploring?")
- Progressive image stitching and 3D model building
- Anomaly detection and change tracking
- Extensible custom layers and logic

It works with ANY robot (quadruped, drone, humanoid, wheeled, custom) using ANY autonomy system (DimOS, ROS, proprietary) without requiring modifications to either.

---

## Key Innovation: The Architecture

### Three Core Insights

#### 1. **Map-Centric vs Robot-Centric**
- ❌ DimOS approach: Make each robot smart → fragmentation
- ✅ PyTerrainMap approach: Make the map smart, keep robots simple

Robots are stateless clients. If a robot dies, restart it. If the map dies, reconstruct from observations. This scales.

#### 2. **Sensor Layers, Not Robot Platforms**
- ❌ Traditional: "Support Go2, then G1, then xArm..." (N×M problem)
- ✅ PyTerrainMap: "Support thermal, LiDAR, camera..." (N+M problem)

All observations feed into per-sensor-type layers. A new robot with an existing sensor set is instantly valuable.

#### 3. **3D Spatial + Temporal + Image Timeline**
Like FPS games' fog-of-war system but for robotics:
- Ground floor ≠ second floor ≠ roof (elevation-aware)
- Yesterday's data < today's data (temporal decay)
- 3 images from different bots → 3D reconstruction
- Image registration over time → change detection

---

## What PyTerrainMap Does

### Core Capabilities

| Capability | Description |
|------------|-------------|
| **Multi-perspective terrain reconstruction** | Fuses observations from different bots, viewpoints, sensor types into coherent spatial model |
| **3D spatial indexing** | Understands elevation/floors; ground floor ≠ 2nd floor ≠ roof |
| **Temporal knowledge management** | Observations decay over time; recent data weighted higher |
| **Real-time context queries** | Bots ask "what should I know before going there?" |
| **Sensor fusion** | Combines thermal + LiDAR + camera + ultrasonic intelligently |
| **Anomaly detection** | Flags: new threats, unexpected presence, structural damage |
| **Image timeline stitching** | Combines images from different bots/times into 3D progression |
| **Fog-of-war tracking** | Explored vs partially-observed vs unknown zones |
| **Extensible layers** | Users define custom layers (crop_health, threat_score, damage_index) |

---

## Concrete Use Cases

### Police Surveillance Fleet
- Multiple patrol units with different sensors
- Query before entering zone: "What did other units find here?"
- Thermal bot detects person, visual bot confirms, threat database alerts
- Context builds: "Person A was here Tuesday, no threat. Back Thursday, flagged as wanted."

### Agricultural Land Inspection  
- Drone captures multispectral imagery (day 1)
- Ground rover measures soil moisture (day 3)
- Quadruped does close inspection (day 5)
- Map builds: Crop health scores, disease progression, irrigation needs
- Timeline shows: Growth vs stagnation, yield prediction

### Construction Site Monitoring
- Pre-scan: LiDAR baseline of site
- Weekly drone flights capture progress
- Ground bot measures excavation depth
- Deviations from design detected automatically
- Timeline: Foundation → structure → finishing → completion

### Building Security
- Humanoid patrols interior, drone surveys exterior
- Stationary sensors monitor entrances
- Normal pattern established (occupancy, times, routines)
- Anomaly detection: "Unusual presence in sector C", "Door left open"
- Temporal tracking: "This person wasn't here yesterday"

### Factory Inspection
- Heat signature maps from thermal drones
- LiDAR point clouds from ground bots
- Camera close-ups from humanoids
- Composite understanding: "HVAC malfunction" (explains heat + sound + pressure)
- Maintenance scheduled automatically

---

## Technical Architecture

### Layers

```
┌─────────────────────────────────────────┐
│    User Application Logic               │
│  (Custom missions, alerting, export)    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    PyTerrainMap Core                      │
│  ├─ 3D Spatial Indexing (H3 + elevation)
│  ├─ Temporal Decay Functions            │
│  ├─ Multi-sensor Fusion Engine          │
│  ├─ Anomaly Detection                   │
│  ├─ Context Synthesis                   │
│  └─ Observation Storage                 │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    PyNoramic (Optional)                 │
│  ├─ Image Registration                  │
│  ├─ Structure from Motion (3D)          │
│  ├─ Temporal Image Stitching            │
│  └─ Change Detection                    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│    Storage                              │
│  ├─ In-memory (recent, fast)            │
│  ├─ Persistent (historical, archive)    │
│  └─ Image storage (optional)            │
└─────────────────────────────────────────┘
```

### Data Model (Simplified)

```rust
Observation {
  robot_id: String,
  location: (lat, lon, elevation),
  timestamp: i64,
  sensor_type: SensorType,  // Thermal, LiDAR, Camera, etc.
  value: SensorValue,        // Typed sensor data
  confidence: f32,           // Observation quality
  robot_context: {...}      // What robot can sense from here
}

SpatialLayer {
  key: (H3Cell, ElevationBucket),
  observations: [Observation...],
  fused_view: FusedData,     // Aggregated understanding
  temporal_validity: f32,    // 0.0-1.0 freshness score
  change_score: f32,         // How much changed since last visit
}

CompositeContext {
  thermal_summary: Option<Stats>,
  obstacle_map: Map,
  detected_objects: [Object...],
  activity_level: Enum,
  temporal_trends: [String...],     // "Temperature rising", "New obstacle"
  suggested_focus_areas: [Location...],
  missing_sensor_layers: [SensorType...],
}
```

---

## Protocol & Integration

Any bot, any autonomy system can integrate via simple protocol:

```python
# Initialize
map_service = PyTerrainMap(host="192.168.1.100", port=8080)

# Before mission
context = await map_service.query(
    location=GeoPoint(lat, lon),
    elevation_range=(0, 2),
    radius_m=50,
    interested_sensors=[SensorType.Thermal, SensorType.LiDAR]
)

# Execute mission (your autonomy system)
plan = my_robot_autonomy.plan(context)
my_robot_autonomy.execute(plan)

# After mission
for observation in my_sensor_readings:
    await map_service.push_observation(Observation(
        robot_id=my_id,
        location=gps,
        sensor_type=observation.type,
        value=observation.data,
        confidence=observation.confidence,
    ))
```

**No robot code changes needed.** Same protocol works for:
- Boston Dynamics Spot
- Custom quadrupeds
- DJI drones
- Humanoids
- Wheeled rovers
- Custom robots

---

## Competitive Positioning

### vs Existing Solutions

| Aspect | ArcGIS | ROS SLAM | DimOS | Palantir | PyTerrainMap |
|--------|--------|----------|-------|----------|-----------|
| **Collaborative mapping** | ✓ Maps only | ✓ Per-robot | ✗ No | ✓ Enterprise | ✓ Core |
| **3D elevation support** | ✓ Limited | ✗ Rare | ✗ No | ✓ Yes | ✓ First-class |
| **Temporal validity** | ✗ Static | ✗ No | ✗ No | ✓ Limited | ✓ Built-in |
| **Multi-bot context** | ✗ No | ✗ Per-robot | ✗ Messaging | ✓ Limited | ✓ Core |
| **Image timeline** | ✗ No | ✗ No | ✗ No | ✓ Limited | ✓ PyNoramic |
| **Decentralized** | ✗ Cloud | ✓ Local | ✗ Dist | ✗ Cloud | ✓ Local |
| **Open source** | ✗ Proprietary | ✓ Yes | ✗ Research | ✗ Proprietary | ✓ MIT |
| **Price** | $$$$ | Free | N/A | $$$$$ | Free |

**PyTerrainMap's unique position:** ROS modularity + Palantir semantics + temporal awareness + zero cloud dependency + MIT license.

---

## Roadmap

### Phase 0-1: MVP (Months 1-6)
- Core spatial engine (H3 indexing, elevation buckets)
- Basic observation storage & queries
- Temporal decay functions
- Sensor fusion (temperature, obstacles, detections)
- Python bindings (PyO3)
- Self-hosted HTTP API
- Basic examples (police, construction)

### Phase 2-3: Knowledge Graph (Months 7-12)
- Semantic layer (entity classification, relationships)
- Anomaly detection (deviation from baseline)
- Threat/damage scoring
- Integration with external databases (threat lists, wanted persons)
- Multi-bot task allocation ("where should I go next?")
- Visualization dashboard

### Phase 4-5: Image & Ecosystem (Months 13-18)
- PyNoramic image registration & stitching
- Structure from Motion (3D reconstruction)
- Change detection across image timelines
- Ecosystem integrations (ROS, DimOS, simulation)
- Commercial pilot support
- v1.0 release

---

## Getting Started

### For Users
1. Download PyTerrainMap (GitHub releases)
2. Provide terrain (OSM data, LiDAR scan, floor plans)
3. Deploy your bot fleet (any autonomy system)
4. Bots query for context, push observations
5. PyTerrainMap builds shared knowledge

### For Contributors
1. Clone repository
2. Set up Rust + Python environment
3. Run tests: `cargo test && pytest`
4. Read ARCHITECTURE.md
5. Pick an issue, submit PR

### For Researchers
- Sensor fusion algorithms (weighted averaging, Bayesian grid, NMS)
- Temporal decay models (exponential, Gaussian)
- Image registration and SfM
- Change detection heuristics

---

## Why PyTerrainMap

### For Roboticists
- Stop rebuilding collaborative mapping
- Focus on robot capabilities, not knowledge infrastructure
- Proven architecture (battle-tested in games, robotics research)

### For Organizations
- Heterogeneous fleet support (any robot, any sensor)
- Self-hosted (no vendor lock-in, no cloud bills)
- MIT licensed (modify freely, use commercially)
- Progressive deployment (start with 2 bots, scale to 100)

### For Researchers
- Open platform for collaborative robotics research
- Extensible architecture (add custom layers, fusion algorithms)
- Benchmark datasets (shared across research teams)

---

## Conclusion

PyTerrainMap is the knowledge layer roboticists have been rebuilding in every project. It transforms robot fleets from isolated agents into a coherent collective intelligence system.

Like how multiplayer FPS games handle shared maps, PyTerrainMap handles shared terrain understanding. Download it, plug in your bots, and focus on what makes your application unique.

The future of robotics is collaborative. PyTerrainMap enables it.

---

**Status:** Vision document (in development)  
**License:** MIT  
**Repository:** github.com/Mullassery/pyterrain-map
