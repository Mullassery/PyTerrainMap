"""
LiDAR sensor adapter for PyTerrainMap ROS bridge.

Converts ROS sensor_msgs (LaserScan, PointCloud2) to observations.
"""

import json
import math
from typing import Dict, List, Optional, Tuple

import numpy as np

from ..transforms.coordinate_frames import CoordinateConverter, ENUPoint
from .base import SensorAdapter, StorageObservation

_POINT_FIELD_DTYPES = {
    1: "i1",
    2: "u1",
    3: "i2",
    4: "u2",
    5: "i4",
    6: "u4",
    7: "f4",
    8: "f8",
}


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
            # No z_sensor: 2D scan, no height dimension.

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
                # Cell center in local coords, converted to real lat/lon via
                # the supplied converter -- this branch is only reached when
                # `converter` is truthy (see the `if converter:` above).
                local_x = (grid_x + 0.5) * self.voxel_size
                local_y = (grid_y + 0.5) * self.voxel_size
                geo = converter.enu_to_geodetic(ENUPoint(east=local_x, north=local_y, up=0.0))

                # Statistics
                intensity = sum(ranges) / len(ranges)  # Average range
                count = len(ranges)

                # Create observation
                obs = StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=timestamp,
                    location_lat=geo.lat,
                    location_lon=geo.lon,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps(
                        {
                            "intensity": round(intensity, 2),
                            "range_m": round(intensity, 2),
                            "points": count,
                            "grid_x": grid_x,
                            "grid_y": grid_y,
                        }
                    ),
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
                    value_json=json.dumps(
                        {
                            "intensity": round(intensity, 2),
                            "points": count,
                        }
                    ),
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

        Decodes the binary x/y/z fields directly with NumPy, filters by range,
        transforms into the map frame when a pose is available, and voxelizes
        into 3D cells. No ROS, PCL, or Open3D runtime is required.
        """
        points, intensity = self._decode_point_cloud2(msg)
        if points.size == 0:
            return []

        ranges = np.linalg.norm(points, axis=1)
        valid = np.isfinite(ranges) & (ranges >= self.min_range) & (ranges <= self.max_range)
        points = points[valid]
        if intensity is not None:
            intensity = intensity[valid]
        if points.size == 0:
            return []

        if robot_pose is not None:
            points = self._transform_points(points, robot_pose)

        voxel_coords = np.floor(points / self.voxel_size).astype(np.int64)
        cells, inverse = np.unique(voxel_coords, axis=0, return_inverse=True)
        counts = np.bincount(inverse)
        z_sum = np.bincount(inverse, weights=points[:, 2])
        z_min = np.full(len(cells), np.inf)
        z_max = np.full(len(cells), -np.inf)
        np.minimum.at(z_min, inverse, points[:, 2])
        np.maximum.at(z_max, inverse, points[:, 2])
        intensity_mean = None
        if intensity is not None:
            intensity_sum = np.bincount(inverse, weights=intensity)
            intensity_mean = intensity_sum / counts

        timestamp = self._ros_time_to_us(msg.header.stamp)
        observations: List[StorageObservation] = []
        for index, (grid_x, grid_y, grid_z) in enumerate(cells):
            count = int(counts[index])
            local_x = float(grid_x) * self.voxel_size
            local_y = float(grid_y) * self.voxel_size
            local_z = float(z_sum[index] / count)
            if converter:
                geo = converter.enu_to_geodetic(ENUPoint(east=local_x, north=local_y, up=local_z))
                lat, lon = geo.lat, geo.lon
            else:
                lat, lon = local_x, local_y

            value: Dict[str, object] = {
                "points": count,
                "grid_x": int(grid_x),
                "grid_y": int(grid_y),
                "grid_z": int(grid_z),
                "z_mean": round(local_z, 3),
                "z_min": round(float(z_min[index]), 3),
                "z_max": round(float(z_max[index]), 3),
            }
            if intensity_mean is not None:
                value["intensity"] = round(float(intensity_mean[index]), 2)

            observations.append(
                StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=timestamp,
                    location_lat=lat,
                    location_lon=lon,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps(value),
                    confidence=min(0.95, count / 10),
                )
            )
        return observations

    @staticmethod
    def _transform_points(points: np.ndarray, pose: Tuple) -> np.ndarray:
        """Transform ``[N, 3]`` sensor-frame points by (x, y, z, qx, qy, qz, qw)."""
        qx, qy, qz, qw = pose[3:7]
        rotation = np.array(
            [
                [
                    1 - 2 * (qy * qy + qz * qz),
                    2 * (qx * qy - qw * qz),
                    2 * (qx * qz + qw * qy),
                ],
                [
                    2 * (qx * qy + qw * qz),
                    1 - 2 * (qx * qx + qz * qz),
                    2 * (qy * qz - qw * qx),
                ],
                [
                    2 * (qx * qz - qw * qy),
                    2 * (qy * qz + qw * qx),
                    1 - 2 * (qx * qx + qy * qy),
                ],
            ],
            dtype=np.float64,
        )
        return points @ rotation.T + np.array(pose[:3], dtype=np.float64)

    def _decode_point_cloud2(self, msg) -> Tuple[np.ndarray, Optional[np.ndarray]]:
        """Decode x/y/z (and optional intensity) from a duck-typed PointCloud2."""
        fields = getattr(msg, "fields", None)
        if not fields:
            raise ValueError("PointCloud2 message has no fields")

        by_name = {
            str(self._field_attr(field, "name")): field
            for field in fields
            if self._field_attr(field, "name") is not None
        }
        missing = [name for name in ("x", "y", "z") if name not in by_name]
        if missing:
            raise ValueError(f"PointCloud2 is missing fields: {missing}")

        width = int(getattr(msg, "width", 0))
        height = int(getattr(msg, "height", 1))
        point_step = int(getattr(msg, "point_step", 0))
        if width <= 0 or height <= 0 or point_step <= 0:
            raise ValueError("PointCloud2 has invalid width, height, or point_step")

        row_step = int(getattr(msg, "row_step", width * point_step))
        raw = self._message_bytes(msg)
        required = row_step * (height - 1) + width * point_step
        if raw.size < required:
            raise ValueError(f"PointCloud2 data is {raw.size} bytes, need at least {required}")
        endian = ">" if bool(getattr(msg, "is_bigendian", False)) else "<"

        x = self._field_values(by_name["x"], raw, width, height, point_step, row_step, endian)
        y = self._field_values(by_name["y"], raw, width, height, point_step, row_step, endian)
        z = self._field_values(by_name["z"], raw, width, height, point_step, row_step, endian)
        points = np.column_stack(
            [
                np.asarray(x, dtype=np.float64),
                np.asarray(y, dtype=np.float64),
                np.asarray(z, dtype=np.float64),
            ]
        )

        intensity = None
        for name in ("intensity", "i"):
            if name in by_name:
                intensity = np.asarray(
                    self._field_values(
                        by_name[name], raw, width, height, point_step, row_step, endian
                    ),
                    dtype=np.float64,
                )
                break
        return points, intensity

    @staticmethod
    def _field_attr(field, name: str):
        if isinstance(field, dict):
            return field.get(name)
        return getattr(field, name, None)

    @staticmethod
    def _message_bytes(msg) -> np.ndarray:
        data = getattr(msg, "data", None)
        if data is None:
            raise ValueError("PointCloud2 message has no data")
        if isinstance(data, np.ndarray):
            return data.view(np.uint8).reshape(-1)
        return np.frombuffer(bytes(data), dtype=np.uint8)

    def _field_values(
        self,
        field,
        raw: np.ndarray,
        width: int,
        height: int,
        point_step: int,
        row_step: int,
        endian: str,
    ) -> np.ndarray:
        datatype = int(self._field_attr(field, "datatype"))
        if datatype not in _POINT_FIELD_DTYPES:
            raise ValueError(f"unsupported PointField datatype {datatype}")
        count = int(self._field_attr(field, "count") or 1)
        offset = int(self._field_attr(field, "offset") or 0)
        dtype = np.dtype(endian + _POINT_FIELD_DTYPES[datatype])
        if count > 1:
            dtype = np.dtype((dtype, (count,)))

        if height == 1 or row_step == width * point_step:
            return np.ndarray(
                shape=(width * height,),
                dtype=dtype,
                buffer=raw,
                offset=offset,
                strides=(point_step,),
            )

        rows = [
            np.ndarray(
                shape=(width,),
                dtype=dtype,
                buffer=raw,
                offset=row * row_step + offset,
                strides=(point_step,),
            )
            for row in range(height)
        ]
        return np.concatenate(rows, axis=0)

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
