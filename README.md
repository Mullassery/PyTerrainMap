# PyTerrainMap

> **Spatial intelligence platform for multi-robot terrain mapping.** Rust core (H3 spatial indexing, temporal decay, sensor fusion, traversability graph, Gaussian-splatting probabilistic mapping) with Python bindings via PyO3, plus a real HTTP/HTTPS API server.

[![CI](https://github.com/Mullassery/PyTerrainMap/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/PyTerrainMap/actions/workflows/ci.yml)
![Python](https://img.shields.io/badge/Python-3.10+-blue.svg)
![Distribution](https://img.shields.io/badge/Distribution-Wheels--Only-blue.svg)
This project is licensed under the [Apache License 2.0](LICENSE).

---

## What this actually is

PyTerrainMap is an append-only observation store for multi-robot terrain
data (`TerrainMap`), indexed spatially (H3 hexagonal grid) and temporally
(exponential/linear confidence decay), with:

- **Immutable, append-only storage** -- observations can be added and
  queried, never deleted or overwritten through the normal API (see
  [`docs/DATA_INTEGRITY.md`](docs/DATA_INTEGRITY.md)).
- **A real HTTP/HTTPS API server** you can actually start and hit with
  HTTP requests (`pyterrain_map.start_server(...)`) -- not just typed
  request/response structs.
- **Spatial + temporal querying**: find observations near a point within
  a time window, with confidence that decays with age.
- **Terrain intelligence helpers**: `analyze_terrain()`,
  `assess_mobility()`, `is_accessible()`, `explain_field()` for
  persona-aware (drone / wheeled / quadruped / humanoid) terrain
  assessment -- currently fixed/demo logic, not backed by a real
  elevation/DEM data source (see Features table below).
- **3D reconstruction + Gaussian-splatting probabilistic mapping** in the
  Rust core (SLAM, photogrammetry, traversability graphs) -- exposed to
  Python via PyO3 bindings.

If you're looking for something else -- a full SLAM pipeline you point a
camera at, a hosted service, ML-based object classification -- this isn't
that (yet). This README describes what's implemented and testable today.

---

## Installation

```bash
pip install pyterrainMap
# or with uv
uv pip install pyterrainMap
```

```bash
python -c "import pyterrain_map; print(pyterrain_map.__version__)"
```

### Requirements
- Python 3.10+
- Precompiled wheel on PyPI: **macOS arm64 only**
  (`cp310-abi3-macosx_11_0_arm64`), verified against the published
  files for every release through 1.5.0. There is currently no Linux
  or x86_64 macOS wheel, and no sdist to fall back to -- on those
  platforms, build from source (below) instead of `pip install`.

### From source

```bash
git clone https://github.com/Mullassery/PyTerrainMap.git
cd PyTerrainMap
pip install maturin
maturin develop --release
```

---

## Quick Start

```python
import json
import time

from pyterrain_map import TerrainMap, Observation, GeoPoint, analyze_terrain, assess_mobility

# Create an in-memory, append-only terrain map
terrain_map = TerrainMap()

# Add a sensor observation
obs = Observation(
    robot_id="robot-1",
    timestamp=int(time.time()),
    lat=40.7128,
    lon=-74.0060,
    sensor_type="thermal",
    value_json=json.dumps({"celsius": 22.5}),
    confidence=0.95,
)
terrain_map.push_observation(obs)

# Query observations near a point, within a time window
result = terrain_map.query(
    GeoPoint(40.7128, -74.0060),
    region_radius_km=1.0,
    time_window_seconds=3600,
)
print(f"Found {result.count} observations, avg confidence {result.avg_confidence:.1%}")

# Terrain intelligence: what can traverse this area?
terrain = analyze_terrain(40.7128, -74.0060, radius_km=1.0)
for robot_type in ("drone", "wheeled", "quadruped", "humanoid"):
    mobility = assess_mobility(terrain, robot_type)
    print(f"{robot_type}: traversable={mobility.traversable}, "
          f"difficulty={mobility.difficulty_label()}")
```

See `python/examples/01_quick_start.py` for a fuller runnable version of
the above, and `tests/test_terrain_map_core.py` / `tests/test_server.py`
for more usage patterns backed by real tests.

### Running the API server

```python
from pyterrain_map import start_server

# Plain HTTP, for local development
handle = start_server(host="127.0.0.1", port=8080)
print(handle)  # ServerHandle(host="127.0.0.1", port=8080, tls=false, running=true)

# ... make requests: GET /health, GET /stats, POST /observations,
#     POST /query/spatial ...

handle.stop()
```

```python
# HTTPS with a self-signed dev certificate (generated on the fly).
# Self-signed certs are for local dev/test only -- real clients won't
# trust them without extra configuration. Pass cert_path/key_path for a
# real certificate in production.
handle = start_server(host="127.0.0.1", port=8443, tls=True)
```

```bash
curl -X POST http://127.0.0.1:8080/observations \
  -H "Content-Type: application/json" \
  -d '{"robot_id":"robot-1","timestamp":1700000000000000,"latitude":40.7128,"longitude":-74.0060,"sensor_type":"thermal","sensor_value":{"celsius":22.5},"confidence":0.9,"metadata":{}}'

curl http://127.0.0.1:8080/stats
```

---

## Features

| Area | Status |
|---|---|
| Append-only observation storage (H3 spatial + temporal-decay index) | Implemented, tested |
| Real HTTP/HTTPS API server (`start_server()`) | Implemented, tested end-to-end (Rust + Python) |
| Terrain intelligence (`analyze_terrain`, `assess_mobility`, `is_accessible`) | Fixed/demo output, not real terrain analysis -- `analyze_terrain()` returns the same risk score and "retrieved from SRTM" text regardless of the coordinates passed in (no elevation/DEM source is actually queried); `assess_mobility()` is a static lookup table keyed only on robot type. Fine for exercising the API shape; don't rely on it for real terrain assessment. |
| Anomaly detection (z-score, IQR, rogue-bot, drift, spike) + temporal quality weighting | Implemented; one Rust unit test (`test_anomaly_detection_spike`) is currently failing on `main` -- the other detectors pass. |
| Traversability knowledge graph | Implemented, tested |
| Gaussian-splatting probabilistic mapping (fusion, frontier detection, fleet learning) | Partially implemented -- core fusion/storage works, but frontier "strategic value" scoring and semantic terrain classification are hardcoded placeholders, and splat temporal decay (`apply_decay_to_store`) is currently a no-op. 3 related Rust unit tests are currently failing on `main` (frontier scoring, fleet learning, semantic terrain cost). |
| 3D reconstruction (SLAM, photogrammetry, 3D Tiles export) | Mixed -- SLAM (loop closure, BoW) and 3D Tiles export are real implementations; photogrammetry's bundle adjustment, pose estimation, and point-cloud color estimation are explicitly-marked placeholders, not a real SfM solver. One SLAM unit test (`test_loop_closure_detector`) is currently failing on `main`. |
| SQLite/PostgreSQL/BigQuery persistence backends | **Not implemented** -- config/schema types only, no live DB connection. Use the in-memory store above for real use today. |

---

## Development

```bash
# Rust
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Python (after `maturin develop`)
pip install -e ".[dev,imaging]"
pytest tests/ -v
```

---

## Cross-repo compatibility

This repo is one of several independently-published robotics packages by
the same author (`PyRoboSimulator`, `pyroboreplay`, `PyRoboFrames`,
`PyRoboVision`). Verified by reading every `Cargo.toml`/`pyproject.toml`
across all of them, plus grepping source in both directions: **none has a
Cargo or pip dependency on this repo, and this repo has none on them.**
`pyroboreplay`'s README describes a "PyTerrainMap Integration" phase, and
this project's own `CLAUDE.md` used to describe integrations with
`pyroboreplay`, `StatGuardian`, and `PyStreamMCP` — none of those are
implemented in code today (verified via grep across all four repos in both
directions); `CLAUDE.md` now marks that section as aspirational, and
`pyroboreplay`'s README has the corresponding correction.

## Support

**mullassery@gmail.com**

---

**License**: Proprietary -- free to use with explicit attribution. See [LICENSE](LICENSE) for full terms.
