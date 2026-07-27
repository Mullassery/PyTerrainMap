# Universal Robot Interoperability Architecture

## Foundational Principle

**The platform understands capabilities, not brands.**

All robotic systems—wheeled, legged, aerial, manipulative, autonomous vehicles, humanoids, simulations, and humans—interact through a unified capability-driven world model. Hardware diversity becomes an implementation detail, not an architectural constraint.

---

## Core Abstraction: Agent

Every participant in the system (robot, human, simulation, AI) is an **Agent**.

```rust
pub struct Agent {
    pub id: AgentId,
    pub identity: AgentIdentity,
    pub state: AgentState,
    pub capabilities: CapabilityManifest,
    pub health: AgentHealth,
    pub trust_metadata: TrustMetadata,
    pub timestamp_us: i64,
}

pub struct AgentIdentity {
    pub name: String,
    pub type_: AgentType,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
}

pub enum AgentType {
    // Wheeled/Tracked
    WheeledRover,
    TrackedRover,
    DeliveryRobot,
    WarehouseRobot,
    AGV,
    AMR,

    // Legged
    Quadruped,
    Humanoid,
    Biped,
    Hexapod,
    Custom { legs: u8 },

    // Aerial
    Drone,
    Multicopter,
    FixedWing,
    VTOL,
    Airship,

    // Manipulation
    RoboticArm,
    MobileManipulator,
    Gripper,
    EndEffector,

    // Vehicles
    AutonomousVehicle,
    AgriculturalVehicle,
    IndustrialVehicle,
    MiningEquipment,

    // Other
    HumanOperator,
    AiAgent,
    Simulation,
    Custom { description: String },
}

pub struct AgentState {
    pub location: GeoPoint,
    pub orientation: Orientation,  // Roll, pitch, yaw (radians)
    pub velocity: Vector3D,
    pub angular_velocity: Vector3D,
    pub battery_percent: Option<f32>,
    pub operating_mode: OperatingMode,
    pub is_active: bool,
    pub timestamp_us: i64,
}

pub enum OperatingMode {
    Idle,
    Operating,
    Charging,
    Transmitting,
    Calibrating,
    Error,
    Shutdown,
}
```

---

## Capability Model

Agents advertise what they can do through a **CapabilityManifest**.

### Capability Categories

```rust
pub struct CapabilityManifest {
    pub mobility: MobilityCapabilities,
    pub sensing: SensingCapabilities,
    pub manipulation: ManipulationCapabilities,
    pub communication: CommunicationCapabilities,
    pub processing: ProcessingCapabilities,
    pub payload: PayloadCapabilities,
    pub custom: HashMap<String, String>,
}

// ============================================================================
// Mobility Capabilities
// ============================================================================

pub struct MobilityCapabilities {
    pub modes: Vec<MobilityMode>,
    pub max_speed_ms: f32,
    pub max_acceleration_ms2: f32,
    pub max_slope_degrees: f32,
    pub max_step_height_m: Option<f32>,  // For legged robots
    pub max_water_depth_m: Option<f32>,  // For aquatic robots
    pub range_km: f32,
    pub operating_temperature_c: (f32, f32),  // Min, Max
    pub all_terrain: bool,
    pub gps_required: bool,
    pub indoor_capable: bool,
    pub outdoor_capable: bool,
}

pub enum MobilityMode {
    // Wheeled/Tracked
    Wheeled { drive_type: DriveType },
    Tracked,

    // Legged
    Walk { legs: u8 },
    Run { legs: u8 },
    Climb,
    Swim,

    // Aerial
    Fly { rotor_count: u8 },
    FixedWing,
    Hover,

    // Other
    Custom { name: String },
}

pub enum DriveType {
    Differential,
    Ackermann,
    Mecanum,
    Omni,
}

// ============================================================================
// Sensing Capabilities
// ============================================================================

pub struct SensingCapabilities {
    pub cameras: Vec<CameraSensor>,
    pub lidars: Vec<LidarSensor>,
    pub radars: Vec<RadarSensor>,
    pub imus: Vec<ImuSensor>,
    pub temperature_sensors: Vec<TemperatureSensor>,
    pub pressure_sensors: Vec<PressureSensor>,
    pub humidity_sensors: Vec<HumiditySensor>,
    pub gas_sensors: Vec<GasSensor>,
    pub custom_sensors: HashMap<String, SensorSpec>,
}

pub struct CameraSensor {
    pub id: String,
    pub type_: CameraType,
    pub resolution_mp: f32,
    pub fov_degrees: f32,
    pub framerate_hz: u32,
    pub has_depth: bool,
    pub mount_point: String,
}

pub enum CameraType {
    RGB,
    Thermal,
    NIR,
    Multispectral,
    Hyperspectral,
    EventCamera,
    StructuredLight,
}

pub struct LidarSensor {
    pub id: String,
    pub range_m: f32,
    pub points_per_second: u32,
    pub has_intensity: bool,
    pub has_rgb: bool,
}

pub struct RadarSensor {
    pub id: String,
    pub range_m: f32,
    pub resolution_m: f32,
    pub velocity_measurement: bool,
}

pub struct ImuSensor {
    pub id: String,
    pub has_accelerometer: bool,
    pub has_gyroscope: bool,
    pub has_magnetometer: bool,
}

pub struct TemperatureSensor {
    pub id: String,
    pub range_c: (f32, f32),
    pub accuracy_c: f32,
}

pub struct PressureSensor {
    pub id: String,
    pub range_pa: (f32, f32),
}

pub struct HumiditySensor {
    pub id: String,
    pub range_percent: (f32, f32),
}

pub struct GasSensor {
    pub id: String,
    pub gas_type: String,
    pub range_ppm: (f32, f32),
}

pub struct SensorSpec {
    pub capabilities: HashMap<String, String>,
}

// ============================================================================
// Manipulation Capabilities
// ============================================================================

pub struct ManipulationCapabilities {
    pub arms: Vec<ArmSpec>,
    pub grippers: Vec<GripperSpec>,
    pub max_payload_kg: f32,
    pub reach_m: f32,
    pub dexterity_level: DexterityLevel,
}

pub enum DexterityLevel {
    None,
    Basic,      // 2-3 DOF gripper
    Moderate,   // 6 DOF arm, basic gripper
    Advanced,   // Multi-DOF arm, sophisticated gripper
    Humanlike,  // 5+ finger hand with full dexterity
}

pub struct ArmSpec {
    pub id: String,
    pub dof: u8,
    pub reach_m: f32,
    pub payload_kg: f32,
}

pub struct GripperSpec {
    pub id: String,
    pub type_: GripperType,
    pub fingers: u8,
    pub force_n: f32,
}

pub enum GripperType {
    Parallel,
    Underactuated,
    Adaptive,
    Magnetic,
    Vacuum,
    Soft,
}

// ============================================================================
// Communication Capabilities
// ============================================================================

pub struct CommunicationCapabilities {
    pub protocols: Vec<CommunicationProtocol>,
    pub bandwidth_mbps: f32,
    pub latency_ms: u32,
    pub reliability_percent: f32,
    pub encryption_supported: bool,
    pub ros_support: bool,
    pub ros2_support: bool,
    pub mavlink_support: bool,
}

pub enum CommunicationProtocol {
    WiFi,
    Cellular,
    LoRaWAN,
    Zigbee,
    Bluetooth,
    MQTT,
    ROS,
    ROS2,
    MAVLink,
    RTCM,
    Custom { name: String },
}

// ============================================================================
// Processing Capabilities
// ============================================================================

pub struct ProcessingCapabilities {
    pub onboard_compute: Option<ComputeSpec>,
    pub edge_support: bool,
    pub cloud_capable: bool,
    pub inference_capable: bool,
    pub max_model_params_m: Option<u32>,
}

pub struct ComputeSpec {
    pub processor: String,
    pub ram_gb: u32,
    pub storage_gb: u32,
    pub gpu_capable: bool,
}

// ============================================================================
// Payload Capabilities
// ============================================================================

pub struct PayloadCapabilities {
    pub max_kg: f32,
    pub max_volume_m3: f32,
    pub mounting_points: u8,
    pub power_available_w: f32,
}
```

---

## Dynamic Capability Discovery

Agents advertise capabilities at startup and update on changes.

```rust
pub struct CapabilityAdvertisement {
    pub agent_id: AgentId,
    pub manifest: CapabilityManifest,
    pub timestamp_us: i64,
    pub ttl_seconds: u32,  // How long is this valid?
}

pub struct CapabilityQuery {
    pub required_mobility: Option<Vec<MobilityMode>>,
    pub required_sensors: Option<Vec<SensorType>>,
    pub required_payload_kg: Option<f32>,
    pub required_range_km: Option<f32>,
    pub team_composition: Option<TeamRequirements>,
}

pub struct TeamRequirements {
    pub min_agents: usize,
    pub required_roles: Vec<AgentRole>,
    pub specialization_needed: Vec<String>,
}

pub enum AgentRole {
    Scout,           // Reconnaissance
    Collector,       // Gather observations
    Analyst,         // Process data
    Executor,        // Perform actions
    Coordinator,     // Manage team
    Safety,          // Monitor safety
}
```

---

## Unified Observation Model

All observations flow through a canonical **spatio-temporal observation** regardless of agent type.

```rust
pub struct UniversalObservation {
    pub observation_id: ObservationId,
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub timestamp_metadata: TemporalMetadata,
    pub location: GeoPoint,
    pub orientation: Orientation,
    pub confidence: f32,
    pub observation_type: ObservationType,
    pub payload: ObservationPayload,
    pub provenance: ProvenanceChain,
    pub compliance_metadata: ComplianceMetadata,
}

pub enum ObservationType {
    // Mobility observations (position, movement)
    Position,
    Movement,
    Trajectory,
    Collision,

    // Sensing observations (camera, lidar, radar, etc.)
    Image,
    PointCloud,
    Radar,
    Thermal,
    Gas,
    Audio,

    // Manipulation observations (object interaction)
    Grasp,
    Place,
    Manipulation,
    ObjectInteraction,

    // Health observations (battery, temperature, status)
    BatteryStatus,
    TemperatureReading,
    ErrorReport,
    DiagnosticData,

    // Communication observations
    Message,
    Alert,
    Report,

    // Human observations
    HumanAnnotation,
    Approval,
    Instruction,

    // Custom
    Custom { type_name: String },
}

pub enum ObservationPayload {
    Position { lat: f64, lon: f64, elevation_m: f32 },
    Movement { velocity_ms: f32, acceleration_ms2: f32, heading: f32 },
    Image { data: Vec<u8>, width: u32, height: u32, format: String },
    PointCloud { data: Vec<Point3D>, count: u32 },
    ObjectDetection { detections: Vec<Detection> },
    BatteryStatus { percent: f32, voltage_v: f32 },
    TemperatureReading { celsius: f32, location: String },
    HumanAnnotation { text: String, confidence: f32 },
    Custom { json: String },
}

pub struct Detection {
    pub class_label: String,
    pub confidence: f32,
    pub location: Option<GeoPoint>,
    pub bbox: Option<BoundingBox>,
}

pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

---

## Multi-Agent Collaboration

The platform enables agents to reason about each other.

```rust
pub struct AgentTeam {
    pub team_id: TeamId,
    pub members: Vec<Agent>,
    pub roles: HashMap<AgentId, AgentRole>,
    pub communication_topology: CommunicationTopology,
    pub mission: Option<Mission>,
}

pub enum CommunicationTopology {
    Centralized { coordinator: AgentId },
    Hierarchical { hierarchy: Vec<AgentId> },
    Mesh,
    Gossip,
    Custom { description: String },
}

pub struct Mission {
    pub mission_id: MissionId,
    pub description: String,
    pub objectives: Vec<Objective>,
    pub constraints: Vec<Constraint>,
    pub expected_duration_seconds: Option<u32>,
    pub required_capabilities: CapabilityQuery,
}

pub struct Objective {
    pub objective_id: ObjectiveId,
    pub description: String,
    pub required_agent_role: AgentRole,
    pub required_capabilities: Vec<String>,
    pub location: Option<GeoPoint>,
    pub deadline_us: Option<i64>,
}

pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub value: String,
}

pub enum ConstraintType {
    TimeLimit,
    EnergyBudget,
    NoFly,
    OperatingHours,
    WeatherConditions,
    TerrainType,
}
```

---

## Native Ecosystem Integration

### ROS/ROS 2 Bridge

```rust
pub struct ROSIntegration {
    pub node_name: String,
    pub ros_version: RosVersion,
    pub subscribed_topics: Vec<String>,
    pub published_topics: Vec<String>,
    pub services: Vec<String>,
    pub tf_frame: String,
}

pub enum RosVersion {
    ROS1,
    ROS2,
}

// Auto-convert ROS messages to UniversalObservations
pub fn ros_topic_to_observation(
    topic_name: &str,
    message: &[u8],
) -> Result<UniversalObservation> {
    // sensor_msgs/Image → ObservationType::Image
    // sensor_msgs/PointCloud2 → ObservationType::PointCloud
    // geometry_msgs/Twist → ObservationType::Movement
    // nav_msgs/Odometry → ObservationType::Position
    // tf messages → ObservationType::Position
}
```

### PX4/ArduPilot Bridge

```rust
pub struct UAVIntegration {
    pub flight_controller: FlightController,
    pub mavlink_version: String,
    pub heartbeat_interval_ms: u32,
    pub home_location: Option<GeoPoint>,
}

pub enum FlightController {
    PX4,
    ArduPilot,
    Custom { name: String },
}

// Auto-convert MAVLink messages to UniversalObservations
pub fn mavlink_to_observation(
    msg_id: u32,
    payload: &[u8],
) -> Result<UniversalObservation> {
    // HEARTBEAT → AgentHealth
    // GLOBAL_POSITION_INT → ObservationType::Position
    // ATTITUDE → ObservationType::Movement
    // BATTERY_STATUS → ObservationType::BatteryStatus
}
```

### Simulation Bridge (Gazebo, Isaac Sim, etc.)

```rust
pub struct SimulationAgent {
    pub sim_name: String,
    pub simulation_time: SimulationTime,
    pub physics_paused: bool,
    pub render_enabled: bool,
}

pub struct SimulationTime {
    pub sim_seconds: f64,
    pub wall_time_us: i64,
    pub time_scale: f32,
}

// Simulated agents appear identical to physical counterparts
pub fn simulation_update_to_observation(
    agent_id: AgentId,
    update: SimulationStateUpdate,
) -> UniversalObservation {
    // Position, orientation, sensor readings all normalized
}
```

---

## Capability-Based Query API

Applications target capabilities, not hardware.

```rust
pub struct CapabilityBasedQuery {
    pub capability: Capability,
    pub location: Option<GeoPoint>,
    pub available_agents: Vec<Agent>,
}

pub trait Capability {
    fn can_execute(&self, agent: &Agent) -> bool;
    fn estimated_duration(&self, agent: &Agent) -> u32;
    fn risk_level(&self, agent: &Agent) -> RiskLevel;
}

pub struct NavigateCapability {
    pub destination: GeoPoint,
    pub via_points: Vec<GeoPoint>,
    pub terrain_type: String,
    pub all_weather: bool,
}

impl Capability for NavigateCapability {
    fn can_execute(&self, agent: &Agent) -> bool {
        // Does agent have navigation capability?
        // Can it reach destination?
        // Does it support terrain type?
    }
}

pub struct InspectCapability {
    pub target_location: GeoPoint,
    pub inspection_type: InspectionType,
    pub required_sensors: Vec<SensorType>,
}

pub enum InspectionType {
    Visual,
    Thermal,
    Structural,
    Environmental,
}

pub struct ManipulateCapability {
    pub object_class: String,
    pub required_dexterity: DexterityLevel,
    pub force_required_n: f32,
}

pub struct CollaborateCapability {
    pub required_roles: Vec<AgentRole>,
    pub coordination_model: CommunicationTopology,
}
```

---

## Adapter Pattern for New Robot Types

Adding support for new robot types requires only:

1. Define AgentType variant
2. Implement CapabilityManifest for that type
3. Define message converters (ROS, MAVLink, etc.)
4. No changes to core platform

```rust
pub trait RobotAdapter {
    fn agent_type(&self) -> AgentType;
    fn read_state(&self) -> Result<AgentState>;
    fn read_capabilities(&self) -> Result<CapabilityManifest>;
    fn convert_observations(&self, raw: Vec<u8>) -> Result<Vec<UniversalObservation>>;
    fn execute_command(&self, cmd: Command) -> Result<()>;
}

pub struct CustomRobotAdapter {
    // Implement RobotAdapter for any new robot type
}
```

---

## Future-Proof Extensibility

The platform is ready for:

- **Multi-legged robots** (octopods, robots with 12+ legs)
- **Swarm robots** (hundreds/thousands coordinating)
- **Snake robots** (soft, undulating locomotion)
- **Underwater robots** (pressure, buoyancy, depth)
- **Space robots** (microgravity, vacuum operations)
- **Hybrid robots** (aerial-ground vehicles)
- **General-purpose humanoids** (full manipulation + locomotion)
- **AI agents** (software robots, digital twins)
- **Biological systems** (animals as data collectors)

All through the same capability-driven architecture without core platform changes.

---

## Guiding Philosophy

```
        Wheeled         Legged         Aerial
          ↓               ↓              ↓
    [Agent Abstraction]
        ↓
    [Capability Query]
        ↓
    [Unified Observation]
        ↓
    [World Intelligence]
        ↓
    [Applications]

Hardware details fade. Intelligence remains portable.
```

**One platform. Infinite robot forms. Shared world understanding.**

