"""
Coordinate frame transformations for PyTerrainMap ROS bridge.

Handles conversions between:
- Local frame (ENU: East-North-Up)
- Robot frame (base_link)
- Geodetic frame (lat/lon/alt)
"""

import math
from dataclasses import dataclass
from typing import Tuple, Optional
from functools import lru_cache


@dataclass
class GeoPoint:
    """Geodetic coordinate (latitude, longitude, altitude)."""
    lat: float  # degrees
    lon: float  # degrees
    alt: float  # meters MSL


@dataclass
class ENUPoint:
    """Local ENU coordinate (East, North, Up)."""
    east: float  # meters
    north: float  # meters
    up: float  # meters


class CoordinateConverter:
    """Convert between ENU (local) and geodetic (global) coordinates."""

    # WGS84 ellipsoid constants
    WGS84_A = 6378137.0  # Semi-major axis (meters)
    WGS84_E2 = 0.00669438  # First eccentricity squared

    def __init__(self, origin_lat: float, origin_lon: float, origin_alt: float = 0.0):
        """
        Initialize converter with origin point.

        Args:
            origin_lat: Origin latitude (degrees)
            origin_lon: Origin longitude (degrees)
            origin_alt: Origin altitude (meters MSL, default 0)
        """
        self.origin = GeoPoint(lat=origin_lat, lon=origin_lon, alt=origin_alt)
        self._precompute_transforms()

    def _precompute_transforms(self):
        """Precompute transformation matrices for origin."""
        lat_rad = math.radians(self.origin.lat)
        lon_rad = math.radians(self.origin.lon)

        sin_lat = math.sin(lat_rad)
        cos_lat = math.cos(lat_rad)
        sin_lon = math.sin(lon_rad)
        cos_lon = math.cos(lon_rad)

        # Rotation matrix from geodetic to ENU
        # ENU = R * (ECEF - ECEF_origin)
        self._R_ecef_to_enu = [
            [-sin_lon, cos_lon, 0],
            [-sin_lat * cos_lon, -sin_lat * sin_lon, cos_lat],
            [cos_lat * cos_lon, cos_lat * sin_lon, sin_lat],
        ]

        # Inverse rotation matrix (ENU to ECEF)
        self._R_enu_to_ecef = [
            [-sin_lon, -sin_lat * cos_lon, cos_lat * cos_lon],
            [cos_lon, -sin_lat * sin_lon, cos_lat * sin_lon],
            [0, cos_lat, sin_lat],
        ]

        # Origin ECEF coordinates
        self._ecef_origin = self._geodetic_to_ecef(self.origin)

    def enu_to_geodetic(self, enu: ENUPoint) -> GeoPoint:
        """
        Convert ENU local coordinates to geodetic.

        Args:
            enu: Local ENU point (east, north, up)

        Returns:
            Geodetic point (lat, lon, alt)
        """
        # Transform ENU to ECEF
        enu_vec = [enu.east, enu.north, enu.up]
        ecef_vec = self._matrix_vector_mult(self._R_enu_to_ecef, enu_vec)

        # Add origin offset
        ecef = [
            self._ecef_origin[0] + ecef_vec[0],
            self._ecef_origin[1] + ecef_vec[1],
            self._ecef_origin[2] + ecef_vec[2],
        ]

        # Convert ECEF to geodetic
        return self._ecef_to_geodetic(ecef)

    def geodetic_to_enu(self, geo: GeoPoint) -> ENUPoint:
        """
        Convert geodetic coordinates to ENU local.

        Args:
            geo: Geodetic point (lat, lon, alt)

        Returns:
            Local ENU point (east, north, up)
        """
        # Convert to ECEF
        ecef = self._geodetic_to_ecef(geo)

        # Compute difference from origin
        delta_ecef = [
            ecef[0] - self._ecef_origin[0],
            ecef[1] - self._ecef_origin[1],
            ecef[2] - self._ecef_origin[2],
        ]

        # Transform to ENU
        enu_vec = self._matrix_vector_mult(self._R_ecef_to_enu, delta_ecef)

        return ENUPoint(east=enu_vec[0], north=enu_vec[1], up=enu_vec[2])

    def _geodetic_to_ecef(self, geo: GeoPoint) -> list:
        """Convert geodetic to ECEF coordinates."""
        lat_rad = math.radians(geo.lat)
        lon_rad = math.radians(geo.lon)

        sin_lat = math.sin(lat_rad)
        cos_lat = math.cos(lat_rad)
        sin_lon = math.sin(lon_rad)
        cos_lon = math.cos(lon_rad)

        # Prime vertical radius of curvature
        N = self.WGS84_A / math.sqrt(1 - self.WGS84_E2 * sin_lat * sin_lat)

        x = (N + geo.alt) * cos_lat * cos_lon
        y = (N + geo.alt) * cos_lat * sin_lon
        z = (N * (1 - self.WGS84_E2) + geo.alt) * sin_lat

        return [x, y, z]

    def _ecef_to_geodetic(self, ecef: list) -> GeoPoint:
        """Convert ECEF to geodetic coordinates."""
        x, y, z = ecef

        # Longitude
        lon_rad = math.atan2(y, x)

        # Latitude (iterative calculation)
        p = math.sqrt(x * x + y * y)
        lat_rad = math.atan2(z, p * (1 - self.WGS84_E2))

        # Iterate to convergence
        for _ in range(3):
            sin_lat = math.sin(lat_rad)
            N = self.WGS84_A / math.sqrt(1 - self.WGS84_E2 * sin_lat * sin_lat)
            lat_rad = math.atan2(z + self.WGS84_E2 * N * sin_lat, p)

        # Altitude
        sin_lat = math.sin(lat_rad)
        cos_lat = math.cos(lat_rad)
        N = self.WGS84_A / math.sqrt(1 - self.WGS84_E2 * sin_lat * sin_lat)
        alt = p / cos_lat - N if cos_lat > 0 else z / sin_lat - N * (1 - self.WGS84_E2)

        return GeoPoint(
            lat=math.degrees(lat_rad),
            lon=math.degrees(lon_rad),
            alt=alt,
        )

    @staticmethod
    def _matrix_vector_mult(matrix: list, vector: list) -> list:
        """Multiply 3x3 matrix by 3x1 vector."""
        return [
            matrix[0][0] * vector[0] + matrix[0][1] * vector[1] + matrix[0][2] * vector[2],
            matrix[1][0] * vector[0] + matrix[1][1] * vector[1] + matrix[1][2] * vector[2],
            matrix[2][0] * vector[0] + matrix[2][1] * vector[1] + matrix[2][2] * vector[2],
        ]

    def distance_m(self, geo1: GeoPoint, geo2: GeoPoint) -> float:
        """
        Calculate distance between two geodetic points (Haversine).

        Args:
            geo1: First point
            geo2: Second point

        Returns:
            Distance in meters
        """
        R = 6371000.0  # Earth radius in meters

        lat1_rad = math.radians(geo1.lat)
        lat2_rad = math.radians(geo2.lat)
        delta_lat = math.radians(geo2.lat - geo1.lat)
        delta_lon = math.radians(geo2.lon - geo1.lon)

        a = (
            math.sin(delta_lat / 2) ** 2
            + math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(delta_lon / 2) ** 2
        )
        c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))

        return R * c

    def bearing_deg(self, geo1: GeoPoint, geo2: GeoPoint) -> float:
        """
        Calculate bearing from geo1 to geo2 (degrees, 0=North, 90=East).

        Args:
            geo1: Starting point
            geo2: Ending point

        Returns:
            Bearing in degrees (0-360)
        """
        lat1_rad = math.radians(geo1.lat)
        lat2_rad = math.radians(geo2.lat)
        delta_lon = math.radians(geo2.lon - geo1.lon)

        y = math.sin(delta_lon) * math.cos(lat2_rad)
        x = math.cos(lat1_rad) * math.sin(lat2_rad) - math.sin(lat1_rad) * math.cos(
            lat2_rad
        ) * math.cos(delta_lon)

        bearing_rad = math.atan2(y, x)
        bearing_deg = math.degrees(bearing_rad)

        return (bearing_deg + 360) % 360


class QuaternionRotation:
    """Handle quaternion rotations for TF transforms."""

    def __init__(self, x: float, y: float, z: float, w: float):
        """
        Initialize quaternion (x, y, z, w).

        Args:
            x, y, z, w: Quaternion components (should be normalized)
        """
        self.x = x
        self.y = y
        self.z = z
        self.w = w

    def to_euler_zyx(self) -> Tuple[float, float, float]:
        """
        Convert quaternion to Euler angles (roll, pitch, yaw) in radians.

        ZYX (yaw-pitch-roll) order: commonly used in robotics.

        Returns:
            Tuple of (roll, pitch, yaw) in radians
        """
        # Roll (X-axis rotation)
        sinr_cosp = 2 * (self.w * self.x + self.y * self.z)
        cosr_cosp = 1 - 2 * (self.x * self.x + self.y * self.y)
        roll = math.atan2(sinr_cosp, cosr_cosp)

        # Pitch (Y-axis rotation)
        sinp = 2 * (self.w * self.y - self.z * self.x)
        sinp = max(-1.0, min(1.0, sinp))  # Clamp to [-1, 1]
        pitch = math.asin(sinp)

        # Yaw (Z-axis rotation)
        siny_cosp = 2 * (self.w * self.z + self.x * self.y)
        cosy_cosp = 1 - 2 * (self.y * self.y + self.z * self.z)
        yaw = math.atan2(siny_cosp, cosy_cosp)

        return (roll, pitch, yaw)

    def to_rotation_matrix(self) -> list:
        """Convert quaternion to 3x3 rotation matrix."""
        x, y, z, w = self.x, self.y, self.z, self.w

        return [
            [1 - 2 * (y * y + z * z), 2 * (x * y - w * z), 2 * (x * z + w * y)],
            [2 * (x * y + w * z), 1 - 2 * (x * x + z * z), 2 * (y * z - w * x)],
            [2 * (x * z - w * y), 2 * (y * z + w * x), 1 - 2 * (x * x + y * y)],
        ]

    @staticmethod
    def from_euler_zyx(roll: float, pitch: float, yaw: float) -> "QuaternionRotation":
        """Create quaternion from Euler angles (roll, pitch, yaw) in radians."""
        cy = math.cos(yaw * 0.5)
        sy = math.sin(yaw * 0.5)
        cp = math.cos(pitch * 0.5)
        sp = math.sin(pitch * 0.5)
        cr = math.cos(roll * 0.5)
        sr = math.sin(roll * 0.5)

        w = cr * cp * cy + sr * sp * sy
        x = sr * cp * cy - cr * sp * sy
        y = cr * sp * cy + sr * cp * sy
        z = cr * cp * sy - sr * sp * cy

        return QuaternionRotation(x, y, z, w)

    def normalize(self) -> "QuaternionRotation":
        """Return normalized quaternion."""
        mag = math.sqrt(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w)
        if mag == 0:
            return QuaternionRotation(0, 0, 0, 1)
        return QuaternionRotation(self.x / mag, self.y / mag, self.z / mag, self.w / mag)
