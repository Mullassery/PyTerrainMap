# Isaac ROS cuVSLAM/nvBlox Workflow

Status: **PointCloud2, odometry, IMU, and GPS adapters implemented; live Isaac ROS validation pending.**

PyTerrainMap's ROS bridge accepts duck-typed ROS messages and does not import `rclpy` or `rospy`. The caller owns subscriptions, message types, QoS, and the running Isaac ROS graph.

## Supported inputs

| Adapter key | ROS message | Status |
|---|---|---|
| `lidar` | `sensor_msgs/LaserScan` | Implemented |
| `lidar` | `sensor_msgs/PointCloud2` | Implemented |
| `odom`, `odometry` | `nav_msgs/Odometry` | Implemented |
| `rgb`, `camera` | `sensor_msgs/Image` | Implemented |
| `thermal` | `sensor_msgs/Image` | Implemented |
| `imu` | `sensor_msgs/Imu` | Implemented |
| `gps`, `navsat`, `navsatfix` | `sensor_msgs/NavSatFix` | Implemented |

PointCloud2 decoding supports ROS `PointField` datatypes 1-8, little- or big-endian payloads, organized clouds with row padding, and optional `intensity`/`i` fields. It does not require PCL, Open3D, or ROS Python packages.

The IMU adapter preserves orientation, angular velocity, linear acceleration, and all three covariance arrays. It takes its storage location from the bridge's TF pose; without TF or a `robot_pose`, it records `location_source: unavailable` with `0,0` coordinates. The GPS adapter stores `NavSatFix` latitude, longitude, altitude, status, covariance, and covariance type directly; no ENU conversion is applied. No-fix or non-finite GPS messages are dropped and counted as adapter errors.

## Intended profile

```yaml
robot_id: isaac_ros_robot
base_frame: base_link
reference_frame: map
sensors:
  odometry:
    enabled: true
    topic: <cuVSLAM odometry topic>
    frame_id: base_link
    adapter: odometry
  nvblox_cloud:
    enabled: true
    topic: <nvBlox point cloud topic>
    frame_id: map
    adapter: lidar
    params:
      voxel_size_m: 0.05
      min_range_m: 0.1
      max_range_m: 50.0
```

Capture the exact topic names from the running Isaac ROS launch graph. Do not guess them.

## Python usage

```python
from pyterrain_map.ros import PyTerrainROSBridge

bridge = PyTerrainROSBridge(
    {
        "robot_id": "robot_1",
        "base_frame": "base_link",
        "reference_frame": "map",
        "sensors": {
            "odometry": {
                "adapter": "odometry",
                "topic": "/visual_slam/tracking/odometry",
                "frame_id": "base_link",
            },
            "nvblox": {
                "adapter": "lidar",
                "topic": "/nvblox_node/pointcloud",
                "frame_id": "map",
                "params": {"voxel_size_m": 0.05},
            },
        },
    }
)
bridge.set_origin(lat=37.7749, lon=-122.4194)

observations = bridge.on_message("/visual_slam/tracking/odometry", odom_msg)
```

The bridge looks up TF at the message timestamp and passes a `(x, y, z, qx, qy, qz, qw)` sensor-to-map transform to the adapter. PointCloud2 points are transformed before voxelization. Odometry observations use the pose from the message. IMU observations use the TF pose only for their storage location, not for the inertial values.

## Validation

CPU-only tests:

```bash
pytest tests/test_ros_pointcloud_odometry.py tests/test_ros_imu_gps.py tests/test_pyterrain_map_ros_bridge.py
```

Selected live baseline: **Ubuntu 24.04 + ROS 2 Jazzy + Isaac ROS 4.6**. Isaac ROS 5.0 / ROS 2 Lyrical is not the current target. Live validation requires an NVIDIA GPU. Record a ROS 2 bag and replay it with the optional `rosbags` reader:

```bash
python3 -m pip install rosbags
ROS_BAG=/data/rosbags/isaac_validation \
  pytest tests/test_rosbag_live.py
```

Set `ROS_BAG_ORIGIN=lat,lon[,alt]` to emit geodetic locations. The live test auto-detects PointCloud2, LaserScan, Odometry, Imu, and NavSatFix topics, feeds TF when present, and skips images because RGB and thermal share `sensor_msgs/Image`. Verify:

- Point count and voxel coordinates.
- Frame IDs and sensor-to-map transforms.
- ENU-to-geodetic conversion.
- Odometry-only and TF-only pose paths.

## Limitations

- No live ROS or Isaac ROS test has been run in this environment.
- IMU location is unavailable unless the bridge has TF at the message timestamp or the caller passes `robot_pose`.
- GPS no-fix and non-finite messages are dropped; only fixed positions are stored.
- Unknown adapters are logged and skipped.
- nvBlox configuration and topic names vary by launch setup.
- The bridge does not start Isaac ROS nodes or manage Docker/NITROS.

Portfolio-level licensing and hardware notes are in
`reports/NVIDIA_INTEGRATION_WORKFLOW.md`.
