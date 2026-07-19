"""
Base storage backend interface for PyTerrainMap.

Storage is append-only, immutable observations stored as NDJSON (newline-delimited JSON).
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, asdict
from typing import List, Dict, Any, Optional
import json
from datetime import datetime


@dataclass
class StorageObservation:
    """Single observation from a sensor."""
    id: str
    robot_id: str
    timestamp: int  # microseconds since epoch
    location_lat: float  # degrees
    location_lon: float  # degrees
    sensor_type: str  # "lidar", "thermal", "rgb", etc.
    value_json: str  # JSON payload
    confidence: float  # 0.0-1.0

    def to_json(self) -> str:
        """Serialize to JSON."""
        return json.dumps(asdict(self))

    @classmethod
    def from_json(cls, json_str: str) -> "StorageObservation":
        """Deserialize from JSON."""
        data = json.loads(json_str)
        return cls(**data)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


class StorageBackend(ABC):
    """
    Abstract base class for storage backends.

    Storage model: Append-only NDJSON (observations stored as one JSON per line)
    Structure: One file per day per robot, partitioned by location grid
    """

    def __init__(self, name: str):
        """
        Initialize storage backend.

        Args:
            name: Backend identifier ("local", "s3", "gcs", "adls")
        """
        self.name = name
        self.stats = {"observations_written": 0, "observations_read": 0}

    @abstractmethod
    async def connect(self) -> bool:
        """
        Test connection to storage.

        Returns:
            True if connection successful
        """
        pass

    @abstractmethod
    async def write_observation(self, obs: StorageObservation) -> bool:
        """
        Write single observation.

        Args:
            obs: Observation to write

        Returns:
            True if successful
        """
        pass

    @abstractmethod
    async def write_batch(self, observations: List[StorageObservation]) -> int:
        """
        Write batch of observations.

        Args:
            observations: List of observations

        Returns:
            Number of observations written successfully
        """
        pass

    @abstractmethod
    async def query(
        self,
        robot_id: Optional[str] = None,
        start_time: Optional[int] = None,
        end_time: Optional[int] = None,
        sensor_type: Optional[str] = None,
        lat_min: Optional[float] = None,
        lat_max: Optional[float] = None,
        lon_min: Optional[float] = None,
        lon_max: Optional[float] = None,
        limit: int = 10000,
    ) -> List[StorageObservation]:
        """
        Query observations with filters.

        Args:
            robot_id: Filter by robot ID
            start_time: Start timestamp (microseconds)
            end_time: End timestamp (microseconds)
            sensor_type: Filter by sensor type
            lat_min, lat_max: Latitude range
            lon_min, lon_max: Longitude range
            limit: Max results to return

        Returns:
            List of matching observations
        """
        pass

    @abstractmethod
    async def get_stats(self) -> Dict[str, Any]:
        """
        Get storage statistics.

        Returns:
            Stats dict (size, count, last_update, etc.)
        """
        pass

    @abstractmethod
    async def delete_old(self, days: int) -> int:
        """
        Delete observations older than N days.

        Args:
            days: Delete observations older than this many days

        Returns:
            Number of observations deleted
        """
        pass

    async def health_check(self) -> bool:
        """
        Health check - test if storage is accessible.

        Returns:
            True if healthy
        """
        return await self.connect()

    def _partition_key(self, obs: StorageObservation) -> str:
        """
        Generate partition key for observation.

        Format: YYYY/MM/DD/{robot_id}/{grid_cell}
        This enables efficient filtering and parallel reads.
        """
        dt = datetime.utcfromtimestamp(obs.timestamp / 1_000_000)
        date_str = dt.strftime("%Y/%m/%d")

        # Grid cell: 0.1 degree resolution (~10km)
        grid_lat = int(obs.location_lat * 10) / 10
        grid_lon = int(obs.location_lon * 10) / 10
        grid_cell = f"grid_{grid_lat:+.1f}_{grid_lon:+.1f}"

        return f"{date_str}/{obs.robot_id}/{grid_cell}"

    def _matches_filters(
        self,
        obs: StorageObservation,
        robot_id: Optional[str] = None,
        start_time: Optional[int] = None,
        end_time: Optional[int] = None,
        sensor_type: Optional[str] = None,
        lat_min: Optional[float] = None,
        lat_max: Optional[float] = None,
        lon_min: Optional[float] = None,
        lon_max: Optional[float] = None,
    ) -> bool:
        """Check if observation matches all filters."""
        if robot_id and obs.robot_id != robot_id:
            return False
        if start_time and obs.timestamp < start_time:
            return False
        if end_time and obs.timestamp > end_time:
            return False
        if sensor_type and obs.sensor_type != sensor_type:
            return False
        if lat_min and obs.location_lat < lat_min:
            return False
        if lat_max and obs.location_lat > lat_max:
            return False
        if lon_min and obs.location_lon < lon_min:
            return False
        if lon_max and obs.location_lon > lon_max:
            return False
        return True


class StorageFactory:
    """Factory for creating storage backends based on configuration."""

    _backends = {}

    @classmethod
    def register(cls, backend_type: str, backend_class: type):
        """Register a backend type."""
        cls._backends[backend_type] = backend_class

    @classmethod
    def create(cls, backend_type: str, config: Dict[str, Any]) -> StorageBackend:
        """
        Create a storage backend instance.

        Args:
            backend_type: Type of backend ("local", "s3", "gcs", "adls")
            config: Configuration dict with credentials

        Returns:
            StorageBackend instance

        Raises:
            ValueError if backend_type not recognized
        """
        if backend_type not in cls._backends:
            raise ValueError(
                f"Unknown backend type: {backend_type}. "
                f"Available: {', '.join(cls._backends.keys())}"
            )

        backend_class = cls._backends[backend_type]
        return backend_class(config)

    @classmethod
    def list_backends(cls) -> List[str]:
        """List available backend types."""
        return list(cls._backends.keys())


# Auto-register built-in backends
def _register_backends():
    """Register all built-in backends."""
    from .local import LocalStorageBackend
    from .s3 import S3StorageBackend
    from .gcs import GCSStorageBackend
    from .adls import ADLSStorageBackend

    StorageFactory.register("local", LocalStorageBackend)
    StorageFactory.register("s3", S3StorageBackend)
    StorageFactory.register("gcs", GCSStorageBackend)
    StorageFactory.register("adls", ADLSStorageBackend)


_register_backends()
