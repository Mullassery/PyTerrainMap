"""OKF Sensor Profiles for PyTerrainMap.

Sensor calibration history, drift tracking, and reliability profiles
for multi-robot terrain mapping.
"""

from pathlib import Path
from typing import Dict, Optional
import json
from dataclasses import dataclass


@dataclass
class SensorProfile:
    """Sensor calibration and performance profile."""

    sensor_id: str
    sensor_type: str  # lidar, camera, imu
    robot_id: str
    calibration_date: str
    accuracy: float  # 0-100%
    drift_rate: float  # mm/hour
    reliability: float
    last_calibrated: str


class OKFSensorProfiles:
    """Manage sensor calibration profiles."""

    def __init__(self, profiles_dir: Path = None):
        self.profiles_dir = profiles_dir or Path.cwd() / "sensor_profiles"
        self.profiles_dir.mkdir(exist_ok=True)

    def save_sensor(self, profile: SensorProfile) -> None:
        """Save sensor profile."""
        filename = f"sensor_{profile.sensor_id}.json"
        with open(self.profiles_dir / filename, 'w') as f:
            json.dump({
                'sensor_id': profile.sensor_id,
                'sensor_type': profile.sensor_type,
                'robot_id': profile.robot_id,
                'accuracy': profile.accuracy,
                'drift_rate': profile.drift_rate,
                'reliability': profile.reliability,
                'last_calibrated': profile.last_calibrated
            }, f, indent=2)

    def needs_calibration(self, sensor_id: str, max_drift: float = 5.0) -> bool:
        """Check if sensor needs recalibration."""
        filename = f"sensor_{sensor_id}.json"
        filepath = self.profiles_dir / filename

        if not filepath.exists():
            return True

        with open(filepath) as f:
            data = json.load(f)
            return data['drift_rate'] > max_drift or data['reliability'] < 90.0

    def get_reliable_sensors(self, min_reliability: float = 95.0) -> Dict:
        """Get all sensors above reliability threshold."""
        reliable = {}

        for f in self.profiles_dir.glob("sensor_*.json"):
            with open(f) as fp:
                data = json.load(fp)
                if data['reliability'] >= min_reliability:
                    reliable[data['sensor_id']] = data['reliability']

        return reliable
