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
- **Store ALL observations** from all robots (permanent historical record)
- Spatial indexing (H3 cells + elevation buckets)
- **Provide raw observations** via query API (no filtering, no decay applied)
- Basic sensor fusion (temperature averaging, obstacle grids, detection voting)
- Elevation-aware 3D spatial organization
- Multiple layers (thermal, LiDAR, camera, custom)

### Key Principle
**PyTerrainMap = Permanent Data Warehouse (No Decay Enforcement)**
- Stores all observations permanently (never deletes)
- **HAS** timestamp and location data (decay is possible)
- **DOES NOT ENFORCE** temporal decay (returns raw confidence values)
- Returns observations as-is (all confidence values original, all timestamps intact)
- Acts as source of truth for all historical and current data
- Decay logic lives in PyTerrainAI (query/middleware layer)

### Does NOT Include
- Temporal decay functions (PyTerrainAI applies these)
- Mission-based filtering (PyTerrainAI applies these)
- Image processing/stitching
- Access control/security
- Machine learning models
- Knowledge graphs
- Anomaly detection

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

## PyTerrainAI (Middleware + Intelligence Layer)

**Repository:** `github.com/Mullassery/pyterrain-ai`  
**License:** MIT  
**Language:** Python (primary)

### Responsibilities
- **Temporal decay:** Apply time-decay functions to observations (old observations weighted less)
- **Mission alignment:** Ensure context matches bot's mission requirements
- **Access control:** Enforce RBAC (what each bot can see/access)
- **Information filtering:** Return only mission-relevant observations
- **Image registration & stitching:** Combine images from multiple robots/times
- **Anomaly detection:** Statistical, rule-based, and ML-based
- **Context enrichment:** Add intelligence to filtered observations
- **Change detection:** Temporal image comparison and trend analysis
- **Audit logging:** Track what each bot accessed

### Key Principle
**PyTerrainAI = Smart Middleware + Temporal Intelligence (Applies Decay)**
- Queries PyTerrainMap for RAW observations (with timestamps intact)
- **APPLIES temporal decay** (uses timestamp to weight old observations less)
- Filters by mission (bot A sees security data only)
- Enriches with intelligence
- Returns: "Here's what you need, weighted by freshness, relevant to your mission"
- Decay happens at query-time, not storage-time

### Does NOT Include
- Observation storage (PyTerrainMap handles)
- Spatial indexing (reads from PyTerrainMap)
- Raw data access (all data flows through decay/filtering)
- Robot autonomy (external systems call PyTerrainAI)

### Integration with PyTerrainMap

```python
from pyterrain_map import PyTerrainMap
from pyterrain_ai import PyTerrainAI

# PyTerrainAI is middleware client of PyTerrainMap
map_service = PyTerrainMap()
ai_service = PyTerrainAI(map_service=map_service, access_policy=policy)

# Bot queries AI (not map directly)
context = await ai_service.get_context(
    bot_id="security_1",
    mission="security",
    location=GeoPoint(lat, lon),
)

# AI does:
# 1. Check if bot authorized for this location/mission
# 2. Query PyTerrainMap for raw observations
# 3. Apply temporal decay
# 4. Filter mission-irrelevant data
# 5. Enrich with intelligence
# 6. Return filtered, decayed, enriched context
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

### Step 1: Observation → PyTerrainMap (Storage)

```python
Observation = {
    robot_id: String,
    location: GeoPoint,
    elevation_asl: Float,  # Optional
    timestamp: Int64,  # STORED AS-IS (no decay)
    sensor_type: SensorType,  # Thermal, LiDAR, Camera, etc.
    value: Dict,  # Sensor-specific
    confidence: Float,  # 0.0-1.0 (stored original)
    metadata: Dict,
}
```
**PyTerrainMap:** Stores EVERYTHING, FOREVER, NO MODIFICATIONS

### Step 2: Bot Query → PyTerrainAI (Middleware)

```python
Bot Query = {
    bot_id: String,
    mission: String,  # "security", "inspection", etc.
    location: GeoPoint,
    radius_m: Float,
}
```

### Step 3: PyTerrainAI Processes (Temporal + Mission Alignment)

```python
# PyTerrainAI workflow:

1. Check Authorization
   ├─ Is this bot allowed for this mission?
   ├─ Does this mission have access to this location?
   └─ Reject if unauthorized

2. Query PyTerrainMap for RAW observations
   └─ Get all observations with timestamps (no decay applied yet)

3. Apply Temporal Decay
   └─ For each observation:
       ├─ Calculate age = now - observation.timestamp
       ├─ Apply decay function: weight = exp(-age / half_life)
       └─ Decayed confidence = original_confidence * weight

4. Filter by Mission
   └─ Keep only sensors relevant to mission
       ├─ "security" mission: keep Camera, Thermal, Movement
       ├─ "inspection" mission: keep LiDAR, Camera, Ultrasonic
       └─ "monitoring" mission: keep Environmental, Thermal, Occupancy

5. Enrich with Intelligence
   ├─ Detect anomalies
   ├─ Stitch images
   ├─ Compute 3D models
   └─ Add context

6. Return Filtered, Decayed, Enriched Context
```

### Step 4: PyTerrainAI → Bot (Mission-Filtered Context)

```python
MissionContext = {
    location: GeoPoint,
    timestamp_query: Int64,
    
    # Only relevant to mission
    observations: [
        {
            sensor_type: ...,
            value: ...,
            confidence: ...,  # DECAYED
            age_seconds: ...,
            temporal_weight: ...,  # e.g., 0.5 for 2hr old data
        }
    ],
    
    # Mission-specific intelligence
    anomalies: [...],
    trends: [...],
    suggested_focus_areas: [...],
    
    # Metadata
    authorization_level: String,
    audit_log_entry: String,
}
```

### Interface Contract

**PyTerrainMap:**
- Exposes read-only query API
- Returns RAW observations (no filtering, no decay, no time modifications)
- Permanent storage (never deletes)
- Single source of truth for all historical data

**PyTerrainAI:**
- Client of PyTerrainMap
- Applies temporal decay (dynamically, on read)
- Enforces mission-based filtering
- Enriches with intelligence
- Does NOT modify PyTerrainMap data
- Does NOT store observations

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
