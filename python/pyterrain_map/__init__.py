"""PyTerrainMap: Spatial Intelligence Companion for Multi-Robot Terrain Mapping

A high-performance Rust core for collaborative terrain mapping.

Quick Start:
    >>> from pyterrain_map import TerrainMap, Observation, GeoPoint
    >>> map_engine = TerrainMap()
    >>> obs = Observation(
    ...     robot_id="robot-1",
    ...     timestamp=1000,
    ...     lat=40.7128,
    ...     lon=-74.0060,
    ...     sensor_type="thermal",
    ...     value_json='{"celsius": 25.0}',
    ...     confidence=0.95,
    ... )
    >>> map_engine.push_observation(obs)
    >>> result = map_engine.query(
    ...     GeoPoint(40.7128, -74.0060),
    ...     region_radius_km=10.0,
    ...     time_window_seconds=10000
    ... )
    >>> print(f"Found {result.count} observations")

Core Classes:
    - TerrainMap: Main mapping engine
    - Observation: Single sensor observation
    - GeoPoint: Latitude/longitude coordinate
    - Region: Geographic bounding box
    - QueryResult: Results from spatial-temporal queries

Documentation:
    https://github.com/Mullassery/pyterrain-map/blob/main/PYTHON_BINDINGS.md

License:
    MIT
"""

__version__ = "0.0.1"
__author__ = "Georgi Mammen Mullassery"
__email__ = "mullassery@gmail.com"
__license__ = "MIT"

# Import Rust extension
try:
    from . import pyterrain_map as _core
except ImportError as e:
    raise ImportError(
        "Failed to import PyTerrainMap Rust extension. "
        "Please install from PyPI: pip install pyterrainMap"
    ) from e

# User-friendly aliases for Rust classes
TerrainMap = _core.PyTerrainMap
Observation = _core.PyObservation
GeoPoint = _core.PyGeoPoint
Region = _core.PyRegion
QueryResult = _core.PyQueryResult

# Personas
class Persona:
    """Analysis persona/context."""
    MobileRobot = "mobile_robot"
    Drone = "drone"
    Farmer = "farmer"
    DisasterResponse = "disaster_response"
    Vehicle = "vehicle"
    Analyst = "analyst"
    MissionPlanner = "mission_planner"

# CLI
from . import cli  # noqa: F401, E402

# Public API
__all__ = [
    "TerrainMap",
    "Observation",
    "GeoPoint",
    "Region",
    "QueryResult",
    "Persona",
    "cli",
]
