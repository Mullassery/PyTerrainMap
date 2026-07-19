"""
Thermal camera adapter for PyTerrainMap ROS bridge.

Converts ROS sensor_msgs/Image (thermal) to temperature observations.
"""

from typing import List, Optional, Tuple
import json
import numpy as np
from .base import SensorAdapter, StorageObservation
from ..transforms.coordinate_frames import CoordinateConverter


class ThermalAdapter(SensorAdapter):
    """Adapter for thermal camera sensors."""

    def __init__(
        self,
        robot_id: str,
        frame_id: str,
        grid_size: int = 8,
        min_temp: float = -40.0,
        max_temp: float = 85.0,
        confidence_threshold: float = 0.7,
    ):
        """
        Initialize thermal adapter.

        Args:
            robot_id: Robot identifier
            frame_id: TF frame ID for thermal camera
            grid_size: Downsample thermal image to N×N grid
            min_temp: Minimum valid temperature (C)
            max_temp: Maximum valid temperature (C)
            confidence_threshold: Confidence threshold for observations
        """
        super().__init__(robot_id, frame_id)
        self.grid_size = grid_size
        self.min_temp = min_temp
        self.max_temp = max_temp
        self.confidence_threshold = confidence_threshold

    @property
    def sensor_type(self) -> str:
        return "thermal"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Convert thermal image to temperature observations.

        Args:
            msg: ROS Image message
            robot_pose: (x, y, z, qx, qy, qz, qw) from TF
            converter: CoordinateConverter for transforms

        Returns:
            List of observations
        """
        try:
            self.message_count += 1
            timestamp = self._ros_time_to_us(msg.header.stamp)

            # Convert ROS Image to numpy array
            image = self._image_to_array(msg)
            if image is None:
                self.error_count += 1
                return []

            # Downsample and grid
            observations = self._process_thermal_grid(
                image, timestamp, robot_pose, converter
            )

            return observations

        except Exception as e:
            print(f"Thermal adapter error: {e}")
            self.error_count += 1
            return []

    def _image_to_array(self, msg) -> Optional[np.ndarray]:
        """Convert ROS Image message to numpy array."""
        try:
            # Handle different image encodings
            encoding = msg.encoding

            if encoding == "mono16":
                # 16-bit grayscale (common for thermal)
                data = np.frombuffer(msg.data, dtype=np.uint16)
                image = data.reshape((msg.height, msg.width))
                # Normalize to 0-1
                image = image.astype(float) / 65535.0
            elif encoding == "mono8":
                # 8-bit grayscale
                data = np.frombuffer(msg.data, dtype=np.uint8)
                image = data.reshape((msg.height, msg.width))
                # Normalize to 0-1
                image = image.astype(float) / 255.0
            elif encoding == "32FC1":
                # 32-bit float
                data = np.frombuffer(msg.data, dtype=np.float32)
                image = data.reshape((msg.height, msg.width))
            else:
                print(f"Unsupported image encoding: {encoding}")
                return None

            return image

        except Exception as e:
            print(f"Image conversion error: {e}")
            return None

    def _process_thermal_grid(
        self,
        image: np.ndarray,
        timestamp: int,
        robot_pose: Optional[Tuple] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Grid thermal image and create observations.

        Downsample image into grid_size × grid_size cells,
        extract statistics, and create observations.
        """
        observations = []
        height, width = image.shape

        # Calculate cell dimensions
        cell_height = max(1, height // self.grid_size)
        cell_width = max(1, width // self.grid_size)

        # Process each grid cell
        for row in range(self.grid_size):
            for col in range(self.grid_size):
                # Extract cell
                y_start = row * cell_height
                y_end = min((row + 1) * cell_height, height)
                x_start = col * cell_width
                x_end = min((col + 1) * cell_width, width)

                cell = image[y_start:y_end, x_start:x_end]

                if cell.size == 0:
                    continue

                # Compute statistics
                mean_val = np.mean(cell)
                std_val = np.std(cell)
                min_val = np.min(cell)
                max_val = np.max(cell)

                # Scale from [0, 1] to temperature range
                mean_temp = self.min_temp + mean_val * (self.max_temp - self.min_temp)
                max_temp = self.min_temp + max_val * (self.max_temp - self.min_temp)

                # Compute confidence
                # More uniform cell = higher confidence
                confidence = max(0.5, 1.0 - (std_val * 0.5))

                if confidence < self.confidence_threshold:
                    continue

                # Create observation
                obs = StorageObservation(
                    id=self._generate_id(),
                    robot_id=self.robot_id,
                    timestamp=timestamp,
                    # Use grid indices as location (will be transformed if converter available)
                    location_lat=float(row) / self.grid_size,
                    location_lon=float(col) / self.grid_size,
                    sensor_type=self.sensor_type,
                    value_json=json.dumps({
                        "temperature_c": round(mean_temp, 2),
                        "max_temp_c": round(max_temp, 2),
                        "std_dev": round(std_val, 4),
                        "grid_cell": f"{row}x{col}",
                    }),
                    confidence=min(0.99, confidence),
                )
                observations.append(obs)

        return observations

    @staticmethod
    def _ros_time_to_us(stamp) -> int:
        """Convert ROS time to microseconds since epoch."""
        secs = getattr(stamp, "secs", getattr(stamp, "sec", 0))
        nsecs = getattr(stamp, "nsecs", getattr(stamp, "nanosec", 0))
        return secs * 1_000_000 + nsecs // 1000

    @staticmethod
    def _generate_id() -> str:
        """Generate unique observation ID."""
        import uuid
        return str(uuid.uuid4())
