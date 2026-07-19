# PyPanorama Architecture

## Overview

PyPanorama is a three-layer collaborative spatial intelligence system:

```
Layer 1: Python API (PyO3 bindings)
         ↓
Layer 2: Rust Core (Spatial engine, fusion, storage)
         ↓
Layer 3: Optional PyNoramic (Image stitching, SfM)
         ↓
Layer 4: Persistent Storage (SQLite, PostgreSQL, or in-memory)
```

## Core Data Model

### Observation (Atomic Unit)

```rust
pub struct Observation {
    pub id: Uuid,
    pub robot_id: String,
    pub timestamp: i64,  // microseconds since epoch
    
    // Location in 3D space
    pub location: GeoPoint,  // (lat, lon)
    pub elevation_asl: Option<f32>,  // above sea level
    pub elevation_agl: Option<f32>,  // above ground level
    
    // Sensor reading
    pub sensor_type: SensorType,
    pub value: SensorValue,  // Typed (Thermal, LiDAR, Camera, etc.)
    
    // Quality indicators
    pub confidence: f32,  // Device/sensor confidence (0.0-1.0)
    pub robot_state: RobotContext,  // Where was robot when sensing?
    
    // Metadata
    pub metadata: HashMap<String, String>,
}

pub enum SensorType {
    Thermal { celsius: f32 },
    LiDAR { distances_cm: Vec<u16> },
    Ultrasonic { distance_cm: u16 },
    Camera { detections: Vec<ObjectDetection> },
    Movement { velocity: f32, heading: f32 },
    Custom(String),
}
```

### Spatial Layer (Per Sensor Type)

```rust
pub struct SpatialLayer {
    pub sensor_type: SensorType,
    pub key: (H3Cell, ElevationBucket),  // (lat/lon bucket, elevation range)
    
    // Observations pooled at this location
    pub observations: Arc<Mutex<VecDeque<Observation>>>,
    
    // Aggregated understanding
    pub fused_view: Arc<Mutex<FusedData>>,
    
    // Temporal tracking
    pub temporal_trend: TemporalTrend,  // Rising, stable, falling, unknown
    pub temporal_validity: f32,  // 0.0-1.0 freshness score
    
    // Anomaly tracking
    pub baseline_stats: Option<BaselineStatistics>,
    pub change_score: f32,  // 0.0-1.0 how different from baseline
    
    pub last_updated: i64,
}

pub struct FusedData {
    pub temperature_estimate: Option<TemperatureEstimate>,
    pub obstacle_map: Option<ObstacleMap>,
    pub object_detections: Vec<FusedDetection>,
    pub movement_activity: ActivityLevel,
    pub confidence: f32,
}
```

### Composite Context (Query Response)

```rust
pub struct CompositeContext {
    pub location: GeoPoint,
    pub elevation_range: (f32, f32),
    pub timestamp_query: i64,
    
    // What we know about this location
    pub thermal_summary: Option<TemperatureEstimate>,
    pub obstacle_map: Option<ObstacleMap>,
    pub detected_objects: Vec<FusedDetection>,
    pub activity_level: ActivityLevel,
    
    // Temporal insights
    pub temporal_trends: Vec<String>,  // "Temperature rising", "New obstacle"
    pub time_since_observation: i64,
    pub observation_count: u32,
    
    // Guidance for next robot
    pub suggested_focus_areas: Vec<(GeoPoint, String)>,
    pub missing_sensor_layers: Vec<SensorType>,
    pub confidence: f32,
}
```

## Spatial Indexing (H3 Hierarchical)

### Why H3?

1. **Hierarchical** — Zoom in/out naturally (resolution 0 = Earth, 15 = 1m²)
2. **Consistent cell size** — No artifacts from axis-aligned grids
3. **Ring queries** — Find neighbors efficiently
4. **Geographic native** — Works with lat/lon natively

### Elevation Bucketing

```rust
pub struct ElevationBucket {
    pub min_m: f32,
    pub max_m: f32,  // Usually 1m or 2m buckets
}

// Key for a spatial layer
pub type SpatialKey = (H3Cell, ElevationBucket);

// Example: Factory floor
// Observation at (40.123, -74.567, 1.5m) maps to:
// SpatialKey(
//   H3Cell::from_lat_lon(40.123, -74.567, resolution=9),
//   ElevationBucket(1.0, 2.0)  // 1-2m bucket
// )
```

## Temporal Decay Functions

### Exponential Decay

```rust
pub fn temporal_decay(age_seconds: i64, half_life: i64) -> f32 {
    let decay_rate = 0.693 / half_life as f32;
    (-decay_rate * age_seconds as f32).exp()
}

// Example: thermal sensor, 2-hour half-life
// At T=0: decay = 1.0 (full value)
// At T=2h: decay = 0.5
// At T=4h: decay = 0.25
// At T=8h: decay = 0.0625
```

### Observation Age Weighting

```rust
pub fn weighted_observation(obs: &Observation) -> (f32, f32) {
    let age = (now() - obs.timestamp);
    let temporal_weight = temporal_decay(age, HALF_LIFE);
    let final_weight = obs.confidence * temporal_weight;
    
    (obs.value, final_weight)
}
```

## Sensor Fusion

### Temperature Fusion (Weighted Average)

```rust
pub fn fuse_temperatures(observations: &[Observation]) -> TemperatureEstimate {
    let (sum, weight_sum): (f32, f32) = observations
        .iter()
        .filter_map(|o| {
            let (temp, weight) = weighted_observation(o);
            Some((temp * weight, weight))
        })
        .fold((0.0, 0.0), |(s, w), (t, wt)| (s + t, w + wt));
    
    let mean = sum / weight_sum;
    
    // Variance: how consistent are readings?
    let variance = observations
        .iter()
        .map(|o| {
            let (temp, weight) = weighted_observation(o);
            weight * (temp - mean).powi(2)
        })
        .sum::<f32>() / weight_sum;
    
    TemperatureEstimate {
        celsius: mean,
        variance,
        num_readings: observations.len() as u32,
    }
}
```

### Obstacle Fusion (Bayesian Grid)

```rust
pub fn update_occupancy_grid(
    grid: &mut OccupancyGrid,
    observation: &Observation,
    sensor_confidence: f32,
) {
    // Each sensor observation updates probability of occupancy
    // P(occupied | obs) ∝ P(obs | occupied) * P(occupied)
    
    for distance in lidar_distances {
        let grid_cell = gps_to_grid(distance, observation.location);
        
        // Bayesian update
        let prior = grid[grid_cell];
        let likelihood = sensor_confidence;
        let posterior = bayesian_update(prior, likelihood);
        
        grid[grid_cell] = posterior;
    }
}
```

### Detection Fusion (Ensemble Voting + NMS)

```rust
pub fn fuse_detections(
    observations: &[Observation],
    nms_threshold: f32,
) -> Vec<FusedDetection> {
    // 1. Group detections by class
    let by_class = group_by_class(observations);
    
    // 2. For each class, ensemble voting
    let mut fused = Vec::new();
    for (class, detections) in by_class {
        let avg_confidence = detections.iter()
            .map(|d| d.confidence)
            .sum::<f32>() / detections.len() as f32;
        
        // 3. Apply Non-Maximum Suppression (remove duplicates)
        let suppressed = nms(&detections, nms_threshold);
        
        fused.push(FusedDetection {
            class_label: class,
            avg_confidence,
            num_detections: detections.len() as u32,
            bbox_ensemble: ensemble_bbox(&detections),
        });
    }
    
    fused
}
```

## Concurrency Model

### Thread-Safe Storage

```rust
pub struct PyPanorama {
    // Multiple robots writing concurrently
    layers: Arc<RwLock<
        HashMap<SpatialKey, SpatialLayer>
    >>,
    
    // Temporal index for range queries
    observations_by_time: Arc<RwLock<
        BTreeMap<i64, Vec<ObservationRef>>
    >>,
    
    // Cache for frequently accessed regions
    context_cache: Arc<RwLock<
        HashMap<SpatialKey, CompositeContext>
    >>,
    
    // Atomic metrics
    total_observations: Arc<AtomicU64>,
    last_update_ts: Arc<AtomicI64>,
}

// Write path: O(log N) for hashmap lookup + append
pub async fn push_observation(&self, obs: Observation) -> Result<()> {
    let key = spatial_key(&obs);
    
    let mut layers = self.layers.write().await;
    let layer = layers.entry(key)
        .or_insert_with(|| SpatialLayer::new(key));
    
    layer.observations.lock().await.push_back(obs.clone());
    
    // Invalidate cache for this location
    self.context_cache.write().await.remove(&key);
    
    Ok(())
}

// Read path: fast (cache hit common)
pub async fn query(&self, location: GeoPoint, ...) -> Result<CompositeContext> {
    let key = spatial_key_from_location(&location);
    
    // Check cache first
    if let Some(cached) = self.context_cache.read().await.get(&key) {
        return Ok(cached.clone());
    }
    
    // Cache miss: compute
    let context = self.compute_context(&key).await?;
    
    // Store in cache
    self.context_cache.write().await.insert(key, context.clone());
    
    Ok(context)
}
```

## Query Engine

### Spatial Range Query

```rust
pub async fn query(
    &self,
    location: GeoPoint,
    radius_m: f32,
    elevation_range: (f32, f32),
    sensor_types: Option<Vec<SensorType>>,
    max_age: Option<i64>,
) -> Result<CompositeContext> {
    // 1. Expand geo region to H3 cells
    let h3_cells = h3::k_ring(
        h3::from_lat_lon(location.lat, location.lon, RESOLUTION),
        ring_radius_cells(radius_m)
    );
    
    // 2. For each H3 cell, find layers matching elevation
    let matching_keys: Vec<SpatialKey> = h3_cells
        .iter()
        .cartesian_product([elevation_range])
        .map(|(cell, elev)| (cell, elev))
        .collect();
    
    // 3. Fetch observations from layers
    let layers = self.layers.read().await;
    let mut all_observations = Vec::new();
    
    for key in matching_keys {
        if let Some(layer) = layers.get(&key) {
            let obs = layer.observations.lock().await;
            
            // Filter by sensor type and age
            for o in obs.iter() {
                if sensor_types.is_none() || sensor_types.as_ref().unwrap().contains(&o.sensor_type) {
                    let age = now() - o.timestamp;
                    if max_age.is_none() || age < max_age.unwrap() {
                        all_observations.push(o.clone());
                    }
                }
            }
        }
    }
    
    // 4. Fuse observations into context
    self.synthesize_context(all_observations).await
}
```

## Anomaly Detection

### Baseline Tracking

```rust
pub struct BaselineStatistics {
    pub temperature_mean: f32,
    pub temperature_std: f32,
    pub typical_obstacles: ObstacleMap,
    pub typical_activity_level: ActivityLevel,
    pub observation_count: u32,
}

pub fn update_baseline(
    baseline: &mut BaselineStatistics,
    current: &FusedData,
) {
    // Running mean/variance update (Welford's method)
    if let Some(temp) = &current.temperature_estimate {
        let n = baseline.observation_count as f32;
        let delta = temp.celsius - baseline.temperature_mean;
        baseline.temperature_mean += delta / (n + 1.0);
        
        let delta2 = temp.celsius - baseline.temperature_mean;
        baseline.temperature_std = (
            (baseline.temperature_std.powi(2) * n + delta * delta2) / (n + 1.0)
        ).sqrt();
    }
    
    baseline.observation_count += 1;
}

pub fn detect_anomalies(
    baseline: &BaselineStatistics,
    current: &FusedData,
) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    
    // Temperature z-score
    if let Some(temp) = &current.temperature_estimate {
        let z_score = (temp.celsius - baseline.temperature_mean).abs() 
            / (baseline.temperature_std + 1e-6);
        
        if z_score > 2.0 {  // 95% confidence threshold
            anomalies.push(Anomaly::TemperatureOutlier {
                value: temp.celsius,
                z_score,
                expected: baseline.temperature_mean,
            });
        }
    }
    
    // Obstacle changes
    let new_obstacles = grid_difference(&current.obstacle_map, &baseline.typical_obstacles);
    if !new_obstacles.is_empty() {
        anomalies.push(Anomaly::NewObstacles { count: new_obstacles.len() });
    }
    
    anomalies
}

pub fn compute_change_score(anomalies: &[Anomaly]) -> f32 {
    // Aggregate anomaly count into 0.0-1.0 score
    let severity: f32 = anomalies.iter()
        .map(|a| match a {
            Anomaly::TemperatureOutlier { z_score, .. } => z_score.min(5.0) / 5.0,
            Anomaly::NewObstacles { count } => (count as f32).min(10.0) / 10.0,
            _ => 0.1,
        })
        .sum();
    
    (severity / anomalies.len().max(1) as f32).min(1.0)
}
```

## Python Bindings (PyO3)

### Rust to Python Bridge

```rust
use pyo3::prelude::*;

#[pyclass]
pub struct PyPanorama {
    inner: Arc<PanoramaCore>,
}

#[pymethods]
impl PyPanorama {
    #[new]
    fn new(config: Option<&str>) -> PyResult<Self> {
        let core = PanoramaCore::new(config)?;
        Ok(PyPanorama { inner: Arc::new(core) })
    }
    
    fn push_observation(&self, py: Python, obs: &PyAny) -> PyResult<PyObject> {
        let obs = extract_observation(obs)?;
        
        let core = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            core.push_observation(obs).await?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }
    
    fn query(&self, py: Python, location: &PyAny, ...) -> PyResult<PyObject> {
        let loc = extract_geopoint(location)?;
        
        let core = self.inner.clone();
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let context = core.query(loc, ...).await?;
            Ok(convert_context_to_python(context, py))
        })
    }
}
```

## Performance Characteristics

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Push observation | O(log N) | Hashmap lookup + append |
| Query context | O(H) | H = number of H3 cells in radius |
| Temporal decay | O(1) | Lazy on read, not on write |
| Sensor fusion | O(M) | M = observations in location |
| Anomaly detection | O(M) | Baseline comparison |

### Space Complexity

| Storage | Size | Notes |
|---------|------|-------|
| In-memory | ~100KB | Per 100 observations |
| Per observation | ~1KB | Includes raw data |
| Cache entry | ~10KB | Fused data + context |
| Temporal index | ~100 bytes | Per observation |

## Extensibility

### Custom Layers

Users can define custom sensor layers in config:

```yaml
custom_layers:
  - name: "threat_score"
    type: "numeric"
    fusion_method: "weighted_average"
    temporal_decay_hours: 12
  
  - name: "crop_health"
    type: "numeric"
    fusion_method: "gaussian_blend"
    temporal_decay_hours: 168
```

### Custom Fusion Functions

```python
def my_fusion_fn(observations):
    """Custom fusion for threat scoring"""
    scores = [o.value for o in observations if o.sensor_type == "threat_detector"]
    if not scores:
        return None
    
    # Exponential average (recent observations weighted more)
    import numpy as np
    weights = np.exp(np.arange(len(scores)) / len(scores))
    return np.average(scores, weights=weights)

map_service.register_fusion("threat_score", my_fusion_fn)
```

---

For implementation details, see source code in `src/` directory.
