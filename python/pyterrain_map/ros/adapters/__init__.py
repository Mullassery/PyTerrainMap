"""Sensor adapters for converting ROS messages to PyTerrainMap observations."""

from .base import SensorAdapter
from .lidar import LiDARAdapter
from .thermal import ThermalAdapter
from .rgb import RGBAdapter

__all__ = [
    "SensorAdapter",
    "LiDARAdapter",
    "ThermalAdapter",
    "RGBAdapter",
]

# Maps the `adapter` key used in platform configs (see `platforms/__init__.py`)
# to the adapter class that implements it. "camera" is accepted as an alias
# for "rgb" since both names show up informally across docs/configs.
ADAPTER_CLASSES = {
    "lidar": LiDARAdapter,
    "thermal": ThermalAdapter,
    "rgb": RGBAdapter,
    "camera": RGBAdapter,
}
