# Bandwidth Optimization: Abstract Representation Model

## FPS Game Insight

First-person shooter games solve this problem: **Show what players need without transmitting massive amounts of data.**

Game approach:
- ❌ DON'T send full 3D model of the map constantly
- ✅ Send abstract representation (fog of war, markers, icons)
- ✅ Players see explored vs unexplored zones
- ✅ Detailed info appears only when looking at that area
- ✅ Massive bandwidth savings

Apply to PyTerrain:

```
❌ Naive Approach (Bandwidth Disaster):
  Bot queries Location_X → PyTerrainAI sends:
  ├─ Full point cloud (millions of points)
  ├─ High-res images (10MB+ each)
  ├─ All 100 observations (raw sensor data)
  └─ Total: 50-100MB per query

✅ FPS Game Approach (Efficient):
  Bot queries Location_X → PyTerrainAI sends:
  ├─ Abstract representation:
  │  ├─ "Zone explored"
  │  ├─ "Temperature: 22°C" (summary, not raw)
  │  ├─ "Last updated: 2h ago"
  │  ├─ "Confidence: HIGH"
  │  └─ Anomaly flags (icons, not data)
  ├─ Markers/icons for detected objects
  └─ Total: <10KB per query

  When bot FOCUSES on specific anomaly:
  ├─ "Zone A has anomaly"
  ├─ Bot queries: "Show me anomaly details at A"
  ├─ Detailed sensor data sent on demand
  └─ Bandwidth: 100KB (only when needed)
```

---

## Abstract Representation Layers

### Level 1: Minimap View (Ultra-Low Bandwidth)

What: Zoomed-out view of all observations  
Size: <1KB per query

```json
{
  "map_view": {
    "zoom": "world",
    "tiles": [
      {
        "h3_cell": "8c283...",
        "status": "explored",
        "anomalies": 2,
        "last_update_hours": 2,
        "confidence": 0.95
      },
      {
        "h3_cell": "8c284...",
        "status": "explored",
        "anomalies": 0,
        "last_update_hours": 24,
        "confidence": 0.85
      },
      {
        "h3_cell": "8c285...",
        "status": "unexplored"
      }
    ]
  }
}
```

### Level 2: Regional View (Low Bandwidth)

What: Specific region with summary data  
Size: 5-20KB per query

```json
{
  "region_view": {
    "location": {"lat": 40.123, "lon": -74.567},
    "radius_m": 100,
    "zones": [
      {
        "zone_id": "A",
        "elevation_m": 42.5,
        "temperature": {
          "current": 22.1,
          "min": 20.0,
          "max": 25.0,
          "observations": 15,
          "confidence": 0.92
        },
        "obstacles": {
          "count": 3,
          "types": ["wall", "debris"],
          "density": "medium"
        },
        "objects_detected": [
          {"type": "person", "count": 2, "confidence": 0.87},
          {"type": "vehicle", "count": 1, "confidence": 0.92}
        ],
        "anomalies": [
          {
            "type": "temperature_spike",
            "severity": "high",
            "z_score": 3.2,
            "needs_verification": true
          }
        ],
        "last_updated": 7200,
        "verification_status": "PENDING"
      }
    ]
  }
}
```

### Level 3: Detailed View (High Bandwidth, On-Demand)

What: Full sensor data for anomalies or critical zones  
Size: 100KB-10MB per query (only when needed)

```json
{
  "detailed_view": {
    "zone_id": "A",
    "location": {"lat": 40.123, "lon": -74.567},
    "all_observations": [
      {
        "robot_id": "thermal_1",
        "timestamp": 1234567890,
        "value": 22.1,
        "confidence": 0.95,
        "metadata": {...}
      },
      {
        "robot_id": "thermal_2",
        "timestamp": 1234567900,
        "value": 21.8,
        "confidence": 0.92,
        "metadata": {...}
      }
    ],
    "point_cloud": {...},  // Only if requested
    "images": [...]         // Only if requested
  }
}
```

---

## Bandwidth Strategy by Bot Type & Mission

### Security Patrol Bot

**Mission:** "Detect threats quickly"  
**Needs:** Abstract view with anomaly flags  
**Bandwidth:** Ultra-low

```
Request: "What's the status of Building_A?"
Response (0.5KB):
{
  "zone": "Building_A",
  "anomalies": [
    {"type": "human_presence", "location": "entrance", "confidence": 0.91},
    {"type": "unusual_activity", "zone_id": "basement", "time_since_activity": "5min"}
  ],
  "last_update": "1min",
  "recommendation": "INVESTIGATE"
}

TOTAL: <1KB
```

### Inspection Bot

**Mission:** "Document damage"  
**Needs:** Regional view with object details  
**Bandwidth:** Low-Medium

```
Request: "What damage is at Building_A?"
Response (15KB):
{
  "damage_zones": [
    {
      "zone_id": "roof",
      "damage_type": "structural",
      "severity": "high",
      "observations": 5,
      "confidence": 0.88
    },
    {
      "zone_id": "entrance",
      "damage_type": "cosmetic",
      "severity": "low",
      "observations": 3,
      "confidence": 0.92
    }
  ],
  "recommended_actions": [...]
}

TOTAL: 15KB
```

### 3D Reconstruction Need

**Only when** explicitly requested for detailed analysis:

```
Request: "Show me 3D model of Building_A damage"
Response (5MB):
{
  "point_cloud": {...},
  "mesh": {...},
  "textures": [...]
}

TOTAL: 5MB (only when bot says "I need details")
```

---

## Aerial View for Planning (Critical)

**For mission planning, bots need top-down view with minimal objects.**

Like a game's strategic map:
- See entire region from above
- Only critical objects shown (anomalies, obstacles, changes)
- Very low bandwidth
- Perfect for path planning and threat assessment

```
Aerial View Request (Planning):
{
  "view_type": "aerial",
  "zoom": "region",
  "show_only": ["anomalies", "obstacles", "changes"],
  "minimal_objects": true
}

Response (<5KB):
[
  {x: 100, y: 50, type: "obstacle", size: "large", passable: false},
  {x: 150, y: 200, type: "anomaly", severity: "high", color: "red"},
  {x: 200, y: 175, type: "change", description: "new equipment", when: "2h ago"},
  {x: 80, y: 300, type: "zone", status: "unexplored"},
  {x: 50, y: 100, type: "zone", status: "explored_safe"}
]
```

**Benefits for Planning:**
- Aerial perspective: Natural for navigation/pathing
- Minimal objects: Only what matters for decision-making
- Fast transmission: <5KB for entire region view
- Supports path planning: See obstacles before moving
- Threat assessment: Red flags for anomalies
- Mission optimization: Identify best route

### Example: Security Bot Planning Route

```
Bot: "Planning patrol route through Building_A"

Query: Aerial view of Building_A (minimal objects, anomalies only)
Response (2KB):
[
  {x: 0, y: 0, type: "zone", size: "large", status: "safe"},
  {x: 200, y: 50, type: "anomaly", severity: "high", description: "unusual activity"},
  {x: 350, y: 100, type: "obstacle", size: "medium", passable: false},
  {x: 180, y: 200, type: "zone", status: "unexplored"}
]

Bot computes: "Route around obstacle at (350,100), investigate anomaly at (200,50)"
Detailed threat data only retrieved AFTER reaching that location.
```

### Example: Inspection Bot Planning Damage Assessment

```
Bot: "Planning inspection of Building_A"

Query: Aerial view with damage zones (minimal objects)
Response (3KB):
[
  {x: 50, y: 100, type: "damage", severity: "high", zone: "roof"},
  {x: 180, y: 50, type: "damage", severity: "low", zone: "entrance"},
  {x: 300, y: 200, type: "obstacle", size: "small", passable: true},
  {x: 100, y: 150, type: "safe_zone", status: "staging_area"}
]

Bot computes: "Visit high-severity damage first (roof), then low-severity (entrance)"
Detailed inspection data retrieved on-site as needed.
```

---

## Implementation: Tiered Query API

PyTerrainAI returns data at appropriate abstraction level:

```python
class ContextLevel(Enum):
    MINIMAP = 1          # <1KB: Explore overall status
    REGIONAL = 2         # 5-20KB: Plan next move
    DETAILED = 3         # 100KB-10MB: Investigate anomaly
    RAW_SENSOR = 4       # Unlimited: Full sensor data

class ContextQuery:
    location: GeoPoint
    radius_m: float
    mission: Mission
    detail_level: ContextLevel  # User controls bandwidth
    
async def get_context(query: ContextQuery) -> Context:
    """
    Return context at appropriate abstraction level.
    Higher detail = more bandwidth, only send when needed.
    """
    if query.detail_level == ContextLevel.MINIMAP:
        return self._get_minimap_view(query)
    elif query.detail_level == ContextLevel.REGIONAL:
        return self._get_regional_view(query)
    elif query.detail_level == ContextLevel.DETAILED:
        return self._get_detailed_view(query)
    else:
        return self._get_raw_sensor_data(query)

# Bot usage example:
security_bot.get_context(
    location=building_a,
    detail_level=ContextLevel.MINIMAP  # "<1KB, just anomalies"
)

inspector_bot.get_context(
    location=building_a,
    detail_level=ContextLevel.REGIONAL  # "15KB, damage details"
)

research_bot.get_context(
    location=anomaly_zone,
    detail_level=ContextLevel.RAW_SENSOR  # "Full data for analysis"
)
```

---

## Bandwidth Comparison

### Scenario: 100 observations at one location

**Naive approach (send everything):**
```
Raw observations: 100 × 1KB = 100KB
Point cloud: 1MB
Images: 10MB per image × 5 = 50MB
TOTAL: ~51MB per query
```

**FPS abstraction approach:**
```
Minimap: <1KB (just status icons)
Regional: 10KB (summaries, not raw)
Detailed (on demand): 100KB + 1MB + images
TOTAL (typical): 10KB
TOTAL (detailed): 51MB only when explicitly requested
```

**Savings:** 99.98% for typical queries, full detail available on demand.

---

## Progressive Disclosure Pattern

Like game maps revealing detail as you explore:

```
Bot approaches unknown zone:
├─ Minimap: "Zone unexplored"
└─ Confidence: Low

Bot moves closer:
├─ Minimap: "Zone explored, 2 anomalies"
└─ Confidence: Medium

Bot enters zone:
├─ Regional: "Temperature 22°C, 5 people, 1 obstacle"
└─ Confidence: High

Bot focuses on anomaly:
├─ Detailed: Full sensor data, point clouds, images
└─ Confidence: Very high
```

Each step reveals only necessary detail, saves bandwidth until needed.

---

## Application to PyTerrain

### PyTerrainMap Storage
- Stores ALL raw data (immutable)
- Never deletes
- Ready for any level of detail query

### PyTerrainAI Query Response
- Bot queries "give me context"
- PyTerrainAI returns appropriate abstraction
- Minimap for overview: <1KB
- Regional for planning: 10-20KB
- Detailed for investigation: 100KB-10MB
- Raw for research: unlimited

### Mission-Aligned Defaults
- Security: Minimap + anomaly alerts (optimized for speed)
- Inspection: Regional + zone summaries (planning)
- Research: Detailed/Raw (understanding)
- Real-time ops: Minimap + delta updates (streaming)

---

## Benefits

✅ **Bandwidth efficient:** 99% reduction for typical queries  
✅ **Scalable:** Can handle 1000s of observations without overwhelming  
✅ **Progressive:** Detail on demand  
✅ **Mission-aligned:** Security gets speed, research gets data  
✅ **Flexible:** Bots control what they see  
✅ **Game-proven:** FPS games do this at massive scale  

---

## Progressive View Hierarchy (Planning → Arrival → Detailed)

**Like FPS games, different views for different phases:**

### Phase 1: Planning (Far Away)
View: **Aerial/minimap**  
Bandwidth: <5KB  
Objects: Minimal (anomalies, obstacles, zones only)  
Use: Route planning, threat assessment  

```
Bot location: 500m away
Bot query: "Aerial overview for planning"
Response: Simplified map with critical objects only
```

### Phase 2: Approach (Getting Closer)
View: **Aerial with progressive detail**  
Bandwidth: 10-50KB  
Objects: More detail as bot gets closer  
Use: Fine-tuning route, identifying entry points  

```
Bot location: 100m away
Bot query: "High-res aerial view"
Response: More detailed zone information, specific coordinates
```

### Phase 3: On-Site (Arrived at Location)
View: **First-person / Detailed sensor view**  
Bandwidth: 100KB-10MB  
Objects: Full sensor data, images, point clouds  
Use: Detailed investigation, damage assessment  

```
Bot location: At target zone
Bot query: "First-person view with all sensor data"
Response: Full observations, images, point cloud, detailed readings
```

### Example: Inspection Mission Flow

```
T=0: Bot receives mission "Inspect Building_A for damage"
     ├─ Query: Aerial view (2KB)
     ├─ Response: Red flag at roof, orange flag at entrance
     ├─ Decision: "Plan route to roof first"
     └─ Bandwidth: 2KB

T=1: Bot navigates toward Building_A
     ├─ Query: Progressive aerial view (15KB)
     ├─ Response: More detail, specific damage zones visible
     ├─ Decision: "Refine route, enter through south entrance"
     └─ Bandwidth: 15KB

T=2: Bot arrives at Building_A, at roof location
     ├─ Query: First-person detailed view (500KB)
     ├─ Response: High-res images, point cloud, measurements
     ├─ Decision: "Document damage, take samples"
     └─ Bandwidth: 500KB

Total mission bandwidth: 517KB
(Equivalent naive approach: 51MB for all observations sent at start)

SAVINGS: 99% bandwidth reduction through progressive disclosure
```

---

## Key Design Principle

**"Send appropriate abstraction for current phase: Plan aerially, arrive in detail."**

1. **Planning phase:** Aerial view, minimal objects, <5KB
2. **Navigation phase:** Progressive detail as bot approaches
3. **Execution phase:** Full first-person sensor data on-site

This matches FPS game design:
- Strategic map view for planning
- Progressive reveals as you approach
- First-person immersion when you arrive
- Massive bandwidth savings

Don't send full detail until bot needs it. Provide aerial overview for planning, detailed FPS view when bot reaches location.
