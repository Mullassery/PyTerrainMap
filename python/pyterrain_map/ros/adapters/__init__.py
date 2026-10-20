"""Sensor adapters for converting ROS messages to PyTerrainMap observations."""

from .base import SensorAdapter
from .gps import GPSAdapter
from .imu import IMUAdapter
from .lidar import LiDARAdapter
from .odometry import OdometryAdapter
from .rgb import RGBAdapter
from .thermal import ThermalAdapter

__all__ = [
    "SensorAdapter",
    "GPSAdapter",
    "IMUAdapter",
    "LiDARAdapter",
    "OdometryAdapter",
    "ThermalAdapter",
    "RGBAdapter",
]

# Maps the `adapter` key used in platform configs (see `platforms/__init__.py`)
# to the adapter class that implements it. "camera" is accepted as an alias
# for "rgb" since both names show up informally across docs/configs.
ADAPTER_CLASSES = {
    "lidar": LiDARAdapter,
    "gps": GPSAdapter,
    "navsat": GPSAdapter,
    "navsatfix": GPSAdapter,
    "imu": IMUAdapter,
    "odom": OdometryAdapter,
    "odometry": OdometryAdapter,
    "thermal": ThermalAdapter,
    "rgb": RGBAdapter,
    "camera": RGBAdapter,
}
