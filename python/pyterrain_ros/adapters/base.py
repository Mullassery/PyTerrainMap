"""Base class for sensor adapters."""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import List, Optional
import json


@dataclass
class StorageObservation:
    """Normalized observation for PyTerrainMap backend."""
    robot_id: str
    timestamp: int  # microseconds since epoch
    location_lat: float
    location_lon: float
    sensor_type: str
    value_json: str  # JSON serialized sensor value
    confidence: float  # 0.0-1.0

    def to_dict(self):
        return {
            "robot_id": self.robot_id,
            "timestamp": self.timestamp,
            "location_lat": self.location_lat,
            "location_lon": self.location_lon,
            "sensor_type": self.sensor_type,
            "value_json": self.value_json,
            "confidence": self.confidence,
        }


class SensorAdapter(ABC):
    """Base class for adapters converting ROS messages to observations."""

    def __init__(self, robot_id: str, frame_id: str):
        """
        Args:
            robot_id: Identifier for the robot (e.g., "spot_1", "warthog_1")
            frame_id: TF frame for this sensor (e.g., "lidar_link")
        """
        self.robot_id = robot_id
        self.frame_id = frame_id
        self.message_count = 0
        self.error_count = 0

    @abstractmethod
    def on_message(
        self,
        msg,
        robot_pose: Optional[tuple] = None,
        converter = None,
    ) -> List[StorageObservation]:
        """
        Convert a ROS message to observations.

        Args:
            msg: ROS message (sensor_msgs or similar)
            robot_pose: (x, y, z, qx, qy, qz, qw) from TF at message timestamp
            converter: CoordinateConverter for ENU → geodetic transforms

        Returns:
            List of observations (may be empty if message is invalid)
        """
        pass

    @property
    @abstractmethod
    def sensor_type(self) -> str:
        """Sensor type identifier (e.g., 'lidar', 'thermal', 'rgb')."""
        pass

    def get_stats(self) -> dict:
        """Return adapter statistics."""
        return {
            "sensor_type": self.sensor_type,
            "frame_id": self.frame_id,
            "messages_processed": self.message_count,
            "errors": self.error_count,
        }


class AdapterRegistry:
    """Registry for sensor adapters."""

    def __init__(self):
        self._adapters = {}

    def register(self, topic: str, adapter: SensorAdapter):
        """Register an adapter for a topic."""
        self._adapters[topic] = adapter

    def get(self, topic: str) -> Optional[SensorAdapter]:
        """Get adapter for topic."""
        return self._adapters.get(topic)

    def get_all(self) -> dict:
        """Get all registered adapters."""
        return self._adapters.copy()
