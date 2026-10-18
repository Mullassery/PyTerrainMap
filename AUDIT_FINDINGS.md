# Audit findings — 2026-10-19

This file records the repository-specific findings from the portfolio audit
conducted on 2026-10-19. It is documentation only; no code or release artifacts
were changed in this pass unless explicitly noted under "Implementation status".

Sources: the repository's own README/source/workflows, GitHub metadata and
Actions runs, PyPI metadata, and official competitor documentation. Claims are
marked as verified, inferred, or open questions. "Absent" means not found in the
inspected repository/docs, not proof that it cannot exist.

## What it is

Multi-robot terrain intelligence: append-only H3-indexed observations, temporal decay, HTTP/TLS API, Postgres persistence, slope/mobility analysis, and traversability.

## Verified strengths

- Append-only H3 store; temporal decay; HTTP/TLS; Postgres; slope/mobility; traversability graph; Rust SLAM/photogrammetry; Cesium 3D Tiles; 19 test files.
- Shared H3-indexed terrain memory across robots is a real niche.

## Verified gaps and user-facing risks

- SQLite/BigQuery are schema-only; photogrammetry is Rust-only; Gaussian frontier/semantic partial; one failing test.
- macOS arm64 wheel only; no fleet deployment; no hosted service.

## Direct competitors

- Open3D, PDAL, OctoMap, RTAB-Map.

## Ranked next actions

- P1: publish Linux/Windows wheels (S/M).
- P2: real SQLite/BigQuery backends (M).
- P3: expose photogrammetry/SLAM via Python (M).

## Do not build

- Full SLAM from scratch; integrate existing stacks.

## Evidence gaps / validation needed

- No multi-robot field test; no data-integrity benchmark.

## Recommended disposition

Maintain as a niche fleet-memory component; do not build SLAM.

## Implementation status

No code changes in this pass.
