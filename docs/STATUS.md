# Project status

**Version:** 1.10.2 (alpha) · **Updated:** 2026-10-19

## Working today

- Append-only `TerrainMap` observation store keyed by `(H3Cell, ElevationBucket)`
  with exponential/linear confidence decay.
- Spatial + temporal queries (`TerrainMap.query`).
- HTTP/HTTPS API server (`start_server`): `/health`, `/stats`,
  `/observations`, `/query/spatial`.
- PostgreSQL persistence: write-through and `restore_from_backend()`.
- Terrain intelligence: `analyze_terrain()` (real elevation API + measured
  slope) and `assess_mobility()` per robot type.
- `pytm` CLI: `setup`, `configure`, `version`, `query`. `pytm-setup`.
- Grid-map JSON export (`pyterrain_map.gridmap.write_grid_map`) as a custom
  interchange (not the ROS `grid_map` format).

## Partial / Rust-only

- SLAM (loop closure, bag-of-words), photogrammetry geometry, and Cesium 3D
  Tiles export are implemented in the Rust core but not exposed through Python.
- Gaussian-splatting mapping: fusion/storage work; frontier strategic-value
  scoring and semantic classification are partial; splat temporal decay is a
  no-op.
- SQLite and BigQuery are configuration/schema types without live connections.
- Known failing Rust test is documented in the README
  (`test_score_frontier_with_high_uncertainty`).

## Verified gaps

| Gap | Impact | Status |
|---|---|---|
| Prebuilt wheels are macOS arm64 only | Linux/Windows/Intel users must build from source | Open |
| No GitHub releases/tags | No release feed despite PyPI 1.10.2 | Open |
| PyPI license metadata is `None` | License scanners/trust signals | Open |
| Python API does not expose SLAM/photogrammetry | Capability is Rust-only | Planned |
| Not fully offline (`analyze_terrain`) | Requires network for elevation | By design |
| Not a hosted service | Self-hosted only | By design |

## Not a goal

- A full camera-to-map SLAM pipeline.
- ML-based object classification.
- A hosted/managed service.

See [Comparison](COMPARISON.md), [Installation](INSTALLATION.md),
[Data integrity](DATA_INTEGRITY.md), and
[`ROADMAP_HONEST.md`](../ROADMAP_HONEST.md).
