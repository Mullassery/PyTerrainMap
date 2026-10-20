# Isaac ROS cuVSLAM/nvBlox Workflow

Status: **PointCloud2 and odometry adapters implemented; live Isaac ROS validation pending.**

PyTerrainMap's ROS bridge accepts duck-typed ROS messages and does not import `rclpy` or `rospy`. The caller owns subscriptions, message types, QoS, and the running Isaac ROS graph.

## Supported inputs

| Adapter key | ROS message | Status |
|---|---|---|
| `lidar` | `sensor_msgs/LaserScan` | Implemented |
| `lidar` | `sensor_msgs/PointCloud2` | Implemented |
| `odom`, `odometry` | `nav_msgs/Odometry` | Implemented |
| `rgb`, `camera` | `sensor_msgs/Image` | Implemented |
| `thermal` | `sensor_msgs/Image` | Implemented |
| `imu` | `sensor_msgs/Imu` | Not implemented; platform configs warn |
| `gps` | `sensor_msgs/NavSatFix` | Not implemented; platform configs warn |

PointCloud2 decoding supports ROS `PointField` datatypes 1-8, little- or big-endian payloads, organized clouds with row padding, and optional `intensity`/`i` fields. It does not require PCL, Open3D, or ROS Python packages.

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

The bridge looks up TF at the message timestamp and passes a `(x, y, z, qx, qy, qz, qw)` sensor-to-map transform to the adapter. PointCloud2 points are transformed before voxelization. Odometry observations use the pose from the message.

## Validation

CPU-only tests:

```bash
pytest tests/test_ros_pointcloud_odometry.py tests/test_pyterrain_map_ros_bridge.py
```

Live validation requires Ubuntu 24.04, ROS 2 Jazzy or Lyrical, an NVIDIA GPU, and Isaac ROS 4.6 or 5.0. Record a ROS 2 bag, replay it, and verify:

- Point count and voxel coordinates.
- Frame IDs and sensor-to-map transforms.
- ENU-to-geodetic conversion.
- Odometry-only and TF-only pose paths.

## Limitations

- No live ROS or Isaac ROS test has been run in this environment.
- IMU and GPS adapters are still missing; unknown adapters are logged and skipped.
- nvBlox configuration and topic names vary by launch setup.
- The bridge does not start Isaac ROS nodes or manage Docker/NITROS.

Portfolio-level licensing and hardware notes are in
`reports/NVIDIA_INTEGRATION_WORKFLOW.md`.
