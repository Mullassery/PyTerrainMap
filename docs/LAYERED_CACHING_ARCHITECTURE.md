# Layered Caching and Progressive World Understanding

## Foundational Principle

Most places, objects, and environments have already been observed many times before. **Agents should not be required to reprocess entire observation history before operating effectively.**

Understanding should be **progressive, not all-or-nothing**. Start with the best available summary. Load deeper information only when needed.

## The Problem: Scalability at Billions of Observations

A single geographic location may contain:

- **Millions of sensor observations** from multiple robots over months/years
- **Years of historical telemetry** (position, orientation, sensor readings)
- **Images and videos** from multiple cameras
- **3D terrain models** (SLAM, photogrammetry reconstructions)
- **Map layers** (OSM, SRTM, satellite imagery)
- **Prior agent observations** (decision logs, paths, plans)
- **AI-generated annotations** (detections, classifications, segmentations)
- **User-contributed information** (annotations, corrections, preferences)

**Current approach:** Load all available data before making decisions
- ❌ Not practical: terabytes per location
- ❌ Not scalable: cannot serve billions of locations
- ❌ Not responsive: seconds to minutes latency
- ❌ Not necessary: most decisions don't need full history

**Solution:** Layered knowledge representation with progressive retrieval

---

## Layered World Knowledge

Represent information in hierarchical layers of increasing detail and specificity:

### **Layer 0: Summary (KB range)**

Extracted essence of everything known about this location.

```json
{
  "location": "Central Park, New York",
  "primary_characteristics": [
    "Urban park",
    "Public access",
    "High pedestrian traffic"
  ],
  "area_km2": 3.4,
  "last_updated_s": 300,
  "confidence": 0.98
}
```

**Use cases:** Quick context, global queries, initial agent briefing

### **Layer 1: Important Facts (100 KB range)**

Curated facts necessary for basic reasoning and planning.

```json
{
  "static_features": [
    { "name": "walking_paths", "count": 45 },
    { "name": "water_bodies", "count": 3 },
    { "name": "playgrounds", "count": 8 }
  ],
  "dynamic_features": [
    { "name": "pedestrian_density", "value": "high", "time_s": 120 },
    { "name": "weather", "condition": "clear", "temp_c": 22 }
  ],
  "restrictions": ["No vehicle access", "Leash required for dogs"],
  "hazards": ["Construction at west entrance", "Slippery grass after rain"],
  "recent_events": [
    { "type": "event", "description": "Concert 2 hours ago", "age_s": 7200 }
  ]
}
```

**Use cases:** Route planning, mission feasibility, quick assessment

### **Layer 2: Local Context (1-10 MB range)**

Detailed regional information sufficient for effective operation.

```json
{
  "terrain_zones": [
    {
      "region": "H3_cell_8f...",
      "elevation_m": 40.5,
      "slope_pct": 3.2,
      "soil_type": "loamy",
      "vegetation": ["grass", "trees"],
      "traversability": 0.95
    }
  ],
  "recent_observations": {
    "last_24h": 1523,
    "last_7d": 8945,
    "sensor_types": ["camera", "lidar", "gps"]
  },
  "obstacle_map": {
    "static_obstacles": [...],
    "dynamic_obstacles": [...]
  },
  "behavioral_patterns": {
    "pedestrian_flows": [...],
    "vehicle_patterns": [...]
  }
}
```

**Use cases:** Navigation, obstacle avoidance, behavior prediction

### **Layer 3: Detailed Observations (100 MB - 1 GB range)**

Individual sensor observations and derived features.

```json
{
  "observations": [
    {
      "id": "obs-12345",
      "type": "camera",
      "timestamp": 1721431351000000,
      "data_reference": "s3://bucket/obs-12345.jpg",
      "features": {
        "detected_objects": [...],
        "terrain_classification": [...],
        "embeddings": [...]
      }
    },
    ...
  ],
  "derived_features": {
    "point_cloud": "ply://...",
    "depth_map": "npz://...",
    "feature_vectors": "hdf5://..."
  }
}
```

**Use cases:** Forensic analysis, model retraining, detailed reconstruction

### **Layer 4: Raw Historical Data (GB - TB range)**

Complete observation history, raw telemetry, unprocessed sensor data.

```json
{
  "observations": [all observations in chronological order],
  "telemetry_streams": [all robot telemetry],
  "raw_sensor_data": [all unprocessed data],
  "version_history": [all historical versions]
}
```

**Use cases:** Scientific analysis, accident reconstruction, system auditing

---

## Progressive Retrieval Strategy

**Principle:** Retrieve information based on specific need, not by default.

```rust
pub enum InformationNeed {
    BasicContext,           // Layer 0
    PlanningDecision,       // Layer 0-1
    RouteOptimization,      // Layer 1-2
    ObstacleAvoidance,      // Layer 2
    TrajectoryTracking,     // Layer 2-3
    FeatureExtraction,      // Layer 3
    HistoricalAnalysis,     // Layer 3-4
    ForensicReconstruction, // Layer 4
}

impl LocationCache {
    pub fn get_for_need(&self, need: InformationNeed) -> CachedData {
        match need {
            InformationNeed::BasicContext => self.get_layer(0),
            InformationNeed::PlanningDecision => self.get_layers(0..=1),
            InformationNeed::RouteOptimization => self.get_layers(1..=2),
            InformationNeed::ObstacleAvoidance => self.get_layers(2..=2),
            InformationNeed::TrajectoryTracking => self.get_layers(2..=3),
            InformationNeed::FeatureExtraction => self.get_layers(3..=3),
            InformationNeed::HistoricalAnalysis => self.get_layers(3..=4),
            InformationNeed::ForensicReconstruction => self.get_layers(4..=4),
        }
    }
}
```

**Latency implications:**
- Basic context: <10 ms
- Planning decision: 50-100 ms
- Route optimization: 100-500 ms
- Obstacle avoidance: 200-1000 ms
- Forensic reconstruction: hours (batch processing)

---

## Multi-Level Cache Hierarchy

Cache information at different proximity levels to computation:

```
                    L1: Computation Layer
                    (GPU/CPU working set)
                           ↑
                    L2: Regional Cache
                    (Hot locations)
                           ↑
                    L3: Location Cache
                    (Observed area)
                           ↑
                    L4: Global Cache
                    (Persistent storage)
                           ↑
                    L5: Archive
                    (Historical data)
```

### **L1: Computation Cache (MB)**
- GPU-resident tensors for immediate inference
- CPU working set for active agent
- Eviction: LRU when full

### **L2: Regional Cache (100 MB - 1 GB)**
- Nearby locations (H3 neighbors)
- Recent observations (last 24 hours)
- Eviction: LRU by access time

### **L3: Location Cache (1-10 GB per location)**
- Complete knowledge for actively observed areas
- Indexed for fast spatial/temporal queries
- Eviction: Time-based (older observations → archive)

### **L4: Global Cache (Persistent)**
- SQLite/PostgreSQL or distributed database
- Organized by region, time, type
- Compression for storage efficiency
- Eviction: Archive policy (e.g., compress after 30 days, delete after 1 year)

### **L5: Archive (Cold Storage)**
- S3/Cloud storage for historical data
- Compressed, deduplicated
- Retrieval: Hours to days

---

## Semantic Summarization

Instead of storing raw observations, cache extracted meaning.

### **Aggregation**

```rust
pub struct LocationSummary {
    /// Aggregate statistics
    pub terrain_stats: TerrainStatistics,
    pub obstacle_density: f32,
    pub pedestrian_traffic: TrafficPattern,
    
    /// Extracted features
    pub dominant_terrain_types: Vec<(TerrainType, f32)>,
    pub common_obstacles: Vec<ObstacleType>,
    pub hazard_zones: Vec<HazardZone>,
    
    /// Temporal patterns
    pub daily_activity_cycles: Vec<TimeWindow>,
    pub seasonal_variations: Vec<SeasonalPattern>,
    
    /// Confidence and age
    pub created_at_us: i64,
    pub updated_at_us: i64,
    pub confidence: f32,
}
```

### **Compression**

```
1,000,000 raw observations (5 TB)
    ↓
Aggregate by region, time window
    ↓
Extract features (terrain, obstacles, patterns)
    ↓
Compress with codec (zstd)
    ↓
100 MB summary (50,000x compression)
```

### **Example**

**Raw observations (5 TB):**
- 1M camera images (each 5 MB)
- 1M point clouds (each 10 MB)
- 1M sensor readings

**Compressed summary (100 MB):**
```json
{
  "terrain": {
    "mean_elevation": 40.5,
    "std_elevation": 2.3,
    "slope_distribution": {...},
    "traversability": 0.92
  },
  "obstacles": {
    "total_detected": 234,
    "types": {"tree": 120, "rock": 50, "water": 20, ...},
    "density_per_km2": 68.8
  },
  "activity": {
    "observation_count": 1000000,
    "unique_agents": 45,
    "first_observation": "2024-01-01T00:00:00Z",
    "last_observation": "2024-07-19T12:00:00Z"
  }
}
```

---

## Cache-Aware Agents

Agents should understand cache quality and decide whether fresh data is needed.

```rust
pub struct CacheQuality {
    /// How old is this information?
    pub age_s: u32,
    
    /// How confident are we in this?
    pub confidence: f32,
    
    /// What layers are currently cached?
    pub cached_layers: Vec<u8>,
    
    /// Has anything changed since last update?
    pub change_detected: bool,
}

pub trait Agent {
    fn decide(&self, location: Location, cache_quality: CacheQuality) -> Decision {
        // Can I make a decision with current cache quality?
        if cache_quality.age_s < 60 && cache_quality.confidence > 0.9 {
            // Fresh and confident—use cache
            self.decide_from_cache(location, cache_quality)
        } else if cache_quality.age_s < 3600 && cache_quality.confidence > 0.7 {
            // Somewhat stale but acceptable for most decisions
            self.decide_from_cache(location, cache_quality)
        } else {
            // Cache is too old or confidence is low
            // Request fresh observations
            self.request_fresh_observations(location)
        }
    }
}
```

---

## Incremental Refinement

World understanding improves continuously as new observations arrive.

```
Initial Knowledge
(from cache)
    ↓ 
Cached Summary
(Layer 0-1)
    ↓
Recent Observations
(Layer 2, last 24h)
    ↓
Live Sensor Data
(real-time updates)
    ↓
Updated Cache
(automatic refresh)
    ↓
Refined World Model
```

### **Update Strategy**

```rust
pub struct CacheUpdateStrategy {
    /// When to update Layer 0 summary
    pub layer0_refresh_interval_s: u32,  // 5 minutes
    
    /// When to update Layer 1 facts
    pub layer1_refresh_interval_s: u32,  // 30 minutes
    
    /// When to expire Layer 2 local context
    pub layer2_expiry_s: u32,            // 24 hours
    
    /// When to archive Layer 3 observations
    pub layer3_archive_s: u32,           // 30 days
}

impl CacheManager {
    pub async fn update_cache(&self, location: Location) {
        // Check what needs refreshing
        if self.layer0_stale(location) {
            self.refresh_layer0_summary(location).await;
        }
        
        if self.layer1_stale(location) {
            self.refresh_layer1_facts(location).await;
        }
        
        // Expire old data
        if self.layer2_expired(location) {
            self.expire_layer2(location).await;
        }
        
        // Archive even older data
        if self.layer3_expired(location) {
            self.archive_layer3(location).await;
        }
    }
}
```

---

## Cache Invalidation

Detect when information changes and requires refresh.

```rust
pub enum InvalidationReason {
    TimeExpired,            // Age threshold exceeded
    NewObservations,        // Fresh data contradicts cache
    SignificantChange,      // Detected substantial difference
    ManualInvalidation,     // User/admin requested refresh
    HighErrorRate,          // Predictions from cache had high error
}

pub struct CacheInvalidationPolicy {
    /// Trigger refresh if this many new observations received
    pub observation_threshold: usize,
    
    /// Trigger refresh if change magnitude > threshold
    pub change_magnitude_threshold: f32,
    
    /// Trigger refresh if prediction error > threshold
    pub error_threshold: f32,
}

impl CacheManager {
    pub async fn check_validity(&self, location: Location) -> InvalidationReason {
        if self.is_expired(location) {
            return InvalidationReason::TimeExpired;
        }
        
        if self.new_observations_exceed_threshold(location).await {
            return InvalidationReason::NewObservations;
        }
        
        if let Some(change) = self.detect_significant_change(location).await {
            return InvalidationReason::SignificantChange;
        }
        
        // ... check other conditions
        
        InvalidationReason::None  // Still valid
    }
}
```

---

## Scalability Principle

**Most decisions require summaries. Some require details. Very few require all data.**

```
Decision Type                Layers Needed    Typical Decisions
─────────────────────────────────────────────────────────────
Context awareness            Layer 0          "What is this place?"
Route planning              Layer 0-1         "Can I get there?"
Path optimization           Layer 1-2         "What's the best route?"
Obstacle avoidance          Layer 2           "What's in my way?"
Forensic analysis           Layer 3-4         "What happened?"

                            Amount of Data
Layer 0 (Summary):          < 1 KB
Layer 1 (Facts):            100 KB
Layer 2 (Context):          10 MB
Layer 3 (Observations):     1 GB
Layer 4 (Raw):              100 GB

Distribution:
• 70% of queries use only Layer 0
• 20% of queries use Layers 0-1
• 5% of queries use Layers 1-2
• 4% of queries use Layers 2-3
• 1% of queries use Layers 3-4
```

This distribution enables:
- **99% fast responses** (<100ms) using cached summaries
- **99.95% queries served** from cache without hitting persistent storage
- **Scalability** to billions of observations without memory explosion

---

## Integration with PyTerrainMap

### **Extend ObservationStore**

```rust
impl ObservationStore {
    /// Sequential query (current)
    pub fn query(&self, request: QueryRequest) -> Vec<Observation> { ... }
    
    /// Query with progressive retrieval (new)
    pub fn query_progressive(
        &self,
        request: QueryRequest,
        need: InformationNeed,
    ) -> CachedResult { ... }
    
    /// Get location summary (Layer 0)
    pub fn get_location_summary(&self, location: GeoPoint) -> LocationSummary { ... }
    
    /// Get region cache (Layers 0-2)
    pub fn get_region_cache(&self, region: RegionId) -> RegionCache { ... }
}
```

### **Extend Query Engine**

```rust
impl Query {
    /// Route to appropriate cache layer
    pub fn execute_with_cache(&self) -> QueryResult {
        // Determine information need
        let need = self.estimate_information_need();
        
        // Get appropriate cache layers
        let cache = self.cache_manager.get_for_need(need);
        
        // Execute query against cache
        // Fall through to persistent storage if needed
        self.execute_against_cache(&cache)
    }
}
```

### **Extend Temporal Index**

```rust
impl TemporalIndex {
    /// Query with temporal caching
    pub fn range_query_cached(&self, start: i64, end: i64) -> TemporalWindow {
        // Check if window is in Layer 2 cache (last 24h)
        if within_cache_window(start, end) {
            return self.get_from_layer2_cache(start, end);
        }
        
        // Otherwise load from persistent storage
        self.range_query(start, end)
    }
}
```

---

## Implementation Roadmap

### **Phase 1 (Week 21-22): Cache Foundation**
- [ ] Cache layer abstraction (0-4)
- [ ] Cache manager with eviction policies
- [ ] Simple LRU + time-based eviction
- [ ] Layer 0 (summary) generation
- [ ] Layer 1 (facts) curation

### **Phase 2 (Week 23-24): Semantic Summarization**
- [ ] LocationSummary type and generation
- [ ] Aggregation engine (terrain, obstacles, patterns)
- [ ] Compression (zstd, quantization)
- [ ] Layer 2-3 summarization

### **Phase 3 (Week 25-26): Cache-Aware Agents**
- [ ] CacheQuality tracking
- [ ] Agent decision-making on cache freshness
- [ ] Automatic fresh observation requests
- [ ] Confidence-based cache usage

### **Phase 4 (Week 27-28): Invalidation and Updates**
- [ ] Change detection
- [ ] Incremental update strategy
- [ ] Cache invalidation policies
- [ ] Background refresh workers

### **Phase 5 (Week 29+): Optimization**
- [ ] Performance tuning
- [ ] Memory profiling
- [ ] Large-scale testing (billions of observations)
- [ ] Distributed cache coordination

---

## Performance Targets

| Operation | Without Cache | With L1-L2 Cache | Improvement |
|-----------|---------------|-----------------|-------------|
| Basic context query | 500ms | 10ms | 50x |
| Planning decision | 2000ms | 100ms | 20x |
| Route optimization | 5000ms | 500ms | 10x |
| Obstacle avoidance | 3000ms | 200ms | 15x |
| Real-time navigation | 4000ms | 50ms | 80x |

---

## Guiding Philosophy

An agent should be able to begin reasoning immediately using the best available summary, while deeper information is loaded only when needed.

This enables:

1. **Responsiveness**: Quick decisions from cached summaries
2. **Scalability**: Billions of observations without memory explosion
3. **Efficiency**: Only fetch what's actually needed
4. **Progressive understanding**: Continuously refine knowledge
5. **Flexibility**: Different needs get different layers
6. **Intelligence**: Agents understand cache quality and act accordingly

The system treats cached summaries as first-class citizens, not second-class approximations. A good summary is often sufficient—and always faster than raw data.
