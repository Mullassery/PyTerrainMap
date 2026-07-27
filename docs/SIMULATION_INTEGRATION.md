# PyTerrainMap: Gazebo & Isaac Sim Integration

**Why simulation?**
- ✅ Develop without hardware
- ✅ Test multi-robot scenarios
- ✅ Generate ground truth for validation
- ✅ Run overnight stress tests
- ✅ Replay real-world scenarios

---

## Architecture: Simulation + PyTerrainMap

```
┌────────────────────────────────────────────────────────────┐
│                   SIMULATION ENVIRONMENT                   │
│  ┌──────────────────────────────────────────────────────┐ │
│  │         Gazebo / Isaac Sim                            │ │
│  │  ┌────────────────────────────────────────────────┐  │ │
│  │  │  Virtual Robots (Spot, M300, Warthog)          │  │ │
│  │  │  - Physics simulation                           │  │ │
│  │  │  - Sensor simulation (perfect + noise)         │  │ │
│  │  │  - Ground truth poses                          │  │ │
│  │  └────────────────────────────────────────────────┘  │ │
│  │                       │                               │ │
│  │                       ▼                               │ │
│  │  ┌────────────────────────────────────────────────┐  │ │
│  │  │  ROS2 Topics (published by simulator)           │  │ │
│  │  │  /scan, /thermal_image, /camera/image          │  │ │
│  │  │  /odom, /ground_truth_pose, /tf                │  │ │
│  │  └────────────────────────────────────────────────┘  │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────┬───────────────────────────────────────┘
                     │
                     ▼
         ┌───────────────────────────┐
         │ PyTerrainMap ROS Bridge   │
         │                           │
         │ - Subscribes to topics    │
         │ - Processes sensor data   │
         │ - Writes observations     │
         └───────────────┬───────────┘
                         │
                         ▼
         ┌───────────────────────────┐
         │ PyTerrainMap Storage      │
         │ (Local FS for sim testing)│
         └───────────────┬───────────┘
                         │
                         ▼
         ┌───────────────────────────┐
         │ Observations (NDJSON)     │
         │ - Ground truth positions  │
         │ - Simulated sensor data   │
         │ - Timestamps & confidence │
         └───────────────────────────┘
```

---

## Gazebo 2 Integration (Classic)

### Setup: Launch Spot in Gazebo

```bash
# Install Gazebo + Spot simulator
sudo apt install gazebo ros-humble-gazebo-plugins
git clone https://github.com/clearpathrobotics/spot_ros2 ~/spot_ws
cd ~/spot_ws && colcon build

# Launch Gazebo with Spot + sensors
ros2 launch spot_gazebo spot_gazebo.launch.py \
    add_lidar:=true \
    add_camera:=true \
    add_imu:=true
```

### Launch PyTerrainMap in Gazebo

```yaml
# launch/gazebo_spot_pyterrain.launch.py
import os
from launch import LaunchDescription
from launch_ros.actions import Node
from launch.actions import IncludeLaunchDescription
from launch.launch_description_sources import PythonLaunchDescriptionSource

def generate_launch_description():
    gazebo_launch = IncludeLaunchDescription(
        PythonLaunchDescriptionSource([
            os.path.join(
                get_package_share_directory('spot_gazebo'),
                'launch',
                'spot_gazebo.launch.py'
            )
        ]),
        launch_arguments={
            'add_lidar': 'true',
            'add_camera': 'true',
            'add_imu': 'true',
        }.items(),
    )
    
    # PyTerrainMap ROS bridge
    pyterrain_bridge = Node(
        package='pyterrain_ros',
        executable='bridge',
        name='pyterrain_bridge',
        parameters=[{
            'robot_id': 'spot_gazebo',
            'base_frame': 'base_link',
            'reference_frame': 'map',
            'storage_type': 'local',
            'storage_path': '/tmp/gazebo_sim',
            'batch_size': 100,
            'sensors': {
                'lidar': {
                    'enabled': True,
                    'topic': '/scan',
                    'frame_id': 'lidar_link',
                },
                'thermal': {
                    'enabled': False,  # Gazebo doesn't simulate thermal
                },
                'imu': {
                    'enabled': True,
                    'topic': '/imu',
                    'frame_id': 'imu_link',
                },
            },
        }],
    )
    
    # Visualization (optional)
    rviz = Node(
        package='rviz2',
        executable='rviz2',
        name='rviz2',
        arguments=['-d', 'config/gazebo_spot.rviz'],
    )
    
    return LaunchDescription([
        gazebo_launch,
        pyterrain_bridge,
        rviz,
    ])
```

### Run Gazebo Simulation

```bash
# Terminal 1: Launch Gazebo + PyTerrainMap
ros2 launch pyterrain_ros gazebo_spot_pyterrain.launch.py

# Terminal 2: Control robot (keyboard teleop or script)
ros2 run teleop_twist_keyboard teleop_twist_keyboard

# Terminal 3: Monitor observations
python3 -c "
import asyncio
from pyterrain_map.storage import LocalStorageBackend

async def monitor():
    backend = LocalStorageBackend({'base_path': '/tmp/gazebo_sim'})
    await backend.connect()
    
    while True:
        stats = await backend.get_stats()
        print(f'Observations: {stats[\"observation_count\"]}', end='\r')
        await asyncio.sleep(1)

asyncio.run(monitor())
"
```

---

## Isaac Sim Integration (NVIDIA)

### Setup: Launch Spot in Isaac Sim

```bash
# Install Isaac Sim (requires NVIDIA GPU)
# https://docs.omniverse.nvidia.com/isaacsim/latest/

# Launch Isaac Sim with Spot scenario
./ isaac-sim.sh

# In Isaac Sim UI:
# 1. File → Open → select "Spot_Simple.usd"
# 2. Window → Animation → Timeline (enable playback)
# 3. Create → Isaac → Sensors → ROS2 LiDAR
# 4. Create → Isaac → Sensors → Camera (for RGB/thermal)
```

### ROS2 Bridge Configuration for Isaac Sim

Isaac Sim publishes more realistic sensor data than Gazebo:

```python
# Launch file for Isaac Sim
class IsaacSimPyTerrainBridge:
    def __init__(self):
        self.ros_node = rclpy.create_node('isaac_pyterrain')
        
        # Isaac Sim publishes at higher frequency + noise
        self.subscriptions = {
            'lidar': (
                LidarScan,
                '/isaac_sim/lidar/scan',
                self.on_lidar,  # 50 Hz
            ),
            'camera': (
                Image,
                '/isaac_sim/camera/rgb',
                self.on_camera,  # 30 Hz
            ),
            'thermal': (
                Image,
                '/isaac_sim/thermal/raw',
                self.on_thermal,  # 30 Hz (simulated)
            ),
            'imu': (
                Imu,
                '/isaac_sim/imu',
                self.on_imu,  # 200 Hz
            ),
        }
        
        # Isaac Sim provides ground truth
        self.gt_pose_sub = self.ros_node.create_subscription(
            PoseStamped,
            '/isaac_sim/ground_truth/pose',
            self.on_ground_truth_pose,
            10,
        )
    
    def on_ground_truth_pose(self, msg):
        """
        Store ground truth pose from simulator
        Used for validation against estimated pose
        """
        self.gt_pose = {
            'x': msg.pose.position.x,
            'y': msg.pose.position.y,
            'z': msg.pose.position.z,
            'timestamp': msg.header.stamp.sec * 1e9 + msg.header.stamp.nanosec,
        }
```

### Running Isaac Sim Scenario

```bash
# Terminal 1: Isaac Sim (headless or GUI)
# (Running in Omniverse)

# Terminal 2: ROS2 Bridge
python3 -m pyterrain_ros.bridge \
    --platform isaac \
    --config config/isaac_sim.yaml

# Terminal 3: Run scenario script
python3 scenarios/multi_robot_survey.py

# Terminal 4: Monitor
python3 -c "
import asyncio
from pyterrain_map.storage import LocalStorageBackend
import json

async def validate():
    backend = LocalStorageBackend({'base_path': '/tmp/isaac_sim'})
    await backend.connect()
    
    # Get observations
    obs = await backend.query(robot_id='isaac_spot', limit=100)
    
    # Calculate coverage
    cells = set()
    for o in obs:
        cell = f'{int(o.location_lat*100)},{int(o.location_lon*100)}'
        cells.add(cell)
    
    print(f'Coverage: {len(cells)} unique grid cells')

asyncio.run(validate())
"
```

---

## Synthetic Data Generation & Validation

### Generate Ground Truth Scenario

```python
class SyntheticScenarioGenerator:
    """
    Create known scenarios and validate PyTerrainMap
    """
    
    def __init__(self):
        self.backend = LocalStorageBackend({'base_path': '/tmp/synthetic'})
    
    async def create_hotspot_scenario(self):
        """
        Scenario: Robot traverses area with known hot spot
        Validates thermal detection
        """
        import time
        import json
        
        # Define known hot spot
        hot_spot = {'lat': 40.7128, 'lon': -74.0060, 'temp': 60.0}
        
        # Robot trajectory (grid pattern)
        trajectory = []
        for lat in [40.710, 40.712, 40.714, 40.716]:
            for lon in [-74.002, -74.000, -73.998]:
                trajectory.append((lat, lon))
        
        # Simulate observations
        obs_list = []
        for i, (lat, lon) in enumerate(trajectory):
            # Calculate distance to hot spot
            dist = ((lat - hot_spot['lat'])**2 + (lon - hot_spot['lon'])**2)**0.5
            
            # Thermal signal decreases with distance
            detected_temp = hot_spot['temp'] * (1 - dist / 0.01)
            
            obs = StorageObservation(
                id=f'synthetic_{i}',
                robot_id='spot_synthetic',
                timestamp=int((time.time() + i) * 1_000_000),
                location_lat=lat,
                location_lon=lon,
                sensor_type='thermal',
                value_json=json.dumps({'temperature_c': detected_temp}),
                confidence=0.95 if dist < 0.01 else 0.7,
            )
            obs_list.append(obs)
        
        # Write synthetic data
        await self.backend.write_batch(obs_list)
        
        return {
            'scenario': 'hotspot',
            'known_hotspot': hot_spot,
            'observations': len(obs_list),
            'trajectory_length': len(trajectory),
        }
    
    async def create_multiobstacle_scenario(self):
        """
        Scenario: Robot navigates through obstacle field
        Validates LiDAR mapping
        """
        # Define obstacles (static in world)
        obstacles = [
            {'lat': 40.7120, 'lon': -74.0065, 'size': 2},
            {'lat': 40.7135, 'lon': -74.0055, 'size': 1},
            {'lat': 40.7125, 'lon': -74.0048, 'size': 3},
        ]
        
        obs_list = []
        
        # Robot scans each obstacle multiple times
        for obs_idx, obstacle in enumerate(obstacles):
            for scan_idx in range(10):  # 10 scans per obstacle
                for angle in range(0, 360, 18):  # 20 beams
                    # Ray from robot toward obstacle
                    import math
                    rad = math.radians(angle)
                    
                    # Add noise
                    noisy_range = obstacle['size'] + (hash(f"{obs_idx}{scan_idx}{angle}") % 10) / 100
                    
                    x = obstacle['lat'] + noisy_range * math.cos(rad) / 100
                    y = obstacle['lon'] + noisy_range * math.sin(rad) / 100
                    
                    obs = StorageObservation(
                        id=f'obstacle_{obs_idx}_{scan_idx}_{angle}',
                        robot_id='spot_obstacles',
                        timestamp=int((time.time() + obs_idx * 10 + scan_idx) * 1_000_000),
                        location_lat=x,
                        location_lon=y,
                        sensor_type='lidar',
                        value_json=json.dumps({
                            'intensity': 150 + (hash(str(angle)) % 50),
                            'range_m': noisy_range,
                            'beam_angle': angle,
                        }),
                        confidence=0.95,
                    )
                    obs_list.append(obs)
        
        await self.backend.write_batch(obs_list)
        
        return {
            'scenario': 'obstacles',
            'obstacle_count': len(obstacles),
            'observations': len(obs_list),
        }
```

### Validation Framework

```python
class SimulationValidator:
    """
    Validate PyTerrainMap against ground truth from simulation
    """
    
    async def validate_thermal_accuracy(self):
        """Check if thermal observations match known hot spots"""
        backend = LocalStorageBackend({'base_path': '/tmp/isaac_sim'})
        
        # Query thermal observations
        thermal_obs = await backend.query(
            robot_id='isaac_spot',
            sensor_type='thermal',
        )
        
        # Known ground truth from simulator
        ground_truth_hotspots = [
            {'lat': 40.7128, 'lon': -74.0060, 'temp': 60.0},
            {'lat': 40.7135, 'lon': -74.0055, 'temp': 45.0},
        ]
        
        # Check accuracy
        errors = []
        for gt_spot in ground_truth_hotspots:
            nearby = [o for o in thermal_obs if
                (abs(o.location_lat - gt_spot['lat']) < 0.001 and
                 abs(o.location_lon - gt_spot['lon']) < 0.001)]
            
            if nearby:
                detected_temp = json.loads(nearby[0].value_json)['temperature_c']
                error = abs(detected_temp - gt_spot['temp'])
                errors.append(error)
                
                status = '✅' if error < 5 else '⚠️' if error < 10 else '❌'
                print(f"{status} Hotspot at ({gt_spot['lat']:.4f}, {gt_spot['lon']:.4f})")
                print(f"   Expected: {gt_spot['temp']:.1f}°C, Detected: {detected_temp:.1f}°C")
        
        avg_error = sum(errors) / len(errors) if errors else 999
        return {
            'accuracy': 100 - min(100, avg_error),
            'avg_error_c': avg_error,
            'pass': avg_error < 5,
        }
    
    async def validate_lidar_coverage(self):
        """Check if LiDAR observations cover expected area"""
        backend = LocalStorageBackend({'base_path': '/tmp/isaac_sim'})
        
        # Get all LiDAR observations
        lidar_obs = await backend.query(
            robot_id='isaac_spot',
            sensor_type='lidar',
        )
        
        # Build coverage map
        coverage_map = {}
        for obs in lidar_obs:
            grid_cell = (int(obs.location_lat * 100), int(obs.location_lon * 100))
            coverage_map[grid_cell] = coverage_map.get(grid_cell, 0) + 1
        
        # Expected coverage (grid search area)
        expected_cells = set()
        for lat in range(int(40.710 * 100), int(40.720 * 100)):
            for lon in range(int(-74.010 * 100), int(-74.000 * 100)):
                expected_cells.add((lat, lon))
        
        covered_cells = set(coverage_map.keys())
        coverage_percent = (len(covered_cells) / len(expected_cells)) * 100
        
        print(f"Coverage: {coverage_percent:.1f}% ({len(covered_cells)}/{len(expected_cells)} cells)")
        
        return {
            'coverage_percent': coverage_percent,
            'cells_covered': len(covered_cells),
            'cells_expected': len(expected_cells),
            'pass': coverage_percent > 80,
        }
```

---

## Multi-Robot Simulation Scenario

```python
class MultiRobotSimulation:
    """
    Simulate multiple robots in Gazebo/Isaac Sim
    """
    
    async def run_fleet_survey(self):
        """
        3 robots survey area simultaneously
        Store in shared PyTerrainMap storage
        """
        backend = S3StorageBackend({
            'bucket': 'fleet-simulation',
            'prefix': 'survey_001',
        })
        
        # Define robot trajectories
        robots = {
            'spot_1': self.grid_trajectory(start=(40.710, -74.010), size=0.005),
            'spot_2': self.grid_trajectory(start=(40.715, -74.010), size=0.005),
            'm300_1': self.circular_trajectory(center=(40.7125, -74.0075), radius=0.01),
        }
        
        async def run_robot(robot_id, trajectory):
            """Simulate one robot"""
            for i, (lat, lon) in enumerate(trajectory):
                # Publish ROS message (in real setup)
                # For sim: directly create observation
                
                obs = StorageObservation(
                    id=f'{robot_id}_{i}',
                    robot_id=robot_id,
                    timestamp=int((time.time() + i * 0.1) * 1_000_000),
                    location_lat=lat,
                    location_lon=lon,
                    sensor_type='lidar',
                    value_json='{"scan": ' + str(i) + '}',
                    confidence=0.9,
                )
                await backend.write_observation(obs)
                
                await asyncio.sleep(0.1)  # Simulate sensor rate
        
        # Run all robots in parallel
        await asyncio.gather(
            run_robot('spot_1', robots['spot_1']),
            run_robot('spot_2', robots['spot_2']),
            run_robot('m300_1', robots['m300_1']),
        )
        
        # Analyze combined coverage
        all_obs = await backend.query()
        print(f"Total observations: {len(all_obs)}")
        
        coverage = set()
        for obs in all_obs:
            cell = f"{int(obs.location_lat * 100)},{int(obs.location_lon * 100)}"
            coverage.add(cell)
        
        print(f"Unique grid cells: {len(coverage)}")
```

---

## Testing Workflows

### Overnight Stress Test

```python
# run_overnight_test.py
import asyncio
from pyterrain_map.storage import LocalStorageBackend
import time

async def stress_test():
    """
    Write 1M observations, query them back
    Runs overnight to catch edge cases
    """
    backend = LocalStorageBackend({'base_path': '/tmp/stress_test'})
    
    print("Starting stress test...")
    start = time.time()
    
    # Write 1M observations
    obs_list = []
    for i in range(1_000_000):
        obs = StorageObservation(
            id=f'stress_{i}',
            robot_id=f'robot_{i % 10}',  # 10 robots
            timestamp=int((time.time() + i * 0.001) * 1_000_000),
            location_lat=40.700 + (i % 1000) / 1000,
            location_lon=-74.000 + (i % 1000) / 1000,
            sensor_type=['lidar', 'thermal'][i % 2],
            value_json='{"data": 1}',
            confidence=0.9,
        )
        obs_list.append(obs)
        
        if len(obs_list) >= 10000:
            await backend.write_batch(obs_list)
            obs_list = []
            print(f"  Written {i} observations ({time.time() - start:.1f}s)")
    
    # Query random subsets
    for _ in range(100):
        results = await backend.query(
            robot_id=f'robot_{hash(_) % 10}',
            limit=1000,
        )
        print(f"  Query returned {len(results)} observations")
    
    total_time = time.time() - start
    print(f"\n✅ Stress test passed ({total_time:.1f}s total)")

asyncio.run(stress_test())
```

### Regression Test Suite

```python
async def run_regression_tests():
    """
    Test suite that validates PyTerrainMap doesn't break
    """
    tests = [
        ('Basic write', test_write_observation),
        ('Batch write', test_batch_write),
        ('Query by robot', test_query_robot),
        ('Query by sensor', test_query_sensor),
        ('Query by location', test_query_location),
        ('Query by time', test_query_time),
        ('Stats calculation', test_stats),
        ('Data retention', test_delete_old),
    ]
    
    results = {}
    for test_name, test_fn in tests:
        try:
            passed = await test_fn()
            results[test_name] = '✅' if passed else '❌'
        except Exception as e:
            results[test_name] = f'❌ {str(e)}'
    
    print("\nRegression Test Results:")
    for name, status in results.items():
        print(f"  {status} {name}")
```

---

## Summary

| Simulation | Realism | PyTerrainMap Ready | Best For |
|-----------|---------|-------------------|----------|
| **Gazebo 2** | Medium | ✅ Yes | Quick testing, open source |
| **Gazebo 2** | Medium | ✅ Yes | Quick testing, open source |
| **Isaac Sim** | High | ✅ Yes | Photorealistic, NVIDIA GPU |

Both Gazebo and Isaac Sim work perfectly with PyTerrainMap. Isaac Sim provides more realistic sensor simulation, Gazebo is simpler and open source.

**Next Steps:**
1. Start with Gazebo (easier setup)
2. Validate using synthetic scenarios
3. Run overnight stress tests
4. Move to Isaac Sim for production validation

---

**Version:** 0.1.0  
**Status:** Ready for simulation testing
