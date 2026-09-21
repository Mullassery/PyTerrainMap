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

## Use cases

- **Fusing terrain observations from multiple robots into one spatial/
  temporal index** — H3-indexed, confidence decaying with age, queryable
  by location and time window.
- **Standing up a real HTTP API over that store** for other services to
  query, without building your own server layer.
- **3D Tiles export for real SLAM/photogrammetry output** — the SLAM and
  export paths are real implementations, not placeholders.
- **Not yet a good fit for:** real terrain/mobility analysis (currently
  fixed/demo output, no real elevation source), persistence from Python
  (the Postgres backend is real at the Rust level but not yet wired into
  the Python API -- `pip install pyterrainMap` users still get in-memory-only
  behavior today; SQLite/BigQuery remain config/schema types with no live
  connection either way), or real photogrammetry (bundle adjustment/pose
  estimation are explicit placeholders) — see [Features](#features) below
  for the full per-feature breakdown.

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
| Persistent storage backends (SQLite/PostgreSQL/BigQuery) | **Mixed, Rust-level only.** Postgres write-through + restore-on-startup (`ServerState::with_backend()`, `src/storage/postgres.rs`) is a real, working implementation as of commit `64e119e` (2026-08-24) -- but it is **not exposed through the Python API**, so `pip install pyterrainMap` users still get in-memory-only behavior regardless. SQLite and BigQuery remain config/schema types only, with no live DB connection. Use the in-memory store above for real use from Python today. |

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

## Known Issues

- **`cargo clippy` reports 227 errors** (`continue-on-error: true` in CI, so this
  doesn't block the pipeline), almost all `non_snake_case` field-naming lint
  violations (e.g. `geometricError`, `POINTS_LENGTH`, `BATCH_ID`) in
  `src/tiles_3d/mod.rs` and related export code. These are **not naming mistakes** —
  the fields match the real [Cesium 3D Tiles](https://github.com/CesiumGS/3d-tiles)
  JSON spec's field names exactly, since this code serializes/deserializes that
  format. The correct fix is `#[allow(non_snake_case)]` on the affected structs (or
  `#[serde(rename = "...")]` with snake_case Rust field names) — renaming the fields
  outright would break the actual Cesium format contract. Not fixed in this pass:
  227 individual annotations is a large mechanical change better done as its own
  reviewed pass, not blind inside an audit.
- **2026-09-13: `cargo audit` is now fully clean — 0 vulnerabilities** (was 9 at the
  start of this pass). Fixed across two passes the same day:
  - `rcgen` 0.11→0.14, `rustls`/`tokio-rustls`/`rustls-pemfile` bumped, `sqlx` 0.7→0.9,
    `geo` 0.27→0.28, and removed the entirely-unused `image`/`statrs` dependencies
    (verified zero usage anywhere in this repo) — see git history for the full
    per-crate rationale.
  - `pyo3` 0.22→0.29 (its 2 remaining advisories): `PyObject` was removed as a type
    alias (→ `Py<PyAny>` everywhere), `IntoPy` was removed in favor of the fallible
    `IntoPyObject`/`IntoPyObjectExt` (`.into_py(py)` → `.into_py_any(py).expect(...)`,
    since these are all primitive conversions that cannot fail in practice),
    `Python::with_gil` → `Python::attach`, `PyDict`/`PyList::new_bound` → plain `::new`.
  - `hyper` 0.14→1.x (a new `h2` advisory that only surfaced *after* the pyo3 fix):
    the old all-in-one `hyper::Server`/`AddrIncoming`/`make_service_fn` was replaced
    with a transport-agnostic `serve_connection` API — `run_http_from_listener` is now
    a manual accept loop (mirroring the pattern `run_https_from_listener` already used
    for TLS termination), using `hyper-util`'s `TokioIo` adapter and
    `http-body-util`'s `Full<Bytes>`/`BodyExt::collect`.
  - Verified: `cargo check`/`build --release` clean, `maturin develop --release` + the
    full 205-test `pytest` suite passing, including `tests/test_server.py`'s 7 real
    end-to-end tests (a genuine TLS handshake among them) against the actual compiled
    extension. `tests/server_integration.rs` (the Rust-side equivalent) compiles
    cleanly but can't run standalone via plain `cargo test` on macOS specifically —
    this crate's PyO3 extension-module build defers Python C-API symbol resolution to
    a real embedding Python process, so a bare test executable started outside Python
    has nothing to resolve those symbols against. Confirmed pre-existing (present on a
    clean checkout before any of this pass's changes too) and CI runs on Linux.
  - `rustls-pemfile` still shows as "unmaintained" regardless of version (the whole
    project is archived) — no upstream fix exists to adopt; this is not a vulnerability.
- **`black --check` failure (18 files) — fixed in this pass.** Reformatted with the
  same black release CI installs; also fixed a `pyproject.toml` deprecation warning
  (`[tool.ruff]`'s `select`/`ignore` keys needed to move under `[tool.ruff.lint]`).
- **Fixed in a prior pass**: `tests/test_statguardian_integration.py::test_valid_coordinates`
  used timestamps 1000s apart as "valid" test data, which correctly tripped
  `TemporalCoordinateContract`'s real `max_temporal_gap_seconds=60` warning check —
  a test-data bug, not a logic bug in the validator (a genuine >60s gap between
  observations *should* warn). Adjusted the test's timestamps to 10s apart.
- **3 workflow steps use `actions-rs/toolchain@v1`**, an action whose maintaining org
  archived its repos years ago — still technically functional (not the cause of any
  of the above), but a maintenance risk worth planning a `dtolnay/rust-toolchain`
  migration for.

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

**License**: Apache License 2.0. See [LICENSE](LICENSE) for full terms.
