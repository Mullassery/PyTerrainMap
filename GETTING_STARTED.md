# PyTerrainMap: Getting Started Guide

**What is this?** A collaborative terrain mapping platform for multi-robot fleets that stores sensor observations in your choice of cloud storage (S3, GCS, ADLS) or local disk.

**What can it do?**
- ✅ Collect LiDAR and thermal data from robots
- ✅ Automatically geo-localize observations (lat/lon)
- ✅ Store observations immutably in cloud/local storage
- ✅ Query observations by robot, time, location, sensor type
- ✅ Coordinate observations across multiple robots

**What it is NOT:**
- ❌ Not a database you install (you use YOUR storage)
- ❌ Not a visualization tool (observations are raw data)
- ❌ Not a machine learning platform (just the data layer)

---

## 5-Minute Quick Start

### Step 1: Install PyTerrainMap

```bash
pip install pyterrainMap
```

### Step 2: Configure Your Storage

```bash
pytm setup
```

You'll be asked:

```
Which data warehouse would you like to use?

1. PostgreSQL
2. BigQuery
3. Snowflake
4. S3 + Iceberg
5. DuckDB
6. All Five (Multi-Warehouse)

OR: Just use simple object storage:

1. Local Disk (for testing)
2. AWS S3 (production)
3. Google Cloud Storage (production)
4. Azure Data Lake (production)

Select: 1

Database file path [~/.pyterrain/pyterrain.duckdb]: 

✅ Connection successful!
Setup Complete!
```

### Step 3: Try It

```python
from pyterrain_map.storage import LocalStorageBackend
from pyterrain_map.storage import StorageObservation
import asyncio
import time

async def demo():
    # Create storage backend
    backend = LocalStorageBackend({
        "base_path": "~/.pyterrain/observations"
    })
    
    # Test connection
    if not await backend.connect():
        print("Storage connection failed!")
        return
    
    # Create a test observation
    obs = StorageObservation(
        id="obs-001",
        robot_id="robot-1",
        timestamp=int(time.time() * 1_000_000),  # microseconds
        location_lat=40.7128,  # New York
        location_lon=-74.0060,
        sensor_type="lidar",
        value_json='{"intensity": 128, "range_m": 15.3}',
        confidence=0.95,
    )
    
    # Write observation
    success = await backend.write_observation(obs)
    print(f"Write successful: {success}")
    
    # Query it back
    results = await backend.query(robot_id="robot-1")
    print(f"Found {len(results)} observations")
    
    # Get stats
    stats = await backend.get_stats()
    print(f"Storage stats: {stats}")

asyncio.run(demo())
```

**Output:**
```
Write successful: True
Found 1 observations
Storage stats: {
    'backend': 'local',
    'base_path': '/Users/you/.pyterrain/observations',
    'total_size_bytes': 245,
    'total_size_mb': 0.00023,
    'file_count': 1,
    'observation_count': 1,
    'observations_written': 1,
    'observations_read': 1
}
```

✅ **That's it! You now have PyTerrainMap storing observations.**

---

## What Just Happened? (Understanding the Basics)

### The Three Layers

```
┌─────────────────────────────────────────────────────────────┐
│ LAYER 3: YOUR APPLICATION                                   │
│ Your code that collects sensor data and writes observations │
│ (ROS bridge, custom sensors, simulations, etc.)             │
└────────────────────────────┬────────────────────────────────┘
                             │
                    Observation objects
                    (robot_id, timestamp,
                     lat, lon, sensor_type, etc.)
                             │
┌────────────────────────────▼────────────────────────────────┐
│ LAYER 2: PYTERRAIN_MAP STORAGE ENGINE                       │
│ Write, query, and manage observations                       │
│ - StorageObservation (data model)                           │
│ - StorageBackend (interface)                                │
│ - Query filters & buffering                                 │
└────────────────────────────┬────────────────────────────────┘
                             │
                    NDJSON format (one JSON per line)
                    Partitioned by: YYYY/MM/DD/robot/grid
                             │
┌────────────────────────────▼────────────────────────────────┐
│ LAYER 1: STORAGE (YOU CHOOSE ONE)                           │
│ ┌──────────┬──────────┬──────────┬──────────┐              │
│ │ Local    │ S3       │ GCS      │ ADLS     │              │
│ │ Disk     │ (AWS)    │ (Google) │ (Azure)  │              │
│ └──────────┴──────────┴──────────┴──────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### Example: Multi-Robot Fleet

```python
import asyncio
from pyterrain_map.storage import S3StorageBackend, StorageObservation

# Your robots: Spot, M300, Warthog
# All write to SAME S3 bucket, DIFFERENT robot_id

async def collect_from_robot(robot_id: str, backend):
    """Simulated robot collecting data"""
    for i in range(10):
        obs = StorageObservation(
            id=f"{robot_id}-{i}",
            robot_id=robot_id,
            timestamp=int(time.time() * 1_000_000) + (i * 1_000_000),
            location_lat=40.7128 + (i * 0.001),
            location_lon=-74.0060 + (i * 0.001),
            sensor_type="lidar",
            value_json=f'{{"scan": {i}}}',
            confidence=0.9,
        )
        await backend.write_observation(obs)
        print(f"{robot_id}: Wrote observation {i}")

async def main():
    # All robots write to SAME S3 bucket
    backend = S3StorageBackend({
        "bucket": "my-fleet-bucket",
        "prefix": "observations",
        "region": "us-east-1",
    })
    
    # Collect from 3 robots in parallel
    await asyncio.gather(
        collect_from_robot("spot-1", backend),
        collect_from_robot("m300-1", backend),
        collect_from_robot("warthog-1", backend),
    )
    
    # Query observations from ALL robots
    all_obs = await backend.query(limit=1000)
    print(f"Total observations: {len(all_obs)}")
    
    # Query from ONE robot
    spot_obs = await backend.query(robot_id="spot-1")
    print(f"Spot observations: {len(spot_obs)}")
    
    # Query by location (5km box around NYC)
    location_obs = await backend.query(
        lat_min=40.70,
        lat_max=40.72,
        lon_min=-74.01,
        lon_max=-74.00,
    )
    print(f"Observations in NYC box: {len(location_obs)}")

asyncio.run(main())
```

---

## Real-World Workflow: Robot + PyTerrainMap

### Scenario: Deploying Spot on a construction site

#### 1. Pre-deployment Setup

```bash
# Install on robot
ssh spot-robot
pip install pyterrainMap

# Configure storage (choose ONE)
pytm setup

# Select: 2 (AWS S3)
# Bucket: "construction-site-001"
# Region: "us-west-2"
# AWS Access Key: ***
# AWS Secret Key: ***
```

#### 2. Connect to ROS Bridge (Future: Coming in Phase 2)

```python
# launch/spot_deployment.launch.py (pseudo-code, coming soon)
from pyterrain_ros.bridge import PyTerrainROSBridge
from pyterrain_ros.platforms import get_platform_config

# Load pre-configured Spot setup
config = get_platform_config("spot")

# Launch bridge
bridge = PyTerrainROSBridge(config)
bridge.run()  # Subscribes to /scan, /camera/*, /imu
              # Publishes observations to S3
```

#### 3. During Deployment

Spot wanders the site for 1 hour:
- Collects 3600 LiDAR scans @ 1Hz
- Each scan: ~500 grid cells = 1.8M observations
- Writes to S3 in batches
- Uses TF tree for geo-localization
- Thermal data from helmet camera

#### 4. Post-Deployment Analysis

```python
from pyterrain_map.storage import S3StorageBackend

backend = S3StorageBackend({
    "bucket": "construction-site-001",
    "prefix": "observations",
    "region": "us-west-2",
})

# Query specific area
results = await backend.query(
    robot_id="spot-1",
    sensor_type="lidar",
    lat_min=37.760,  # Site boundary
    lat_max=37.765,
    lon_min=-122.510,
    lon_max=-122.505,
    limit=100000,
)

# Export for downstream processing
import json
with open("site_scan.ndjson", "w") as f:
    for obs in results:
        f.write(obs.to_json() + "\n")

# Use with point cloud tools (CloudCompare, etc.)
```

---

## Architecture: How Everything Fits Together

### Block Diagram

```
ROBOT SIDE                          CLOUD/LOCAL SIDE
═════════════════════════════════════════════════════════════════

ROS2 Sensors                    PyTerrainMap Storage Engine
  │                                      │
  ├─ /scan (LaserScan)    ┌──────────────┼──────────────┐
  │   Lidar data          │              │              │
  │                       │         StorageBackend      │
  ├─ /thermal (Image)     │         (you choose)        │
  │   Thermal camera      │              │              │
  │                       ▼              │              ▼
  ├─ /camera (Image)   ┌──────────────┐ │         ┌────────────┐
  │   RGB camera       │  LiDAR       │ │         │ Local Disk │
  │                    │  Adapter     │ │         │ (NDJSON)   │
  ├─ /imu (IMU)        └──────────────┘ │         └────────────┘
  │   Motion data            │           │
  │                          │           │    OR
  ├─ /tf (Transforms)  ┌──────────────┐ │
  │   GPS, odometry    │ Thermal      │ │    ┌────────────┐
  │                    │ Adapter      │ │    │ AWS S3     │
  └─ /tf_static        └──────────────┘ │    │ (NDJSON)   │
     Static TF links        │           │    └────────────┘
                            │           │
                    ┌───────────────────┘    OR
                    │
                    ▼                        ┌────────────┐
            ┌──────────────────┐            │ GCS        │
            │ Coordinate       │            │ (NDJSON)   │
            │ Converter        │            └────────────┘
            │ (TF → Geodetic)  │
            └──────────────────┘    OR
                    │
                    ▼                        ┌────────────┐
            ┌──────────────────┐            │ ADLS       │
            │ StorageObservation           │ (NDJSON)   │
            │ (normalized)     │            └────────────┘
            └──────────────────┘
                    │
                    ▼
        ┌─────────────────────────┐
        │ NDJSON (immutable log)  │
        │ Partitioned by:         │
        │ YYYY/MM/DD/robot/grid   │
        └─────────────────────────┘
```

### Data Flow: From Sensor to Storage

```
1. ROS2 publishes /scan
   └─ LaserScan message (270 beams, 10Hz)

2. LiDAR Adapter processes it
   └─ Voxelizes into 0.1m grid cells
   └─ Creates ~500 observations

3. TF Listener provides transforms
   └─ Spot location at scan timestamp
   └─ Lidar mounting offset

4. Coordinate Converter transforms
   └─ Local (ENU) → Geodetic (lat/lon)
   └─ Each grid cell → lat/lon + confidence

5. StorageObservation objects created
   {
     "id": "uuid",
     "robot_id": "spot-1",
     "timestamp": 1721683200000000,  # microseconds
     "location_lat": 40.7128,
     "location_lon": -74.0060,
     "sensor_type": "lidar",
     "value_json": "{\"intensity\": 128, \"points\": 42}",
     "confidence": 0.92
   }

6. Written to storage (S3/GCS/ADLS/local)
   s3://bucket/observations/2024/07/19/spot-1/grid_40.7_-74.0.ndjson
   [newline-delimited JSON]
```

---

## Common Use Cases & Examples

### Use Case 1: Thermal Inspection

**Scenario:** Find hot spots on building facade

```python
from pyterrain_map.storage import GCSStorageBackend

backend = GCSStorageBackend({
    "bucket": "thermal-inspections",
    "project_id": "my-project",
})

# Query thermal observations
hot_spots = await backend.query(
    robot_id="dji-m300",
    sensor_type="thermal",
    start_time=int(time.time() * 1_000_000) - 3600 * 1_000_000,  # last hour
)

# Filter for hot temperatures (parsed from value_json)
import json
very_hot = []
for obs in hot_spots:
    data = json.loads(obs.value_json)
    if data["temperature_c"] > 50:  # 50°C+
        very_hot.append(obs)

print(f"Found {len(very_hot)} hot spots")
for obs in very_hot:
    data = json.loads(obs.value_json)
    print(f"  {obs.location_lat:.4f}, {obs.location_lon:.4f}: {data['temperature_c']:.1f}°C")
```

### Use Case 2: Multi-Robot Mapping

**Scenario:** 3 robots mapping forest, create unified map

```python
async def create_unified_map():
    backend = S3StorageBackend({
        "bucket": "forest-survey",
        "prefix": "lidar",
    })
    
    # Get ALL lidar observations (all robots, last 8 hours)
    all_scans = await backend.query(
        sensor_type="lidar",
        start_time=int(time.time() * 1_000_000) - (8 * 3600 * 1_000_000),
    )
    
    # Convert to standard format for PCL/CloudCompare
    points = []
    for obs in all_scans:
        data = json.loads(obs.value_json)
        # Simple example: treat intensity as Z coordinate
        points.append([
            obs.location_lat,
            obs.location_lon,
            data.get("intensity", 0),
        ])
    
    # Save as XYZ file
    with open("unified_map.xyz", "w") as f:
        for p in points:
            f.write(f"{p[0]:.6f} {p[1]:.6f} {p[2]:.2f}\n")
    
    print(f"Created map with {len(points)} points")
```

### Use Case 3: Time-Series Analysis

**Scenario:** Track how a location changes over time

```python
async def location_timeseries():
    backend = LocalStorageBackend({"base_path": "~/.pyterrain/obs"})
    
    # Query all observations at a specific location (1km radius)
    target_lat, target_lon = 40.7128, -74.0060
    
    results = await backend.query(
        lat_min=target_lat - 0.01,
        lat_max=target_lat + 0.01,
        lon_min=target_lon - 0.01,
        lon_max=target_lon + 0.01,
        limit=100000,
    )
    
    # Sort by timestamp
    results.sort(key=lambda x: x.timestamp)
    
    # Show evolution
    print("Time-series of observations at target location:")
    for obs in results[:20]:  # First 20
        dt = datetime.utcfromtimestamp(obs.timestamp / 1_000_000)
        print(f"  {dt}: {obs.sensor_type} confidence={obs.confidence}")
```

### Use Case 4: Robot Diagnostics

**Scenario:** Check if robot's sensors are working

```python
async def sensor_diagnostics(robot_id: str):
    backend = S3StorageBackend({"bucket": "fleet-data"})
    
    # Count observations by sensor type (last hour)
    one_hour_ago = int((time.time() - 3600) * 1_000_000)
    
    sensor_counts = {}
    results = await backend.query(
        robot_id=robot_id,
        start_time=one_hour_ago,
    )
    
    for obs in results:
        sensor_counts[obs.sensor_type] = sensor_counts.get(obs.sensor_type, 0) + 1
    
    print(f"\n{robot_id} Sensor Status (last hour):")
    print("Sensor         | Observations | Status")
    print("─" * 45)
    
    expected = {"lidar": 3600, "thermal": 600, "rgb": 300, "imu": 3600}
    for sensor, expected_count in expected.items():
        actual = sensor_counts.get(sensor, 0)
        status = "✅ OK" if actual > expected_count * 0.8 else "❌ FAIL"
        print(f"{sensor:14} | {actual:12} | {status}")
```

---

## Command Reference

### Setup & Configuration

```bash
# Interactive setup (first time)
pytm setup

# Update configuration
pytm configure

# Show version
pytm version

# View current config
cat ~/.pyterrain/config.json
```

### Python API

```python
# Import storage backend
from pyterrain_map.storage import (
    LocalStorageBackend,      # Local disk
    S3StorageBackend,         # AWS S3
    GCSStorageBackend,        # Google Cloud Storage
    ADLSStorageBackend,       # Azure Data Lake
)

# Import data model
from pyterrain_map.storage import StorageObservation

# Import transforms (ROS bridge)
from pyterrain_ros.transforms import (
    CoordinateConverter,      # ENU ↔ Geodetic
    TFListener,              # Transform cache
)

# Import adapters (ROS bridge)
from pyterrain_ros.adapters import (
    LiDARAdapter,            # LiDAR processor
    ThermalAdapter,          # Thermal camera processor
)

# Import platforms (pre-configs)
from pyterrain_ros.platforms import get_platform_config
```

### Common Operations

```python
import asyncio
from pyterrain_map.storage import StorageObservation, S3StorageBackend

async def common_ops():
    backend = S3StorageBackend({...})
    
    # 1. Write single observation
    obs = StorageObservation(...)
    await backend.write_observation(obs)
    
    # 2. Write batch
    observations = [obs1, obs2, obs3, ...]
    written = await backend.write_batch(observations)
    
    # 3. Query all
    all_obs = await backend.query()
    
    # 4. Query with filters
    filtered = await backend.query(
        robot_id="spot-1",
        sensor_type="lidar",
        lat_min=40.0,
        lat_max=41.0,
        lon_min=-75.0,
        lon_max=-74.0,
        start_time=1000000,
        end_time=2000000,
        limit=1000,
    )
    
    # 5. Get statistics
    stats = await backend.get_stats()
    print(stats)
    
    # 6. Delete old data (older than 30 days)
    deleted = await backend.delete_old(days=30)
    
    # 7. Health check
    healthy = await backend.health_check()

asyncio.run(common_ops())
```

---

## Troubleshooting

### Problem: "PyTerrainMap not configured"

**Solution:**
```bash
pytm setup
# Select your storage option and provide credentials
```

### Problem: "S3 connection failed"

**Check:**
```bash
# Verify AWS credentials
aws s3 ls

# Verify bucket exists and is accessible
aws s3 ls s3://your-bucket

# Check PyTerrainMap config
cat ~/.pyterrain/config.json
cat ~/.pyterrain/credentials.json  # Check file permissions: should be 0600
```

### Problem: "Observations not appearing"

**Check:**
```python
# 1. Verify write was successful
success = await backend.write_observation(obs)
print(f"Write result: {success}")

# 2. Check storage stats
stats = await backend.get_stats()
print(f"Total observations: {stats.get('observation_count')}")

# 3. Verify timestamp (should be microseconds since epoch)
import time
now_us = int(time.time() * 1_000_000)
print(f"Current time (us): {now_us}")
```

### Problem: "Query returns no results"

**Check:**
```python
# 1. Query WITHOUT filters first
all_results = await backend.query(limit=100)
print(f"Total in storage: {len(all_results)}")

# 2. Check filter ranges
results = await backend.query(
    lat_min=0,
    lat_max=90,
    lon_min=-180,
    lon_max=180,
)
print(f"With wide filters: {len(results)}")

# 3. Print first observation to see structure
if all_results:
    obs = all_results[0]
    print(f"Robot: {obs.robot_id}")
    print(f"Location: {obs.location_lat}, {obs.location_lon}")
    print(f"Sensor: {obs.sensor_type}")
```

---

## Next Steps: ROS Bridge (Coming Soon)

Once Phase 2 is complete, you'll be able to:

```bash
# Launch Spot + PyTerrainMap
ros2 launch pyterrain_ros spot_deployment.launch.py

# This will:
# ✅ Subscribe to ROS2 sensor topics
# ✅ Process LiDAR & thermal data
# ✅ Geo-localize observations via TF
# ✅ Write observations to your configured storage
# ✅ Handle failures & reconnections
```

---

## Key Concepts Summary

| Term | Meaning | Example |
|------|---------|---------|
| **Observation** | Single sensor reading at a location | LiDAR point, thermal pixel grid |
| **Robot ID** | Identifier for robot | "spot-1", "m300-1", "warthog-1" |
| **Timestamp** | When observation was captured (microseconds) | 1721683200000000 |
| **Location** | Geographic coordinates | lat=40.7128, lon=-74.0060 |
| **Sensor Type** | Type of sensor | "lidar", "thermal", "rgb", "imu" |
| **Confidence** | Data quality (0.0-1.0) | 0.95 = high quality |
| **Storage Backend** | Where observations are stored | Local, S3, GCS, ADLS |
| **Partition** | How data is organized | YYYY/MM/DD/robot_id/grid_cell |

---

## Architecture Cheat Sheet

```
┌──────────────────────────────────────────────────────────────┐
│                    PYTERRAIN MAP PLATFORM                    │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  YOUR STORAGE LAYER (Choose One):                            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Local | S3 | GCS | ADLS                                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │ (NDJSON format)                   │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │              StorageBackend Interface                    │ │
│  │  ┌────────────────────────────────────────────────────┐ │ │
│  │  │ write_observation(obs)                            │ │ │
│  │  │ write_batch(observations)                         │ │ │
│  │  │ query(robot_id, sensor_type, location, time)     │ │ │
│  │  │ get_stats()                                       │ │ │
│  │  │ delete_old(days)                                  │ │ │
│  │  └────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │ StorageObservation                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │           Your Application Layer                         │ │
│  │  ┌────────────────────────────────────────────────────┐ │ │
│  │  │ ROS Bridge (Phase 2)                              │ │ │
│  │  │  - Sensor Adapters (LiDAR, Thermal)              │ │ │
│  │  │  - Coordinate Transforms (TF → Geodetic)         │ │ │
│  │  │  - Platform Configs (Spot, DJI M300, Warthog)    │ │ │
│  │  └────────────────────────────────────────────────────┘ │ │
│  │                    OR                                   │ │
│  │  ┌────────────────────────────────────────────────────┐ │ │
│  │  │ Your Custom Code                                  │ │ │
│  │  │  - Parse sensor data                              │ │ │
│  │  │  - Create StorageObservation                      │ │ │
│  │  │  - Write to backend                               │ │ │
│  │  └────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │ (Sensor data)                     │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │           Sensors / ROS2 Topics                          │ │
│  │  - /scan (LiDAR)    - /thermal (Thermal camera)         │ │
│  │  - /camera (RGB)    - /imu (Motion)                     │ │
│  │  - /tf (Transforms) - /gps (Location)                   │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

---

## What You Can Build

With PyTerrainMap, you can build:

✅ **Persistent Robot Maps** — Robots collect data over weeks, stored immutably
✅ **Multi-Robot Surveys** — Coordinate observations across fleet
✅ **Change Detection** — Query same location at different times
✅ **Data Lakes** — Archive all sensor data (no lock-in)
✅ **Analytics** — Export to tools (PCL, QGIS, Python, etc.)
✅ **Compliance** — Prove what happened when (immutable log)

---

## Support & Documentation

- 📖 Full API Reference: `API_REFERENCE.md` (coming soon)
- 🏗️ Architecture: `ROS_BRIDGE_ARCHITECTURE.md`
- 🚀 ROS Bridge: `ROS_BRIDGE_IMPLEMENTATION_COMPLETE.md`
- 📦 Storage backends: `python/pyterrain_map/storage/`
- 🤖 ROS adapters: `python/pyterrain_ros/adapters/`
- 📍 Coordinate transforms: `python/pyterrain_ros/transforms/`

---

**Version:** 0.1.0  
**Last Updated:** July 19, 2026  
**Status:** Production Ready (Core) | Phase 2 In Progress (ROS Bridge)
