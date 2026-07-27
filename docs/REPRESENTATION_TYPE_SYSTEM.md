# Representation and Type System Architecture

## Foundational Principle

**Space-time intelligence is a problem of representation, not only space and time.**

The platform must treat representation diversity as a fundamental property of reality. Observations originate from heterogeneous sources (sensors, applications, users, agents, models, external systems) with different formats, schemas, coordinate systems, unit systems, memory layouts, and confidence models.

**Design Approach:** Normalize observations without simplifying. Preserve original information while exposing canonical representation for reasoning.

## The Problem: Representation Diversity

Today, observations come from:

**Physical Sensors:**
- Camera (JPEG, PNG, RGB, infrared, multi-spectral)
- LiDAR (point clouds, 3D, varying density)
- IMU (acceleration, rotation, magnetic)
- GPS/GNSS (lat/lon, altitude, accuracy)
- Ultrasonic (distance, polarization)
- Radar (velocity, range, angle)

**AI Models:**
- Object detection (bounding boxes, classifications, confidence)
- Semantic segmentation (per-pixel classifications)
- Terrain classification (type, traversability)
- Agent predictions (future trajectories)
- LLM outputs (text, embeddings, reasoning)

**External Systems:**
- Map APIs (GeoJSON, vector tiles)
- Weather APIs (time-series measurements)
- Satellite imagery (multi-band, multi-temporal)
- User-uploaded content (images, text, annotations)

**Different Observers:**
- Autonomous agents (structured telemetry)
- Mobile devices (sensor fusion)
- Humans (subjective annotations)
- Simulations (synthetic data)

**Current Problem:** Forcing all into generic JSON or flattening loses critical information:
- Camera matrix (focal length, principal point)
- Point cloud encoding (color, intensity, normals)
- Embedding dimensionality and model provenance
- Sensor fusion methodology
- Coordinate system and transformations

## Solution: Observation Abstraction Layer

All observations, regardless of source, transform into canonical envelope:

```rust
pub struct ObservationEnvelope {
    /// Unique identifier
    pub id: Uuid,
    
    /// Type of observation
    pub observation_type: ObservationType,
    
    /// Where did this come from?
    pub source: ObservationSource,
    
    /// Original/processed payload
    pub payload: ObservationPayload,
    
    /// Spatial coordinates and reference frame
    pub spatial: SpatialInfo,
    
    /// Temporal dimensions and clock source
    pub temporal: TemporalMetadata,
    
    /// Confidence and uncertainty
    pub confidence: ConfidenceModel,
    
    /// Additional provenance and metadata
    pub metadata: HashMap<String, String>,
}
```

### ObservationType: Strongly Typed

Never treat everything as generic JSON. Instead:

```rust
pub enum ObservationType {
    ImageObservation,
    PointCloudObservation,
    LocationObservation,
    MotionObservation,
    AudioObservation,
    TextObservation,
    VideoObservation,
    TerrainObservation,
    WeatherObservation,
    DetectionObservation,
    SegmentationObservation,
    EmbeddingObservation,
    GraphObservation,
    TelemetryObservation,
    EventObservation,
    AgentObservation,
    Custom(String),  // Extensible for new types
}
```

### ObservationPayload: Type-Specific Data

```rust
pub enum ObservationPayload {
    Image(ImagePayload),
    PointCloud(PointCloudPayload),
    Location(LocationPayload),
    Motion(MotionPayload),
    Detection(DetectionPayload),
    Segmentation(SegmentationPayload),
    Embedding(EmbeddingPayload),
    Telemetry(TelemetryPayload),
    Text(TextPayload),
    Custom(Vec<u8>),  // Opaque binary with type hint
}

pub struct ImagePayload {
    /// Raw image data (JPEG, PNG, RAW, etc.)
    pub data: Vec<u8>,
    
    /// Image format
    pub format: ImageFormat,
    
    /// Dimensions
    pub width: u32,
    pub height: u32,
    
    /// Color space (RGB, HSV, YCbCr, etc.)
    pub color_space: ColorSpace,
    
    /// Camera intrinsics (if known)
    pub camera_matrix: Option<CameraIntrinsics>,
    
    /// Distortion coefficients
    pub distortion: Option<Vec<f32>>,
}

pub struct PointCloudPayload {
    /// Point data
    pub points: Vec<Point3D>,
    
    /// Encoding (binary, compressed, sparse)
    pub encoding: PointCloudEncoding,
    
    /// Compression ratio
    pub compression_ratio: Option<f32>,
    
    /// Associated colors (if present)
    pub colors: Option<Vec<RGB>>,
    
    /// Associated normals (if computed)
    pub normals: Option<Vec<Vector3>>,
    
    /// Point confidence values
    pub confidence: Option<Vec<f32>>,
}

pub struct DetectionPayload {
    /// Detected objects
    pub detections: Vec<Detection>,
    
    /// Model used for detection
    pub model_info: ModelInfo,
    
    /// Threshold used
    pub confidence_threshold: f32,
}

pub struct EmbeddingPayload {
    /// Raw embedding vector
    pub vector: Vec<f32>,
    
    /// Dimensionality (redundant but explicit)
    pub dimension: u32,
    
    /// Model that generated embedding
    pub model: String,
    
    /// Model version
    pub model_version: String,
    
    /// Vector normalization (None, L2, etc.)
    pub normalization: Option<String>,
}
```

### ObservationSource: Complete Lineage

```rust
pub struct ObservationSource {
    /// Primary source (sensor, model, user, API, etc.)
    pub source_type: SourceType,
    
    /// Unique identifier for source (camera ID, model name, agent ID)
    pub source_id: String,
    
    /// Processing pipeline
    pub pipeline: Vec<ProcessingStep>,
}

pub enum SourceType {
    Sensor,        // Physical sensor
    Model,         // AI model output
    User,          // Human input
    API,           // External API
    Simulation,    // Synthetic data
    Fusion,        // Fused from multiple sources
    Transformation, // Derived from other observation
}

pub struct ProcessingStep {
    /// What operation was performed?
    pub operation: String,
    
    /// When?
    pub timestamp: i64,
    
    /// By whom/what?
    pub executor: String,
    
    /// Any parameters that affected output?
    pub parameters: HashMap<String, String>,
    
    /// Did this change the semantics?
    pub semantic_preserving: bool,
}
```

### SpatialInfo: Coordinate Awareness

```rust
pub struct SpatialInfo {
    /// Location in canonical coordinates
    pub location: GeoPoint,
    
    /// Elevation above sea level
    pub elevation_asl: Option<f32>,
    
    /// Coordinate system used (WGS84, ECEF, UTM, pixel, camera, etc.)
    pub coordinate_system: CoordinateSystem,
    
    /// Reference frame or camera pose
    pub reference_frame: Option<ReferenceFr ame>,
    
    /// Spatial uncertainty
    pub uncertainty: SpatialUncertainty,
    
    /// Geometry (if applicable)
    pub geometry: Option<Geometry>,
}

pub enum CoordinateSystem {
    WGS84,          // Latitude/longitude
    ECEF,           // Earth-Centered, Earth-Fixed
    UTM(Zone),      // UTM zone
    LocalTangent,   // Local tangent plane
    CameraFrame,    // Relative to camera
    RobotFrame,     // Relative to robot
    PixelCoords,    // Image pixel coordinates
    Custom(String),
}

pub struct SpatialUncertainty {
    /// Horizontal uncertainty in meters
    pub horizontal_m: f32,
    
    /// Vertical uncertainty in meters
    pub vertical_m: Option<f32>,
    
    /// Covariance matrix (if available)
    pub covariance: Option<Matrix3x3>,
}
```

### ConfidenceModel: Not One-Size-Fits-All

```rust
pub enum ConfidenceModel {
    /// Simple 0.0-1.0 confidence
    Simple(f32),
    
    /// Confidence interval (mean ± std)
    Gaussian {
        mean: f32,
        std: f32,
    },
    
    /// Beta distribution (for probabilities)
    Beta {
        alpha: f32,
        beta: f32,
    },
    
    /// Per-class confidence (for classifications)
    Categorical {
        classes: Vec<(String, f32)>,
    },
    
    /// Quantile-based confidence
    Quantiles {
        q05: f32,
        q25: f32,
        q50: f32,  // median
        q75: f32,
        q95: f32,
    },
    
    /// Per-element confidence (for detections, segmentation)
    PerElement(Vec<f32>),
    
    /// Unknown/not specified
    Unknown,
}
```

## Type Registry: Extensibility

New observation types can be added without core platform changes.

### Plugin Pattern

```rust
pub trait ObservationAdapter: Send + Sync {
    /// Observation type this adapter handles
    fn observation_type(&self) -> ObservationType;
    
    /// Convert incoming data to canonical envelope
    fn to_envelope(&self, raw: Vec<u8>) -> Result<ObservationEnvelope>;
    
    /// Convert envelope back to native format (for export)
    fn from_envelope(&self, envelope: &ObservationEnvelope) -> Result<Vec<u8>>;
    
    /// Estimate memory footprint
    fn memory_size(&self, payload: &ObservationPayload) -> usize;
    
    /// Route to appropriate execution engine
    fn execution_backend(&self) -> ExecutionBackend;
}

pub struct HyperspectralImageAdapter { /* ... */ }
pub struct RadarObservationAdapter { /* ... */ }
pub struct QuantumSensorAdapter { /* ... */ }
pub struct CustomEnterpriseAdapter { /* ... */ }
```

### Registry

```rust
pub struct TypeRegistry {
    adapters: HashMap<ObservationType, Arc<dyn ObservationAdapter>>,
}

impl TypeRegistry {
    pub fn register(&mut self, adapter: Arc<dyn ObservationAdapter>) {
        self.adapters.insert(adapter.observation_type(), adapter);
    }
    
    pub fn to_envelope(&self, obs_type: ObservationType, raw: Vec<u8>) -> Result<ObservationEnvelope> {
        self.adapters
            .get(&obs_type)
            .ok_or("Unknown type")?
            .to_envelope(raw)
    }
}
```

## GPU-Aware Type Routing

Different observation types route to optimal execution engines:

```rust
pub enum ExecutionBackend {
    ImageTensorPipeline,      // JPEG → Bitmap → Tensor → GPU
    PointCloudSpatialPipeline,  // PCL → Spatial Index → GPU
    TelemetryStreamPipeline,  // Measurements → Stream processing → CPU
    EmbeddingVectorPipeline,  // Embeddings → Vector ops → GPU
    GraphComputePipeline,     // Knowledge graphs → GCN → GPU
    AudioProcessingPipeline,  // Audio → Spectrogram → GPU
    CustomPipeline(String),
}
```

**Automatic routing in query execution:**

```rust
impl ObservationStore {
    pub fn query_gpu_aware(
        &self,
        request: QueryRequest,
    ) -> ResultStream {
        observations
            .iter()
            .group_by(|o| o.execution_backend())
            .into_iter()
            .par_flat_map(|(backend, group)| {
                match backend {
                    ImageTensorPipeline => self.execute_on_vision_gpu(group),
                    PointCloudSpatialPipeline => self.execute_on_spatial_gpu(group),
                    TelemetryStreamPipeline => self.execute_on_cpu_stream(group),
                    EmbeddingVectorPipeline => self.execute_on_vector_gpu(group),
                    _ => self.execute_generic(group),
                }
            })
            .collect()
    }
}
```

## Zero-Copy Representation

Avoid repeated conversions: JPEG → Bitmap → Tensor → Embedding

Instead:

1. **Store original**: JPEG in observation
2. **Lazy decode**: On-demand bitmap decoding in GPU pipeline
3. **Shared tensors**: Multiple models access same GPU tensor
4. **Pointer passing**: Embeddings reference original tensor memory

```rust
pub struct ImagePayload {
    pub data: Arc<Bytes>,           // Shared JPEG data
    pub decoded_cache: Arc<Mutex<Option<Bitmap>>>,  // Lazy bitmap
    pub tensor_cache: Arc<Mutex<Option<GpuTensor>>>, // Lazy tensor
}

impl ImagePayload {
    pub fn get_tensor(&self) -> Arc<GpuTensor> {
        // Check cache first
        if let Some(tensor) = self.tensor_cache.lock().as_ref() {
            return tensor.clone();
        }
        
        // Decode if needed
        let bitmap = self.get_bitmap();
        
        // Transfer to GPU without intermediate copy
        let tensor = self.to_gpu_tensor_zero_copy(&bitmap);
        
        // Cache for reuse
        *self.tensor_cache.lock() = Some(tensor.clone());
        
        tensor
    }
}
```

## Unit System Tracking

Never assume meters, feet, latitude, longitude, pixels are interchangeable.

```rust
pub struct UnitedValue<T> {
    pub value: T,
    pub unit: Unit,
}

pub enum Unit {
    Meters,
    Feet,
    Miles,
    Kilometers,
    Degrees,  // Angular
    Radians,
    Seconds,
    Milliseconds,
    Celsius,
    Fahrenheit,
    Kelvin,
    Pixels,  // Image coordinates
    Custom(String),
}

impl<T: NumericOps> UnitedValue<T> {
    pub fn convert_to(&self, target_unit: Unit) -> Result<UnitedValue<T>> {
        // Conversion matrix ensures type safety
        conversion_matrix::convert(self.value, self.unit, target_unit)
    }
}
```

Usage:
```rust
let distance_m = UnitedValue {
    value: 100.0,
    unit: Unit::Meters,
};

let distance_ft = distance_m.convert_to(Unit::Feet)?;
// Result: 328.1 feet

let lat = UnitedValue {
    value: 40.7128,
    unit: Unit::Degrees,  // Degrees latitude
};

// This would fail (can't convert degrees to meters)
let invalid = lat.convert_to(Unit::Meters)?;  // Error!
```

## AI-Native Observations

Treat AI outputs as first-class observations:

```rust
pub struct DetectionObservation {
    pub source_image: Option<Arc<ImageObservation>>,
    pub detections: Vec<Detection>,
    pub model_info: ModelInfo,
    pub execution_latency_ms: u32,
}

pub struct SegmentationObservation {
    pub source_image: Arc<ImageObservation>,
    pub class_map: Arc<Bytes>,  // GPU-resident tensor
    pub class_info: Vec<ClassInfo>,
    pub pixel_confidences: Option<Arc<Bytes>>,
}

pub struct PredictionObservation {
    pub agent_id: String,
    pub predicted_trajectory: Vec<Point3D>,
    pub confidence: Vec<f32>,
    pub reasoning: String,  // Why this prediction?
}
```

## Ingestion Pipeline

Every incoming observation follows this pipeline:

```
Raw Input (JPEG, PCL, JSON, etc.)
    ↓
Type Detection (file extension, header, schema)
    ↓
Adapter Lookup (registry)
    ↓
Validation (schema, units, coordinates)
    ↓
Canonicalization (ObservationEnvelope)
    ↓
Spatial/Temporal Normalization
    ↓
Confidence Modeling
    ↓
Storage (immutable append-only)
    ↓
Indexing (spatial, temporal, type)
    ↓
Availability for Query
```

## Integration with Existing Types

### Extend SensorValue Enum

Currently:
```rust
pub enum SensorValue {
    Temperature { celsius: f32 },
    LiDAR { distances_cm: Vec<u16> },
    Camera { detections: Vec<ObjectDetection> },
    // ... etc
}
```

Replace with:
```rust
pub enum SensorValue {
    Observation(Arc<ObservationEnvelope>),  // Any observation type
    // Keep simple types for backward compatibility
    Temperature { celsius: f32 },
    // ...
}
```

### Extend Observation Type

```rust
pub struct Observation {
    // Existing fields
    pub id: Uuid,
    pub robot_id: String,
    pub timestamp: i64,
    pub location: GeoPoint,
    pub sensor_type: SensorType,
    pub confidence: f32,
    pub temporal: TemporalMetadata,
    
    // NEW: Unified representation
    pub envelope: Arc<ObservationEnvelope>,
    
    pub metadata: HashMap<String, String>,
}
```

## Implementation Roadmap

### Phase 1 (Week 20-21)
- [ ] ObservationEnvelope struct
- [ ] ObservationType enum (16 standard types)
- [ ] ObservationPayload (Image, PointCloud, Detection, Embedding, etc.)
- [ ] ObservationSource with pipeline tracking
- [ ] SpatialInfo and CoordinateSystem
- [ ] ConfidenceModel (multiple types)
- [ ] Type Registry and ObservationAdapter trait

### Phase 2 (Week 22-23)
- [ ] GPU-aware routing by type
- [ ] Zero-copy buffers and lazy conversion
- [ ] Unit system with conversions
- [ ] Ingestion pipeline validation
- [ ] Adapter implementations (10+ standard types)

### Phase 3 (Week 24-25)
- [ ] Custom type registration (plugin system)
- [ ] Integration with parallel execution
- [ ] Type-specific indexes
- [ ] Export in native formats

### Phase 4 (Week 26+)
- [ ] Performance optimization
- [ ] Memory-mapped observations
- [ ] Distributed type registry

## Guiding Philosophy

**Space-time intelligence is not only about space and time. It is about representation.**

The platform's role is not to force uniformity at the source, but to provide:

1. **Canonical representation** — Stable envelope regardless of source
2. **Type preservation** — Explicit types, not generic JSON
3. **Fidelity maintenance** — No information loss through normalization
4. **Provenance tracking** — Complete lineage from source to reasoning
5. **Extensibility** — New types without core changes
6. **Efficiency** — Zero-copy, GPU-aware routing
7. **Correctness** — Unit-aware, coordinate-system-explicit

This allows heterogeneous observations to participate in a shared world model while preserving their original meaning, fidelity, and optimal computational path.
