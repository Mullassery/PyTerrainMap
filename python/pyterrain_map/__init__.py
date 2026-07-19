"""PyTerrainMap: Spatial Intelligence Companion for Multi-Robot Terrain Mapping

A high-performance Rust core exposing collaborative terrain mapping, multi-source
spatial reasoning, anomaly detection, 3D reconstruction, and autonomous-friendly
APIs for robot fleets, agricultural systems, and geospatial analysis.

Quick Start:
    >>> from pyterrain_map import TerrainMap, Persona
    >>> engine = TerrainMap()
    >>> analysis = engine.analyze(40.71, -74.00, Persona.MobileRobot)
    >>> print(analysis.summary())

Features:
    - Multi-robot terrain mapping with consensus fusion
    - Append-only immutable observation storage
    - Temporal decay and temporal reasoning
    - H3 hierarchical spatial indexing
    - 8-failure anomaly detection taxonomy
    - Multi-source geospatial data integration
    - 3D reconstruction (SLAM + Photogrammetry)
    - Natural language CLI interface
    - REST API with TLS/mTLS
    - Persona-driven analysis (7 personas)
    - Change detection and temporal trends
    - Weather/soil integration
    - RBAC + privacy controls
    - GIS export (GeoJSON, KML, WKT, OBJ, 3D Tiles)

Documentation:
    https://github.com/Mullassery/pyterrain-map/blob/main/PYTHON_BINDINGS.md

License:
    MIT
"""

__version__ = "0.0.1"
__author__ = "Georgi Mammen Mullassery"
__email__ = "mullassery@gmail.com"
__license__ = "MIT"

from typing import Tuple, List, Dict, Optional

# Import Rust extension
try:
    from . import _core as _rust_core  # noqa: F401
except ImportError as e:
    raise ImportError(
        "Failed to import PyTerrainMap Rust extension. "
        "Please install from source: pip install -e . or maturin develop"
    ) from e

# Public API
__all__ = [
    # Main engine
    "TerrainMap",

    # Analysis results
    "TerrainAnalysis",
    "AnalysisReport",
    "Risk",
    "MobilityAssessment",
    "EnvironmentalConditions",
    "TemporalReasoning",

    # Personas
    "Persona",

    # Data explanation
    "DataExplanation",

    # Spatial reasoning
    "SpatialReasoningEngine",
    "DataProvenance",
    "Uncertainty",
    "PositionAnswer",

    # CLI
    "CLICommand",
    "CLIResponse",
]


class TerrainMap:
    """Main terrain mapping engine.

    Provides high-level API for terrain analysis, mobility assessment,
    environmental conditions, and mission planning.
    """

    def __init__(self):
        """Initialize terrain mapping engine."""
        pass

    def analyze(
        self,
        lat: float,
        lon: float,
        persona: str = "Analyst",
    ) -> "TerrainAnalysis":
        """Analyze a location for a given persona.

        Args:
            lat: Latitude in decimal degrees
            lon: Longitude in decimal degrees
            persona: Analysis context (MobileRobot, Drone, Farmer, etc.)

        Returns:
            TerrainAnalysis with persona-specific insights
        """
        pass

    def assess_mobility(
        self,
        lat: float,
        lon: float,
        robot_type: str = "rover",
    ) -> "MobilityAssessment":
        """Assess robot traversability at location.

        Args:
            lat: Latitude
            lon: Longitude
            robot_type: Type of robot (rover, drone, legged, etc.)

        Returns:
            MobilityAssessment with traversability details
        """
        pass

    def environmental_conditions(
        self,
        lat: float,
        lon: float,
    ) -> "EnvironmentalConditions":
        """Get weather and soil conditions at location.

        Args:
            lat: Latitude
            lon: Longitude

        Returns:
            EnvironmentalConditions with weather and soil data
        """
        pass


class TerrainAnalysis:
    """Result of location analysis."""

    def __init__(self):
        """Initialize analysis result."""
        self.location: Tuple[float, float]
        self.summary: str
        self.observations: List[str]
        self.risks: List["Risk"]
        self.recommendations: Dict[str, List[str]]


class AnalysisReport:
    """Structured analysis report."""
    pass


class Risk:
    """Risk assessment."""

    def severity_label(self) -> str:
        """Get severity label (Critical, High, Medium, Low)."""
        pass


class MobilityAssessment:
    """Robot traversability analysis."""

    def difficulty_label(self) -> str:
        """Get difficulty label."""
        pass


class EnvironmentalConditions:
    """Weather and soil context."""

    @property
    def is_flight_safe(self) -> bool:
        """Check if conditions are safe for aerial operations."""
        pass

    @property
    def is_ground_safe(self) -> bool:
        """Check if conditions are safe for ground robots."""
        pass


class TemporalReasoning:
    """Temporal trends and projections."""
    pass


class Persona:
    """Analysis persona/context."""

    MobileRobot = "mobile_robot"
    Drone = "drone"
    Farmer = "farmer"
    DisasterResponse = "disaster_response"
    Vehicle = "vehicle"
    Analyst = "analyst"
    MissionPlanner = "mission_planner"


class DataExplanation:
    """Self-documenting data field for AI agents."""

    @staticmethod
    def soil_moisture() -> "DataExplanation":
        """Get explanation for soil_moisture field."""
        pass

    @staticmethod
    def temperature() -> "DataExplanation":
        """Get explanation for temperature field."""
        pass

    @staticmethod
    def visibility() -> "DataExplanation":
        """Get explanation for visibility field."""
        pass

    @staticmethod
    def slope() -> "DataExplanation":
        """Get explanation for slope field."""
        pass


class SpatialReasoningEngine:
    """Multi-source spatial reasoning with regional context."""

    def __init__(self):
        """Initialize with default regional preferences."""
        pass


class DataProvenance:
    """Data source attribution and confidence tracking."""
    pass


class Uncertainty:
    """Confidence intervals and limitation modeling."""
    pass


class PositionAnswer:
    """Position with full provenance and reasoning."""
    pass


class CLICommand:
    """Natural language command parser."""

    @staticmethod
    def parse(input_str: str) -> "CLICommand":
        """Parse natural language command."""
        pass


class CLIResponse:
    """Human-friendly CLI response."""

    def format_terminal(self) -> str:
        """Format for terminal output."""
        pass

    def format_json(self) -> str:
        """Format as JSON."""
        pass
