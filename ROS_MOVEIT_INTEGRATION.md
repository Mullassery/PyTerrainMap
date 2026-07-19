# PyTerrainMap: ROS2 & MoveIt Integration Guide

**What you'll learn:**
- How PyTerrainMap integrates with ROS2 ecosystem
- How to use observations with MoveIt motion planning
- How to coordinate with Nav2 navigation
- Real-world multi-robot scenarios

---

## Overview: PyTerrainMap in the ROS2 Ecosystem

```
┌─────────────────────────────────────────────────────────────────┐
│                        ROS2 ECOSYSTEM                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Nav2       │  │   MoveIt2    │  │   tf2        │         │
│  │ (Navigation) │  │ (Motion Plng)│  │ (Transforms) │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                  │                  │                 │
│         └──────────────────┼──────────────────┘                 │
│                            │                                    │
│    Subscribes to Topics & Services                             │
│         /tf, /map, /scan, /costmap, etc.                       │
│                            │                                    │
│  ┌─────────────────────────▼──────────────────────────┐        │
│  │                                                    │        │
│  │      PyTerrainMap ROS Bridge (Phase 2)            │        │
│  │                                                    │        │
│  │  ┌────────────────────────────────────────────┐   │        │
│  │  │ Subscribe to:                              │   │        │
│  │  │ - /scan (LaserScan) → LiDARAdapter        │   │        │
│  │  │ - /thermal_image → ThermalAdapter         │   │        │
│  │  │ - /tf & /tf_static → TFListener           │   │        │
│  │  │ - /imu/data → IMUAdapter                  │   │        │
│  │  │                                            │   │        │
│  │  │ Transform & Normalize                      │   │        │
│  │  │ - Robot pose from TF tree                 │   │        │
│  │  │ - Sensor frame to base_link               │   │        │
│  │  │ - Base_link to map/odom (ENU→Geodetic)    │   │        │
│  │  │                                            │   │        │
│  │  │ Publish:                                   │   │        │
│  │  │ - /pyterrain/observations (topic)         │   │        │
│  │  │ - /pyterrain/status (diagnostics)         │   │        │
│  │  │ - /pyterrain/query (service for queries)  │   │        │
│  │  └────────────────────────────────────────────┘   │        │
│  │                                                    │        │
│  └────────────────────┬───────────────────────────────┘        │
│                       │                                         │
│                Storage Backend                                  │
│           (Local FS | S3 | GCS | ADLS)                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## How MoveIt2 + PyTerrainMap Work Together

### Use Case: Manipulation in Unmapped Environment

**Scenario:** Spot with an arm needs to pick objects on a construction site

#### 1. MoveIt2 Plans Motion Based on:
- Robot's current pose (from TF tree)
- Collision map (typically from /occupancy_grid or /scan)
- Target location (gripper target)

#### 2. PyTerrainMap Captures Environment:
- Stores all sensor observations (LiDAR scans, thermal data)
- Geo-indexes by location
- Builds historical map

#### 3. Post-Task Analysis:
```python
from pyterrain_map.storage import S3StorageBackend
from datetime import datetime, timedelta

# Query observations from during task execution
backend = S3StorageBackend({"bucket": "spot-tasks"})

# Get all sensor data from task timeframe
start_time = datetime(2026, 7, 19, 14, 30, 0)
end_time = datetime(2026, 7, 19, 14, 35, 0)

observations = await backend.query(
    robot_id="spot-arm-1",
    start_time=int(start_time.timestamp() * 1_000_000),
    end_time=int(end_time.timestamp() * 1_000_000),
)

# Analyze: Which areas did robot sense?
print(f"Robot scanned {len(observations)} locations")

# Which objects were nearby?
import json
for obs in observations:
    data = json.loads(obs.value_json)
    if data.get("intensity", 0) > 100:  # High-intensity reflective object
        print(f"  Found object at {obs.location_lat}, {obs.location_lon}")

# Export for 3D reconstruction
with open("task_pointcloud.xyz", "w") as f:
    for obs in observations:
        f.write(f"{obs.location_lat} {obs.location_lon} "
                f"{json.loads(obs.value_json).get('intensity', 0)}\n")
```

---

## ROS2 Topic Architecture

### Topics PyTerrainMap Subscribes To

```python
# In the ROS bridge node (Python with rclpy):

self.node.create_subscription(
    LaserScan,
    '/scan',
    self.on_scan,
    qos_profile=QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
)

self.node.create_subscription(
    Image,
    '/thermal_image',
    self.on_thermal,
    qos_profile=QoSProfile(depth=1, reliability=ReliabilityPolicy.BEST_EFFORT)
)

self.node.create_subscription(
    TFMessage,
    '/tf',
    self.on_tf,
    qos_profile=QoSProfile(depth=100, reliability=ReliabilityPolicy.BEST_EFFORT)
)

self.node.create_subscription(
    TFMessage,
    '/tf_static',
    self.on_tf_static,
    qos_profile=QoSProfile(depth=100, reliability=ReliabilityPolicy.TRANSIENT_LOCAL)
)
```

### Topics PyTerrainMap Publishes

```python
# Observations as they're processed
self.obs_pub = self.node.create_publisher(
    StorageObservation,  # Custom message type
    '/pyterrain/observations',
    qos_profile=QoSProfile(depth=10)
)

# Bridge status/health
self.status_pub = self.node.create_publisher(
    DiagnosticArray,
    '/diagnostics',
    qos_profile=QoSProfile(depth=1)
)
```

### Services PyTerrainMap Provides

```python
# Query service: request observations
self.query_service = self.node.create_service(
    PyTerrainQuery,
    '/pyterrain/query',
    self.handle_query
)

# Service definition
class PyTerrainQuery(srv.Service):
    class Request:
        string robot_id
        int64 start_time  # microseconds
        int64 end_time
        string sensor_type
        float32 lat_min
        float32 lat_max
        float32 lon_min
        float32 lon_max
        int32 limit
    
    class Response:
        StorageObservation[] observations
        int32 total_count
        string status
```

### Services PyTerrainMap Consumes

```python
# Gets current robot pose from TF2
from tf2_ros import TransformListener, Buffer

self.tf_buffer = Buffer()
self.tf_listener = TransformListener(self.tf_buffer, self.node)

try:
    transform = self.tf_buffer.lookup_transform(
        target_frame='map',
        source_frame='base_link',
        time=rclpy.time.Time()
    )
    x = transform.transform.translation.x
    y = transform.transform.translation.y
except tf2.LookupException:
    self.node.get_logger().warning("No transform available")
```

---

## MoveIt2 Integration Examples

### Example 1: Safe Manipulation Using PyTerrainMap

**Scenario:** Verify if grasped object matches expected material (thermal data)

```python
import rclpy
from moveit_commander import MoveGroupCommander
from pyterrain_map.storage import S3StorageBackend
import asyncio
import json

class ManipulationWithTerrain:
    def __init__(self):
        rclpy.init()
        self.node = rclpy.create_node('manipulation_node')
        self.storage = S3StorageBackend({"bucket": "spot-tasks"})
        self.move_group = MoveGroupCommander("arm")
    
    async def pick_object(self, object_location_lat, object_location_lon):
        """
        1. Move arm to object location
        2. Grasp object
        3. Query thermal data from that location
        4. Verify material matches expectations
        """
        # 1. Plan and execute move
        self.move_group.set_pose_target([
            object_location_lat,
            object_location_lon,
            1.0,  # height
        ])
        self.move_group.go(wait=True)
        
        # 2. Execute grasp (actuate gripper)
        await self.gripper.close()
        
        # 3. Query thermal observations from that location
        thermal_obs = await self.storage.query(
            robot_id="spot-arm-1",
            sensor_type="thermal",
            lat_min=object_location_lat - 0.0001,
            lat_max=object_location_lat + 0.0001,
            lon_min=object_location_lon - 0.0001,
            lon_max=object_location_lon + 0.0001,
            limit=100,
        )
        
        # 4. Analyze thermal signature
        if thermal_obs:
            data = json.loads(thermal_obs[0].value_json)
            temp_c = data["temperature_c"]
            
            if temp_c < -20:
                print("❌ Object is too cold - likely ice/frozen")
                await self.gripper.open()
                return False
            elif temp_c > 60:
                print("❌ Object is too hot - safety hazard")
                await self.gripper.open()
                return False
            else:
                print(f"✅ Object temperature OK: {temp_c}°C")
                return True
        
        return True
```

### Example 2: Dynamic Obstacle Avoidance

**Scenario:** MoveIt2 avoids obstacles detected by PyTerrainMap

```python
class DynamicPathPlanning:
    async def plan_with_terrain_avoidance(self, goal_pose):
        """
        1. Get recent sensor observations
        2. Update collision map
        3. Plan path avoiding obstacles
        """
        # 1. Query recent LiDAR observations
        recent_scans = await self.storage.query(
            robot_id="spot-1",
            sensor_type="lidar",
            start_time=int((time.time() - 30) * 1_000_000),  # last 30 seconds
            limit=50000,
        )
        
        # 2. Build occupancy grid from observations
        obstacles = set()
        for obs in recent_scans:
            data = json.loads(obs.value_json)
            if data.get("intensity", 0) > 150:  # Strong reflector = obstacle
                grid_x = int(obs.location_lat * 100)
                grid_y = int(obs.location_lon * 100)
                obstacles.add((grid_x, grid_y))
        
        # 3. Add obstacles to collision map
        for grid_x, grid_y in obstacles:
            self.add_collision_box(grid_x, grid_y)
        
        # 4. Plan with MoveIt2
        self.move_group.set_pose_target(goal_pose)
        plan = self.move_group.plan()
        
        if plan:
            self.move_group.execute(plan, wait=True)
            return True
        else:
            print("❌ No collision-free path found")
            return False
```

### Example 3: Grasp Pose Estimation from Thermal Data

```python
class GraspPlanning:
    async def estimate_grasp_from_thermal(self, location_lat, location_lon):
        """
        Use thermal observations to estimate object characteristics
        and select best grasp pose
        """
        # Query thermal observations
        thermal_data = await self.storage.query(
            robot_id="spot-arm-1",
            sensor_type="thermal",
            lat_min=location_lat - 0.001,
            lat_max=location_lat + 0.001,
            lon_min=location_lon - 0.001,
            lon_max=location_lon + 0.001,
        )
        
        if not thermal_data:
            return None
        
        # Analyze thermal pattern
        temperatures = []
        for obs in thermal_data:
            data = json.loads(obs.value_json)
            temperatures.append(data.get("temperature_c", 20))
        
        avg_temp = sum(temperatures) / len(temperatures)
        temp_variance = sum((t - avg_temp) ** 2 for t in temperatures) / len(temperatures)
        
        # High variance = non-uniform object = complex grasp needed
        if temp_variance > 10:
            grasp_strategy = "multi_finger_grasp"
            confidence = 0.7
        else:
            grasp_strategy = "simple_grasp"
            confidence = 0.9
        
        return {
            "strategy": grasp_strategy,
            "confidence": confidence,
            "avg_temperature": avg_temp,
            "location": (location_lat, location_lon),
        }
```

---

## Nav2 Integration Examples

### Example: Multi-Waypoint Navigation Using Terrain Data

```python
class TerrainAwareNavigation:
    async def navigate_with_terrain_avoidance(self):
        """
        Use PyTerrainMap observations to guide Nav2 navigation
        """
        # 1. Get all recent observations (last hour)
        recent_obs = await self.storage.query(
            robot_id="spot-1",
            start_time=int((time.time() - 3600) * 1_000_000),
        )
        
        # 2. Find hazardous areas (high thermal or dense obstacles)
        hazard_zones = []
        obstacle_count = {}
        
        for obs in recent_obs:
            grid_cell = f"{int(obs.location_lat*100)},{int(obs.location_lon*100)}"
            obstacle_count[grid_cell] = obstacle_count.get(grid_cell, 0) + 1
            
            if obs.sensor_type == "thermal":
                data = json.loads(obs.value_json)
                if data.get("temperature_c", 20) > 60:  # Hot area
                    hazard_zones.append((obs.location_lat, obs.location_lon))
        
        # 3. Identify safe corridors (low obstacle count)
        safe_cells = [
            cell for cell, count in obstacle_count.items() 
            if count < 10  # Less than 10 observations
        ]
        
        # 4. Plan waypoints avoiding hazard zones
        waypoints = []
        for cell in safe_cells[:5]:  # 5 waypoints
            lat, lon = cell.split(',')
            waypoints.append({
                "pose": {
                    "position": {"x": float(lat), "y": float(lon), "z": 0}
                }
            })
        
        # 5. Send waypoints to Nav2
        for waypoint in waypoints:
            self.nav2_client.send_goal(waypoint)
            await self.nav2_client.get_result()
            
            # Update observations as we go
            await self.storage.write_observation(
                self.create_nav_observation()
            )
```

---

## Multi-Robot Coordination with PyTerrainMap

### Fleet Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     FLEET CONTROL CENTER                    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │ PyTerrainMap Fleet Database                          │  │
│  │ S3://fleet-data/                                     │  │
│  │  ├─ 2024/07/19/spot-1/grid_40.7_-74.0.ndjson       │  │
│  │  ├─ 2024/07/19/spot-2/grid_40.7_-74.0.ndjson       │  │
│  │  ├─ 2024/07/19/m300-1/grid_40.7_-74.0.ndjson       │  │
│  │  └─ 2024/07/19/warthog-1/grid_40.7_-74.0.ndjson   │  │
│  └─────────────────────────────────────────────────────┘  │
│                           ▲                                 │
│        All robots write to same S3 bucket                  │
│        Partitioned by (date, robot_id, location grid)      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
         ▲          ▲          ▲          ▲
         │          │          │          │
    ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐
    │ Spot-1 │  │ Spot-2 │  │M300-1  │  │Warthog-│
    └────────┘  └────────┘  └────────┘  └────────┘
```

### Fleet Coordination Example

```python
class FleetCoordinator:
    async def coordinate_fleet(self):
        """
        Central controller that coordinates multiple robots
        """
        backend = S3StorageBackend({"bucket": "fleet-data"})
        
        robots = ["spot-1", "spot-2", "m300-1", "warthog-1"]
        
        # 1. Get current status of all robots
        fleet_status = {}
        for robot_id in robots:
            recent = await backend.query(
                robot_id=robot_id,
                start_time=int((time.time() - 60) * 1_000_000),  # last 60s
            )
            fleet_status[robot_id] = {
                "active": len(recent) > 0,
                "observation_count": len(recent),
            }
        
        # 2. Find coverage gaps
        all_observations = await backend.query()
        covered_cells = set()
        for obs in all_observations:
            cell = f"{int(obs.location_lat*100)},{int(obs.location_lon*100)}"
            covered_cells.add(cell)
        
        # 3. Assign robots to unvisited areas
        target_grid = generate_coverage_grid()
        for robot_id in robots:
            if not fleet_status[robot_id]["active"]:
                continue
            
            # Find nearest uncovered cell
            unvisited = [c for c in target_grid if c not in covered_cells]
            if unvisited:
                nearest = min(unvisited, key=lambda c: distance_to(robot_id, c))
                self.send_goal_to_robot(robot_id, nearest)
        
        # 4. Monitor coverage
        coverage_percent = (len(covered_cells) / len(target_grid)) * 100
        print(f"Fleet coverage: {coverage_percent:.1f}%")
```

---

## Data Sharing Between Robots

```python
class MultiRobotDataSharing:
    async def share_observations(self):
        """
        Robot A discovers something, Robot B benefits from it
        """
        backend = S3StorageBackend({"bucket": "shared-data"})
        
        # Robot A (Spot) finds hot object
        spot_thermal = await backend.query(
            robot_id="spot-1",
            sensor_type="thermal",
            start_time=int((time.time() - 10) * 1_000_000),
        )
        
        if spot_thermal:
            for obs in spot_thermal:
                data = json.loads(obs.value_json)
                if data.get("temperature_c", 0) > 80:
                    # Publish alert
                    self.publish_alert(
                        f"Hot object at {obs.location_lat}, {obs.location_lon}: "
                        f"{data['temperature_c']}°C"
                    )
                    
                    # Robot B (M300) can now avoid this area
                    # Or Robot C (Warthog) can investigate
```

---

## Performance Optimization for Real-Time Operations

### Buffering & Batching

```python
class OptimizedPyTerrainBridge:
    def __init__(self):
        self.observation_buffer = []
        self.buffer_size = 500
        self.flush_timeout = 5.0  # seconds
    
    async def on_sensor_message(self, obs):
        """
        Buffer observations and batch-write to storage
        """
        self.observation_buffer.append(obs)
        
        # Flush when buffer is full OR timeout
        if len(self.observation_buffer) >= self.buffer_size:
            await self.flush_buffer()
    
    async def flush_buffer(self):
        """Batch write for efficiency"""
        if not self.observation_buffer:
            return
        
        written = await self.backend.write_batch(self.observation_buffer)
        self.node.get_logger().info(
            f"Flushed {written} observations to storage"
        )
        self.observation_buffer = []
```

### Query Optimization

```python
# ✅ GOOD: Filter early (storage does the work)
results = await backend.query(
    robot_id="spot-1",
    sensor_type="lidar",
    lat_min=40.70,
    lat_max=40.72,
    lon_min=-74.01,
    lon_max=-74.00,
    start_time=recent_time,
    limit=1000,
)

# ❌ BAD: Get everything then filter (wastes bandwidth)
all_results = await backend.query(limit=999999)
filtered = [r for r in all_results if r.robot_id == "spot-1"]
```

---

## Troubleshooting ROS Integration

### Problem: TF Transforms Not Available

```python
# Check if TF is publishing
ros2 topic list | grep tf

# Monitor TF tree
ros2 run tf2_tools view_frames.py
# Generates frames.pdf showing transform tree

# In PyTerrainMap, verify TF lookup
try:
    tf = tf_listener.lookup_transform(
        target_frame="map",
        source_frame="base_link",
        timestamp=int(time.time() * 1e9)
    )
    print(f"✅ Transform found: {tf}")
except tf2.LookupException as e:
    print(f"❌ Transform lookup failed: {e}")
```

### Problem: Observations Not Getting Stored

```bash
# 1. Check ROS bridge is running
ps aux | grep pyterrain

# 2. Check topic subscription
ros2 topic list -v | grep pyterrain

# 3. Check storage backend connection
python3 -c "
import asyncio
from pyterrain_map.storage import S3StorageBackend

async def test():
    backend = S3StorageBackend({'bucket': 'test'})
    result = await backend.connect()
    print(f'Storage connected: {result}')

asyncio.run(test())
"

# 4. Check observation count in storage
python3 -c "
import asyncio
from pyterrain_map.storage import S3StorageBackend

async def check():
    backend = S3StorageBackend({'bucket': 'your-bucket'})
    stats = await backend.get_stats()
    print(f'Observations: {stats[\"observation_count\"]}')

asyncio.run(check())
"
```

### Problem: High Latency in Storage Writes

```python
# Solution: Increase batch size and timeout
bridge = PyTerrainROSBridge({
    "batch_size": 1000,       # Default 100
    "batch_timeout_ms": 10000, # Default 1000
    "max_queue_size": 5000,
})

# Monitor queue depth
self.node.create_timer(1.0, self.log_queue_stats)

def log_queue_stats(self):
    queue_depth = len(self.observation_buffer)
    self.node.get_logger().info(f"Queue depth: {queue_depth}")
```

---

## Best Practices

### ✅ DO

- ✅ Use high QoS for /tf (TRANSIENT_LOCAL for static transforms)
- ✅ Batch observations (write 100+ at a time, not one-by-one)
- ✅ Filter queries by robot_id, sensor_type, time, location
- ✅ Use local storage during development, cloud in production
- ✅ Monitor queue depth and backlog
- ✅ Export observations in NDJSON for offline analysis

### ❌ DON'T

- ❌ Query without filters (gets everything)
- ❌ Write single observations one-by-one (too slow)
- ❌ Assume TF transforms are always available (check first)
- ❌ Store sensitive data without encryption
- ❌ Ignore storage connection failures (implement retry logic)

---

## Summary

PyTerrainMap integrates seamlessly with ROS2:

| Component | PyTerrainMap Role |
|-----------|-------------------|
| **ROS2 Core** | Subscribes to /tf, sensors; publishes observations |
| **MoveIt2** | Uses terrain data for grasp planning & verification |
| **Nav2** | Receives obstacle/hazard info for path planning |
| **TF2** | Provides coordinate transformations |
| **RQT** | Can visualize observations via custom plugins |

Everything is **first-class**: not bolted on, but designed from the ground up to work with ROS2.

---

**Next:** See `GETTING_STARTED.md` for installation and `ROS_BRIDGE_IMPLEMENTATION_COMPLETE.md` for code reference.
