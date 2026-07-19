"""
TF (Transform) listener for PyTerrainMap ROS bridge.

Subscribes to /tf and /tf_static topics and maintains a cache of transforms.
Provides methods to look up transforms between frames at specific timestamps.
"""

from typing import Optional, Dict, Tuple, List
from dataclasses import dataclass
from collections import defaultdict
import math


@dataclass
class Transform:
    """Represents a TF transformation between two frames."""
    parent_frame: str
    child_frame: str
    timestamp: int  # nanoseconds since epoch
    x: float
    y: float
    z: float
    qx: float  # Quaternion
    qy: float
    qz: float
    qw: float

    def inverse(self) -> "Transform":
        """Return the inverse transformation."""
        # Inverse rotation (conjugate of quaternion)
        qx_inv = -self.qx
        qy_inv = -self.qy
        qz_inv = -self.qz
        qw_inv = self.qw

        # Rotate position by inverse rotation
        # p_inv = R_inv * (-p)
        pos = [-self.x, -self.y, -self.z]
        R = self._quat_to_matrix(qx_inv, qy_inv, qz_inv, qw_inv)
        x_inv = R[0][0] * pos[0] + R[0][1] * pos[1] + R[0][2] * pos[2]
        y_inv = R[1][0] * pos[0] + R[1][1] * pos[1] + R[1][2] * pos[2]
        z_inv = R[2][0] * pos[0] + R[2][1] * pos[1] + R[2][2] * pos[2]

        return Transform(
            parent_frame=self.child_frame,
            child_frame=self.parent_frame,
            timestamp=self.timestamp,
            x=x_inv,
            y=y_inv,
            z=z_inv,
            qx=qx_inv,
            qy=qy_inv,
            qz=qz_inv,
            qw=qw_inv,
        )

    @staticmethod
    def _quat_to_matrix(qx: float, qy: float, qz: float, qw: float) -> list:
        """Convert quaternion to 3x3 rotation matrix."""
        return [
            [1 - 2 * (qy * qy + qz * qz), 2 * (qx * qy - qw * qz), 2 * (qx * qz + qw * qy)],
            [2 * (qx * qy + qw * qz), 1 - 2 * (qx * qx + qz * qz), 2 * (qy * qz - qw * qx)],
            [2 * (qx * qz - qw * qy), 2 * (qy * qz + qw * qx), 1 - 2 * (qx * qx + qy * qy)],
        ]


class TransformCache:
    """In-memory cache of TF transforms with temporal indexing."""

    # Keep transforms for 10 seconds by default
    CACHE_DURATION_NS = 10 * 1_000_000_000

    def __init__(self, max_history_ns: int = CACHE_DURATION_NS):
        """
        Initialize transform cache.

        Args:
            max_history_ns: How far back to keep transforms (nanoseconds)
        """
        self.max_history_ns = max_history_ns
        # Map: (parent_frame, child_frame) -> list of Transform
        self._transforms: Dict[Tuple[str, str], List[Transform]] = defaultdict(list)
        self._latest_timestamp = 0

    def add_transform(self, tf: Transform):
        """Add a transform to the cache."""
        key = (tf.parent_frame, tf.child_frame)
        self._transforms[key].append(tf)
        self._latest_timestamp = max(self._latest_timestamp, tf.timestamp)

        # Prune old transforms
        cutoff = self._latest_timestamp - self.max_history_ns
        self._transforms[key] = [t for t in self._transforms[key] if t.timestamp > cutoff]

    def get_transform(
        self,
        parent_frame: str,
        child_frame: str,
        timestamp: int,
        interpolate: bool = True,
    ) -> Optional[Transform]:
        """
        Get transform at a specific timestamp.

        Args:
            parent_frame: Parent frame name
            child_frame: Child frame name
            timestamp: Query timestamp (nanoseconds)
            interpolate: Interpolate if exact match not found

        Returns:
            Transform or None if not available
        """
        key = (parent_frame, child_frame)

        if key not in self._transforms or not self._transforms[key]:
            return None

        transforms = self._transforms[key]

        # Find exact or nearest match
        for tf in transforms:
            if tf.timestamp == timestamp:
                return tf

        if not interpolate:
            # Return nearest transform
            return min(transforms, key=lambda t: abs(t.timestamp - timestamp))

        # Linear interpolation between two closest transforms
        before = None
        after = None

        for tf in transforms:
            if tf.timestamp <= timestamp:
                if before is None or tf.timestamp > before.timestamp:
                    before = tf
            if tf.timestamp >= timestamp:
                if after is None or tf.timestamp < after.timestamp:
                    after = tf

        if before is None:
            return after
        if after is None:
            return before

        # Interpolate
        return self._interpolate_transforms(before, after, timestamp)

    def _interpolate_transforms(self, t1: Transform, t2: Transform, timestamp: int) -> Transform:
        """Linear interpolation between two transforms."""
        alpha = (timestamp - t1.timestamp) / (t2.timestamp - t1.timestamp)
        alpha = max(0.0, min(1.0, alpha))

        # Interpolate position (linear)
        x = t1.x + alpha * (t2.x - t1.x)
        y = t1.y + alpha * (t2.y - t1.y)
        z = t1.z + alpha * (t2.z - t1.z)

        # Interpolate rotation (SLERP - spherical linear interpolation)
        qx, qy, qz, qw = self._slerp(
            t1.qx, t1.qy, t1.qz, t1.qw,
            t2.qx, t2.qy, t2.qz, t2.qw,
            alpha,
        )

        return Transform(
            parent_frame=t1.parent_frame,
            child_frame=t1.child_frame,
            timestamp=timestamp,
            x=x,
            y=y,
            z=z,
            qx=qx,
            qy=qy,
            qz=qz,
            qw=qw,
        )

    @staticmethod
    def _slerp(
        qx1: float, qy1: float, qz1: float, qw1: float,
        qx2: float, qy2: float, qz2: float, qw2: float,
        alpha: float,
    ) -> Tuple[float, float, float, float]:
        """Spherical linear interpolation between two quaternions."""
        # Dot product
        dot = qx1 * qx2 + qy1 * qy2 + qz1 * qz2 + qw1 * qw2

        # If dot < 0, negate one quaternion to take shorter path
        if dot < 0:
            qx2, qy2, qz2, qw2 = -qx2, -qy2, -qz2, -qw2
            dot = -dot

        # Clamp dot product
        dot = max(-1.0, min(1.0, dot))

        # If quaternions are very close, use linear interpolation
        if dot > 0.9995:
            qx = qx1 + alpha * (qx2 - qx1)
            qy = qy1 + alpha * (qy2 - qy1)
            qz = qz1 + alpha * (qz2 - qz1)
            qw = qw1 + alpha * (qw2 - qw1)
            # Normalize
            mag = math.sqrt(qx * qx + qy * qy + qz * qz + qw * qw)
            return qx / mag, qy / mag, qz / mag, qw / mag

        # Calculate angle between quaternions
        theta = math.acos(dot)
        sin_theta = math.sin(theta)

        w1 = math.sin((1 - alpha) * theta) / sin_theta
        w2 = math.sin(alpha * theta) / sin_theta

        qx = w1 * qx1 + w2 * qx2
        qy = w1 * qy1 + w2 * qy2
        qz = w1 * qz1 + w2 * qz2
        qw = w1 * qw1 + w2 * qw2

        return qx, qy, qz, qw

    def get_all_frames(self) -> list:
        """Get list of all frames in the tree."""
        frames = set()
        for (parent, child) in self._transforms.keys():
            frames.add(parent)
            frames.add(child)
        return sorted(frames)


class TFListener:
    """
    Listens to /tf and /tf_static topics and maintains transform cache.

    In ROS, this would be initialized with a ROS node and subscriptions.
    For now, we provide manual add_transform() for testing.
    """

    def __init__(self):
        """Initialize TF listener."""
        self.cache = TransformCache()
        self._ros_node = None  # Will be set by bridge when available

    def on_tf_message(self, transforms_list):
        """
        Called when /tf message received.

        Args:
            transforms_list: List of geometry_msgs/TransformStamped
        """
        for stamped_tf in transforms_list:
            tf = Transform(
                parent_frame=stamped_tf.header.frame_id,
                child_frame=stamped_tf.child_frame_id,
                timestamp=self._ros_time_to_ns(stamped_tf.header.stamp),
                x=stamped_tf.transform.translation.x,
                y=stamped_tf.transform.translation.y,
                z=stamped_tf.transform.translation.z,
                qx=stamped_tf.transform.rotation.x,
                qy=stamped_tf.transform.rotation.y,
                qz=stamped_tf.transform.rotation.z,
                qw=stamped_tf.transform.rotation.w,
            )
            self.cache.add_transform(tf)

    def lookup_transform(
        self,
        target_frame: str,
        source_frame: str,
        timestamp: int,
    ) -> Optional[Transform]:
        """
        Look up transform from source to target frame.

        Args:
            target_frame: Target frame
            source_frame: Source frame
            timestamp: Query timestamp (nanoseconds)

        Returns:
            Transform or None if not available
        """
        # Try direct transform first
        tf = self.cache.get_transform(target_frame, source_frame, timestamp)
        if tf:
            return tf

        # Try inverse transform
        tf = self.cache.get_transform(source_frame, target_frame, timestamp)
        if tf:
            return tf.inverse()

        # Could implement frame tree traversal here for complex cases
        return None

    def transform_point(
        self,
        point: Tuple[float, float, float],
        source_frame: str,
        target_frame: str,
        timestamp: int,
    ) -> Optional[Tuple[float, float, float]]:
        """
        Transform a point from source frame to target frame.

        Args:
            point: (x, y, z) in source frame
            source_frame: Source frame
            target_frame: Target frame
            timestamp: Timestamp (nanoseconds)

        Returns:
            (x, y, z) in target frame or None
        """
        tf = self.lookup_transform(target_frame, source_frame, timestamp)
        if tf is None:
            return None

        # Rotate point
        R = Transform._quat_to_matrix(tf.qx, tf.qy, tf.qz, tf.qw)
        x = R[0][0] * point[0] + R[0][1] * point[1] + R[0][2] * point[2] + tf.x
        y = R[1][0] * point[0] + R[1][1] * point[1] + R[1][2] * point[2] + tf.y
        z = R[2][0] * point[0] + R[2][1] * point[1] + R[2][2] * point[2] + tf.z

        return (x, y, z)

    @staticmethod
    def _ros_time_to_ns(stamp) -> int:
        """Convert ROS time to nanoseconds since epoch."""
        # ROS time has secs and nsecs fields
        return stamp.secs * 1_000_000_000 + stamp.nsecs
