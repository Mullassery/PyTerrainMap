# Temporal Normalization: Time as First-Class Coordinate

## Principle

Time is treated with the same rigor and precision as spatial coordinates. Every observation carries complete temporal provenance enabling reasoning about out-of-order events, late arrivals, and multi-clock synchronization across heterogeneous robot fleets.

## Core Architecture

### The Problem
In distributed robot systems, observations DO NOT arrive in chronological order:
- GPS satellites have different clock skew
- LiDAR packets are batched before transmission
- Camera frames are buffer-delayed by processing
- Network latency varies (WiFi vs cellular vs satellite)
- Different sensors have different synchronization sources

**Treating arrival order as chronological order is incorrect and leads to wrong conclusions.**

### The Solution
Distinguish five temporal dimensions for every observation:

1. **event_time** — When the event actually occurred in reality (ground truth)
2. **capture_time** — When the sensor physically recorded it
3. **transmission_time** — When it left the robot/gateway
4. **ingestion_time** — When it entered the backend system
5. **processing_time** — When it was indexed/analyzed

Plus metadata:
- **clock_source** — Where the timestamp came from (GPS, NavIC, system clock, etc.)
- **precision_us** — Timestamp granularity (1us, 1ms, 1s, etc.)
- **estimated_latency_us** — Time from event to ingestion
- **sync_confidence** — 0.0-1.0 confidence in clock synchronization
- **jitter_us** — Variability in transmission delay
- **temporal_confidence** — 0.0-1.0 overall temporal quality

## Rust API

### TemporalMetadata Structure
```rust
pub struct TemporalMetadata {
    pub event_time_us: i64,          // Reality coordinate
    pub capture_time_us: i64,        // Sensor timestamp
    pub transmission_time_us: i64,   // Network departure
    pub ingestion_time_us: i64,      // Backend arrival
    pub processing_time_us: i64,     // Analysis completion
    
    pub clock_source: ClockSource,    // GPS/NavIC/UTC/etc.
    pub precision_us: u32,            // Timestamp precision
    pub estimated_latency_us: u32,   // Propagation time
    pub sync_confidence: f32,         // Sync quality
    pub is_late_arrival: bool,        // Out-of-order flag
    pub jitter_us: u32,               // Delay variance
    pub temporal_confidence: f32,     // Overall confidence
}
```

### ClockSource Enum
```rust
pub enum ClockSource {
    GPS,            // Standard GPS time
    NavIC,          // Indian GNSS (prioritized in India)
    Galileo,        // European GNSS (prioritized in Europe)
    GLONASS,        // Russian GNSS
    BeiDou,         // Chinese GNSS
    IMU,            // IMU integrated time
    Camera,         // Camera frame timestamp
    LiDAR,          // LiDAR pulse time
    SystemClock,    // Local monotonic clock
    UTC,            // UTC epoch time (default)
    NTP,            // Network Time Protocol
    PTP,            // Precision Time Protocol
}
```

### Observation Enhancement
```rust
pub struct Observation {
    pub id: Uuid,
    pub robot_id: String,
    pub timestamp: i64,           // Legacy: kept for compatibility
    pub location: GeoPoint,
    pub elevation_asl: Option<f32>,
    pub sensor_type: SensorType,
    pub value: SensorValue,
    pub confidence: f32,
    
    pub temporal: TemporalMetadata,  // NEW: Full temporal provenance
    
    pub metadata: HashMap<String, String>,
}
```

### Creation APIs
```rust
// Simple creation with UTC time (default)
let obs = Observation::new(
    "robot-01".to_string(),
    timestamp_us,
    location,
    elevation,
    SensorType::Camera,
    value,
    confidence,
);

// Creation with explicit clock source
let obs = Observation::with_clock_source(
    "robot-01".to_string(),
    timestamp_us,
    location,
    elevation,
    SensorType::Camera,
    value,
    confidence,
    ClockSource::NavIC,  // Explicitly use NavIC in India
);

// Creation with full temporal control
let mut temporal = TemporalMetadata::from_event_time(
    event_time_us,
    ClockSource::GPS,
    1000,  // millisecond precision
);
temporal.jitter_us = 50;
temporal.sync_confidence = 0.99;

let obs = Observation::new(...).with_temporal(temporal);
```

### Temporal Reasoning Methods
```rust
// Check temporal validity
obs.temporal.is_temporally_valid()          // confidence >= 0.7

// Compute latencies
obs.temporal.total_latency_us()             // event to processing
obs.temporal.transmission_latency_us()      // capture to ingestion

// Order events chronologically
obs1.event_occurred_before(&obs2)           // Use event_time, not arrival

// Detect late arrivals
obs.check_late_arrival()                    // Mark if >100ms delay
```

## Data Model Example

### Before (No Temporal Provenance)
```json
{
  "id": "obs-123",
  "robot_id": "rover-01",
  "timestamp": 1721431351000000,
  "location": [12.9716, 77.5946],
  "sensor_type": "gps",
  "value": {"lat": 12.9716, "lon": 77.5946},
  "confidence": 0.95
}
```

### After (With Temporal Normalization)
```json
{
  "id": "obs-123",
  "robot_id": "rover-01",
  "timestamp": 1721431351000000,
  
  "temporal": {
    "event_time_us": 1721431351000000,
    "capture_time_us": 1721431351001000,
    "transmission_time_us": 1721431351050000,
    "ingestion_time_us": 1721431351120000,
    "processing_time_us": 1721431351150000,
    
    "clock_source": "navic",
    "precision_us": 1000,
    "estimated_latency_us": 120000,
    "sync_confidence": 0.98,
    "is_late_arrival": false,
    "jitter_us": 20,
    "temporal_confidence": 0.96
  },
  
  "location": [12.9716, 77.5946],
  "sensor_type": "gps",
  "value": {"lat": 12.9716, "lon": 77.5946},
  "confidence": 0.95
}
```

## Integration Points

### Temporal Query Engine (temporal/mod.rs)
- Queries use `event_time_us` not arrival order
- Support watermarking (max timestamp seen)
- Handle late arrivals by event window
- Re-execute queries when late data arrives

### Anomaly Detection (anomaly/mod.rs)
- Factor temporal confidence into anomaly scoring
- Distinguish slow-changing sensors from fast ones
- Account for transmission delays in spike detection

### Spatial Reasoning (spatial_reasoning/mod.rs)
- Weight position by temporal confidence
- Reject observations with poor sync
- Use clock source for regional preferences

### Fusion (fusion/mod.rs)
- Weight fused values by temporal quality
- Later event time takes precedence (if confident)
- Track temporal agreement between sensors

## Regional Clock Preferences

The system understands regional GNSS availability:

```rust
// India: NavIC prioritized
// Europe: Galileo prioritized
// China: BeiDou prioritized
// Russia: GLONASS prioritized
// Global: GPS as fallback
```

Integration in `spatial_reasoning/mod.rs`:
```rust
let answer = engine.reason_position(
    "India",
    vec![
        ("NavIC", 12.971598, 77.594566, 0.94),
        ("GPS", 12.971600, 77.594568, 0.91),
    ],
);

// NavIC weighted higher due to regional preference
// Position: weighted average
// Confidence: composite of source confidences and temporal quality
```

## Late-Arriving Data Handling

### Out-of-Order Event Resolution
```rust
// Observation arrives with timestamp earlier than previously
// processed observation

// System checks: event_time of new obs < max_event_time_seen
if obs.temporal.event_time_us < watermark {
    // Mark as late arrival
    obs.temporal.is_late_arrival = true;
    
    // Decision: reprocess affected analysis or queue for batch
    match strategy {
        Strategy::Immediate => reprocess_affected_queries(),
        Strategy::Batch => queue_for_window_update(),
    }
}
```

### Temporal Windows
```rust
// Process observations in windows bounded by event time
// Not arrival time

let events = observations
    .iter()
    .filter(|o| o.temporal.event_time_us >= window_start_us
        && o.temporal.event_time_us < window_end_us)
    .collect();
```

### State Corrections
```rust
// Late observation may correct earlier assumptions
// e.g., sensor malfunction diagnosis

let late_obs = get_late_observation();
if late_obs.temporal.temporal_confidence > threshold {
    // Revert earlier anomaly classification
    revert_anomaly_flags(
        late_obs.robot_id,
        late_obs.temporal.event_time_us
    );
}
```

## Implementation Checklist

- [x] **Phase 1 - Foundation (Week 19):**
  - [x] TemporalMetadata struct with 5 dimensions
  - [x] ClockSource enum with 12 GNSS/sensor sources
  - [x] Integration into Observation type
  - [x] Creation APIs with clock source support
  - [x] Temporal validity checking
  - [x] Late arrival detection
  - [x] Export types to lib.rs

- [ ] **Phase 2 - Integration (Week 20-21):**
  - [ ] Update temporal query engine to use event_time
  - [ ] Watermarking in TemporalIndex
  - [ ] Out-of-order event reprocessing
  - [ ] Late arrival batch window
  - [ ] Update anomaly detection with temporal confidence
  - [ ] Update fusion with temporal weighting
  - [ ] Update spatial reasoning with clock sources

- [ ] **Phase 3 - Testing (Week 22):**
  - [ ] Unit tests for out-of-order handling
  - [ ] Late arrival detection tests
  - [ ] Multi-clock synchronization tests
  - [ ] Watermark advancement tests
  - [ ] State correction scenarios

- [ ] **Phase 4 - Documentation (Week 23):**
  - [ ] Python API docs for temporal metadata
  - [ ] Temporal reasoning examples
  - [ ] Late arrival handling guide
  - [ ] Clock source selection guide

## Performance Considerations

- **Memory**: TemporalMetadata adds ~100 bytes per observation (negligible)
- **CPU**: Sorting by event_time vs arrival time is same complexity
- **Storage**: Extra temporal fields compress well in SQLite/PostgreSQL
- **Latency**: Event ordering adds no observable latency to critical path

## Example: Handling Delayed GPS Fix

```rust
// Scenario: Robot in urban canyon loses GPS for 2 minutes
// Then gets GPS fix for time period when it was disconnected

// Earlier observation (from dead reckoning)
let imu_obs = Observation::new(
    "rover-01",
    t1_us,  // 2 minutes ago
    location_estimated,
    None,
    SensorType::Movement,
    value,
    0.3,  // Low confidence (estimate)
);

// Later observation (GPS fix for same time period)
let gps_obs = Observation::with_clock_source(
    "rover-01",
    t1_us,  // SAME EVENT TIME
    location_precise,
    None,
    SensorType::GPS,
    value,
    0.99,  // High confidence (GPS fix)
);

// Mark as late arrival
gps_obs.temporal.is_late_arrival = true;

// System recognizes:
// - event_time is same (both represent same moment)
// - gps_obs has higher temporal_confidence
// - Should use GPS location in final answer
// - No contradiction, just out-of-order arrival
```

## References

- Kafka: Event time vs processing time watermarking
- Google Cloud Dataflow: Windowing by event time
- Apache Flink: Late data handling
- Apache Beam: Temporal reasoning patterns
