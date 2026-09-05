# PyTerrainMap: Vision

## Honest status (read this first)

The previous version of this document described a **"PyTerrain ecosystem"** of
two separate products — PyTerrainMap (storage/mapping) and **PyTerrainAI**
(intelligence layer: anomaly detection, image stitching) — plus a sub-component
called **PyNoramic** (image registration, structure-from-motion). Checked
against reality: **neither PyTerrainAI nor PyNoramic exist as repositories**
(`gh repo view Mullassery/PyTerrainAI` and `Mullassery/PyNoramic` both fail to
resolve), and neither is mentioned anywhere in this repo's current README. This
was never a two-layer ecosystem; it's one repo, and everything it does, it does
itself. The document also repeatedly described the project as "MIT-licensed" —
licensing is handled by a separate process and isn't addressed here.

The README's own "What this actually is" section and its Features status table
are already a rigorous, current account of what's implemented, partially
implemented, or not implemented — that table is the primary source of truth
this rewrite builds on, cross-checked directly against `src/` and recent commits
where the table's dating was in question (see the Postgres note below).

## What's real and working today

- **Append-only observation storage**, H3-spatial-indexed and
  temporal-decay-weighted (`TerrainMap`, `Observation`) — implemented and
  tested (README Features table; `docs/DATA_INTEGRITY.md`).
- **A real HTTP/HTTPS API server** (`pyterrain_map.start_server(...)`) you can
  actually start and hit — `GET /health`, `GET /stats`, `POST /observations`,
  `POST /query/spatial` — tested end-to-end in both Rust and Python.
- **Traversability knowledge graph** — implemented and tested
  (`src/traversability/`).
- **SLAM** (loop closure, bag-of-words) and **3D Tiles export** are real
  implementations, not stubs (`src/slam/`, `src/tiles_3d/`) — though one SLAM
  unit test (`test_loop_closure_detector`) is currently failing on `main`.
- **Anomaly detection** (z-score, IQR, rogue-bot, drift, spike) with temporal
  quality weighting is implemented; one unit test
  (`test_anomaly_detection_spike`) is currently failing on `main`, the rest pass.
- **Postgres persistence is now real at the Rust level** — this update
  supersedes the README's Features table, which was written before commit
  `64e119e` ("wire Postgres persistence into live server + H3 rollup
  compaction," 2026-08-24) landed. `ServerState::with_backend()` /
  `restore_from_backend()` (`src/server.rs`, `src/storage/persistence_bridge.rs`)
  now write-through every observation to a configured `StorageBackend` (e.g.
  `PostgresBackend`, `src/storage/postgres.rs`) and repopulate in-memory state +
  indices on startup — a server configured with a backend no longer loses
  history on restart. **This is Rust-level only: the Python API's backend
  factory still isn't wired up**, per the same commit's own message, so
  `pip install pyterrainMap` users still get in-memory-only behavior today.
  SQLite/BigQuery backends remain schema/config types with no live connection.
- **H3 rollup compaction** (`src/storage/rollup.rs`, `CompactionPolicy`) is
  real — reads a time window of observations and produces real
  per-parent-H3-cell, per-time-bucket, per-sensor-type summaries. Also landed
  in `64e119e`, also not yet Python-exposed.

## What's partially built or scaffolding

- **Terrain intelligence helpers** (`analyze_terrain()`, `assess_mobility()`,
  `is_accessible()`) are fixed/demo logic, not real terrain analysis:
  `analyze_terrain()` returns the same risk score and "retrieved from SRTM"
  text regardless of the coordinates passed in — no elevation/DEM source is
  actually queried. `assess_mobility()` is a static lookup table keyed only on
  robot type. Useful for exercising the API shape; not for real terrain
  assessment.
- **Gaussian-splatting probabilistic mapping**: core fusion/storage works, but
  frontier "strategic value" scoring and semantic terrain classification are
  hardcoded placeholders, and splat temporal decay (`apply_decay_to_store`) is
  currently a no-op. Three related unit tests are failing on `main` (frontier
  scoring, fleet learning, semantic terrain cost).
- **Photogrammetry**: bundle adjustment, pose estimation, and point-cloud color
  estimation are explicitly-marked placeholders, not a real structure-from-motion
  solver, despite SLAM and 3D Tiles export (both real) living in the same
  "3D reconstruction" bucket.
- **Platform distribution**: the published PyPI wheel is macOS arm64 only
  (`cp310-abi3-macosx_11_0_arm64`), verified against every published release
  through 1.5.0. No Linux or x86_64 macOS wheel, and no sdist fallback — other
  platforms must build from source via `maturin develop --release`.
- **The "two-layer ecosystem" and image-stitching vision** (PyTerrainAI,
  PyNoramic) described in earlier versions of this document is not built and
  has no corresponding repository — see above. If pursued at all, it would be
  new work, not a layer that's "in progress."
- **Cross-repo integration claims from earlier internal docs** (`pyroboreplay`,
  StatGuardian, PyStreamMCP integrations previously described in this repo's
  `CLAUDE.md`) were checked via grep across all repos in both directions and
  found not implemented anywhere. `CLAUDE.md` and `pyroboreplay`'s README have
  already been corrected to mark this as aspirational, not shipped.

## Realistic near-term roadmap

The most recent real engineering here was driven by an external critique
review (`bb7474d`) that flagged two concrete gaps — no durable persistence, no
compaction — both of which were then actually implemented at the Rust level
within the same day (`64e119e`). That's the operating pattern worth continuing:
identify a specific, verifiable gap, close it, verify the close. On that basis:

1. **Wire the Python API's storage-backend factory** to the already-real
   `PostgresBackend`/`ServerState::with_backend()` path, so `pip install
   pyterrainMap` users get durable persistence, not just Rust-level consumers.
   This is explicitly called out as the immediate follow-up in `64e119e`'s own
   commit message.
2. **Fix the five currently-failing unit tests on `main`**
   (`test_anomaly_detection_spike`, the SLAM loop-closure test, and the three
   Gaussian-splatting tests for frontier scoring / fleet learning / semantic
   terrain cost) before adding new surface area in those modules.
3. **Either wire `analyze_terrain()`/`assess_mobility()` to a real elevation/DEM
   source, or relabel them clearly as demo/API-shape-only in every place they're
   documented** — right now the README already does this correctly; keep it
   that way as the functions evolve, rather than letting the fixed-output
   behavior quietly get presented as real terrain analysis elsewhere (as the
   old PyTerrainAI/PyNoramic framing did).
4. **Publish a Linux/x86_64 wheel or an sdist** so the project isn't
   effectively macOS-arm64-only for anyone trying `pip install`.

No multi-month phase timeline is given here — the prior version of this
document promised "Phase 0-1: MVP (Months 1-6)" style plans that have no
supporting evidence of that cadence in this repo's actual commit history. The
verifiable pattern is same-day critique-to-fix turnaround on scoped items, not
a fixed multi-phase schedule.

## Long-term direction (aspirational — not a status claim)

The original framing still describes a reasonable long-term bet, stripped of
the fictional second product:

**PyTerrainMap aims to be a self-hosted, shared spatial-knowledge layer for
heterogeneous robot fleets** — any robot, any sensor type, any autonomy stack,
querying and contributing to one map instead of each robot rebuilding terrain
understanding from scratch. The differentiators worth keeping from the earlier
pitch: map-centric rather than robot-centric (robots are simple, disposable
clients; the map is the durable asset), sensor-layer composition (support a
sensor type once, any robot with that sensor benefits) rather than per-robot
integration, and first-class temporal + 3D-spatial awareness (data decays,
elevation matters).

Whether real terrain intelligence (DEM-backed `analyze_terrain`), real
photogrammetry, an image-stitching/change-detection layer, or a second
"intelligence" product ever get built is future direction, not a commitment —
each would need the same real-code-and-test treatment the storage/server/SLAM
work already has, not a document describing it as already underway.
