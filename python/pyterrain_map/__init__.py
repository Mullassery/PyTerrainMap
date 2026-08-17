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

Running the API server:
    >>> from pyterrain_map import start_server
    >>> handle = start_server(host="127.0.0.1", port=8080)  # real HTTP server
    >>> # ... make requests against http://127.0.0.1:8080/health etc ...
    >>> handle.stop()

Core Classes:
    - TerrainMap: Main mapping engine
    - Observation: Single sensor observation
    - GeoPoint: Latitude/longitude coordinate
    - Region: Geographic bounding box
    - QueryResult: Results from spatial-temporal queries
    - TerrainAnalysis, Risk, MobilityAssessment, EnvironmentalConditions,
      DataExplanation: Intelligence/analysis layer (see analyze_terrain() etc.)
    - ServerHandle: Handle to a running HTTP(S) API server (see start_server())

Documentation:
    https://github.com/Mullassery/pyterrain-map/blob/main/PYTHON_BINDINGS.md

License:
    Proprietary -- free to use with explicit attribution. See LICENSE.
"""

__author__ = "Georgi Mammen Mullassery"
__email__ = "mullassery@gmail.com"
__license__ = "Proprietary"

# Import Rust extension
try:
    from . import pyterrain_map as _core
except ImportError as e:
    raise ImportError(
        "Failed to import PyTerrainMap Rust extension. "
        "Please install from PyPI: pip install pyterrainMap"
    ) from e

# Re-export every class/function the Rust extension registers (PyTerrainAnalysis,
# PyGaussianSplatStore, PyUnifiedPathCost, PyFrontier, PyBotObservationMessage,
# etc.) under their original Py-prefixed names too. Without this, only names
# explicitly aliased below were importable from `pyterrain_map` at all --
# `from pyterrain_map import PyGaussianSplatStore` (as this project's own test
# suite under tests/ does, e.g. test_gaussian_splatting_python.py) raised
# ImportError, meaning most of the test suite could not even be collected.
# `__version__`/`__doc__` are not affected: names starting with `_` are never
# pulled in by `import *`.
from .pyterrain_map import *  # noqa: F401,F403

# `__version__` isn't pulled in by the wildcard import above (leading
# underscore), so read it explicitly from the compiled extension -- this is
# the single source of truth (set from Cargo.toml's `CARGO_PKG_VERSION` in
# src/py.rs) rather than a second hand-maintained copy that can drift.
__version__ = _core.__version__

# User-friendly aliases for Rust classes
TerrainMap = _core.PyTerrainMap
Observation = _core.PyObservation
GeoPoint = _core.PyGeoPoint
Region = _core.PyRegion
QueryResult = _core.PyQueryResult

# Intelligence / analysis layer
TerrainAnalysis = _core.PyTerrainAnalysis
Risk = _core.PyRisk
MobilityAssessment = _core.PyMobilityAssessment
EnvironmentalConditions = _core.PyEnvironmentalConditions
DataExplanation = _core.PyDataExplanation

# High-level query/intelligence functions
analyze_terrain = _core.analyze_terrain
assess_mobility = _core.assess_mobility
detect_changes = _core.detect_changes
fuse_observations = _core.fuse_observations
query_by_sensor = _core.query_by_sensor
explain_field = _core.explain_field
is_accessible = _core.is_accessible

# Real HTTP/HTTPS API server
start_server = _core.start_server
ServerHandle = _core.PyServerHandle

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
    "TerrainAnalysis",
    "Risk",
    "MobilityAssessment",
    "EnvironmentalConditions",
    "DataExplanation",
    "analyze_terrain",
    "assess_mobility",
    "detect_changes",
    "fuse_observations",
    "query_by_sensor",
    "explain_field",
    "is_accessible",
    "start_server",
    "ServerHandle",
    "Persona",
    "cli",
]
