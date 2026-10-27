# SyncFleet integration

PyTerrainMap observations can be replicated through
[SyncFleet](../SyncFleet) by recording them as SyncFleet
*operations*. SyncFleet replicates its operation journal over mutual-TLS 1.3;
it does not replicate arbitrary SQLite tables.

## What is implemented and tested

`pyterrain_map.syncfleet.SyncFleetBridge`:

- serialises an `Observation` (or a dict) into a stable JSON payload;
- derives a deterministic `entity_id` (retrying the same observation is
  idempotent at the entity level);
- registers the `terrain` dataset with the `terrain_observations` table
  allowlist if it does not exist;
- records one `insert` operation per observation via
  `syncfleet dataset apply`.

The cross-repo test (`tests/test_syncfleet_bridge.py`) runs the **real**
`syncfleet` binary against a temporary node and verifies the SQLite
`operation_journal` rows and payloads.

## Setup

SyncFleet needs a config file and an initialized node. It does not create the
config file for you.

```toml
# syncfleet.toml
[node]
node_name = "robot-1"
data_dir = "/var/lib/syncfleet"
log_level = "warn"

[storage]
wal_mode = true
```

```bash
syncfleet init --node robot-1 --config syncfleet.toml
```

## Usage

```python
from pyterrain_map import TerrainMap, Observation
from pyterrain_map.syncfleet import SyncFleetBridge

terrain = TerrainMap()
terrain.push_observation(Observation(
    robot_id="robot-1",
    timestamp=1_700_000_000,
    lat=40.7128,
    lon=-74.0060,
    sensor_type="thermal",
    value_json='{"celsius": 22.5}',
    confidence=0.95,
))

bridge = SyncFleetBridge(
    binary="syncfleet",
    config="syncfleet.toml",
    dataset="terrain",
    table="terrain_observations",
)
bridge.apply_observations(terrain.observations())
```

Verify:

```bash
syncfleet journal stats --config syncfleet.toml
syncfleet dataset show --config syncfleet.toml terrain
```

Then use SyncFleet's own commands for transport and peer management
(`peer token/redeem`, `daemon start`, `sync now`). The bridge does not enroll
peers or open connections.

## Payload

```json
{
  "robot_id": "robot-1",
  "timestamp": 1700000000,
  "latitude": 40.7128,
  "longitude": -74.006,
  "sensor_type": "thermal",
  "sensor_value": {"celsius": 22.5},
  "confidence": 0.95
}
```

## What is not implemented

- **Automatic capture:** `TerrainMap.push_observation` does not call the bridge;
  the application must invoke it.
- **Typed materialization:** SyncFleet records operations; it does not currently
  materialize them into a typed `terrain_observations` table.
- **PostgreSQL outbox/CDC:** PyTerrainMap's live PostgreSQL backend is not
  captured automatically.
- **Transport testing:** peer enrollment and network sync are SyncFleet's own
  tested surface, not re-tested here.

These are tracked integration gaps, not silent claims.
