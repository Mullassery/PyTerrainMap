# Changelog

All notable changes to PyTerrainMap will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.8.0] - 2026-09-28

### Added

- **Real elevation/DEM data wired into `analyze_terrain()`/`assess_mobility()`.**
  `analyze_terrain()` previously returned fixed output regardless of the
  coordinates passed in -- a hardcoded slope-risk severity of exactly
  `0.4` every time, and a summary claiming "Elevation data retrieved from
  SRTM" despite never actually querying any elevation source at all. It
  now queries real elevation at the center point and 4 cardinal points at
  the requested radius from Open-Meteo's free, no-API-key elevation API
  (Copernicus DEM GLO-90, `src/elevation/mod.rs`), and derives a real
  measured max slope (`atan(rise/run)`) from those samples. This makes a
  real network call and raises a real `PyRuntimeError` if it fails, rather
  than fabricating data. `PyTerrainAnalysis` gained two new real fields,
  `elevation_m` and `max_slope_degrees`. `assess_mobility()` now scales
  difficulty, recommended speed, battery impact, and traversability from
  that real slope, per robot type (wheeled robots become non-traversable
  above a real 20-degree slope limit; quadruped 35, humanoid 25; drones
  are slope-independent, matching flight not ground contact). Verified
  live against two real locations with genuinely different terrain:
  Zermatt, Switzerland (alpine, measured ~25 degree slope, elevation
  ~1613m, wheeled robots correctly marked non-traversable) vs. flat Iowa
  farmland (measured ~1.7 degree slope). 8 new unit tests in
  `src/elevation/mod.rs` (pure slope/offset math, deterministic, run in
  CI) plus one `#[ignore]`d real-network integration test (run explicitly
  with `--ignored`, not part of the default CI run since it depends on
  outbound network access).

### Fixed

- Adding `reqwest` (for the elevation lookup above) put both the `ring`
  and `aws-lc-rs` rustls crypto providers into the dependency graph
  (`reqwest`'s `rustls-tls` feature supports either, leaving the choice to
  the application), which broke the existing HTTPS server's
  `tls_acceptor_from_pem()`: `rustls::ServerConfig::builder()` could no
  longer auto-detect a single provider and started panicking ("Could not
  automatically determine the process-level CryptoProvider"). This was a
  real regression surfaced by the full test suite
  (`server::tests::test_tls_acceptor_from_generated_cert`), not a
  pre-existing issue. Fixed by explicitly installing the `ring` provider
  (matching what `rcgen`, used for dev certificate generation, is already
  built with) at the top of `tls_acceptor_from_pem()`. Verified via the
  full unit test suite plus the real end-to-end HTTPS TLS handshake
  integration test (`tests/server_integration.rs`).

## [1.7.0] - 2026-09-28

### Added

- **PostgreSQL persistence exposed through the Python API.**
  `TerrainMap.with_postgres(connection_string, pool_size)` connects to a
  real Postgres instance and initializes schema; `push_observation`/
  `push_batch` now write through to it; `restore_from_backend()` reloads
  prior observations back into memory (e.g. after a process restart).
  Previously this persistence existed only at the Rust level
  (`ServerState::with_backend()`) and was unreachable from
  `pip install pyterrainMap`. Verified against a real local Postgres 16
  container, which surfaced 6 real, previously-undiscovered bugs in the
  Postgres backend itself -- see Fixed, below.
- `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `ROADMAP_HONEST.md`.
- `.github/dependabot.yml` (cargo, pip, github-actions ecosystems).
- `.github/ISSUE_TEMPLATE/bug_report.yml`, `feature_request.yml`, `.github/pull_request_template.md`.
- `pip-audit` step in CI's `lint` job (`continue-on-error: true`; advisory only, no historical baseline yet).

### Fixed

- **Six real bugs in the PostgreSQL storage backend (`src/storage/postgres.rs`),
  found by wiring it up to the Python API and verifying against a real
  local Postgres 16 instance** -- this backend had apparently never
  actually been exercised against a live Postgres server before, despite
  being described as "real, working." (1) `initialize_schema()` used
  invalid MySQL-only inline `INDEX name (cols)` syntax inside `CREATE
  TABLE`, and bundled the table creation with a separate `CREATE INDEX`
  statement into one `sqlx::query()` call, which `sqlx` rejects outright
  ("cannot insert multiple commands into a prepared statement") -- every
  connection attempt failed at schema init, for every caller. Fixed by
  splitting into valid, separate `CREATE TABLE`/`CREATE INDEX` statements,
  and making the PostGIS-dependent geospatial GIST index genuinely
  optional (warns and continues if the `postgis` extension isn't
  available, rather than failing the whole backend). (2) The `value_json`
  column is `JSONB`, but the insert bound it as a plain string with no
  cast -- every insert failed with a type error. Fixed by casting to
  `::jsonb` in the `INSERT` statement. (3) `query_spatial_temporal`'s
  `limit: usize` parameter was bound as `limit as i64` with no bounds
  check; a large limit (e.g. the natural `usize::MAX` "no limit" value)
  silently reinterpreted as a negative `i64`, and Postgres rejected the
  query ("LIMIT must not be negative"). Fixed by clamping to
  `limit.min(i64::MAX as usize)` before the cast. (4) The `id` column is
  `UUID`, but was decoded as `String` -- `sqlx::Row::get` panics (not just
  errors) on a decode-type mismatch, so every read of an existing row
  crashed the process (a Rust panic across the PyO3 FFI boundary, from
  Python's perspective). Fixed by decoding as `uuid::Uuid` and converting
  to a string. (5) Likewise `value_json` (`JSONB`) was decoded as `String`
  instead of being cast to text in the `SELECT`, also panicking on every
  read. Fixed by selecting `value_json::text AS value_json`. (6)
  `confidence` (`FLOAT`, i.e. Postgres `FLOAT8`/`f64`) was decoded as
  `f32`, also panicking on every read. Fixed by decoding as `f64` and
  narrowing to `f32` after the fact. Verified via a full push -> restart
  (fresh instance) -> `restore_from_backend()` round trip against a real
  Postgres container, confirming `robot_id`, `location`, `sensor_type`,
  `value` (JSON), `confidence`, and `timestamp` all survive correctly.
- **A real correctness bug in streaming throughput statistics**
  (`advanced::streaming::StreamingPipeline::calculate_throughput()`,
  `src/advanced/streaming.rs`), found as a flake
  (`test_streaming_statistics`) while re-running the full suite for the
  Postgres work above. The function returned a hardcoded `0.0` whenever
  `current_batch.latency_us() == 0` -- but on fast/optimized `--release`
  hardware, processing a small batch can genuinely complete in under 1
  microsecond of wall-clock time, and that guard silently misreported
  near-instantaneous processing as "zero throughput" instead of "very high
  throughput." Fixed by flooring the latency at 1 microsecond (which also
  guards against negative values from clock skew) instead of
  special-casing it to a false zero. This test had previously been
  documented in `ROADMAP_HONEST.md` as "genuinely flaky, not fixed"; it is
  now fixed and stable (verified across 5 isolated reruns).
- **8 of the 9 deterministic known-failing Rust unit tests, individually
  root-caused and fixed** (927 passed / 1 failed / 1 ignored, up from
  918-919 passed / 9-10 failed): 4 were real production logic bugs
  (`pyroboframes_adapter`'s outlier `sync_confidence` constant,
  `pyrobovision_adapter`'s model-selection tiering that let a generic
  fallback model outrank a model built for the exact requested conditions,
  `quality_gates`'s anomaly window including the tested point in its own
  baseline plus a std_dev skip-guard that silently ignored spikes off a
  flat baseline, and `fleet::learning`'s confidence formula that crashed
  from 0.5 to ~0.14 on the very first real corroborating observation); 4
  were bad test data/fixtures where the real production code was already
  correct (`fleet::consensus`'s z-tolerance test data, `fleet_learning`'s
  geographic-distance test data, `semantic`'s test splat never actually
  setting `terrain_type`, and `loop_closure`'s untrained test vocabulary).
  The 1 remaining failure (`gaussian_frontier_integration`) was
  re-diagnosed precisely: the scoring formula and weights are real, but
  one input (`terrain_difficulty`) is hardcoded to `0.0` pending real
  splat-traversability wiring (already honestly commented in the source) —
  left failing rather than loosening the test assertion, since that would
  hide the real, still-open gap. Full root-cause detail for all 9 in
  `ROADMAP_HONEST.md`.
- Stale `License: Proprietary` metadata left over from before the Apache-2.0
  relicense: `python/pyterrain_map/__init__.py`'s `__license__` string
  (was actually shipped in package metadata), plus doc references in
  `CLAUDE.md`, `docs/PRODUCT_VISION.md`, `docs/PYTHON_BINDINGS.md`,
  `docs/ROS_BRIDGE_DELIVERY.md`, `docs/ARCHITECTURE_BOUNDARIES.md`,
  `docs/index.rst`.
- `Cargo.lock` was gitignored and had never been committed in this repo's
  history -- removed from `.gitignore` and committed, since this crate
  builds a distributable extension/application artifact via maturin, not a
  library meant to float its dependency versions for downstream consumers.
- Repo URL casing (`Mullassery/pyterrain-map` -> `Mullassery/PyTerrainMap`)
  across `Cargo.toml`, `pyproject.toml`, `CHANGELOG.md`, and several `docs/`
  files.
- `pyproject.toml` trove classifiers were missing Python 3.13, even though
  CI has tested it since it was added to the test matrix.
- CI: replaced several actions that GitHub now refuses to run (archived
  `actions-rs/toolchain@v1`; `actions/setup-python@v4`;
  `codecov/codecov-action@v3`; `actions/upload-artifact@v3` /
  `download-artifact@v3` in `publish.yml`; `actions/create-release@v1`) --
  confirmed via `actionlint`, which reported all of these as
  no-longer-runnable before the fix and clean after. `publish.yml` was
  effectively broken (would fail on any real run) until this fix.
- `docs/ARCHITECTURE.md` described a fictional "Layer 3: Optional PyNoramic
  (Image stitching, SfM)" in its top-of-file architecture diagram --
  contradicting `docs/VISION.md`'s own "Honest status" section, which
  already explicitly disclaims PyNoramic as never having existed as a
  repository. Removed the layer from the diagram and added a note pointing
  to `VISION.md` and `ROADMAP_HONEST.md` for the real status of
  photogrammetry/SfM code that does exist (`src/photogrammetry/`,
  `src/reconstruction_3d/`, both explicit placeholders, not a working
  solver).
- **`cargo test --workspace` could not run at all on macOS** (crashed at
  runtime with a `dyld` symbol-resolution `SIGABRT`,
  `_PyBaseObject_Type not found in flat namespace`) because PyO3's
  `extension-module` feature was hardcoded as always-on in `Cargo.toml`,
  so even a standalone `cargo test` binary (never loaded by a real Python
  process) was built expecting Python C-API symbols to resolve via dlopen at
  import time, which never happens outside `maturin`/Python. Feature-gated
  `extension-module` behind a new `[features]` flag (same pattern already
  used in the org's `ClusterAudienceKit`), on by default so `cargo build`,
  `cargo bench`, `cargo clippy`, `maturin develop`, and `maturin build` are
  all unaffected. Run `cargo test --workspace --no-default-features
  --features database` to build a test binary that links normally against
  libpython instead. Verified: this now actually runs on macOS for the
  first time -- 918-919 passed, 9-10 failed, 1 ignored across repeated runs
  (previously 0 could even be collected). 7 failures were already
  known/documented (`adapters::pyroboframes_adapter`,
  `adapters::pyrobovision_adapter`, `exploration::gaussian_frontier_integration`,
  `gaussian_splatting::fleet_learning`, `gaussian_splatting::semantic`,
  `slam::loop_closure::test_loop_closure_detector`,
  `temporal::quality_gates::test_anomaly_detection_spike`); 3 are newly
  discovered by this fix, not previously documented anywhere -- 2
  deterministic (`fleet::consensus::tests::test_consensus_engine_majority`,
  `fleet::learning::tests::test_learned_pattern`) and 1 intermittent/timing-dependent
  (`advanced::streaming::tests::test_streaming_statistics`, root-caused to
  `calculate_throughput()` returning `0.0` when a batch completes in under
  1 microsecond on fast release-mode hardware) -- see `ROADMAP_HONEST.md`
  for the updated failing-test list and root causes. `maturin build
  --release` and `pytest tests/` (205/205) re-verified unaffected by this
  change.
- Dead TODO block in `src/lib.rs` (previously around line 256): commented-out
  `// TODO: Implement in future weeks` listing `storage`, `fusion`,
  `anomaly`, `query`, and PyO3 `python` bindings as not-yet-implemented --
  all five are already real, implemented modules declared earlier in the
  same file (`pub mod storage`, `pub mod fusion`, `pub mod anomaly`,
  `pub mod query`, `pub mod py`). Removed the stale comment block outright.

## [1.6.0]

### Added

- **Real persistence wiring for the live server** (`server.rs`,
  `storage::persistence_bridge`): `ServerState::with_backend()` takes any
  `StorageBackend` (e.g. `PostgresBackend`), `handle_submit` write-throughs
  every observation to it, and `ServerState::restore_from_backend()`
  repopulates the in-memory store on startup. A reboot with a configured
  backend no longer loses terrain history -- previously `PostgresBackend`
  was real and tested but never constructed anywhere outside its own file.
  Currently Rust-level only; Python API wiring is a follow-up.
- **H3 parent-cell rollup/compaction** (`storage::rollup::CompactionPolicy`):
  reads a time window of observations and produces real per-parent-cell,
  per-time-bucket, per-sensor-type summaries (count, mean confidence,
  timestamp range, contributing robots) -- genuine data reduction for
  bounding long-term storage growth. Additive only: `ObservationStore`'s
  no-delete/no-mutate guarantee is unchanged; this doesn't evict or archive
  anything yet, it computes the rollups a future archival flow would use.

## [1.5.0] - 2026-08-17

### Fixed

- **Immutability contradiction (correctness/trust fix)**: `ObservationStore`
  (Rust core) and `TerrainMap` (Python-facing class) both documented an
  append-only, immutable observation log, but each had a `clear()` method
  that could wipe the entire store through the normal API -- directly
  contradicting that guarantee (see `docs/DATA_INTEGRITY.md`). Neither
  method was used anywhere in this codebase or its tests. Both were removed
  outright rather than gated, since there was no legitimate production use
  case for erasing the audit trail. If you need an empty store, construct a
  new `TerrainMap()` / `ObservationStore::new()` instead.
- **`TerrainMap.query()` returned zero results for every query.** The
  longitude-delta calculation passed a latitude in *degrees* directly into
  `.cos()` (which expects radians), which for most latitudes produces a
  *negative* `lon_delta` and makes the region filter reject every
  observation unconditionally. Fixed by converting to radians first. Added
  regression tests (`tests/test_terrain_map_core.py`) covering a range of
  latitudes.
- **Python package couldn't be tested.** `python/pyterrain_map/__init__.py`
  only aliased a handful of classes (`TerrainMap`, `Observation`, ...); every
  other Rust-registered class (`PyGaussianSplatStore`, `PyUnifiedPathCost`,
  `PyFrontier`, `PyBotObservationMessage`, `PyTerrainAnalysis`, etc.) and
  every module-level function (`analyze_terrain`, `assess_mobility`,
  `detect_changes`, `fuse_observations`, `query_by_sensor`, `explain_field`,
  `is_accessible`) was unimportable from `pyterrain_map`. 9 of this
  project's own test files could not even be collected as a result. Now
  re-exported.
- **`cargo build`/`cargo test`/`cargo clippy` couldn't link on macOS** without
  going through `maturin`, because the crate had no `build.rs` calling
  `pyo3_build_config::add_extension_module_link_args()`. Added `build.rs` so
  plain `cargo test --workspace` etc. work directly.

### Added

- **Real HTTP/HTTPS API server**, actually bound to the previously
  type-only `src/api/mod.rs` / `src/api_tls/mod.rs` layer. New
  `src/server.rs` wires a real `hyper` server (backed by the real
  `ObservationStore` + spatial/temporal indices) to genuine TCP listeners,
  with TLS termination via `rustls` (self-signed dev certs via `rcgen`,
  or real cert/key files for production). Callable from Python via
  `pyterrain_map.start_server(host, port, tls=False, cert_path=None,
  key_path=None) -> ServerHandle`. Endpoints: `GET /health`, `GET
  /version`, `GET /stats`, `POST /observations`, `POST /query/spatial`.
  Proven end-to-end with real HTTP/HTTPS requests in both
  `tests/server_integration.rs` (Rust) and `tests/test_server.py`
  (Python) -- including a real rustls client that validates the
  self-signed certificate rather than bypassing validation.
- Wired in `src/py_query_functions.rs` and `src/temporal_anomaly.rs`
  (previously written but never added to `lib.rs`/the PyO3 module, so
  unreachable from Python and untested). Fixed two compile bugs found while
  wiring `temporal_anomaly.rs` in (`SensorType::Temperature`, which doesn't
  exist -- the real variant is `SensorType::Thermal`; and an `i64` value
  assigned into a `u32` struct field without a cast).
- `analyze_terrain`, `assess_mobility`, `detect_changes`,
  `fuse_observations`, `query_by_sensor`, `explain_field`, `is_accessible`,
  `TerrainAnalysis`, `Risk`, `MobilityAssessment`, `EnvironmentalConditions`,
  `DataExplanation` are now importable from `pyterrain_map` (previously
  documented in this changelog and `docs/` but not actually reachable).
- CI workflow (`.github/workflows/ci.yml`) and PyPI publish workflow
  (`.github/workflows/publish.yml`).
- Sphinx docs scaffolding (`docs/conf.py`, `docs/index.rst`,
  `docs/installation.rst`, `docs/quickstart.rst`).

### Known issues (pre-existing, not introduced by this release)

- 5 Rust unit tests fail, unrelated to the areas touched in this release:
  `adapters::pyroboframes_adapter::tests::test_temporal_metadata_preservation`,
  `adapters::pyrobovision_adapter::tests::test_select_best_model_rocky_night`,
  `exploration::gaussian_frontier_integration::tests::test_score_frontier_with_high_uncertainty`,
  `gaussian_splatting::fleet_learning::tests::test_fleet_learning_objects_near`,
  `gaussian_splatting::semantic::tests::test_mission_terrain_cost_delivery`.
- 1 Python test fails:
  `tests/test_statguardian_integration.py::TestTemporalCoordinateContract::test_valid_coordinates`
  (a units mismatch between the temporal-gap threshold and the test's
  timestamp values in the optional StatGuardian integration shim).
- `cargo clippy --workspace -- -D warnings` reports ~200 pre-existing
  warnings across the codebase (mostly non_snake_case fields mirroring
  external JSON/3D-Tiles schemas, and a PyO3-macro-generated
  `useless_conversion` false positive on `PyResult<T>` return types).
  `cargo fmt --check` likewise reports formatting drift across most of the
  pre-existing codebase. Neither is introduced by this release; both are
  left as a follow-up cleanup pass rather than an unreviewed
  codebase-wide reformat/relint bundled into this change.
- The `persistence` module (`src/persistence/mod.rs`) documents
  SQLite/PostgreSQL/BigQuery backends but, like the API layer was before
  this release, is type/config definitions only -- no `sqlx`/`rusqlite`
  connection is ever opened. Not fixed in this release (deferred as an
  enterprise/storage-backend feature per this release's scope); the
  in-memory `ObservationStore` used by the real HTTP server above is the
  actual, working storage path today.

## [1.0.0] - 2024-07-26

### 🎉 Production Release

PyTerrainMap v1.0.0 is the first stable release featuring temporal-normalization enabled spatial intelligence for multi-robot terrain mapping.

### Added

#### Core Features
- **Multi-Robot Terrain Mapping**: Heterogeneous robot fleets (drones, wheeled, quadrupeds, humanoids) collaboratively build shared terrain understanding
- **Temporal Normalization**: Time as first-class coordinate with 5D temporal metadata (event_time, capture_time, transmission_time, ingestion_time, processing_time)
- **Late-Arrival Handling**: Automatic detection and reprocessing of out-of-order observations
- **Multi-Clock Synchronization**: 12 GNSS sources with regional preferences (NavIC/India, Galileo/Europe, BeiDou/China, GLONASS/Russia)

#### Python API
- `TerrainMap`: Main mapping engine with observation storage and querying
- `Observation`: Single sensor reading with spatial-temporal coordinates
- `TerrainAnalysis`: Comprehensive terrain intelligence with persona-specific recommendations
- `MobilityAssessment`: Robot suitability scoring with difficulty levels and battery impact
- `EnvironmentalConditions`: Weather and soil condition tracking
- `DataExplanation`: Field metadata and provenance for agent introspection
- 7 high-level query functions: `analyze_terrain()`, `assess_mobility()`, `detect_changes()`, `fuse_observations()`, `query_by_sensor()`, `explain_field()`, `is_accessible()`

#### High-Performance Features
- **H3 Hexagonal Indexing**: ~0.1km² hexagons at resolution 14 with elevation bucketing
- **Temporal Quality Weighting**: Confidence scaling based on latency and clock synchronization
- **Multi-Sensor Fusion**: Weighted consensus combining sensor confidence + temporal quality
- **Anomaly Detection**: 8-failure taxonomy (z-score, IQR, rogue bot, spike, drift, etc.)
- **Change Detection**: 3D model diffing with heatmaps and temporal trends

#### 3D Reconstruction
- **SLAM Integration**: Visual odometry + IMU fusion with pose graph optimization
- **Photogrammetry**: Structure-from-Motion, NeRF, and Gaussian Splatting support
- **3D Tiles Export**: Hierarchical point cloud streaming for web visualization
- **Cesium.js Integration**: Web-based multi-layer visualization

#### Privacy & Security
- **RBAC**: 5-role access control (Admin, Analyst, Robot, Public, Restricted)
- **Coordinate Degradation**: 2-4 decimal place privacy levels
- **Robot Anonymization**: Identity masking for fleet privacy
- **Audit Logging**: Tamper-evident operation trails
- **TLS/mTLS Support**: Encrypted communication with certificate-based authentication

#### Storage & Persistence
- **Append-Only Storage**: Immutable observation history for audit trail
- **Multi-Backend Support**: SQLite, PostgreSQL, DuckDB with query federation
- **Archival Policies**: Automatic retention and purging based on time/volume
- **Data Tiering**: Hot (active), warm (queryable), cold (archived) tiers

#### External Data Integration
- 10+ data sources: OSM, SRTM, Sentinel-2, Landsat, SoilGrids, NOAA, etc.
- Reference image georeferencing via visual descriptors
- Regional GNSS preference weighting
- Backup data source fallback chains

#### Developer Experience
- **Type Hints**: Complete .pyi stubs for IDE autocomplete
- **Documentation**: 500+ lines of docstrings on all classes/methods
- **CLI**: Natural language parser supporting 14+ command patterns
- **Examples**: Quick-start guide with multi-robot scenarios
- **Testing**: 28+ inline unit tests covering temporal ordering, late arrivals, clock sync

### Technical Details

#### Architecture
- **Language**: Rust core (performance) + Python bindings (usability)
- **Build**: PyO3 0.22 with abi3-py310 for forward compatibility
- **Wheel**: Single wheel supports Python 3.10-3.13
- **Dependencies**: All OSS (MIT/Apache 2.0/BSD)

#### Performance
- Observation analysis: <5ms per location
- Temporal reprocessing: <2ms per observation
- Multi-sensor fusion: <1ms per fuse operation
- Memory usage: <500MB for 100k observations

#### Testing
- Unit tests for temporal ordering, late arrivals, multi-clock sync
- Integration tests for multi-robot scenarios
- Performance benchmarks with real datasets
- Code coverage: >85%

### Changed

- **Version Numbering**: Semantic versioning (major.minor.patch)
- **Development Status**: Alpha → Beta (production-ready)
- **Classifiers**: Updated PyPI metadata for broader discoverability

### Fixed

- Clock synchronization with >1s latency observations
- Late-arrival confidence penalty calculation
- Temporal quality weighting in anomaly detection
- Multi-region cache invalidation for latency-dependent changes

### Deprecated

- Legacy SimpleTimeIndex (use TemporalIndexEnhanced)
- Synchronous HTTP API (async version preferred)

### Removed

- PyROS compatibility layer (use adapters instead)
- Legacy JSON configuration format (use YAML)
- Deprecated sensor type mappings

### Security

- Added mTLS support for secure robot communication
- Implemented coordinate degradation for privacy
- Audit logging for all data access
- Input validation on all API boundaries

### Known Limitations

1. **Geographic Coverage**: GNSS preferences optimized for 50+ countries; fallback to GPS elsewhere
2. **Temporal Granularity**: Minimum 1 microsecond observation precision
3. **Scalability**: Tested up to 100k observations in memory; use database backend for larger datasets
4. **Real-Time Constraints**: Recommended max 1000 observations/second ingestion rate
5. **3D Reconstruction**: Requires minimum 50 images for Structure-from-Motion

### Migration Guide

For users upgrading from alpha versions:

- **TemporalIndex**: Use `TemporalIndexEnhanced` for event_time ordering
- **Temporal Metadata**: All observations now require 5D temporal fields
- **Clock Sources**: Explicitly specify clock source (GPS, NavIC, etc.)
- **Configuration**: Migrate JSON configs to YAML format

### Roadmap

**Upcoming (v1.1.0)**
- Real-time change detection with streaming support
- Advanced terrain traversability learning from robot logs
- Web-based visualization dashboard
- GraphQL query API

**Future (v2.0.0)**
- Distributed map federation across multiple sites
- Machine learning terrain prediction
- Autonomous exploration guidance
- Integration with ROS 2 middleware

### Contributors

- **Georgi Mammen Mullassery** - Lead architect and developer

### Thank You

Special thanks to:
- PyO3 team for excellent Python-Rust interop
- H3 community for geospatial indexing
- All contributors and early testers

### Getting Started

```bash
pip install pyterrainMap
```

See [Quick Start Guide](https://github.com/Mullassery/PyTerrainMap#quick-start) for usage examples.

### Links

- **GitHub**: https://github.com/Mullassery/PyTerrainMap
- **Issues**: https://github.com/Mullassery/PyTerrainMap/issues
- **Documentation**: https://github.com/Mullassery/PyTerrainMap/blob/main/PYTHON_BINDINGS.md
- **PyPI**: https://pypi.org/project/pyterrainMap/

---

**License**: MIT © 2024 Georgi Mammen Mullassery
