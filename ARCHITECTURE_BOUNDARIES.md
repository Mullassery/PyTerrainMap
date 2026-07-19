# PyTerrain Ecosystem: Architectural Boundaries

## Two-Repository Design

The PyTerrain ecosystem consists of two independent repositories with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Robot Fleets                                  │
│  (DimOS, ROS, Custom Autonomy Systems)                          │
└─────────────────────────┬───────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│  PyTerrainMap    │ │  PyTerrainMap    │ │  PyTerrainMap    │
│  (Spatial Layer) │ │  (Spatial Layer) │ │  (Spatial Layer) │
│                  │ │                  │ │                  │
│ ✓ Observation    │ │ ✓ Observation    │ │ ✓ Observation    │
│   storage        │ │   storage        │ │   storage        │
│ ✓ H3 indexing    │ │ ✓ H3 indexing    │ │ ✓ H3 indexing    │
│ ✓ Elevation      │ │ ✓ Elevation      │ │ ✓ Elevation      │
│ ✓ Basic fusion   │ │ ✓ Basic fusion   │ │ ✓ Basic fusion   │
│ ✓ Temporal decay │ │ ✓ Temporal decay │ │ ✓ Temporal decay │
│ ✓ Query API      │ │ ✓ Query API      │ │ ✓ Query API      │
└──────────────────┘ └──────────────────┘ └──────────────────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                          ▼
        ┌──────────────────────────────────┐
        │    PyTerrainAI (Optional)        │
        │    (Intelligence Layer)          │
        │                                  │
        │ ✓ Image registration & stitching│
        │ ✓ Structure from Motion (3D)    │
        │ ✓ Anomaly detection             │
        │ ✓ Context synthesis             │
        │ ✓ Change detection              │
        │ ✓ Knowledge graphs              │
        └──────────────────────────────────┘
```

## PyTerrainMap (Core Spatial Layer)

**Repository:** `github.com/Mullassery/pyterrain-map`  
**License:** MIT  
**Language:** Rust (core) + Python (bindings)

### Responsibilities
- Store observations from multiple robots
- Spatial indexing (H3 cells + elevation buckets)
- Temporal knowledge management (decay functions)
- Basic sensor fusion (temperature averaging, obstacle grids, detection voting)
- Query API (spatial-temporal range queries)
- Context synthesis (what should a bot know before exploring?)

### Does NOT Include
- Image processing/stitching
- Advanced anomaly detection (beyond baseline z-score)
- Machine learning models
- Knowledge graphs
- Natural language understanding

### Public API

```rust
pub struct PyTerrainMap {
    // Core operations
    pub async fn push_observation(&self, obs: Observation) -> Result<()>;
    pub async fn query(&self, location: GeoPoint, radius: f32, ...) -> Result<CompositeContext>;
    pub async fn get_temporal_trends(&self, location: GeoPoint) -> Result<Vec<Trend>>;
    
    // Extensibility
    pub fn register_custom_layer(&self, config: LayerConfig);
    pub fn register_fusion_fn(&self, layer: String, fn_ptr: FusionFn);
}
```

### Guarantees
- **<1ms observation ingestion** (concurrent writes)
- **<50ms query response** (50m² radius, 100 observations)
- **No external dependencies** except Rust std + tokio
- **Self-contained** (no network calls to other services)

---

## PyTerrainAI (Intelligence Layer)

**Repository:** `github.com/Mullassery/pyterrain-ai`  
**License:** MIT  
**Language:** Python (primary), Rust (optional performance modules)

### Responsibilities
- Image registration and stitching (from multiple robots/times)
- Structure from Motion (3D reconstruction from images)
- Advanced anomaly detection (statistical, ML-based)
- Context enrichment (combining map data with external knowledge)
- Change detection (temporal image comparison)
- Knowledge synthesis (entity tracking, threat scoring, etc.)

### Does NOT Include
- Observation storage (uses PyTerrainMap)
- Spatial indexing (reads from PyTerrainMap)
- Basic sensor fusion (delegated to PyTerrainMap)
- Robot autonomy (external systems call PyTerrainAI)

### Integration with PyTerrainMap

```python
from pyterrain_map import PyTerrainMap
from pyterrain_ai import PyTerrainAI

# PyTerrainAI is a client of PyTerrainMap
map_service = PyTerrainMap(...)
ai_service = PyTerrainAI(map_service=map_service)

# Query map for context
context = await map_service.query(location)

# Enhance with intelligence
enriched = await ai_service.analyze(context)
# ├─ Detected anomalies
# ├─ Temporal changes
# ├─ 3D reconstructions
# └─ Confidence scores
```

### Public API

```python
class PyTerrainAI:
    def __init__(self, map_service: PyTerrainMap):
        self.map = map_service
    
    async def get_image_timeline(self, location: GeoPoint) -> ImageTimeline;
    async def get_3d_reconstruction(self, location: GeoPoint) -> PointCloud;
    async def detect_anomalies(self, location: GeoPoint) -> AnomalyReport;
    async def get_context_enriched(self, location: GeoPoint) -> EnrichedContext;
```

### Can Use External Services
- OpenCV (image processing)
- TensorFlow/PyTorch (ML models)
- GraphQL/Neo4j (knowledge graphs)
- LLMs (semantic understanding)

---

## Deployment Scenarios

### Scenario 1: PyTerrainMap Only
**Use case:** Quick deployment, basic multi-robot coordination

```python
from pyterrain_map import PyTerrainMap

# Deploy locally
map_service = PyTerrainMap()

# Bots query for context, push observations
for bot in fleet:
    context = await map_service.query(bot.location)
    bot.execute(context)
    await map_service.push_observation(bot.observations)
```

**Requirements:** Rust, Python 3.10+  
**Footprint:** ~50MB  
**Deployment time:** 5 minutes

### Scenario 2: PyTerrainMap + PyTerrainAI
**Use case:** Advanced inspection, temporal analysis, 3D reconstruction

```python
from pyterrain_map import PyTerrainMap
from pyterrain_ai import PyTerrainAI

# Deploy both services
map_service = PyTerrainMap()
ai_service = PyTerrainAI(map_service=map_service)

# Bots get enriched context
for bot in fleet:
    enriched = await ai_service.get_context_enriched(bot.location)
    bot.execute(enriched)
    await map_service.push_observation(bot.observations)

# Humans analyze results
changes = await ai_service.detect_anomalies(target_location)
timeline = await ai_service.get_image_timeline(target_location)
```

**Requirements:** PyTerrainMap + Python 3.10+ + OpenCV/TensorFlow  
**Footprint:** ~500MB  
**Deployment time:** 15 minutes

### Scenario 3: PyTerrainMap as Microservice
**Use case:** Containerized, multi-tenant

```bash
# Run PyTerrainMap in container
docker run -p 8080:8080 pyterrain-map:latest

# External bots use HTTP API
POST /observations
GET /query?lat=40.123&lon=-74.567&radius_m=50

# Optional: Attach PyTerrainAI
docker run -p 8081:8081 \
  -e TERRAIN_MAP_URL=http://map:8080 \
  pyterrain-ai:latest
```

---

## Data Flow & Contracts

### Observation → PyTerrainMap

```python
Observation = {
    robot_id: String,
    location: GeoPoint,
    elevation_asl: Float,  # Optional
    timestamp: Int64,
    sensor_type: SensorType,  # Enum: Thermal, LiDAR, Camera, etc.
    value: Dict,  # Sensor-specific
    confidence: Float,  # 0.0-1.0
    metadata: Dict,  # Custom fields
}
```

### PyTerrainMap → PyTerrainAI

PyTerrainAI queries PyTerrainMap via:

```python
CompositeContext = {
    location: GeoPoint,
    timestamp: Int64,
    thermal_summary: Optional[Stats],
    obstacle_map: Optional[Grid],
    detected_objects: [Object...],
    temporal_trends: [String...],
    suggested_focus_areas: [(Location, reason)...],
    raw_observations: [Observation...],  # For AI to process
}
```

### Interface Contract
- **PyTerrainMap** exposes read-only query API (PyTerrainAI is a client)
- **PyTerrainAI** does NOT write to PyTerrainMap (no feedback loop)
- **PyTerrainAI** can suggest actions via enriched context

---

## Why Separate Repositories?

### PyTerrainMap (Lean, Fast)
- **Pro:** Minimal dependencies, self-contained, sub-ms latency
- **Pro:** Easy to deploy in resource-constrained environments
- **Pro:** Stable API, rarely changes
- **Pro:** Can run standalone for basic coordination

### PyTerrainAI (Rich, Smart)
- **Pro:** Can iterate rapidly on intelligence algorithms
- **Pro:** Can use heavy dependencies (OpenCV, TensorFlow)
- **Pro:** Optional (users don't pay for features they don't use)
- **Pro:** Can be upgraded independently
- **Pro:** Clear responsibility (intelligence only)

### Trade-offs
- **Trade-off:** Two repos to maintain (mitigated by clear contract)
- **Trade-off:** Network latency between services (acceptable: AI is async)
- **Trade-off:** Learning curve (need to understand both)

### Benefits Over Monolith
- ✅ Users can use PyTerrainMap without AI overhead
- ✅ PyTerrainAI can use expensive libraries without bloating PyTerrainMap
- ✅ Clear separation enables independent scaling
- ✅ Easier to test, deploy, and iterate each layer
- ✅ Reduces dependency conflicts

---

## Development Guidelines

### When to Modify PyTerrainMap
- Observation storage optimization
- Spatial query performance
- Temporal decay logic
- Basic sensor fusion improvements
- Core API changes

### When to Modify PyTerrainAI
- Image stitching algorithms
- Anomaly detection models
- Knowledge synthesis logic
- External system integrations
- Context enrichment

### Never
- ❌ Add ML models to PyTerrainMap
- ❌ Add image processing to PyTerrainMap
- ❌ Add knowledge graphs to PyTerrainMap
- ❌ Add external API calls to PyTerrainMap core
- ❌ Add heavy dependencies to PyTerrainMap

---

## Future Extensibility

### Custom Layers (PyTerrainMap)
```python
map_service.register_custom_layer(
    name="threat_score",
    fusion_fn=my_threat_fn,
    temporal_decay_hours=6
)
```

### Custom AI Modules (PyTerrainAI)
```python
ai_service.register_analyzer(
    name="crop_disease_detector",
    model=my_pytorch_model,
    input_sensors=[SensorType.Camera, SensorType.Multispectral],
    output_type="disease_probability"
)
```

---

## References

- PyTerrainMap Repo: `github.com/Mullassery/pyterrain-map`
- PyTerrainAI Repo: `github.com/Mullassery/pyterrain-ai`
- Shared Wiki: `github.com/Mullassery/pyterrain-ecosystem` (main documentation)
