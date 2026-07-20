# PyTerrainMap: Temporal Integration Guide

## Overview

PyTerrainMap Phase 3 introduces comprehensive temporal normalization, enabling reasoning about observations that arrive out-of-order or with significant latency. The system treats **time as a first-class coordinate** (5 dimensions + clock source + quality metrics) alongside spatial coordinates.

## Core Concepts

### Five Temporal Dimensions

Each observation carries five independent time coordinates:

1. **Event Time** (`event_time_us`) - When the event actually occurred in reality (ground truth)
2. **Capture Time** (`capture_time_us`) - When the sensor physically captured the event
3. **Transmission Time** (`transmission_time_us`) - When data left the robot (may batch or buffer)
4. **Ingestion Time** (`ingestion_time_us`) - When observation entered backend storage
5. **Processing Time** (`processing_time_us`) - When observation was indexed/analyzed

**Typical flow**: Event → Capture (10ms) → Transmission (100ms) → Ingestion (500ms) → Processing (600ms)

### Watermarking

The system maintains a **watermark** (maximum event_time seen) to detect and handle late arrivals:

- Observations with `event_time < watermark` are **late arrivals**
- Late arrivals trigger cache invalidation and query reprocessing
- Watermark never regresses (monotonic guarantee)

### Temporal Quality

Observations are weighted by **temporal quality** (0.0-1.0), computed from latency:

```
latency = ingestion_time - event_time

Quality:
- < 100ms latency   → quality 1.0 (excellent)
- 100-5000ms        → linear interpolation
- > 5000ms latency  → quality 0.3 (poor)
```

**Sensor fusion uses**: `weight = sensor_weight × confidence × temporal_quality`

## 12 Supported Clock Sources

PyTerrainMap is aware of regional GNSS coverage:

| Clock Source | Region | Typical Accuracy |
|---|---|---|
| GPS | Global | ±100ms |
| NavIC | India | ±100ms |
| Galileo | Europe | ±50ms |
| GLONASS | Russia/Former USSR | ±100ms |
| BeiDou | China/Belt & Road | ±100ms |
| IMU (accelerometer) | Global | ±500ms (drifts) |
| Camera frame timer | Global | ±33ms (assumes 30fps) |
| LiDAR timestamp | Global | ±50ms |
| System clock (NTP) | Global | ±1s (if unlocked) |
| UTC (manual) | N/A | ±1s |
| NTP (network) | Global | ±100ms |
| PTP (precision time protocol) | High-precision networks | ±1µs |

**API**: `crate::types::ClockSource` enum with `is_reliable()` and `typical_accuracy_ms()` methods.

## TemporalIndex: Event-Time Ordering

The enhanced temporal index maintains observations sorted by `event_time` (not arrival order):

```rust
use crate::temporal::TemporalIndexEnhanced;

let mut index = TemporalIndexEnhanced::new(DecayFunction::Exponential { half_life_ms: 60_000 });

// Insert observations (event_time_us, arrival_time_us)
let is_late = index.insert(1_000_000, 1_100_000)?;  // Normal
let is_late = index.insert(3_000_000, 3_100_000)?;  // Normal
let is_late = index.insert(2_000_000, 5_000_000)?;  // LATE: event_time < watermark

// Query by event time
let results = index.range_query(1_000_000, 3_000_000)?;

// Check temporal quality
let quality = index.temporal_quality(obs_index)?;  // 0.0-1.0

// Detect late arrivals
let late_indices = index.late_arrivals();
```

## Late-Arrival Reprocessing

When a late arrival is detected:

1. **Impact Analysis** - Compute affected spatial regions
2. **Cache Invalidation** - Mark regions for refresh
3. **Query Reprocessing** - Rerun affected spatial-temporal queries
4. **Anomaly Update** - Recalculate with new temporal context

```rust
use crate::late_arrival::LateArrivalProcessor;

let mut processor = LateArrivalProcessor::new();

// Process a late arrival
let affected_cells = processor.process_late_arrival(&obs, watermark_us)?;

// Get reprocessing work
let regions = processor.get_affected_regions();
let queries = processor.get_affected_queries();

// Estimate cost
let cost_ms = processor.estimate_reprocess_cost_ms();  // ~1ms per region + 10ms per query

// After reprocessing
processor.mark_reprocessing_done(current_time_us);
processor.clear_affected();  // Reset for next batch
```

## Sensor Fusion with Temporal Quality

The fusion engine automatically incorporates temporal quality:

```rust
use crate::fusion::SensorFusion;

let mut fusion = SensorFusion::default();
fusion.set_temporal_quality_enabled(true);  // Default: enabled

// Observations with better temporal quality get higher weight
let fused = fusion.fuse(&[&obs1, &obs2, &obs3])?;

// Result: temperature averaged with temporal-quality-weighted confidence
```

**Example**: 
- `obs1`: temp=20.0°C, confidence=0.9, latency=50ms (quality=1.0) → weight=0.9×1.0=0.9
- `obs2`: temp=19.5°C, confidence=0.9, latency=2500ms (quality~0.65) → weight=0.9×0.65=0.585
- Fused mean = (20.0×0.9 + 19.5×0.585) / (0.9 + 0.585) ≈ 19.8°C

## Performance Characteristics

### Latency (p99)
- **TemporalIndex insert** (10k observations): <1ms
- **Temporal quality calculation**: <0.1ms per observation
- **Late-arrival detection**: <1ms
- **Sensor fusion with temporal weighting**: <5ms for 10 observations
- **TerrainAnalysis with timestamps**: <5ms

### Memory (per 10k observations)
- **TemporalIndexEnhanced**: ~50KB (event/arrival times + watermark)
- **LateArrivalProcessor**: ~10KB (affected regions/queries)
- **Temporal metadata per observation**: ~120 bytes

### Scaling
- Linear in observation count (O(n) storage, O(log n) queries)
- Temporal quality calculation: O(1) per observation
- Late-arrival reprocessing: O(affected_cells + affected_queries)

## Use Cases

### 1. Multi-Robot Consensus (Disaster Response)

Three robots (RGB drone, thermal drone, LiDAR rover) observe same rubble site:

```rust
// RGB arrives at t=100ms with event_time=1000ms
// Thermal arrives at t=500ms with event_time=1050ms
// LiDAR arrives at t=2500ms with event_time=1100ms (LATE)

// When LiDAR arrives:
// - Watermark was 1050ms (from thermal)
// - LiDAR's event_time=1100ms > watermark, not late!
// - If LiDAR's event_time=950ms (2.5s late arrival):
//   - Triggers reprocessing of spatial queries around site
//   - Reduces thermal+RGB observations' temporal quality slightly
//   - Re-fuses with LiDAR context for refined 3D model
```

### 2. Agricultural Monitoring (Multi-Rate Sensors)

Single rover: RGB @10fps, hyperspectral @1fps, soil probe @0.1Hz, GPS @1Hz

```rust
// All sensors report at different rates, may batch:
// RGB captures at 1000ms, arrives at 1050ms (50ms latency)
// Hyperspectral captures at 1000ms, arrives at 2000ms (1s latency)
// Soil probe captures at 1000ms, arrives at 1500ms (500ms latency)
// GPS fix at 1000ms, arrives at 1100ms (100ms latency)

// Temporal quality (quality = f(latency)):
// RGB: 1.0 (fast)
// Hyperspectral: 0.37 (slow)
// Soil probe: 0.74 (medium)
// GPS: 0.95 (fast)

// Fusion weight for NDVI calculation:
// = model_confidence × temporal_quality × sensor_confidence
// Hyperspectral's high confidence is tempered by latency
```

### 3. Autonomous Vehicle (Sub-100ms Requirement)

Vehicle needs <100ms decision latency for safety:

```rust
// Sensor observations:
// - LiDAR: 50ms latency (quality=1.0)
// - Camera: 33ms latency (quality=1.0)
// - Radar: 20ms latency (quality=1.0)
// - GPS/INS fusion: 100ms latency (quality=1.0 threshold)
// - Weather API: 5000ms latency (quality=0.3 — ignored for immediate decisions)

// Temporal quality ensures only fresh, reliable data affects steering
// Late-arrival batch: if GPS fix arrives 500ms late, it's held for post-decision analysis
```

## Troubleshooting

### Clock Skew Detection

**Symptom**: Observations from same sensor have timestamps differing by >1s

**Diagnosis**:
```rust
// Check temporal confidence
if obs.temporal.sync_confidence < 0.7 {
    eprintln!("Warning: low sync confidence = possible clock drift");
}

// Check latency distribution
for obs in observations {
    let latency = obs.temporal.ingestion_time_us - obs.temporal.event_time_us;
    if latency > 5_000_000 { // >5s
        eprintln!("High latency: {} observations from {:?}", latency, obs.temporal.clock_source);
    }
}
```

**Solution**: 
- Adjust clock source if robot has NTP/GPS available
- Add NTP sync before deployment
- Use regional GNSS (NavIC in India, Galileo in Europe, BeiDou in China)

### Causality Violations

**Symptom**: Observation A's result (event_time=1000ms) appears to depend on observation B (event_time=1050ms)

**Diagnosis**:
```rust
// Check observation ordering by event_time
let times: Vec<_> = observations.iter().map(|o| o.temporal.event_time_us).collect();
if times.windows(2).any(|w| w[0] > w[1]) {
    eprintln!("Causality violation detected");
}
```

**Solution**:
- Ensure observations are processed in event_time order, not arrival order
- Use `TemporalIndexEnhanced` which guarantees event_time ordering
- Review robot clock synchronization (see clock skew detection)

### Slow Convergence with Late Arrivals

**Symptom**: Spatial queries keep changing as late observations arrive

**Diagnosis**:
```rust
// Monitor late-arrival rate
let late_arrival_rate = late_arrivals.len() as f32 / total_observations as f32;
if late_arrival_rate > 0.05 { // >5% late
    eprintln!("High late-arrival rate: {:.1}%", late_arrival_rate * 100.0);
}

// Check time skew (transmission - capture):
for obs in observations {
    let skew = obs.temporal.transmission_time_us - obs.temporal.capture_time_us;
    if skew > 1_000_000 { // >1s
        eprintln!("Large transmission delay: {}ms", skew / 1000);
    }
}
```

**Solution**:
- Reduce batching delays in robot software (transmit more frequently)
- Increase network bandwidth
- Use predictive caching to reduce impact of late arrivals

## API Reference

### TemporalIndexEnhanced

```rust
pub struct TemporalIndexEnhanced {
    pub fn new(decay: DecayFunction) -> Self
    pub fn insert(&mut self, event_time_us: i64, arrival_time_us: i64) -> Result<bool>
    pub fn watermark(&self) -> i64
    pub fn late_arrivals(&self) -> &[usize]
    pub fn latency_ms(&self, index: usize) -> Result<i64>
    pub fn temporal_quality(&self, index: usize) -> Result<f32>
    pub fn range_query(&self, from_us: i64, to_us: i64) -> Result<Vec<usize>>
    pub fn since(&self, event_time_us: i64) -> Result<Vec<usize>>
}
```

### LateArrivalProcessor

```rust
pub struct LateArrivalProcessor {
    pub fn new() -> Self
    pub fn process_late_arrival(&mut self, obs: &Observation, watermark_us: i64) -> Result<Vec<String>>
    pub fn get_affected_regions(&self) -> Vec<String>
    pub fn get_affected_queries(&self) -> Vec<String>
    pub fn estimate_reprocess_cost_ms(&self) -> u32
    pub fn mark_reprocessing_done(&mut self, current_time_us: i64)
}
```

### SensorFusion

```rust
pub struct SensorFusion {
    pub fn new(weights: SensorWeights) -> Self
    pub fn set_temporal_quality_enabled(&mut self, enabled: bool)
    fn extract_temporal_quality(&self, obs: &Observation) -> f32
    pub fn fuse(&self, observations: &[&Observation]) -> Result<FusedData>
}
```

## Next Steps

1. **Monitor temporal metrics** in production (latency distribution, late-arrival rate)
2. **Tune reprocessing strategy** based on deployment (aggressive vs lazy)
3. **Implement predictive caching** to mask late-arrival latency
4. **Add observability** hooks to track clock drift over time

## Version History

- **v0.2.0** (Phase 3): TemporalIndexEnhanced, late-arrival reprocessing, temporal-quality fusion
- **v0.0.1** (Phase 1): Basic temporal decay, Python bindings
