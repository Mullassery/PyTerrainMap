# PyTerrainMap Python Bindings

Native Python package exposing PyTerrainMap Rust core via PyO3.

## Installation

### From PyPI (When Available)
```bash
pip install pyterrainMap
```

### From Source (Development)
```bash
# Clone repository
git clone https://github.com/Mullassery/pyterrainMap.git
cd pyterrainMap

# Build Python wheel
maturin develop

# Or with pip
pip install -e .
```

## Quick Start

### Basic Usage
```python
from pyterrain_map import TerrainMap, Persona

# Initialize terrain mapper
map_engine = TerrainMap()

# Analyze a location
analysis = map_engine.analyze(40.71, -74.00, Persona.MobileRobot)

# Print executive summary
print(analysis.summary())
# Output: "This location is suitable for rover movement..."

# Get robot-specific advice
print(analysis.recommendations_for_robot())
# ["Use slow mode", "Monitor battery"]

# Check risks
for risk in analysis.risks:
    print(f"{risk.severity_label()}: {risk.description}")
    print(f"  Mitigations: {', '.join(risk.mitigations)}")
```

### Weather & Soil Integration
```python
from pyterrain_map import EnvironmentalConditions, Persona

# Get environmental context
env = map_engine.environmental_conditions(40.71, -74.00)

# Check if conditions are suitable
if env.is_flight_safe:
    print("✓ Conditions safe for drone mission")
else:
    print("✗ High wind or low visibility - defer mission")

# Check trafficability for ground robot
if env.soil_trafficability > 0.6:
    print("✓ Terrain passable for rover")
    print(f"  Battery impact: {env.battery_impact}x normal")
```

### Mobility Assessment
```python
from pyterrain_map import MobilityAssessment

# Check if robot can traverse location
assessment = map_engine.assess_mobility(40.71, -74.00, robot_type="rover")

print(f"Traversable: {assessment.traversable}")
print(f"Difficulty: {assessment.difficulty_label()}")
print(f"Recommended speed: {assessment.recommended_speed_ms} m/s")
print(f"Hazards: {', '.join(assessment.hazards)}")
```

### Data Explanations (for AI Agents)
```python
from pyterrain_map import DataExplanation

# Get explanation of data field
soil_moisture = DataExplanation.soil_moisture()

print(f"Field: {soil_moisture.field}")
print(f"Description: {soil_moisture.description}")
print(f"Applications: {', '.join(soil_moisture.applications)}")
print(f"Source: {soil_moisture.source}")
print(f"Confidence: {soil_moisture.confidence:.0%}")
```

### Mission Suitability
```python
# Drone mission assessment
drone_suitable = map_engine.assess_for_mission(
    location=(40.71, -74.00),
    mission_type="aerial_survey",
    persona=Persona.Drone
)

print(f"Suitability: {drone_suitable.suitability:.0%}")
print(f"Limiting factors: {', '.join(drone_suitable.limiting_factors)}")
print(f"Recommendations: {', '.join(drone_suitable.recommendations)}")

# Farming suitability
farm_suitable = map_engine.assess_for_mission(
    location=(40.71, -74.00),
    mission_type="wheat_cultivation",
    persona=Persona.Farmer,
    crop_type="wheat"
)

print(f"Yield impact: {farm_suitable.yield_impact_percent:+.1f}%")
print(f"Amendments needed: {', '.join(farm_suitable.amendments)}")
```

### Temporal Reasoning
```python
# Analyze trends over last 7 days
reasoning = map_engine.reason_over_time(40.71, -74.00, days=7)

# Check trends
for trend in reasoning.trends:
    print(f"{trend.metric}: {trend.direction.name} ({trend.magnitude:.1%})")

# Check projections
for projection in reasoning.projections:
    print(f"{projection.metric} in {projection.hours_ahead}h: {projection.projected_value:.1f}")

# Check recommended actions
for action in reasoning.actions:
    print(f"[{action.urgency:.0%} urgent] {action.description}")
```

### Batch Analysis
```python
# Analyze multiple locations
locations = [
    (40.71, -74.00),
    (34.05, -118.24),
    (37.77, -122.41),
]

for lat, lon in locations:
    analysis = map_engine.analyze(lat, lon, Persona.MobileRobot)
    suitability = analysis.mission_suitability()
    print(f"{lat}, {lon}: {suitability:.0%} suitable")
```

### Export & Storage
```python
# Get analysis as JSON
json_output = analysis.to_json()

# Save to file
with open(f"analysis_{lat}_{lon}.json", "w") as f:
    f.write(json_output)

# Get analysis report
report = analysis.to_report()
print(report.summary)
```

## API Reference

### Main Classes

#### `TerrainMap`
Main engine for terrain analysis.

**Methods:**
- `analyze(lat, lon, persona=Persona.Analyst) -> TerrainAnalysis`
  - Comprehensive location analysis for given persona
- `assess_mobility(lat, lon, robot_type) -> MobilityAssessment`
  - Robot traversability analysis
- `assess_for_mission(lat, lon, mission_type, persona, **kwargs) -> MissionAssessment`
  - Mission-specific suitability scoring
- `environmental_conditions(lat, lon) -> EnvironmentalConditions`
  - Weather + soil context
- `reason_over_time(lat, lon, days=7) -> TemporalReasoning`
  - Temporal trends and projections
- `explain(field_name) -> DataExplanation`
  - Self-documentation for data fields

#### `TerrainAnalysis`
Result of location analysis.

**Properties:**
- `location: Tuple[float, float]` - (latitude, longitude)
- `summary: str` - Executive summary
- `observations: List[str]` - Detailed observations
- `risks: List[Risk]` - Identified risks
- `recommendations: Dict[Persona, List[str]]` - Persona-specific advice

**Methods:**
- `summary() -> str` - Get summary
- `advice_for(persona) -> List[str]` - Persona-specific recommendations
- `to_json() -> str` - Serialize to JSON
- `to_report() -> AnalysisReport` - Get structured report

#### `Risk`
Risk assessment.

**Properties:**
- `risk_type: RiskType` - Type of risk (Weather, Terrain, Soil, etc.)
- `severity: float` - 0.0-1.0 severity
- `description: str` - Risk description
- `affected_personas: List[Persona]` - Who this affects
- `mitigations: List[str]` - How to mitigate

**Methods:**
- `severity_label() -> str` - ("Critical", "High", "Medium", "Low")

#### `MobilityAssessment`
Robot traversability analysis.

**Properties:**
- `traversable: bool` - Can robot cross?
- `difficulty: float` - 0.0-1.0 difficulty
- `hazards: List[str]` - Identified hazards
- `recommended_speed_ms: float` - Recommended speed
- `battery_impact: float` - Energy multiplier
- `time_to_cross_100m_seconds: float` - Estimated time

**Methods:**
- `difficulty_label() -> str` - ("Easy", "Slightly difficult", "Moderately difficult", etc.)

#### `EnvironmentalConditions`
Weather + soil context.

**Properties:**
- `weather: Optional[WeatherObservation]` - Weather data
- `soil: Optional[SoilCondition]` - Soil data
- `mission_suitability: float` - 0.0-1.0 combined score
- `is_flight_safe: bool` - Safe for aerial operations
- `is_ground_safe: bool` - Safe for ground robots

#### `Persona` (Enum)
User context for analysis.

**Values:**
- `MobileRobot` - Ground rover/robot
- `Drone` - Aerial vehicle
- `Farmer` - Agricultural user
- `DisasterResponse` - Emergency responder
- `Vehicle` - Autonomous car
- `Analyst` - Geospatial analyst
- `MissionPlanner` - Mission coordinator

### Data Classes

#### `DataExplanation`
Self-documenting data field.

**Pre-built explanations:**
- `DataExplanation.soil_moisture()`
- `DataExplanation.temperature()`
- `DataExplanation.visibility()`
- `DataExplanation.slope()`

**Properties:**
- `field: str` - Field name
- `description: str` - What it measures
- `applications: List[str]` - Use cases
- `confidence: float` - Data confidence
- `source: str` - Data origin
- `units: str` - Measurement units
- `normal_range: str` - Expected range

## Integration with AI Agents

### Claude Code Integration
```python
from pyterrain_map import TerrainMap, MCPTool

# Expose as MCP tool for Claude Code
tools = [
    MCPTool.terrain_assessment(),
    MCPTool.mobility_assessment(),
    MCPTool.explain_field(),
]

# Claude Code can now discover and call PyTerrainMap functions
```

### Custom Python Script
```python
# Script for autonomous decision-making
from pyterrain_map import TerrainMap, Persona

def plan_rover_mission(target_lat, target_lon):
    map_engine = TerrainMap()
    
    # Assess location
    analysis = map_engine.analyze(target_lat, target_lon, Persona.MobileRobot)
    
    # Check mission suitability
    if analysis.mission_suitability < 0.4:
        print("Location not suitable, seeking alternative...")
        return None
    
    # Get mobility assessment
    mobility = map_engine.assess_mobility(target_lat, target_lon, "rover")
    
    if not mobility.traversable:
        print("Terrain impassable!")
        return None
    
    # Plan navigation
    return {
        "target": (target_lat, target_lon),
        "route_difficulty": mobility.difficulty_label(),
        "recommended_speed": mobility.recommended_speed_ms,
        "estimated_time_100m": mobility.time_to_cross_100m_seconds,
        "battery_impact": mobility.battery_impact,
        "hazards": mobility.hazards,
    }

# Run mission planner
mission = plan_rover_mission(40.71, -74.00)
print(mission)
```

## Development

### Building from Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin for building Python wheels
pip install maturin

# Build development wheel
maturin develop

# Run tests
python -m pytest tests/
```

### Running Python Tests
```bash
# Install test dependencies
pip install pytest

# Run all tests
pytest

# Run specific test module
pytest tests/test_terrain_analysis.py

# Run with verbose output
pytest -v
```

## Performance

PyTerrainMap Rust core ensures high performance:
- **Analysis**: ~1-5ms per location
- **Mobility assessment**: ~0.5-2ms
- **Batch processing**: 10,000 locations in ~10-20 seconds
- **Memory**: ~50MB base + ~100MB per 100k cached observations

## Troubleshooting

### Import Error: "module 'pyterrain_map' not found"
```bash
# Reinstall development mode
pip uninstall pyterrainMap
maturin develop
```

### GEOS/GDAL Issues (macOS)
```bash
# Install geospatial dependencies
brew install geos gdal

# Rebuild
maturin develop --release
```

### Python 3.13 Compatibility
PyO3 bindings require Python 3.10-3.12.
- For Python 3.13, use CLI interface (`pytm` command)
- Bindings compatibility being worked on

## License

MIT License - See LICENSE file

## Support

- **CLI**: Use `pytm --help` for command reference
- **API Docs**: See docstrings (e.g., `help(TerrainMap.analyze)`)
- **Examples**: Check `examples/` directory
- **Issues**: Report at https://github.com/Mullassery/pyterrainMap/issues
