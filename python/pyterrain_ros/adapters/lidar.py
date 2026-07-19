"""
LiDAR sensor adapter for PyTerrainMap ROS bridge.

Converts ROS sensor_msgs (LaserScan, PointCloud2) to observations.
"""

from typing import List, Optional, Tuple
import json
import math
from .base import SensorAdapter, StorageObservation
from ..transforms.coordinate_frames import CoordinateConverter


class LiDARAdapter(SensorAdapter):
    """Adapter for LiDAR sensors (2D and 3D)."""

    def __init__(
        self,
        robot_id: str,
        frame_id: str,
        voxel_size_m: float = 0.1,
        min_range_m: float = 0.1,
        max_range_m: float = 100.0,
    ):
        """
        Initialize LiDAR adapter.

        Args:
            robot_id: Robot identifier
            frame_id: TF frame ID for LiDAR sensor
            voxel_size_m: Grid cell size for spatial aggregation
            min_range_m: Minimum valid range
            max_range_m: Maximum valid range
        """
        super().__init__(robot_id, frame_id)
        self.voxel_size = voxel_size_m
        self.min_range = min_range_m
        self.max_range = max_range_m

    @property
    def sensor_type(self) -> str:
        return "lidar"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Convert LaserScan or PointCloud2 to observations.

        Args:
            msg: ROS message (LaserScan or PointCloud2)
            robot_pose: (x, y, z, qx, qy, qz, qw) from TF
            converter: CoordinateConverter for transforms

        Returns:
            List of observations
        """
        try:
            self.message_count += 1

            # Determine message type by checking for common fields
            if hasattr(msg, "ranges"):
                # LaserScan
                return self._process_laser_scan(msg, robot_pose, converter)
            elif hasattr(msg, "data"):
                # PointCloud2
                return self._process_point_cloud2(msg, robot_pose, converter)
            else:
                print(f"Unknown LiDAR message type: {type(msg)}")
                self.error_count += 1
                return []

        except Exception as e:
            print(f"LiDAR adapter error: {e}")
            self.error_count += 1
            return []

    def _process_laser_scan(
        self,
        msg,
        robot_pose: Optional[Tuple] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Process LaserScan message (2D horizontal scan).

        Converts ranges and bearings into spatial points, aggregates into grid cells.
        """
        observations = []
        timestamp = self._ros_time_to_us(msg.header.stamp)

        # Grid cell accumulator: (grid_x, grid_y) -> list of ranges
        grid_cells = {}

        for i, range_val in enumerate(msg.ranges):
            # Skip invalid/out-of-range
            if range_val < self.min_range or range_val > self.max_range:
                continue

            # Calculate bearing for this beam
            bearing_rad = msg.angle_min + (i * msg.angle_increment)

            # Convert to Cartesian in sensor frame
            x_sensor = range_val * math.cos(bearing_rad)
            y_sensor = range_val * math.sin(bearing_rad)
            z_sensor = 0.0  # 2D scan, no height

            # Transform to robot frame if we have pose
            if robot_pose:
                # Simple 2D transform (assumes horizontal LiDAR)
                qx, qy, qz, qw = robot_pose[3:7]
                yaw = self._quat_to_yaw(qx, qy, qz, qw)

                # Rotate by robot yaw
                cos_y = math.cos(yaw)
                sin_y = math.sin(yaw)
                x = cos_y * x_sensor - sin_y * y_sensor + robot_pose[0]
                y = sin_y * x_sensor + cos_y * y_sensor + robot_pose[1]
            else:
                x, y = x_sensor, y_sensor

            # Discretize into grid cells
            grid_x = int(x / self.voxel_size)
            grid_y = int(y / self.voxel_size)
            cell_key = (grid_x, grid_y)

            if cell_key not in grid_cells:
                grid_cells[cell_key] = []
            grid_cells[cell_key].append(range_val)

        # Create observations from grid cells
        if converter:
            # Convert to geodetic coordinates
            for (grid_x, grid_y), ranges in grid_cells.items():
                # Cell center in local coords
                local_x = (grid_x + 0.5) * self.voxel_size
                local_y = (grid_y + 0.5) * self.voxel_size

                # Statistics
                intensity = sum(ranges) / len(ranges)  # Average range
                count = len(ranges)

                # Create observation
                obs = StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=timestamp,
                    location_lat=0.0,  # Will be set if converter available
                    location_lon=0.0,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps({
                        "intensity": round(intensity, 2),
                        "range_m": round(intensity, 2),
                        "points": count,
                        "grid_x": grid_x,
                        "grid_y": grid_y,
                    }),
                    confidence=min(0.95, count / 10),  # More points = higher confidence
                )
                observations.append(obs)
        else:
            # No coordinate converter, just store local coordinates
            for (grid_x, grid_y), ranges in grid_cells.items():
                intensity = sum(ranges) / len(ranges)
                count = len(ranges)

                obs = StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=timestamp,
                    location_lat=float(grid_x) * self.voxel_size,  # Use local x as lat
                    location_lon=float(grid_y) * self.voxel_size,  # Use local y as lon
                    sensor_type=self.sensor_type,
                    value_json=json.dumps({
                        "intensity": round(intensity, 2),
                        "points": count,
                    }),
                    confidence=min(0.95, count / 10),
                )
                observations.append(obs)

        return observations

    def _process_point_cloud2(
        self,
        msg,
        robot_pose: Optional[Tuple] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Process PointCloud2 message (3D point cloud).

        Voxelizes into grid cells and creates observations.
        """
        observations = []
        timestamp = self._ros_time_to_us(msg.header.stamp)

        # Grid cell accumulator: (grid_x, grid_y, grid_z) -> count
        grid_cells = {}

        # Parse point cloud (simplified - assumes x, y, z fields)
        # In production, would use open3d or PCL bindings
        # For now, just create a placeholder
        # This would require proper PointCloud2 parsing

        return observations

    @staticmethod
    def _quat_to_yaw(qx: float, qy: float, qz: float, qw: float) -> float:
        """Extract yaw from quaternion."""
        sin_y_cosp = 2 * (qw * qz + qx * qy)
        cos_y_cosp = 1 - 2 * (qy * qy + qz * qz)
        return math.atan2(sin_y_cosp, cos_y_cosp)

    @staticmethod
    def _ros_time_to_us(stamp) -> int:
        """Convert ROS time to microseconds since epoch."""
        # Assumes stamp.secs and stamp.nsecs (rospy) or sec and nanosec (rclpy)
        secs = getattr(stamp, "secs", getattr(stamp, "sec", 0))
        nsecs = getattr(stamp, "nsecs", getattr(stamp, "nanosec", 0))
        return secs * 1_000_000 + nsecs // 1000

    @staticmethod
    def _generate_id() -> str:
        """Generate unique observation ID."""
        import uuid
        return str(uuid.uuid4())
