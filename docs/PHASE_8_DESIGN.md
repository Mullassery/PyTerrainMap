# Phase 8: Spatial Knowledge Graph

**Objective:** Foundation for traversability intelligence. Build a graph data structure representing the spatial environment (nodes = places, edges = connections) with robot-aware metadata.

**Timeline:** 5-6 days | **Tests:** 15-20 | **Status:** Design Phase (2026-07-20)

---

## Core Problem

Routes aren't just "start → goal via waypoints." They're:
- **Topological:** Must go through doors, corridors, connectors (can't cut through walls)
- **Robot-specific:** A 2m door blocks 2.5m robots but not quadrotors
- **Temporal:** Doors open/close, flooding appears/clears
- **Historical:** 100 robots have successfully crossed here; 1 failed (but they're heavy)

A spatial knowledge graph solves this by making explicit:
1. **What places exist?** (rooms, terrain cells, landmarks, zones)
2. **How are they connected?** (doors, corridors, bridges, slopes)
3. **What are the constraints?** (width, height, weight limit, surface friction)
4. **What do robots know?** (success/failure records with confidence)

---

## Data Model

### Node Types

Each node represents a distinct spatial region with well-defined boundaries and properties.

```rust
pub enum NodeType {
    /// Indoor room with walls
    IndoorRoom {
        dimensions: (f32, f32, f32),  // width, depth, height
        floor_material: String,        // "carpet", "tile", "concrete"
    },
    
    /// Outdoor terrain cell (10m x 10m grid, e.g.)
    TerrainCell {
        surface_type: String,          // "grass", "mud", "pavement", "gravel"
        elevation: f32,                // meters above reference
        slope: f32,                    // degrees (0-90)
        roughness: f32,                // 0.0 (smooth) to 1.0 (rough)
    },
    
    /// Landmark (tree, building corner, pole)
    Landmark {
        landmark_type: String,         // "tree", "pole", "building_corner"
        height: Option<f32>,
        radius: f32,                   // collision radius
    },
    
    /// Zone (parking lot, field, warehouse)
    Zone {
        zone_type: String,             // "parking", "field", "warehouse"
        area_m2: f32,
    },
    
    /// Staircase or ramp
    VerticalTransition {
        transition_type: String,       // "stairs", "ramp", "elevator"
        vertical_rise: f32,            // meters
        slope_angle: f32,              // degrees (for ramps)
    },
}

pub struct Node {
    pub id: String,                    // Unique ID: "room_42", "terrain_x100_y50"
    pub node_type: NodeType,
    pub position: (f64, f64, f32),     // (lat, lon, elevation) in meters
    pub boundary: Option<Polygon>,     // 2D boundary for visualization
    pub created_at: i64,               // Unix timestamp
    pub last_observed: i64,            // When was this last seen?
    pub confidence: f32,               // 0.0-1.0: how certain is this node real?
}
```

### Edge Types

Each edge represents a navigable connection with constraints.

```rust
pub enum EdgeType {
    /// Door between rooms
    Door {
        width: f32,                    // meters
        height: f32,                   // meters
        is_open: bool,                 // State: open or closed?
        requires_key: bool,
        one_way: bool,
    },
    
    /// Corridor or hallway
    Corridor {
        width: f32,
        length: f32,
        surface: String,               // "tile", "carpet", etc.
        obstacles: Vec<String>,        // "pillar at 5m", etc.
    },
    
    /// Direct path (outdoor terrain)
    Path {
        surface_type: String,
        distance: f32,
        clearance_height: Option<f32>, // For low-clearance paths
    },
    
    /// Bridge or overpass
    Bridge {
        span_length: f32,
        width: f32,
        weight_limit_kg: Option<f32>,  // Max load
        surface: String,
    },
    
    /// Elevator or lift
    Elevator {
        capacity_kg: f32,
        height: f32,                   // Travel distance
        accessible: bool,              // ADA compliant?
    },
    
    /// Stairs
    Stairs {
        step_height: f32,
        width: f32,
        count: u32,
    },
    
    /// Ramp
    Ramp {
        length: f32,
        height: f32,
        slope: f32,                    // degrees
        surface: String,
    },
    
    /// Generic connection
    Generic {
        distance: f32,
        traversability_score: f32,     // 0.0-1.0 baseline difficulty
    },
}

pub struct Edge {
    pub id: String,                    // "edge_door_42_to_43"
    pub from_node: String,
    pub to_node: String,
    pub edge_type: EdgeType,
    pub distance: f32,                 // meters (path length)
    pub bidirectional: bool,
    pub created_at: i64,
    pub last_updated: i64,
    pub confidence: f32,               // 0.0-1.0: certainty of connection
}
```

### Metadata Schema

```rust
pub struct SpatialMetadata {
    /// Environment versioning
    pub environment_id: String,        // "farm_2026_q3"
    pub version: u32,
    pub created_at: i64,
    pub description: String,
    
    /// Reference frame
    pub origin_lat: f64,
    pub origin_lon: f64,
    pub origin_elevation: f32,
    pub coordinate_system: String,    // "WGS84", "UTM_10N", etc.
    
    /// Statistics
    pub total_nodes: u32,
    pub total_edges: u32,
    pub average_confidence: f32,
    
    /// Temporal info
    pub last_update: i64,
    pub sensor_types: Vec<String>,    // ["camera", "lidar", "gps"]
    pub observation_count: u32,
}

pub struct TraversabilityObservation {
    pub id: String,
    pub edge_id: String,
    pub robot_id: String,
    pub robot_type: String,           // "wheeled", "legged", "aerial"
    pub outcome: TraversalOutcome,    // Success, Failure(reason), Difficulty(score)
    pub timestamp: i64,
    pub confidence: f32,              // 0.0-1.0: how certain is this observation?
    pub notes: Option<String>,
}

pub enum TraversalOutcome {
    Success { time_ms: u32, energy_used: f32 },
    Failure { reason: String },      // "high_slip", "blocked", "timeout"
    Difficulty { score: f32 },       // 0.0-1.0 (1.0 = extremely difficult)
}
```

---

## PostgreSQL Schema

```sql
-- Environments (versioning)
CREATE TABLE environments (
    id VARCHAR(256) PRIMARY KEY,
    version INT NOT NULL,
    created_at BIGINT NOT NULL,
    description TEXT,
    origin_lat DOUBLE PRECISION,
    origin_lon DOUBLE PRECISION,
    origin_elevation FLOAT,
    coordinate_system VARCHAR(32),
    total_nodes INT DEFAULT 0,
    total_edges INT DEFAULT 0,
    average_confidence FLOAT DEFAULT 0.5,
    last_update BIGINT,
    sensor_types TEXT[],
    observation_count INT DEFAULT 0
);

-- Nodes
CREATE TABLE spatial_nodes (
    id VARCHAR(256) PRIMARY KEY,
    environment_id VARCHAR(256) REFERENCES environments(id),
    node_type VARCHAR(64) NOT NULL,  -- "indoor_room", "terrain_cell", "landmark"
    position_lat DOUBLE PRECISION NOT NULL,
    position_lon DOUBLE PRECISION NOT NULL,
    position_elevation FLOAT NOT NULL,
    boundary GEOMETRY(POLYGON, 4326),
    metadata JSONB,                  -- node_type-specific fields
    created_at BIGINT NOT NULL,
    last_observed BIGINT,
    confidence FLOAT DEFAULT 0.5,
    INDEX (environment_id),
    INDEX (node_type),
    SPATIAL INDEX (position_lat, position_lon)
);

-- Edges
CREATE TABLE spatial_edges (
    id VARCHAR(256) PRIMARY KEY,
    environment_id VARCHAR(256) REFERENCES environments(id),
    from_node VARCHAR(256) REFERENCES spatial_nodes(id),
    to_node VARCHAR(256) REFERENCES spatial_nodes(id),
    edge_type VARCHAR(64) NOT NULL,  -- "door", "corridor", "path", "bridge"
    distance FLOAT NOT NULL,
    bidirectional BOOLEAN DEFAULT TRUE,
    metadata JSONB,                  -- edge_type-specific fields
    created_at BIGINT NOT NULL,
    last_updated BIGINT,
    confidence FLOAT DEFAULT 0.5,
    INDEX (environment_id),
    INDEX (from_node),
    INDEX (to_node),
    INDEX (edge_type)
);

-- Robot-specific observations
CREATE TABLE traversability_observations (
    id VARCHAR(256) PRIMARY KEY,
    edge_id VARCHAR(256) REFERENCES spatial_edges(id),
    robot_id VARCHAR(256) NOT NULL,
    robot_type VARCHAR(64) NOT NULL, -- "wheeled", "legged", "aerial"
    outcome VARCHAR(32) NOT NULL,    -- "success", "failure", "difficulty"
    outcome_details JSONB,            -- {time_ms, energy_used} or {reason} or {score}
    timestamp BIGINT NOT NULL,
    confidence FLOAT DEFAULT 0.5,
    notes TEXT,
    INDEX (edge_id),
    INDEX (robot_id),
    INDEX (robot_type),
    INDEX (timestamp)
);

-- Fleet consensus (aggregated observations)
CREATE TABLE traversability_consensus (
    id VARCHAR(256) PRIMARY KEY,
    edge_id VARCHAR(256) REFERENCES spatial_edges(id),
    robot_type VARCHAR(64) NOT NULL,
    success_count INT DEFAULT 0,
    failure_count INT DEFAULT 0,
    average_difficulty FLOAT,
    last_updated BIGINT NOT NULL,
    confidence FLOAT DEFAULT 0.5,
    UNIQUE (edge_id, robot_type),
    INDEX (edge_id),
    INDEX (robot_type)
);
```

---

## Rust Module Structure

```
src/traversability/
├── mod.rs                    (exports)
├── spatial_graph.rs          (SpatialGraph struct, graph operations)
├── nodes.rs                  (Node, NodeType definitions)
├── edges.rs                  (Edge, EdgeType definitions)
├── observations.rs           (TraversabilityObservation, consensus)
├── metadata.rs               (SpatialMetadata schema)
└── storage.rs                (PostgreSQL backend)
```

---

## Key Operations

### Graph Construction
```rust
pub fn add_node(&mut self, node: Node) -> Result<()>
pub fn add_edge(&mut self, edge: Edge) -> Result<()>
pub fn remove_node(&mut self, node_id: &str) -> Result<()>
pub fn remove_edge(&mut self, edge_id: &str) -> Result<()>
```

### Queries
```rust
pub fn get_node(&self, node_id: &str) -> Option<Node>
pub fn get_edge(&self, edge_id: &str) -> Option<Edge>
pub fn neighbors(&self, node_id: &str) -> Vec<(String, Edge)>
pub fn connected_component(&self, start: &str) -> Vec<String>
pub fn all_edges_from(&self, node_id: &str) -> Vec<Edge>
```

### Traversability Learning
```rust
pub fn record_observation(&mut self, obs: TraversabilityObservation) -> Result<()>
pub fn compute_consensus(&mut self, edge_id: &str) -> ConsensusResult
pub fn get_edge_traversability(&self, edge_id: &str, robot_type: &str) -> f32
```

### Spatial Queries
```rust
pub fn nodes_in_region(&self, lat_range: (f64, f64), lon_range: (f64, f64)) -> Vec<Node>
pub fn shortest_path(&self, start: &str, goal: &str) -> Option<Vec<String>>
pub fn connected_regions(&self, region_id: &str) -> Vec<Node>
```

---

## Testing Strategy

### Unit Tests (15-20)

1. **Node Operations (3 tests)**
   - Create and retrieve nodes
   - Update node metadata
   - Delete nodes and cascading edges

2. **Edge Operations (3 tests)**
   - Create and retrieve edges
   - Bidirectional edge handling
   - Edge type specifics (doors, corridors, paths)

3. **Graph Traversal (3 tests)**
   - Neighbors query
   - Connected components
   - Path existence checks

4. **Observations & Consensus (4 tests)**
   - Record single observation
   - Compute consensus from multiple observations
   - Robot-type specific traversability
   - Confidence weighting

5. **Spatial Queries (2 tests)**
   - Region queries (lat/lon bounding)
   - Shortest path (no obstacles)

6. **Metadata & Versioning (2 tests)**
   - Environment creation
   - Version tracking

---

## Success Criteria

- ✅ All 15-20 tests passing
- ✅ Node/edge types cover common scenarios (rooms, terrain, connectors)
- ✅ PostgreSQL schema ready for production
- ✅ Graph operations have <1ms latency for 10K nodes
- ✅ Observation storage + consensus computation working
- ✅ Code committed and documented

---

## Next Phases

- **Phase 9:** Multi-layer distance models (geometric, topological, cost, semantic)
- **Phase 10:** Robot capability profiles (dimensions, locomotion, weight)
- **Phase 11:** Fleet learning engine (observation aggregation, consensus)

---

## Dependencies

- **Storage:** PostgreSQL (pluggable, optional for tests via in-memory backend)
- **Geometry:** geo crate for spatial queries
- **Serialization:** serde for JSON metadata
- **Database:** sqlx for PostgreSQL access (optional feature)

**No external robot frameworks required.** Pure spatial data structure.

---

**Phase 8 starts 2026-07-20. Target completion: 2026-07-25 (5-6 days).**
