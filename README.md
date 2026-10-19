# PyTerrainMap

Spatial intelligence platform for multi-robot terrain mapping — an append-only, H3-indexed observation store with temporal decay, a real HTTP/HTTPS API server, and terrain-intelligence helpers, built on a Rust core with PyO3 bindings.

[![CI](https://github.com/Mullassery/PyTerrainMap/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/PyTerrainMap/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/pyterrainMap)](https://pypi.org/project/pyterrainMap/)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-blue.svg)](https://www.python.org/)

## What it is

PyTerrainMap is an append-only observation store for multi-robot terrain data (`TerrainMap`), indexed spatially on an H3 hexagonal grid and temporally with confidence that decays with age. It exposes spatial + temporal querying, a real HTTP/HTTPS server you can start from Python (`start_server(...)`), and terrain-intelligence helpers (`analyze_terrain()`, `assess_mobility()`, `is_accessible()`, `explain_field()`). The core is Rust; the Python bindings are built with PyO3/maturin. Observations can be added and queried but never deleted or overwritten through the normal API.

## Features

- **Append-only, H3-indexed store** — observations keyed by `(H3Cell, ElevationBucket)` with exponential/linear confidence decay.
- **Real HTTP/HTTPS API server** — `pyterrain_map.start_server(host, port, tls=...)`; tested end-to-end in Rust and Python (including a TLS handshake). Endpoints include `GET /health`, `GET /stats`, `POST /observations`, `POST /query/spatial`.
- **Terrain intelligence** — `analyze_terrain()` queries real elevation (Open-Meteo / Copernicus DEM GLO-90) at the center and four cardinal points and derives a real measured slope; `assess_mobility()` scales difficulty, speed, and traversability per robot type (drone / wheeled / quadruped / humanoid).
- **Traversability knowledge graph** — implemented and tested.
- **3D reconstruction** — SLAM (loop closure, bag-of-words), real multi-view photogrammetry geometry, and Cesium 3D Tiles export (Rust core; see status for exposure).
- **Gaussian-splatting probabilistic mapping** — fusion and storage work; frontier scoring and semantic classification are partial.
- **Persistent storage** — PostgreSQL is real end-to-end from Python (`TerrainMap.with_postgres(...)`, `push_observation`/`push_batch` write-through, `restore_from_backend()`).
- **CLI** — `pytm` (`setup`, `configure`, `version`, `query`) and `pytm-setup`.

## Who it's for / Use cases

- **Fusing terrain observations from multiple robots** into one spatially and temporally queryable index with age-decayed confidence.
- **Standing up an HTTP API over the store** for other services to query without building a server layer.
- **Robot-specific mobility assessment** — decide whether a drone/wheeled/quadruped/humanoid platform can traverse an area, from real slope data.
- **3D Tiles export** for SLAM/photogrammetry output.
- **Persistence from Python** — write-through and restore-on-startup against PostgreSQL.

## What it is not

- **Not a full camera-to-map SLAM pipeline.** Photogrammetry requires caller-supplied 2D keypoint detections (imaging is deliberately Python-side) and is Rust-level only today, not exposed via the Python API.
- **Not fully offline.** `analyze_terrain()` makes a real network call to a public elevation API and raises an error (rather than fabricating data) if it cannot reach it.
- **Not all storage backends are live.** SQLite and BigQuery are config/schema types with no live connection; only PostgreSQL is real end-to-end.
- **Gaussian-splatting mapping is partially implemented** — frontier "strategic value" scoring and semantic terrain classification are placeholder, and splat temporal decay is a no-op.
- **Not a hosted service** and not ML-based object classification.

## Project status

Alpha (Development Status 3), version **1.10.1**. The append-only store, H3/temporal indexing, HTTP/HTTPS server, PostgreSQL persistence, and terrain-slope intelligence are implemented and tested. The Rust core is now **`cargo clippy --release -- -D warnings` clean (0 warnings)** and `cargo test` passes except for one documented, pre-existing failing test (`exploration::gaussian_frontier_integration::tests::test_score_frontier_with_high_uncertainty`); the remaining dead-code fields have been wired into real behavior (temporal decay application, cache update policy, scheduler temporal-correction queue, fleet conflict resolution, predictive-caching learning rates, Postgres table name, federated query accessors). Prebuilt wheels are published for **macOS arm64 only**; other platforms build from the source distribution with a Rust toolchain. See [docs/DATA_INTEGRITY.md](docs/DATA_INTEGRITY.md) and `docs/` for details.

## Installation

```bash
pip install pyterrainMap

# or with uv
uv pip install pyterrainMap

python -c "import pyterrain_map; print(pyterrain_map.__version__)"
```

Requires Python 3.10+. Prebuilt wheels are macOS arm64 only; on other platforms build from source:

```bash
git clone https://github.com/Mullassery/PyTerrainMap.git
cd PyTerrainMap
pip install maturin
maturin develop --release
```

## Quick start

```python
import json, time
from pyterrain_map import TerrainMap, Observation, GeoPoint, analyze_terrain, assess_mobility

terrain = TerrainMap()  # in-memory, append-only

terrain.push_observation(Observation(
    robot_id="robot-1", timestamp=int(time.time()),
    lat=40.7128, lon=-74.0060, sensor_type="thermal",
    value_json=json.dumps({"celsius": 22.5}), confidence=0.95,
))

result = terrain.query(GeoPoint(40.7128, -74.0060), region_radius_km=1.0, time_window_seconds=3600)
print(f"Found {result.count} observations, avg confidence {result.avg_confidence:.1%}")

terrain_info = analyze_terrain(40.7128, -74.0060, radius_km=1.0)
for robot_type in ("drone", "wheeled", "quadruped", "humanoid"):
    mobility = assess_mobility(terrain_info, robot_type)
    print(robot_type, mobility.traversable, mobility.difficulty_label())
```

## Usage

Start the API server (plain HTTP in-process; `tls=True` uses a self-signed dev certificate):

```python
from pyterrain_map import start_server

handle = start_server(host="127.0.0.1", port=8080)
# GET /health, GET /stats, POST /observations, POST /query/spatial ...
handle.stop()
```

```bash
curl -X POST http://127.0.0.1:8080/observations \
  -H "Content-Type: application/json" \
  -d '{"robot_id":"robot-1","timestamp":1700000000000000,"latitude":40.7128,"longitude":-74.0060,"sensor_type":"thermal","sensor_value":{"celsius":22.5},"confidence":0.9,"metadata":{}}'

curl http://127.0.0.1:8080/stats
```

```bash
pytm setup                       # interactive configuration wizard
pytm query --lat 40.1 --lon -74.5 --radius-km 0.5 --json
```

PostgreSQL persistence from Python:

```python
terrain = TerrainMap.with_postgres("postgres://user:pass@localhost/terrain", pool_size=5)
terrain.restore_from_backend()
```

## Documentation

- [docs/PYTHON_BINDINGS.md](docs/PYTHON_BINDINGS.md) — Python API reference
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — architecture overview
- [docs/DATA_INTEGRITY.md](docs/DATA_INTEGRITY.md) — append-only guarantees
- [docs/TEMPORAL_NORMALIZATION.md](docs/TEMPORAL_NORMALIZATION.md) — temporal decay and clock handling
- [docs/ROS_BRIDGE_ARCHITECTURE.md](docs/ROS_BRIDGE_ARCHITECTURE.md) — ROS integration design
- [docs/](docs/) — full documentation index
- [python/examples/01_quick_start.py](python/examples/01_quick_start.py) — runnable quick start
- [CHANGELOG.md](CHANGELOG.md) — notable changes

## Known issues and deferred work

**Blocked / failed:**
- Linux/Windows wheels; real SQLite/BigQuery backends.

**Rejected / deferred:**
- Full SLAM from scratch.

Full shipped/blocked/rejected detail: [`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Bug reports: [GitHub Issues](https://github.com/Mullassery/PyTerrainMap/issues). Support: mullassery@gmail.com.

## Decentralized fleet sync with SyncFleet (planned integration)

PyTerrainMap pairs with [SyncFleet](https://github.com/Mullassery/SyncFleet), a
secure, local-first, peer-to-peer SQLite synchronization engine. The intended
design gives a multi-robot edge swarm an out-of-the-box decentralized
map-sharing protocol: each robot accumulates terrain observations in its local
PyTerrainMap store, and SyncFleet replicates the selected observation table to
authorized peers over mutual-TLS 1.3 — with no cloud round-trip or broker, so it
keeps working across intermittent or air-gapped links.

**Why they fit together**

- Observations are **append-only**, so cross-robot inserts never conflict — the
  case SyncFleet's single-writer default handles cleanly, without needing CRDTs.
- The observation table maps to a SyncFleet **dataset** (table allowlist), and
  each observation UUID becomes an operation `entity_id`.
- Both are Rust + Tokio + rustls / TLS 1.3 with an edge, local-first focus.
- SyncFleet supplies mutual-TLS transport, peer enrollment, and per-dataset ACLs
  for the replication channel.

**Status: not wired up yet.** This is a design goal, not a current capability.
Before it works, SyncFleet must (1) materialize replicated operations into the
allowlisted table (today it only replicates its journal), (2) share a live
storage substrate (SyncFleet is SQLite; PyTerrainMap's live backend is
PostgreSQL), and (3) capture PyTerrainMap writes as SyncFleet operations via an
outbox/CDC bridge. Until then, treat the two as independent, complementary
tools.

## Part of the robotics & autonomy family

PyTerrainMap is one of six complementary, composable tools in the Mullassery **robotics & autonomy** family. Together they form a single stack: **perceive → map → simulate → learn → debug → coordinate the fleet**.

| Tool | Role | What it does |
|---|---|---|
| [PyRoboVision](https://github.com/Mullassery/PyRoboVision) | Perception | Multi-object tracker (Kalman filter + Hungarian algorithm) and trajectory predictor for robot perception. |
| [PyTerrainMap](https://github.com/Mullassery/PyTerrainMap) | Mapping / SLAM | **This project** — fuses multi-robot terrain observations into a spatial-temporal store, with H3 spatial indexing and SLAM. |
| [PyRoboSimulator](https://github.com/Mullassery/PyRoboSimulator) | Simulation | World simulator for autonomous systems, with a FastAPI backend. |
| [PyRoboFrames](https://github.com/Mullassery/PyRoboFrames) | Learning data | Rust-backed ML dataloader for robot-learning datasets (native LeRobot format, Apple Silicon hardware video decode). |
| [PyRoboReplay](https://github.com/Mullassery/PyRoboReplay) | Mission forensics | Forensic-debugging and causal-analysis engine for autonomous robot mission replay. |
| [SyncFleet](https://github.com/Mullassery/SyncFleet) | Fleet coordination | Rust + SQLite, peer-to-peer state sync for robotics fleets — mutual-TLS 1.3, version-vector replication, mDNS discovery; conflict resolution on the roadmap. |

Each works standalone, but they're complementary by design. The planned integration with SyncFleet (detailed below) will allow terrain maps to sync across fleets without relying on a central database.

[Browse all projects →](https://github.com/Mullassery?tab=repositories)

## License

Apache License 2.0 — see [LICENSE](LICENSE).
