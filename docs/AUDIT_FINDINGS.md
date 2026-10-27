# Audit findings — 2026-10-26

This file records the current portfolio-audit findings for PyTerrainMap. It is
documentation only; no code or release artifacts were changed. The fuller
evidence report is kept in the portfolio workspace at
`.audit/reports/PyTerrainMap/audit.md`.

## What it is

A Rust spatial-intelligence platform for multi-robot terrain mapping: an
append-only H3-indexed observation store with temporal decay, a real HTTP/HTTPS
server, PostgreSQL persistence, terrain/mobility helpers, and a broad set of
robotics-adjacent modules (SLAM/photogrammetry bridge, traversability,
Gaussian splatting, Cesium export, fleet/exploration).

## Verified strengths

- Real append-only observation store, H3 + elevation-bucket spatial index, and
  temporal decay.
- Genuine Hyper HTTP/HTTPS server, not a stub.
- Real PostgreSQL write-through and `restore_from_backend()`.
- Allowlisted `run_op()`/`ops()` JSON bridge for photogrammetry, SLAM,
  Cesium 3D Tiles, and local SRTM lookup.
- SLAM keyframe pose graph, multi-view photogrammetry, traversability graph,
  ROS-style adapters, and Cesium point-cloud export are implemented at varying
  depths.
- CI covers Python 3.10–3.13, cargo tests, clippy, and wheel builds.
- Clippy is clean; `cargo audit` was reported clean as of 2026-10-21.
- 23 Python test files; 248 static Python and 865 static Rust test definitions.

## Verified gaps and user-facing risks

- **Auth is optional and write-only:** without `PYTERRAINMAP_API_TOKEN`, writes
  are open; spatial queries, stats, and health are unauthenticated regardless.
- **No rate limiting or concurrency cap** was found in the server path.
- **Self-signed TLS is the default** unless real cert/key paths are supplied;
  accidental insecure deployment is easy.
- **Version drift:** README/Cargo/pyproject/KNOWN_ISSUES say 1.11.0;
  `docs/STATUS.md` says 1.10.2.
- **Test/failure-list drift:** README cites 845 passed / 2 ignored; internal
  roadmap cites 927/1/1 at an earlier revision; KNOWN_ISSUES lists at least 7
  failures and is stale.
- **Terrain-intelligence maturity conflict:** KNOWN_ISSUES says fixed/demo
  output, while README/STATUS say real elevation API + measured slope.
- **`cargo fmt --check` is not a CI gate;** roadmap documented 814 diff hunks.
- **Partial modules are presented broadly:** Gaussian frontier/semantic scoring,
  path planning, shapefile export, and NN prediction are partial/placeholder.
- **SQLite and BigQuery are schema-only;** only PostgreSQL is live.
- **No durable retry/outbox for Postgres writes.**
- **SyncFleet materialization** into a typed table is incomplete.
- **PyPI wheel is macOS arm64 only;** Linux/Windows are CI artifacts.
- **No multi-robot field test or data-integrity benchmark.**

## Direct positioning

Against Open3D, PDAL, OctoMap, and RTAB-Map. The defensible niche is shared,
append-only H3 terrain memory with temporal confidence and a real HTTP/Postgres
path.

## Ranked next actions

1. P0: require auth for writes on non-loopback binds, or bind loopback-only by
   default and refuse accidental exposure.
2. P0: add rate limiting and a concurrency cap.
3. P0: reconcile the test suite and publish one verified pass/fail command.
4. P1: unify version 1.11.0 and retire duplicate status pages.
5. P1: gate `cargo fmt` in CI and keep clippy clean.
6. P1: add Postgres retry/outbox handling and a restore runbook.
7. P1: clearly label SQLite/BigQuery and partial Gaussian/path-planning modules
   as unsupported/experimental.
8. P2: publish Linux/Windows wheels or scope support to macOS arm64.
9. P2: complete SyncFleet materialization and add a field test.

## Do not build

- A full camera-to-map SLAM stack, ML terrain classification, or a hosted
  service. Integrate existing SLAM stacks and stay the shared terrain-memory
  component.

## Evidence gaps / validation needed

- No multi-robot field test.
- No reproducible load/data-integrity benchmark.
- No server rate-limit/auth penetration test.
- No independent verification of wheel publishing.

## Recommended disposition

Maintain as a niche fleet-memory component with a real core. Do not present it
as a production multi-robot platform until auth, documentation truth, and test
health are fixed.
