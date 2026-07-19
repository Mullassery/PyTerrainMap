# ROS/ROS2 Bridge Architecture

## Overview

Native integration with ROS/ROS2 ecosystems enabling PyTerrainMap to:
- Subscribe to sensor topics (LiDAR, thermal, RGB, IMU)
- Consume TF tree for spatial transformations
- Track robot poses and trajectories
- Publish processed observations
- Support heterogeneous robot fleets in coordinated deployments

## Design Principles

1. **Protocol Native**: Use ROS2 client library (rclrs/rclpy), not middleware
2. **Zero Configuration**: Sensible defaults for common platforms (Clearpath, Boston Dynamics, DJI)
3. **Pluggable Sensors**: Adapter pattern for custom sensor types
4. **Quality First**: Timestamp synchronization, frame transforms, dropout handling
5. **Fleet Aware**: Multi-robot namespacing, collaborative observation tracking

## Architecture Layers

```
┌─────────────────────────────────────────────────┐
│         ROS2 Node (rclpy)                       │
│  - Topic subscriptions (sensors, odometry)      │
│  - TF listener (spatial transforms)             │
│  - Service servers (query, config)              │
└────────────────┬────────────────────────────────┘
                 │
┌─────────────────▼────────────────────────────────┐
│    Sensor Adapters Layer                        │
│  - LiDAR → Point cloud observations             │
│  - Thermal → Temperature grid                   │
│  - RGB → Visual observations                    │
│  - IMU → Motion/confidence metrics              │
└────────────────┬────────────────────────────────┘
                 │
┌─────────────────▼────────────────────────────────┐
│    Normalization Pipeline                       │
│  - Timestamp alignment (PTP clock sync)         │
│  - Frame transform (TF → geodetic)              │
│  - Quality inference (dropout, outlier)         │
│  - Batching (configurable window)               │
└────────────────┬────────────────────────────────┘
                 │
┌─────────────────▼────────────────────────────────┐
│    PyTerrainMap Backend                         │
│  - Observation push (single/batch)              │
│  - Query federation                            │
│  - Multi-backend routing                       │
└─────────────────────────────────────────────────┘
```

## Module Structure

### python/pyterrain_ros/

```
pyterrain_ros/
├── __init__.py
├── bridge.py              # Main ROS2 node
├── adapters/
│   ├── __init__.py
│   ├── base.py            # Adapter interface
│   ├── lidar.py           # PointCloud2 → observations
│   ├── thermal.py         # sensor_msgs/Image → thermal grid
│   ├── rgb.py             # RGB images
│   └── imu.py             # IMU data (confidence metrics)
├── transforms/
│   ├── __init__.py
│   ├── tf_listener.py     # TF tree subscription
│   ├── coordinate_frames.py # Geodetic conversions
│   └── sync.py            # Timestamp sync utils
├── platforms/
│   ├── __init__.py
│   ├── clearpath.py       # Clearpath Warthog/Jackal config
│   ├── spot.py            # Boston Dynamics Spot
│   ├── dji.py             # DJI fleet (M300, Matrice)
│   └── generic.py         # Generic robot template
└── launch/
    ├── sim.launch.py      # Gazebo/Isaac Sim
    ├── hardware.launch.py # Real robot
    └── fleet.launch.py    # Multi-robot coordination
```

## Core Components

### 1. Bridge Node (`bridge.py`)

```python
class PyTerrainROSBridge(Node):
    def __init__(self, config_file: str = None):
        # ROS2 init
        # Load config (YAML)
        # Initialize adapters for enabled sensors
        # Create TF listener
        # Connect to PyTerrainMap backend
        
    def run(self):
        # Main loop: collect → normalize → batch → push
        # Handles backpressure from slow backend
        # Reports diagnostics
```

**Configuration (YAML)**
```yaml
robot_id: "robot-1"
base_frame: "base_link"
reference_frame: "map"  # for geodetic conversions

sensors:
  lidar:
    topic: "/scan"  # or /cloud
    enabled: true
    frame_id: "lidar_link"
    voxel_size: 0.1
    
  thermal:
    topic: "/thermal_image"
    enabled: true
    frame_id: "thermal_link"
    confidence_threshold: 0.7
    
  rgb:
    topic: "/camera/image_raw"
    enabled: false
    
  imu:
    topic: "/imu"
    enabled: true
    frame_id: "imu_link"

backend:
  type: "postgres"  # or "memory" for testing
  connection: "postgresql://user:pass@localhost/pyterrain"
  batch_size: 100
  batch_timeout_ms: 1000

timing:
  sync_mode: "ptp"     # or "ros_time" or "wall"
  max_age_ms: 5000
  interpolation: true
```

### 2. Adapter Base Class (`adapters/base.py`)

```python
class SensorAdapter(ABC):
    @abstractmethod
    def on_message(self, msg) -> List[StorageObservation]:
        """Convert ROS message to observations"""
        pass
    
    @property
    def sensor_type(self) -> str:
        """e.g., "lidar", "thermal", "rgb" """
        pass
```

### 3. LiDAR Adapter (`adapters/lidar.py`)

Converts:
- `sensor_msgs/LaserScan` → radial point cloud
- `sensor_msgs/PointCloud2` → 3D point cloud

Outputs:
- Spatial density map (grid cells with point counts)
- Elevation estimates where applicable
- Returns/intensity confidence scores

### 4. Thermal Adapter (`adapters/thermal.py`)

Converts:
- `sensor_msgs/Image` (8UC1 or 16UC1) → temperature grid
- Color palette → absolute temperature (requires calibration)

Outputs:
- Temperature observations at cell centers
- Confidence from image SNR and emissivity
- Anomaly flags for outliers

### 5. TF Listener (`transforms/tf_listener.py`)

- Subscribe to `/tf` and `/tf_static`
- Cache transforms with 10s history
- Provide:
  - `get_pose(frame, target_time)` → (x, y, z, qx, qy, qz, qw)
  - `transform_point(point, from_frame, to_frame)` → Point3D
  - `get_geodetic(frame)` → (lat, lon, alt) via origin transform

### 6. Coordinate Frame Utilities (`transforms/coordinate_frames.py`)

```python
class CoordinateConverter:
    def __init__(self, origin_lat: float, origin_lon: float):
        # Store origin (first robot pose locks it)
        
    def enu_to_geodetic(self, x: float, y: float, z: float) -> Tuple[float, float, float]:
        # East-North-Up (local) → lat/lon/alt (geodetic)
        # Uses Haversine + vertical offset
        
    def geodetic_to_enu(self, lat: float, lon: float, alt: float) -> Tuple[float, float, float]:
        # Inverse transform
```

### 7. Platform Templates (`platforms/`)

Pre-configured bridges for common robots:

**Spot** (`spot.py`)
```python
SPOT_CONFIG = {
    "sensors": {
        "lidar": {
            "topic": "/scan",
            "frame_id": "lidar",
        },
        "rgb": {
            "topics": [
                "/camera/frontleft/image",
                "/camera/frontright/image",
                "/camera/back/image",
            ]
        }
    },
    "reference_frame": "map",
    "base_frame": "body",
}
```

**DJI M300 RTK** (`dji.py`)
```python
DJI_M300_CONFIG = {
    "sensors": {
        "lidar": {
            "topic": "/lidar_points",
            "frame_id": "lidar_frame",
        },
        "thermal": {
            "topic": "/zenmuse_h20t/thermal/image",
            "frame_id": "thermal_frame",
        },
        "rgb": {
            "topic": "/zenmuse_h20t/rgb/image",
        }
    },
    "reference_frame": "map",  # RTK provides geodetic
    "coordinate_mode": "geodetic",  # Direct lat/lon/alt
}
```

## Data Flow Example

**Scenario**: Warthog with LiDAR collecting terrain data

```
1. ROS2 publishes /warthog/scan (LaserScan) @ 10 Hz
   timestamp: 1721683200.123456
   
2. Bridge subscribes, receives LaserScan
   
3. LiDAR Adapter converts:
   - 270 beams → 270 (range, bearing, intensity) tuples
   - Groups into 5cm grid cells
   - Computes cell center + average intensity
   
4. TF Lookup:
   - Get warthog base_link → map @ timestamp
   - Get lidar_link → base_link (static)
   - Compose: lidar frame at (x=123.4, y=456.7, yaw=2.1)
   
5. Coordinate Transform:
   - Map origin set at first pose: (40.7128, -74.0060)
   - Grid cell at local (x=10, y=5) → geodetic (40.7130, -74.0057)
   
6. Batch Collection:
   - Accumulate observations for 1 second
   - 10 scans × ~50 cells = 500 observations
   
7. Push to Backend:
   ```python
   observations = [
       StorageObservation(
           id=uuid(),
           robot_id="warthog-1",
           timestamp=1721683200123456,  # us
           location_lat=40.7130,
           location_lon=-74.0057,
           sensor_type="lidar_intensity",
           value_json={"intensity": 128, "range_m": 15.3},
           confidence=0.92,
       ),
       # ... 499 more
   ]
   backend.insert_batch(observations)
   ```

## Launch Files

### Gazebo Simulation (`launch/sim.launch.py`)

```python
def generate_launch_description():
    # Start Gazebo with Warthog
    # Start PyTerrainMap ROS bridge
    # Start visualization (Rviz)
    # Start data recorder (bag)
    
    return LaunchDescription([
        gazebo_sim,
        pyterrain_bridge,
        rviz,
    ])
```

### Hardware Deployment (`launch/hardware.launch.py`)

```python
def generate_launch_description():
    # Load robot-specific params
    # Start hardware drivers
    # Start PyTerrainMap bridge
    # Start monitoring/diagnostics
    
    return LaunchDescription([
        robot_drivers,
        pyterrain_bridge,
        diagnostics,
    ])
```

### Multi-Robot Fleet (`launch/fleet.launch.py`)

```python
def generate_launch_description():
    # For each robot in fleet:
    #   - Start bridge with robot_id
    #   - Map topics to fleet namespace
    #   - Connect to shared backend
    # Start fleet coordinator (collision avoidance, etc.)
    
    return LaunchDescription([
        *[bridge_for_robot(robot) for robot in fleet],
        fleet_coordinator,
    ])
```

## Quality Assurance

### Timestamp Synchronization

Challenge: Sensors on different clocks, network delays

Solutions:
1. **PTP Mode** (preferred): Use PTP daemon for HW clock sync (Driftless)
2. **ROS Time Mode**: Use `/clock` topic (works in sim)
3. **Wall Clock Mode**: Host clock (least accurate, fallback)

### Frame Dropout Handling

When sensor misses for >2× expected period:
- Flag observation with `quality_flag: "incomplete_frame"`
- Lower confidence score
- Mark time range as sparse in backend

### Outlier Detection

For each sensor type, run online IQR detection:
- Temperature: flag values >3σ from local mean
- Range: flag returns at impossible distances
- Intensity: flag saturation/null returns

## Integration Testing

### Synthetic Data Generator

```python
class GazeboDataGenerator:
    def __init__(self, bridge_config):
        # Launch Gazebo with robots
        # Simulate sensor noise
        # Inject known targets (retroreflectors, thermal hot-spots)
        # Record ground truth
        
    def verify(self, backend):
        # Query observations from backend
        # Compare to ground truth
        # Report accuracy metrics
```

Example test scenario:
```yaml
scenario: "multi_robot_thermal"
duration_s: 60
robots:
  - name: "spot_1"
    model: "spot"
    trajectory: "circle_10m"
  - name: "spot_2"
    model: "spot"
    trajectory: "circle_10m"
targets:
  - position: [50.0, 50.0, 2.0]  # lat, lon, alt
    type: "thermal_hot_spot"
    temperature_c: 60
  - position: [50.01, 50.00, 1.5]
    type: "lidar_reflector"

verification:
  - query: "thermal observations near (50.0, 50.0)"
    expected_count_min: 100
    expected_temp_mean: [55, 65]
  - query: "lidar high-intensity returns"
    expected_count_min: 500
```

## Deployment Checklist

- [ ] ROS2 Humble or Jazzy installed
- [ ] robot_localization running (odometry + IMU fusion)
- [ ] TF static_transform_publisher for sensor mounts
- [ ] Backend running (PostgreSQL or memory)
- [ ] Config YAML prepared for robot
- [ ] Time sync (NTP or PTP) active
- [ ] Bag recorder running for debug
- [ ] Monitoring dashboard up (Grafana)

## Success Metrics

| Metric | Target | Method |
|--------|--------|--------|
| Data Ingestion Rate | 100K obs/sec | `psql: SELECT COUNT(*) FROM pyterrain_observations WHERE timestamp > now() - 1s` |
| Latency (Sensor→Backend) | <500ms P95 | Trace decorator on adapter callbacks |
| Frame Sync Error | <50ms | Compare TF timestamp vs actual sensor capture time |
| Data Completeness | >95% | Monitor adapter message drop rates |
| Coordinate Accuracy | <0.5m | Compare GPS-equipped robot vs computed poses |

