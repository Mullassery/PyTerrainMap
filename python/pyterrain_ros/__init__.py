"""PyTerrainMap ROS/ROS2 Bridge

Native integration with ROS/ROS2 for real-time multi-robot terrain mapping.
"""

__version__ = "0.1.0"

from .bridge import PyTerrainROSBridge
from .adapters.base import SensorAdapter

__all__ = [
    "PyTerrainROSBridge",
    "SensorAdapter",
]
