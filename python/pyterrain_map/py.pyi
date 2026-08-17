"""Type stubs for PyTerrainMap Python bindings."""

from typing import Dict, List, Optional, Tuple

__version__: str

Persona: Dict[str, str]

class PyGeoPoint:
    """Geographic point (latitude, longitude)."""
    lat: float
    lon: float

    def __init__(self, lat: float, lon: float) -> None: ...
    def __repr__(self) -> str: ...
    def __eq__(self, other: PyGeoPoint) -> bool: ...
    def distance_m(self, other: PyGeoPoint) -> float: ...

class PyRegion:
    """Geographic region (bounding box)."""
    north: float
    south: float
    east: float
    west: float

    def __init__(self, north: float, south: float, east: float, west: float) -> None: ...
    def __repr__(self) -> str: ...
    def contains(self, point: PyGeoPoint) -> bool: ...
    def center(self) -> PyGeoPoint: ...
    @staticmethod
    def world() -> PyRegion: ...

class PyObservation:
    """Single sensor observation."""
    robot_id: str
    timestamp: int
    sensor_type: str
    confidence: float

    def __init__(
        self,
        robot_id: str,
        timestamp: int,
        lat: float,
        lon: float,
        sensor_type: str,
        value_json: str,
        confidence: float,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def location(self) -> PyGeoPoint: ...
    def value(self) -> str: ...

class PyQueryResult:
    """Results from spatial-temporal query."""
    count: int
    observations: List[PyObservation]
    avg_confidence: float

    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __getitem__(self, idx: int) -> PyObservation: ...
    def len(self) -> int: ...
    def to_dict(self) -> Dict[str, object]: ...

class PyTerrainMap:
    """Main terrain mapping engine.

    NOTE: intentionally has no `clear()`/reset method -- see
    docs/DATA_INTEGRITY.md. It documents an append-only, immutable
    observation store; construct a new `TerrainMap()` if you need an
    empty one instead of resetting an existing one.
    """
    def __init__(self) -> None: ...
    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def push_observation(self, obs: PyObservation) -> str: ...
    def push_batch(self, observations: List[PyObservation]) -> int: ...
    def query(
        self,
        location: PyGeoPoint,
        region_radius_km: float,
        time_window_seconds: int,
    ) -> PyQueryResult: ...
    def region_stats(self, region: PyRegion) -> Dict[str, object]: ...
    def observations(self) -> List[PyObservation]: ...

class PyTerrainAnalysis:
    """Terrain intelligence analysis for a location."""
    location: Tuple[float, float]
    summary: str
    observations: List[str]
    risks: List[PyRisk]
    confidence: float

    def __init__(self, lat: float, lon: float) -> None: ...
    def __repr__(self) -> str: ...
    def advice_for(self, persona: str) -> List[str]: ...
    def add_observation(self, obs: str) -> None: ...
    def add_risk(self, risk: PyRisk) -> None: ...
    def add_recommendation(self, persona: str, recommendation: str) -> None: ...

class PyRisk:
    """Risk assessment for terrain analysis."""
    risk_type: str
    severity: float
    description: str
    affected_personas: List[str]
    mitigations: List[str]

    def __init__(self, risk_type: str, severity: float, description: str) -> None: ...
    def __repr__(self) -> str: ...
    def severity_label(self) -> str: ...
    def affects(self, persona: str) -> PyRisk: ...
    def with_mitigation(self, mitigation: str) -> PyRisk: ...

class PyMobilityAssessment:
    """Robot mobility assessment for terrain."""
    traversable: bool
    difficulty: float
    hazards: List[str]
    recommended_speed_ms: float
    battery_impact: float
    time_to_cross_100m_seconds: float

    def __init__(self) -> None: ...
    def __repr__(self) -> str: ...
    def difficulty_label(self) -> str: ...
    def add_hazard(self, hazard: str) -> None: ...

class PyEnvironmentalConditions:
    """Environmental conditions (weather + soil)."""
    location: Tuple[float, float]
    mission_suitability: float

    def __init__(self, lat: float, lon: float) -> None: ...
    def __repr__(self) -> str: ...
    def update_suitability(self, score: float) -> None: ...

class PyDataExplanation:
    """Explanation of a data field for agent introspection."""
    field: str
    description: str
    applications: List[str]
    confidence: float
    source: str
    units: str
    normal_range: str

    def __init__(
        self,
        field: str,
        description: str,
        confidence: float,
        source: str,
        units: str,
        normal_range: str,
    ) -> None: ...
    def __repr__(self) -> str: ...
    def add_application(self, app: str) -> None: ...

    @staticmethod
    def soil_moisture() -> PyDataExplanation: ...

    @staticmethod
    def temperature() -> PyDataExplanation: ...

    @staticmethod
    def visibility() -> PyDataExplanation: ...

    @staticmethod
    def slope() -> PyDataExplanation: ...

# ---------------------------------------------------------------------------
# High-level module-level query/intelligence functions
# ---------------------------------------------------------------------------

def analyze_terrain(lat: float, lon: float, radius_km: float) -> PyTerrainAnalysis: ...
def assess_mobility(
    terrain_analysis: PyTerrainAnalysis, robot_type: str
) -> PyMobilityAssessment: ...
def detect_changes(
    north: float,
    south: float,
    east: float,
    west: float,
    time_start_seconds: int,
    time_end_seconds: int,
) -> Dict[str, object]: ...
def fuse_observations(observations: List[PyObservation]) -> Dict[str, object]: ...
def query_by_sensor(
    sensor_type: str,
    north: float,
    south: float,
    east: float,
    west: float,
    time_start_seconds: int,
    time_end_seconds: int,
) -> PyQueryResult: ...
def explain_field(field_name: str) -> PyDataExplanation: ...
def is_accessible(lat: float, lon: float, robot_type: str) -> Dict[str, object]: ...

# ---------------------------------------------------------------------------
# Real HTTP/HTTPS API server (src/server.rs, src/py_server.rs)
# ---------------------------------------------------------------------------

class PyServerHandle:
    """Handle to a running PyTerrainMap API server started via start_server()."""
    host: str
    port: int
    tls: bool

    def stop(self) -> None: ...
    def is_running(self) -> bool: ...
    def __repr__(self) -> str: ...
    def __enter__(self) -> PyServerHandle: ...
    def __exit__(self, exc_type: object, exc_value: object, traceback: object) -> None: ...

def start_server(
    host: str = "127.0.0.1",
    port: int = 8080,
    tls: bool = False,
    cert_path: Optional[str] = None,
    key_path: Optional[str] = None,
) -> PyServerHandle: ...
