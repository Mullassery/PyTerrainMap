# AI Integration Design

## Core Principle

PyTerrain (PyTerrainMap + PyTerrainAI) is designed to work seamlessly with **both humans and AI systems**:

- **Human programmers:** Import as regular Python library, call directly
- **AI code generators:** Claude Code, Copilot, Codex can generate integration code
- **Scripts/Bots:** Call via HTTP API or direct library import
- **Interactive exploration:** Use in Jupyter, REPL, Claude Code notebooks

Type hints, documentation, and examples serve both humans and AI equally well.

---

## Design for AI Code Generation

### 1. Type Hints Everywhere

AI needs clear type information to generate correct code.

```python
# ✅ GOOD: AI knows exactly what to pass and what to expect
async def get_context(
    bot_id: str,
    mission: Mission,
    location: GeoPoint,
    radius_m: float = 50.0,
) -> Context:
    """Get mission-aligned context for bot"""
    ...

# ❌ BAD: AI might guess wrong
async def get_context(bot_id, mission, location, radius=50):
    """Get context"""
    ...
```

### 2. Clear, Parseable Docstrings

AI extracts meaning from docstrings.

```python
async def get_context(
    bot_id: str,
    mission: Mission,
    location: GeoPoint,
    radius_m: float = 50.0,
) -> Context:
    """
    Get mission-aligned, temporal-decayed context for a bot.
    
    This method:
    1. Checks bot permissions for mission
    2. Queries PyTerrainMap for observations
    3. Applies temporal decay
    4. Filters by mission (RBAC)
    5. Detects anomalies
    
    Args:
        bot_id: Unique bot identifier
        mission: Mission type (security, inspection, monitoring, maintenance)
        location: Target location (lat, lon)
        radius_m: Query radius in meters (default 50)
    
    Returns:
        Context object with:
        - observations: List of filtered, decayed observations
        - anomalies: List of flagged anomalies
        - suggested_actions: Recommended next steps
        - timestamp: Query execution time
    
    Raises:
        PermissionError: If bot not authorized for mission at location
        ValueError: If mission type invalid
    
    Example:
        context = await ai.get_context(
            bot_id="security_1",
            mission=Mission.SECURITY,
            location=GeoPoint(40.123, -74.567)
        )
        print(f"Threats: {context.anomalies}")
    """
```

### 3. Predictable, Deterministic APIs

AI needs to predict outcomes. No hidden state or side effects.

```python
# ✅ GOOD: Pure function, predictable
def compute_z_score(value: float, mean: float, std: float) -> float:
    """Compute z-score relative to mean/std"""
    return abs(value - mean) / (std + 1e-6)

# ❌ BAD: Hidden state, unpredictable
def compute_z_score(value):
    return abs(value - self.baseline.mean) / self.baseline.std  # Depends on mutable state
```

### 4. Rich Examples

AI learns by pattern matching. Provide examples for every major flow.

```python
# examples/security_patrol.py
async def security_patrol():
    """Example: Security bot conducting patrol"""
    
    map_service = PyTerrainMap()
    ai_service = PyTerrainAI(map_service)
    
    # Scenario: Security bot exploring Building_A
    bot_id = "security_1"
    building_location = GeoPoint(40.123, -74.567)
    
    # Step 1: Plan phase (aerial view)
    context = await ai_service.get_context(
        bot_id=bot_id,
        mission=Mission.SECURITY,
        location=building_location,
        radius_m=100,
        detail_level=DetailLevel.AERIAL
    )
    
    # Step 2: Move toward anomalies
    for anomaly in context.anomalies:
        print(f"Investigating {anomaly.description} at {anomaly.location}")
        
        # Step 3: Arrive and query detailed view
        detailed = await ai_service.get_context(
            bot_id=bot_id,
            mission=Mission.SECURITY,
            location=anomaly.location,
            detail_level=DetailLevel.DETAILED
        )
        
        # Step 4: Report findings
        await map_service.push_observation(Observation(
            robot_id=bot_id,
            location=anomaly.location,
            sensor_type=SensorType.Camera,
            value={"findings": "building_secure"},
            confidence=0.95
        ))
```

---

## API Design for AI Consumption

### HTTP REST API (For Agents, Scripts)

```
POST /context
{
  "bot_id": "security_1",
  "mission": "security",
  "location": {"lat": 40.123, "lon": -74.567},
  "radius_m": 50.0,
  "detail_level": "regional"
}

Response:
{
  "timestamp": 1234567890,
  "observations": [
    {
      "sensor_type": "camera",
      "value": {...},
      "confidence": 0.95,
      "temporal_weight": 0.85,
      "status": "verified"
    }
  ],
  "anomalies": [
    {
      "type": "temperature_spike",
      "severity": "high",
      "location": {"lat": 40.124, "lon": -74.568}
    }
  ],
  "suggested_actions": [
    "Investigate thermal anomaly at (40.124, -74.568)",
    "Check obstacle at (40.122, -74.566)"
  ]
}
```

### Python API (For Direct Integration)

```python
from pyterrain_ai import PyTerrainAI, Mission, DetailLevel
from pyterrain_map import PyTerrainMap, Observation, SensorType, GeoPoint

async def my_autonomous_system():
    """AI system using PyTerrain"""
    
    map_svc = PyTerrainMap()
    ai_svc = PyTerrainAI(map_svc)
    
    # AI queries map to understand environment
    context = await ai_svc.get_context(
        bot_id="my_bot",
        mission=Mission.INSPECTION,
        location=GeoPoint(40.123, -74.567),
        detail_level=DetailLevel.REGIONAL
    )
    
    # AI processes context and makes decisions
    if any(a.severity == "high" for a in context.anomalies):
        print("Found critical issues!")
    
    # AI reports findings
    await map_svc.push_observation(Observation(
        robot_id="my_bot",
        location=GeoPoint(40.123, -74.567),
        sensor_type=SensorType.Camera,
        value={"observation": "damage_found"},
        confidence=0.92
    ))
```

### OpenAPI Schema (For AI Agents)

Auto-generate OpenAPI from Python type hints. Claude Code can read and follow schema.

```yaml
/context:
  post:
    summary: "Get mission-aligned context for bot"
    parameters:
      - name: bot_id
        type: string
        description: "Unique bot identifier"
      - name: mission
        type: string
        enum: [security, inspection, monitoring, maintenance]
      - name: location
        type: object
        properties:
          lat: number
          lon: number
    responses:
      200:
        schema: Context
```

---

## Integration with Claude Code / Copilot

### Example: Claude Code Generates Inspection Bot

**Prompt:**
```
Create an inspection bot that:
1. Queries PyTerrainMap for building damage
2. Plans inspection route (avoid anomalies)
3. Documents findings
4. Returns damage report

Use PyTerrainAI to get mission context.
```

**Claude Code generates:**
```python
from pyterrain_ai import PyTerrainAI, Mission, DetailLevel
from pyterrain_map import PyTerrainMap

class InspectionBot:
    def __init__(self, bot_id: str):
        self.bot_id = bot_id
        self.map = PyTerrainMap()
        self.ai = PyTerrainAI(self.map)
    
    async def inspect_building(self, location: GeoPoint) -> dict:
        # Query for damage context
        context = await self.ai.get_context(
            bot_id=self.bot_id,
            mission=Mission.INSPECTION,
            location=location,
            detail_level=DetailLevel.REGIONAL
        )
        
        # Find damage zones
        damage = [a for a in context.anomalies if "damage" in a.type]
        
        # Visit each damage zone
        findings = []
        for zone in damage:
            detailed = await self.ai.get_context(
                bot_id=self.bot_id,
                mission=Mission.INSPECTION,
                location=zone.location,
                detail_level=DetailLevel.DETAILED
            )
            findings.append(self._document_damage(zone, detailed))
        
        return {"bot": self.bot_id, "findings": findings}
```

Why this works:
- ✅ Type hints guide code generation
- ✅ Clear docstrings explain what functions do
- ✅ Example usage shows patterns
- ✅ Predictable API (no hidden state)
- ✅ Returns structured data (JSON-serializable)

---

## Principles for AI Compatibility

### 1. Type Hints Required

Every parameter and return value must have explicit type.

```python
# ✅ GOOD
async def query(location: GeoPoint, radius_m: float) -> list[Observation]:
    ...

# ❌ BAD
async def query(location, radius_m):
    ...
```

### 2. No Magic or Implicit Behavior

AI can't infer intent. Be explicit.

```python
# ✅ GOOD: AI knows what happens
context = await ai.get_context(
    bot_id=bot_id,
    mission=Mission.SECURITY,
    location=location
)
# Returns: filtered, decayed, anomaly-flagged context

# ❌ BAD: AI might not know what query() does
data = await map.query(location)
```

### 3. Structured Returns

Return dataclasses or pydantic models, not dicts or tuples.

```python
# ✅ GOOD: AI can introspect structure
@dataclass
class Context:
    observations: list[Observation]
    anomalies: list[Anomaly]
    suggested_actions: list[str]

# ❌ BAD: AI can't know structure
return {
    "data": [...],
    "issues": [...],
}
```

### 4. Comprehensive Error Handling

Explicit exceptions that AI can catch and handle.

```python
# ✅ GOOD: AI knows what can go wrong
try:
    context = await ai.get_context(...)
except PermissionError:
    print("Bot not authorized for this mission")
except ValueError:
    print("Invalid mission type")

# ❌ BAD: Generic exception
except Exception as e:
    print("Something went wrong")
```

### 5. Default Values for Common Cases

AI should be able to call with minimal args.

```python
# ✅ GOOD: AI can call with just required params
context = await ai.get_context(
    bot_id="bot_1",
    mission=Mission.SECURITY,
    location=GeoPoint(40.123, -74.567)
)
# Uses sensible defaults: radius_m=50, detail_level=REGIONAL

# ❌ BAD: Requires many parameters
context = await ai.get_context(
    bot_id="bot_1",
    mission="security",
    lat=40.123,
    lon=-74.567,
    radius_m=50.0,
    start_time=1234567890,
    end_time=1234567900,
    max_observations=100,
    min_confidence=0.5,
)
```

---

## Testing for AI Usage

Add tests that verify AI can call the API correctly:

```python
async def test_ai_can_query_context():
    """Verify AI agents can call get_context"""
    ai = PyTerrainAI(map_service)
    
    # Minimal call (like AI would)
    context = await ai.get_context(
        bot_id="bot_1",
        mission=Mission.SECURITY,
        location=GeoPoint(40.123, -74.567)
    )
    
    # Verify structure is what AI expects
    assert hasattr(context, 'observations')
    assert hasattr(context, 'anomalies')
    assert hasattr(context, 'suggested_actions')
    assert isinstance(context.observations, list)
```

---

## Documentation for AI

Create a guide for LLMs to understand the system:

```markdown
# PyTerrain for AI Systems

PyTerrain is a collaborative terrain mapping system designed for autonomous AI agents.

## Quick Start for AI Agents

1. **Import the libraries**
   ```python
   from pyterrain_map import PyTerrainMap, Observation, SensorType
   from pyterrain_ai import PyTerrainAI, Mission
   ```

2. **Query for context**
   ```python
   context = await ai.get_context(bot_id, mission, location)
   ```

3. **Process context**
   ```python
   for anomaly in context.anomalies:
       # Decide what to do based on anomaly
   ```

4. **Push observations**
   ```python
   await map_service.push_observation(Observation(...))
   ```

## API Reference

[Full API docs with all type signatures and examples]
```

---

## Summary

PyTerrain is designed so that:
- ✅ Claude Code can generate bot implementations
- ✅ Copilot can suggest PyTerrain calls
- ✅ Codex can write integration code
- ✅ LLMs understand the API from docs
- ✅ Type hints guide code generation
- ✅ Examples show patterns
- ✅ Errors are explicit and catchable

This is **API design for the AI era**.
