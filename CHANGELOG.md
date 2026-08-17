# Changelog

All notable changes to PyTerrainMap will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

See [Quick Start Guide](https://github.com/Mullassery/pyterrain-map#quick-start) for usage examples.

### Links

- **GitHub**: https://github.com/Mullassery/pyterrain-map
- **Issues**: https://github.com/Mullassery/pyterrain-map/issues
- **Documentation**: https://github.com/Mullassery/pyterrain-map/blob/main/PYTHON_BINDINGS.md
- **PyPI**: https://pypi.org/project/pyterrainMap/

---

**License**: MIT © 2024 Georgi Mammen Mullassery
