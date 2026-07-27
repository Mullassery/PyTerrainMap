# PyTerrain Implementation Roadmap

## Phase Overview (18 weeks total)

```
Phase 1: PyTerrainMap Core (Weeks 1-6)
├─ Data structures
├─ H3 spatial indexing
├─ In-memory storage
├─ Basic query API
└─ Python bindings

Phase 2: PyTerrainAI Basics (Weeks 5-8)
├─ RBAC system
├─ Temporal decay
├─ Mission filtering
├─ Anomaly detection
└─ HTTP API

Phase 3: Integration & Testing (Weeks 7-10)
├─ End-to-end tests
├─ Example scenarios
├─ Performance tuning
└─ Documentation

Phase 4: Advanced Features (Weeks 11-18)
├─ Persistent storage
├─ Image stitching (PyNoramic)
├─ ML anomaly detection
└─ Production hardening
```

---

## Phase 1: PyTerrainMap Core (Weeks 1-6)

### Week 1-2: Data Structures & Foundation

**Start here:**

```bash
cd /Users/georgimullassery/PyTerrainMap
cargo new --lib
```

**Create core types (src/types.rs):**

```rust
pub struct Observation {
    pub id: Uuid,
    pub robot_id: String,
    pub timestamp: i64,
    pub location: GeoPoint,
    pub elevation_asl: Option<f32>,
    pub sensor_type: SensorType,
    pub value: SensorValue,
    pub confidence: f32,
}

pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

pub enum SensorType {
    Thermal,
    LiDAR,
    Camera,
    Ultrasonic,
}
```

**Target:** Core data structures compile, basic serialization  
**Time:** 3-4 days  
**Test:** Unit tests for type definitions

---

### Week 2-3: Spatial Indexing (H3)

**Create spatial module (src/spatial/mod.rs):**

```rust
pub struct SpatialIndex {
    // Map H3 cells to observation buckets
    cells: HashMap<(H3Cell, ElevationBucket), Vec<Arc<Observation>>>,
}

impl SpatialIndex {
    pub fn insert(&mut self, obs: Arc<Observation>) {
        let h3_cell = self.location_to_h3(obs.location);
        let elev_bucket = self.elevation_to_bucket(obs.elevation_asl);
        let key = (h3_cell, elev_bucket);
        
        self.cells.entry(key)
            .or_insert_with(Vec::new)
            .push(obs);
    }
    
    pub fn query_radius(&self, location: GeoPoint, radius_m: f32) -> Vec<Arc<Observation>> {
        let h3_cell = self.location_to_h3(location);
        let cells = h3::k_ring(h3_cell, self.ring_radius(radius_m));
        
        let mut results = Vec::new();
        for cell in cells {
            for elev_bucket in self.elevation_buckets() {
                if let Some(obs) = self.cells.get(&(cell, elev_bucket)) {
                    results.extend(obs.iter().cloned());
                }
            }
        }
        results
    }
}
```

**Dependencies to add (Cargo.toml):**
```toml
h3 = "0.11"
uuid = { version = "1.6", features = ["v4"] }
parking_lot = "0.12"
```

**Target:** Spatial queries work  
**Time:** 1 week  
**Test:** Spatial query tests

---

### Week 3-4: In-Memory Storage

**Create storage module (src/storage/mod.rs):**

```rust
pub struct Storage {
    // Multiple indexes for different query patterns
    by_spatial: Arc<RwLock<SpatialIndex>>,
    by_time: Arc<RwLock<BTreeMap<i64, Vec<Arc<Observation>>>>>,
    
    // Observation pool
    observations: Arc<RwLock<Vec<Arc<Observation>>>>,
}

impl Storage {
    pub async fn store(&self, obs: Observation) -> Result<()> {
        let obs = Arc::new(obs);
        
        // Add to spatial index
        self.by_spatial.write().await.insert(obs.clone());
        
        // Add to temporal index
        self.by_time.write().await
            .entry(obs.timestamp)
            .or_insert_with(Vec::new)
            .push(obs.clone());
        
        // Add to observation pool
        self.observations.write().await.push(obs);
        
        Ok(())
    }
    
    pub async fn query(
        &self,
        location: GeoPoint,
        radius_m: f32,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<Observation>> {
        // Get spatial results
        let spatial = self.by_spatial.read().await.query_radius(location, radius_m);
        
        // Filter by time
        let results: Vec<_> = spatial
            .iter()
            .filter(|obs| obs.timestamp >= start_time && obs.timestamp <= end_time)
            .map(|obs| (**obs).clone())
            .collect();
        
        Ok(results)
    }
}
```

**Target:** Store and query observations  
**Time:** 1 week  
**Test:** Storage and query tests

---

### Week 4-5: Query API & Fusion

**Create query module (src/query/mod.rs):**

```rust
pub async fn fuse_observations(observations: &[Observation]) -> FusedData {
    // Group by sensor type
    let by_sensor = group_by_sensor(observations);
    
    // Fuse each sensor type
    FusedData {
        temperature: fuse_temperatures(by_sensor.get(&SensorType::Thermal)),
        obstacles: fuse_obstacles(by_sensor.get(&SensorType::LiDAR)),
        detections: fuse_detections(by_sensor.get(&SensorType::Camera)),
    }
}

// Implement basic fusion algorithms
fn fuse_temperatures(obs: Option<&[Observation]>) -> Option<TemperatureEstimate> {
    if let Some(obs) = obs {
        let values: Vec<f32> = obs.iter()
            .filter_map(|o| extract_temperature(o))
            .collect();
        
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        
        Some(TemperatureEstimate { 
            celsius: mean, 
            variance,
            num_readings: values.len() as u32,
        })
    } else {
        None
    }
}
```

**Target:** Basic fusion working  
**Time:** 1 week  
**Test:** Fusion algorithm tests

---

### Week 5-6: Python Bindings (PyO3)

**Create Python module (src/python.rs):**

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct PyTerrainMap {
    storage: Arc<Storage>,
}

#[pymethods]
impl PyTerrainMap {
    #[new]
    fn new() -> Self {
        PyTerrainMap {
            storage: Arc::new(Storage::new()),
        }
    }
    
    fn push_observation(&self, py: Python, obs: &PyAny) -> PyResult<()> {
        let obs = extract_observation(obs)?;
        
        // Run async operation
        pyo3_asyncio::tokio::future_into_py(py, async move {
            self.storage.store(obs).await?;
            Ok(Python::with_gil(|py| py.None()))
        })?;
        
        Ok(())
    }
    
    fn query(&self, py: Python, location: &PyAny, radius: f32) -> PyResult<PyObject> {
        // Similar async wrapper
        todo!()
    }
}

#[pymodule]
fn pyterrain_map(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyTerrainMap>()?;
    Ok(())
}
```

**Update Cargo.toml:**
```toml
[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
pyo3-asyncio = { version = "0.20", features = ["tokio-runtime"] }
tokio = { version = "1.35", features = ["full"] }

[lib]
crate-type = ["cdylib"]
```

**Target:** Python can import and use PyTerrainMap  
**Time:** 1 week  
**Test:** Python integration tests

---

## Phase 2: PyTerrainAI Basics (Weeks 5-8)

### Week 5-6: RBAC System

**In parallel with PyTerrainMap Week 5-6:**

```python
# pyterrain_ai/access_control.py

from enum import Enum
from dataclasses import dataclass

class Mission(Enum):
    SECURITY = "security"
    INSPECTION = "inspection"
    MONITORING = "monitoring"
    MAINTENANCE = "maintenance"

@dataclass
class Role:
    mission: Mission
    allowed_sensors: list[SensorType]
    allowed_areas: list[str]
    access_level: str  # "standard", "privileged", "admin"

class AccessPolicy:
    def __init__(self):
        self.policies = {
            Mission.SECURITY: Role(
                mission=Mission.SECURITY,
                allowed_sensors=[SensorType.Camera, SensorType.Thermal],
                allowed_areas=["*"],  # All areas
                access_level="standard"
            ),
            Mission.INSPECTION: Role(
                mission=Mission.INSPECTION,
                allowed_sensors=[SensorType.LiDAR, SensorType.Camera],
                allowed_areas=["Building_A", "Building_B"],
                access_level="standard"
            ),
        }
    
    def can_access(self, bot_id: str, mission: Mission, location: GeoPoint) -> bool:
        role = self.policies.get(mission)
        if not role:
            return False
        
        # Check area restrictions
        if "*" not in role.allowed_areas:
            if location not in role.allowed_areas:
                return False
        
        return True
```

**Target:** RBAC checks working  
**Time:** 1 week  
**Test:** Permission tests

---

### Week 6-7: Temporal Decay & Filtering

**Create temporal module (pyterrain_ai/temporal.py):**

```python
import math
from dataclasses import dataclass

class TemporalDecay:
    """Apply time decay to observations"""
    
    @staticmethod
    def exponential_decay(age_seconds: int, half_life_seconds: int) -> float:
        """
        Exponential decay: weight = exp(-age / half_life)
        """
        decay_rate = 0.693 / half_life_seconds
        return math.exp(-decay_rate * age_seconds)
    
    @staticmethod
    def apply_to_observation(obs: Observation, half_life_seconds: int = 7200) -> Observation:
        """Apply decay to observation confidence"""
        now = time.time() * 1e6  # microseconds
        age = (now - obs.timestamp) / 1e6  # seconds
        
        decay_weight = TemporalDecay.exponential_decay(age, half_life_seconds)
        obs.decayed_confidence = obs.confidence * decay_weight
        obs.temporal_weight = decay_weight
        
        return obs

class MissionFilter:
    """Filter observations by mission relevance"""
    
    def filter(self, observations: list[Observation], mission: Mission) -> list[Observation]:
        role = self.policy.get_role(mission)
        
        return [
            obs for obs in observations
            if obs.sensor_type in role.allowed_sensors
        ]
```

**Target:** Decay functions working, filtering working  
**Time:** 1 week  
**Test:** Decay and filter tests

---

### Week 7-8: Anomaly Detection & HTTP API

**Create anomaly module (pyterrain_ai/anomaly.py):**

```python
class AnomalyDetector:
    """Detect observations that deviate from baseline"""
    
    def __init__(self, map_service):
        self.map = map_service
    
    async def detect(self, observation: Observation) -> AnomalyStatus:
        # Get historical baseline
        baseline = await self.map.get_historical_baseline(
            location=observation.location,
            sensor_type=observation.sensor_type,
            days_back=30
        )
        
        if not baseline:
            return AnomalyStatus.UNKNOWN  # No baseline yet
        
        # Compute z-score
        z_score = abs(observation.value - baseline.mean) / (baseline.std + 1e-6)
        
        if z_score > 3.0:
            return AnomalyStatus.ANOMALY_NEEDS_VERIFICATION
        else:
            return AnomalyStatus.VERIFIED
```

**Create HTTP API (pyterrain_ai/server.py):**

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

app = FastAPI()

class ContextRequest(BaseModel):
    bot_id: str
    mission: str
    lat: float
    lon: float
    radius_m: float = 50.0

@app.post("/context")
async def get_context(request: ContextRequest) -> dict:
    """Get mission-aligned, time-decayed context for bot"""
    
    # Check permissions
    if not policy.can_access(request.bot_id, request.mission, request.location):
        raise HTTPException(status_code=403, detail="Not authorized")
    
    # Query PyTerrainMap
    observations = await map_service.query(
        location=GeoPoint(request.lat, request.lon),
        radius_m=request.radius_m
    )
    
    # Apply temporal decay
    decayed = [TemporalDecay.apply_to_observation(obs) for obs in observations]
    
    # Filter by mission
    filtered = mission_filter.filter(decayed, request.mission)
    
    # Detect anomalies
    for obs in filtered:
        obs.status = await anomaly_detector.detect(obs)
    
    return {"observations": filtered}
```

**Target:** API running, can query with access control  
**Time:** 1 week  
**Test:** API tests

---

## Phase 3: Integration & Testing (Weeks 7-10)

### Week 7: End-to-End Setup

Create test scenario:

```python
# tests/end_to_end.py

async def test_security_bot_mission():
    """Test complete security patrol mission"""
    
    # Setup
    map_service = PyTerrainMap()
    ai_service = PyTerrainAI(map_service)
    
    # Simulate observations from exploration bot
    await map_service.push_observation(Observation(
        robot_id="explorer_1",
        location=GeoPoint(40.123, -74.567),
        sensor_type=SensorType.Thermal,
        value=22.0,
        confidence=0.95
    ))
    
    # Security bot queries for context
    context = await ai_service.get_context(
        bot_id="security_1",
        mission="security",
        location=GeoPoint(40.123, -74.567)
    )
    
    # Verify context is security-relevant
    assert context.observations  # Got data
    assert all(obs.decayed_confidence > 0)  # Decay applied
```

**Target:** End-to-end flow works  
**Time:** 1 week

---

### Week 8-9: Performance Testing

```python
# tests/benchmark.py

async def benchmark_storage():
    """Test observation ingestion speed"""
    map_service = PyTerrainMap()
    
    # Push 10K observations
    start = time.time()
    for i in range(10000):
        await map_service.push_observation(...)
    end = time.time()
    
    # Should be <1ms per observation
    assert (end - start) / 10000 < 0.001
```

**Target:** <1ms per observation, <50ms per query  
**Time:** 1 week

---

### Week 10: Documentation & Examples

Create example scenarios:

```python
# examples/police_surveillance.py
# examples/factory_inspection.py
# examples/agricultural_monitoring.py
```

**Target:** Users can understand system via examples  
**Time:** 1 week

---

## Phase 4: Advanced Features (Weeks 11-18)

### Weeks 11-12: Persistent Storage

Add SQLite/PostgreSQL support (optional, not MVP-blocking)

### Weeks 13-15: Image Stitching (PyNoramic)

Image registration, Structure from Motion

### Weeks 16-18: Production Hardening

- Error handling
- Logging
- Monitoring
- Deployment docs

---

## Phase 5: Ecosystem Integration (Weeks 19-26) — NEW

**Strategic integrations with PyRoboFrames and PyRoboVision to enhance terrain intelligence while maintaining architectural boundaries.**

### Weeks 19-20: PyRoboFrames Integration — Sensor Data Pipelines

**What this adds:** High-throughput multi-robot sensor ingest, temporal alignment, sensor composition tracking

**Priority:** P0 (unblocks real-world multi-robot use cases)

**Implementation:**
```python
# pyterrain_map/adapters/pyroboframes_adapter.py

from pyroboframes import RoboticsDataFrame
from pyterrain_map.observation import Observation, TemporalMetadata

class RoboticsDataFrameAdapter:
    """Convert PyRoboFrames RoboticsDataFrame to PyTerrainMap Observations"""
    
    async def ingest_dataframe(self, df: RoboticsDataFrame, robot_id: str) -> List[Observation]:
        """
        Transform aligned, normalized sensor data into georeferenced observations
        
        - Handles multi-rate sensor composition (10fps camera + 1fps probe + GPS)
        - Preserves sensor quality from PyRoboFrames
        - Tracks which sensors contributed to each observation
        - Maintains lineage (episode_id, source_dataset, frame_index)
        """
        observations = []
        
        for frame_idx, row in df.iterrows():
            obs = Observation(
                observation_id=f"{robot_id}_{frame_idx}",
                robot_id=robot_id,
                timestamp=row['timestamp_ns'],
                location=GeoPoint(
                    lat=row['gps_0_lat'],
                    lon=row['gps_0_lon']
                ),
                temporal_metadata=TemporalMetadata(
                    event_time=row['timestamp_ns'],
                    capture_times={
                        'camera_0': row['camera_0_timestamp_ns'],
                        'lidar_0': row['lidar_0_timestamp_ns'],
                        'imu_0': row['imu_0_timestamp_ns'],
                    },
                    quality={
                        'camera_0': row['quality']['camera_0'],
                        'lidar_0': row['quality']['lidar_0'],
                        'imu_0': row['quality']['imu_0'],
                    }
                ),
                sensor_values=self._extract_sensor_values(row),
                provenance={
                    'source_episode': df.metadata.get('episode_id'),
                    'source_dataset': df.metadata.get('dataset_name'),
                    'global_frame_index': frame_idx,
                }
            )
            observations.append(obs)
        
        return observations
```

**Dependencies to add:**
```toml
pyroboframes = "1.2.1"  # Requires PyRoboFrames in Python environment
```

**Success Criteria:**
- ✅ Can ingest MCAP/ROS2 streams via PyRoboFrames
- ✅ Multi-rate sensor alignment working (50-100ms windows)
- ✅ Temporal metadata preserved (capture times, quality per sensor)
- ✅ Provenance tracked (robot_id, episode_id, frame_index)
- ✅ 3 use case tests passing: Construction, Agricultural, Disaster

**Time:** 2 weeks  
**Tests:** Unit tests for adapter, end-to-end tests with 3 use cases

**Use Cases Validated:**
1. **Construction Site Inspection** — 3-drone MCAP streams with 4K RGB, thermal, LiDAR, barometer
2. **Agricultural Monitoring** — Multi-rate sensors (10fps RGB, 1fps hyperspectral, 0.1Hz soil probe)
3. **Security Surveillance** — 24/7 perimeter drone observations with 50ms temporal windows

---

### Weeks 21-22: PyRoboVision Integration — Vision Model Performance Tracking

**What this adds:** Terrain-specific vision model selection, performance degradation tracking, model-aware quality weighting

**Priority:** P0 (enables adaptive perception pipelines)

**Implementation:**
```python
# pyterrain_map/adapters/pyrobovision_adapter.py

from pyrobovision.registry import ModelRegistry
from pyterrain_map.fusion import FusionWeighting

class VisionModelAwareAdapter:
    """Embed PyRoboVision model registry into PyTerrainMap fusion"""
    
    def __init__(self, vision_registry: ModelRegistry):
        self.registry = vision_registry
    
    async def compute_fusion_weight(
        self,
        model_id: str,
        inference_confidence: float,
        sensor_quality: float,
        terrain_type: str
    ) -> float:
        """
        Compute model-aware fusion weight: 
        weight = model_mAP(terrain) × inference_confidence × sensor_quality
        """
        
        # Query model performance for this terrain
        model_perf = await self.registry.performance(
            model_id=model_id,
            terrain_type=terrain_type
        )
        
        if not model_perf:
            # Fallback: use inference confidence alone
            return inference_confidence * sensor_quality
        
        # Weight = registry mAP × inference confidence × sensor quality
        fusion_weight = (
            model_perf.mAP *
            inference_confidence *
            sensor_quality
        )
        
        return fusion_weight
    
    async def select_best_model(
        self,
        task: str,
        terrain_type: str,
        max_latency_ms: int = 100
    ) -> str:
        """Select best-performing model for terrain + task"""
        candidates = await self.registry.models_for_task(task)
        
        best_model = None
        best_score = 0
        
        for model_id in candidates:
            perf = await self.registry.performance(
                model_id=model_id,
                terrain_type=terrain_type
            )
            
            if perf.inference_latency_ms > max_latency_ms:
                continue  # Latency constraint violated
            
            # Score = mAP + (1 - normalized_latency)
            score = perf.mAP + (1.0 - perf.inference_latency_ms / 200)
            
            if score > best_score:
                best_model = model_id
                best_score = score
        
        return best_model
```

**Dependencies:**
```toml
pyrobovision = "1.2.1"  # Vision model registry
```

**Success Criteria:**
- ✅ Model registry lookups working (mAP by terrain)
- ✅ Fusion weights computed correctly (model mAP × confidence × sensor quality)
- ✅ Adaptive model selection by terrain type
- ✅ Model-terrain performance degradation tracked (shadows, weather)
- ✅ 3 use case tests passing: Disaster Response, Agriculture Yield, Tree Detection

**Time:** 2 weeks  
**Tests:** Unit tests for model adapter, end-to-end tests with 3 use cases

**Use Cases Validated:**
1. **Disaster Response** — 3-model ensemble (YOLOv11 RGB, thermal, LiDAR) with multi-model agreement tracking
2. **Agricultural Yield** — Adaptive model selection (wheat/corn/soy) with terrain-aware mAP
3. **Tree Detection** — Cross-terrain model comparison (forest vs. urban) with confidence bounds

---

### Weeks 23-24: Data Type Alignment & Provenance

**What this adds:** Unified data contracts across all 3 projects, end-to-end provenance tracking

**Priority:** P1 (enables future diagnostics with PyVectorHound)

**Implementation:**
```rust
// src/contracts/observation_contract.rs

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ObservationContract {
    /// From PyRoboFrames: sensor composition metadata
    pub temporal_metadata: TemporalMetadata,
    
    /// From PyRoboVision: model performance metadata
    pub vision_metadata: Option<VisionMetadata>,
    
    /// PyTerrainMap: fusion & quality gates
    pub fusion_result: FusionResult,
    
    /// Full provenance chain
    pub lineage: Lineage,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lineage {
    pub robot_id: String,
    pub episode_id: String,
    pub frame_index: u64,
    pub dataset_name: String,
    pub source_models: Vec<String>,
    pub source_sensors: Vec<String>,
    pub timestamp_created: i64,
}
```

**Success Criteria:**
- ✅ Data contracts enforceable at observation boundaries
- ✅ Full provenance chain (robot→episode→frame→models→sensors→fusion)
- ✅ Lineage queryable (reverse tracing for diagnostics)

**Time:** 2 weeks

---

### Weeks 25-26: Integration Testing & Documentation

**What this adds:** Production-ready integration tests, reference architectures

**Priority:** P0 (blocks release)

**Test Cases:**
1. **End-to-End Multi-Robot Survey**
   - PyRoboFrames: Ingest 3 MCAP streams (Drone A/B/C)
   - PyRoboVision: Run 3 models in parallel
   - PyTerrainMap: Fuse with model-aware weighting
   - Output: Georeferenced 3D model with confidence annotations

2. **Temporal Alignment Stress Test**
   - PyRoboFrames: Compose 1000 observations with variable sensor rates
   - PyTerrainMap: Verify temporal metadata preserved
   - Validate: All timestamps, quality scores intact

3. **Model Adaptation Scenario**
   - PyRoboVision: Switch from day model → night model based on time
   - PyTerrainMap: Re-weight observations accordingly
   - Validate: Model switch doesn't lose data

**Documentation:**
- Architecture diagrams (data flow through 3 projects)
- Example: Construction inspection workflow
- Example: Agricultural monitoring workflow
- Example: Disaster response workflow
- Troubleshooting guide (when models unavailable, when PyRoboFrames unavailable, graceful degradation)

**Time:** 2 weeks  
**Deliverables:**
- `ECOSYSTEM_INTEGRATION_GUIDE.md`
- `MULTI_ROBOT_SURVEY_EXAMPLE.py`
- `DISASTER_RESPONSE_EXAMPLE.py`
- 15+ integration tests

---

## Build Priorities (P0-P2 Classification)

### P0 (Blocking Release) — Weeks 1-10
1. PyTerrainMap core (storage, queries, fusion)
2. PyTerrainAI basics (RBAC, decay, anomaly detection)
3. Python bindings working
4. MVP success criteria met

### P0 (Phase 5: Ecosystem) — Weeks 19-24
1. **PyRoboFrames adapter** (weeks 19-20) — Unblocks real multi-robot ingest
2. **PyRoboVision adapter** (weeks 21-22) — Enables terrain-aware model selection
3. **Data type alignment** (weeks 23-24) — Ensures contract between projects
4. **Integration tests** (weeks 25-26) — Validates multi-project workflows

**Rationale:** Without these, PyTerrainMap can only ingest pre-normalized data. With them, PyTerrainMap becomes the central hub for multi-robot terrain intelligence.

### P1 (After MVP, Nice to Have) — Weeks 11-18, 27+
1. Persistent storage (SQLite/PostgreSQL)
2. Image stitching (Structure from Motion)
3. Advanced anomaly detection (ML-based)
4. Performance optimization (1M+ observations)

### P2 (Future Research)
1. Real-time model retraining based on observation feedback
2. Distributed terrain mapping (federated learning)
3. Historical terrain change detection
4. Cost optimization for multi-cloud deployment

---

## Quick Start Command Sequence

```bash
# Week 1-2: Create project structure
cd /Users/georgimullassery/PyTerrainMap
git checkout main

# Create source files
mkdir -p src/{spatial,storage,query,fusion}
touch src/types.rs src/lib.rs

# Week 2: Add dependencies
cargo add h3 uuid parking_lot tokio

# Week 3-4: Implement storage
# (Edit src/storage/mod.rs)

# Week 5-6: Python bindings
cargo add pyo3 pyo3-asyncio --build

# Week 5 (parallel): PyTerrainAI setup
cd /Users/georgimullassery/pyterrain-ai
mkdir -p pyterrain_ai/{access_control,temporal,anomaly,server}

# Test everything
cargo test
pytest tests/

# Build
maturin develop
```

---

## Success Criteria for MVP (Week 10)

✅ PyTerrainMap can store observations  
✅ PyTerrainMap can query by location/time  
✅ PyTerrainAI applies temporal decay  
✅ PyTerrainAI enforces RBAC  
✅ PyTerrainAI detects anomalies (basic z-score)  
✅ Python can call both services  
✅ HTTP API works  
✅ Performance: <1ms ingestion, <50ms query  
✅ Tests pass  
✅ Examples work  

---

## Dependencies Summary

**Rust (Cargo.toml):**
```toml
h3 = "0.11"
uuid = { version = "1.6", features = ["v4"] }
tokio = { version = "1.35", features = ["full"] }
parking_lot = "0.12"
pyo3 = { version = "0.20", features = ["extension-module"] }
pyo3-asyncio = { version = "0.20", features = ["tokio-runtime"] }
```

**Python (pyproject.toml):**
```
fastapi>=0.104
httpx>=0.24
pydantic>=2.0
numpy>=1.24
```

---

## Where to Start: This Week

1. **Today:** Create Cargo.toml structure, add dependencies
2. **Tomorrow:** Implement core types (Observation, GeoPoint, SensorType)
3. **This week:** H3 spatial indexing working
4. **Next week:** In-memory storage working
5. **Week 3:** Query API working

**First code file to write: `src/types.rs`**

Ready to start?
