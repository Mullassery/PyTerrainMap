"""PyTerrainMap ROS/ROS2 Bridge

Native integration with ROS/ROS2 for real-time multi-robot terrain mapping.
"""

__version__ = "0.1.0"

from .adapters.base import SensorAdapter
from .bridge import PyTerrainROSBridge

__all__ = [
    "PyTerrainROSBridge",
    "SensorAdapter",
]
