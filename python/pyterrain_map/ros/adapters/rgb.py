"""
RGB camera sensor adapter for PyTerrainMap ROS bridge.

Converts ROS sensor_msgs/Image (color) to camera detection observations.

Scope note: this uses classical background-subtraction (MOG2) blob
detection, not a learned object classifier. Per this project's dependency
policy (see pyproject.toml — no torch/tensorflow), we don't ship a deep
detector; blob detection genuinely finds moving/foreground regions in the
frame using OpenCV (an existing optional dependency, `imaging` extra), but
it cannot tell *what* an object is. Every detection is reported under the
generic class label "object", with confidence derived from blob area and
stability rather than classification certainty. If callers need labeled
object classes (e.g. "person", "car"), they should run their own detector
upstream and feed `ObjectDetection`s in directly via the core API instead
of relying on this adapter.
"""

from typing import List, Optional, Tuple
import json

try:
    import cv2
    _CV2_IMPORT_ERROR = None
except ImportError as e:  # pragma: no cover - exercised only without the `imaging` extra
    cv2 = None
    _CV2_IMPORT_ERROR = e

import numpy as np

from .base import SensorAdapter, StorageObservation
from ..transforms.coordinate_frames import CoordinateConverter


class RGBAdapter(SensorAdapter):
    """Adapter for RGB/color camera sensors.

    Detects moving foreground objects via MOG2 background subtraction and
    reports them as generic-class detections. Unlike the rest of this
    adapter's siblings, "no detections" is a real, honest result (an empty
    `detections` list on a `StorageObservation` that *was* emitted) —
    distinct from no observation being emitted at all, which is what
    happened before this adapter existed (camera topics had no registered
    adapter and were silently dropped).
    """

    def __init__(
        self,
        robot_id: str,
        frame_id: str,
        min_contour_area_px: int = 500,
        history: int = 500,
        var_threshold: float = 16.0,
        detect_shadows: bool = True,
        max_detections: int = 25,
    ):
        """
        Initialize RGB camera adapter.

        Args:
            robot_id: Robot identifier
            frame_id: TF frame ID for the camera
            min_contour_area_px: Minimum foreground blob area (pixels) to report
            history: MOG2 background model history length (frames)
            var_threshold: MOG2 variance threshold for foreground classification
            detect_shadows: Whether MOG2 should flag (and exclude) cast shadows
            max_detections: Cap on detections reported per frame (largest first)
        """
        super().__init__(robot_id, frame_id)
        if cv2 is None:
            raise ImportError(
                "RGBAdapter requires opencv-python. Install the `imaging` extra: "
                "pip install 'pyterrainmap[imaging]'"
            ) from _CV2_IMPORT_ERROR

        self.min_contour_area_px = min_contour_area_px
        self.max_detections = max_detections
        self._bg_subtractor = cv2.createBackgroundSubtractorMOG2(
            history=history,
            varThreshold=var_threshold,
            detectShadows=detect_shadows,
        )

    @property
    def sensor_type(self) -> str:
        return "rgb"

    def on_message(
        self,
        msg,
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]] = None,
        converter: Optional[CoordinateConverter] = None,
    ) -> List[StorageObservation]:
        """
        Convert an RGB image to a single camera observation carrying zero or
        more foreground-object detections.

        Returns:
            A one-element list on success (even with zero detections), or an
            empty list only when the frame itself couldn't be decoded.
        """
        try:
            self.message_count += 1
            timestamp = self._ros_time_to_us(msg.header.stamp)

            image = self._image_to_array(msg)
            if image is None:
                self.error_count += 1
                return []

            detections = self._detect_foreground_objects(image)
            lat, lon = self._robot_location(robot_pose, converter)

            obs = StorageObservation(
                id=self._generate_id(),
                robot_id=self.robot_id,
                timestamp=timestamp,
                location_lat=lat,
                location_lon=lon,
                sensor_type=self.sensor_type,
                value_json=json.dumps({"detections": detections}),
                confidence=0.9 if detections else 0.5,
            )
            return [obs]

        except Exception as e:
            print(f"RGB adapter error: {e}")
            self.error_count += 1
            return []

    def _image_to_array(self, msg) -> Optional[np.ndarray]:
        """Convert a ROS sensor_msgs/Image message to a BGR numpy array."""
        try:
            encoding = msg.encoding
            data = np.frombuffer(msg.data, dtype=np.uint8)

            if encoding in ("bgr8", "rgb8"):
                image = data.reshape((msg.height, msg.width, 3))
                if encoding == "rgb8":
                    image = image[:, :, ::-1]  # RGB -> BGR for OpenCV
                return image
            elif encoding == "mono8":
                return data.reshape((msg.height, msg.width))
            else:
                print(f"Unsupported image encoding: {encoding}")
                return None

        except Exception as e:
            print(f"Image conversion error: {e}")
            return None

    def _detect_foreground_objects(self, image: np.ndarray) -> List[dict]:
        """Run MOG2 background subtraction and extract bounding boxes."""
        fg_mask = self._bg_subtractor.apply(image)
        # MOG2 marks shadows as gray (127); keep only confident foreground (255).
        _, fg_mask = cv2.threshold(fg_mask, 200, 255, cv2.THRESH_BINARY)
        fg_mask = cv2.morphologyEx(fg_mask, cv2.MORPH_OPEN, np.ones((3, 3), np.uint8))

        contours, _ = cv2.findContours(fg_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

        frame_area = float(image.shape[0] * image.shape[1])
        candidates = []
        for contour in contours:
            area = cv2.contourArea(contour)
            if area < self.min_contour_area_px:
                continue
            x, y, w, h = cv2.boundingRect(contour)
            # Larger blobs relative to frame size => higher confidence, capped
            # well below 1.0 since this is unclassified motion, not identity.
            confidence = min(0.85, 0.3 + (area / frame_area) * 5.0)
            candidates.append((area, {
                "class_label": "object",
                "confidence": round(confidence, 3),
                "bbox": [float(x), float(y), float(w), float(h)],
            }))

        candidates.sort(key=lambda c: c[0], reverse=True)
        return [detection for _, detection in candidates[: self.max_detections]]

    @staticmethod
    def _robot_location(
        robot_pose: Optional[Tuple[float, float, float, float, float, float, float]],
        converter: Optional[CoordinateConverter],
    ) -> Tuple[float, float]:
        """Resolve the observation's (lat, lon) from robot pose + converter.

        `robot_pose` is (x, y, z, qx, qy, qz, qw) in ENU local coordinates.
        When a `CoordinateConverter` is available, x/y are treated as
        east/north offsets from its origin and converted to geodetic
        coordinates; otherwise local x/y are stored directly (consistent
        with how the other adapters fall back when no converter is set).
        """
        if robot_pose is None:
            return 0.0, 0.0

        x, y = robot_pose[0], robot_pose[1]
        if converter is None:
            return x, y

        from ..transforms.coordinate_frames import ENUPoint
        geo = converter.enu_to_geodetic(ENUPoint(east=x, north=y, up=0.0))
        return geo.lat, geo.lon

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
