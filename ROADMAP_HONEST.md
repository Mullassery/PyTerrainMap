# PyTerrainMap: Honest Roadmap and Technical Debt

Last verified: 2026-09-22, against a clean local checkout of `main`
(`081c6ee`). Every number below was re-run in this pass; where the
sandbox couldn't run something (network-gated), that's stated explicitly
rather than repeating an old number as if it were re-verified today.
This pass had network access, so `cargo audit` was re-run for real (see
below) instead of citing the 2026-09-13 result second-hand.

This document exists so nobody has to reverse-engineer project status from
commit messages. See also the README's "Features" table (day-to-day source
of truth for what's implemented) and `docs/KNOWN_ISSUES.md` (build/platform
issues). This file focuses on: what's genuinely missing, what's broken, and
concrete technical debt with file:line references.

---

## 1. Build/lint/test health (re-verified 2026-09-20)

| Check | Command | Result |
|---|---|---|
| `cargo fmt --check` | `cargo fmt --check` | **FAILS.** 814 diff hunks across 109 of 123 `.rs` files under `src/`. **Not run in CI at all** -- `.github/workflows/ci.yml` has no `cargo fmt --check` step in any job, so this drift is invisible to CI and has apparently been accumulating unnoticed. This is a *new* finding this pass (not previously documented anywhere in this repo). |
| `cargo clippy --release -- -D warnings` | same | **FAILS: 232 errors** (`grep -c '^error:'` on the run output, minus the summary line), re-verified 2026-09-22 after the pyo3 feature-gating fix below (default-feature build, so equivalent to prior runs). Was 231 as of 2026-09-20 (227 before that) -- another 1-error drift, not the same number, direction consistent with slow accumulation rather than anything this pass introduced. Overwhelmingly `non_snake_case` on fields that mirror the real Cesium 3D Tiles JSON field names (`src/tiles_3d/mod.rs`, e.g. `QUANTIZED_VOLUME_OFFSET` line 238, `QUANTIZED_VOLUME_SCALE` line 240) -- these are spec-mandated names, not naming mistakes; the correct fix is `#[allow(non_snake_case)]` per struct, not a rename. A few are real: e.g. `src/caching/mod.rs:344` `if start_layer <= 0 && end_layer >= 0` is flagged `unused_comparisons` (likely `<= 0`/`>= 0` on an unsigned type -- worth a look, it's a logic smell not just a style lint), and `src/adapters/pyrobovision_adapter.rs:106` (`let mAP = ...`) is a real non-snake-case variable, not a spec-mirroring field. CI runs this with `continue-on-error: true`, so it doesn't block merges. **Still not fixed** -- 232 individual annotations/renames is a large, mechanical change that deserves its own reviewed PR, not a drive-by inside a quick-fix pass. |
| `cargo test --workspace --release` | plain invocation, then feature-gated | **FIXED in this pass.** Root cause confirmed: `pyo3`'s `extension-module` feature was hardcoded always-on in `Cargo.toml`'s `[dependencies]`, so even a standalone `cargo test` binary (never loaded by a real Python process) was built expecting Python C-API symbols to resolve via `dlopen` at import time -- which never happens outside `maturin`/an embedding Python process, hence the `dyld` `SIGABRT` (`_PyBaseObject_Type not found in flat namespace`) previously documented here. Fixed the same way the org's `ClusterAudienceKit` fixed the identical issue: moved `extension-module` out of the hardcoded `pyo3` features list and into a new Cargo `[features]` flag (`extension-module = ["pyo3/extension-module"]`), kept `default = ["database", "extension-module"]` so `cargo build`/`cargo bench`/`cargo clippy`/`maturin develop`/`maturin build` are all unaffected (re-verified: `cargo build --release` and `maturin build --release` both still succeed, `pytest tests/` still 205/205 against the maturin-built wheel). Run `cargo test --workspace --no-default-features --features database` (plus `PYO3_PYTHON=<a python>=3.10, e.g. python3.11>` if the system default `python3` resolves to something older than the `abi3-py310` floor) to get a real, standalone test binary that links normally against libpython. **Result: it now actually runs on macOS for the first time** -- 918-919 passed, 9-10 failed (one intermittent), 1 ignored, across repeated runs. 7 of the failures match the previously-documented known-failing list (`adapters::pyroboframes_adapter::tests::test_temporal_metadata_preservation`, `adapters::pyrobovision_adapter::tests::test_select_best_model_rocky_night`, `exploration::gaussian_frontier_integration::tests::test_score_frontier_with_high_uncertainty`, `gaussian_splatting::fleet_learning::tests::test_fleet_learning_objects_near`, `gaussian_splatting::semantic::tests::test_mission_terrain_cost_delivery`, `slam::loop_closure::tests::test_loop_closure_detector`, `temporal::quality_gates::tests::test_anomaly_detection_spike`). **3 are newly discovered by this fix, not previously documented anywhere** (2 deterministic, 1 intermittent) -- see the full list and root causes below; these simply couldn't be seen before because the suite never got past the runtime crash. Not triaged/fixed in this pass (out of scope -- this was a build/tooling fix, not a test-content audit); flagged as a follow-up. |
| `cargo audit` | `cargo audit` | **Ran successfully this pass** (network was available) -- **0 vulnerabilities**, 1 pre-existing advisory warning (`rustls-pemfile` 2.2.0 flagged "unmaintained", `RUSTSEC-2025-0134`; already documented elsewhere in this file and in `docs/KNOWN_ISSUES.md`, no fix available upstream). Exit code 0. This supersedes the 2026-09-20 pass's "could not run, no network" note and the 2026-09-13 last-known-good figure -- this is a live, re-confirmed result as of 2026-09-22. |
| `black --check python/` | same | **Passes** -- re-run in this pass in a fresh Python 3.11 venv (25 files, "would be left unchanged"). |
| `ruff check python/` | same | **Passes** -- re-run in this pass, "All checks passed!". |
| `pytest tests/` | `pytest tests/ -q --no-cov` | **Passes: 205/205**, 1.35s -- re-run for real in this pass against a `maturin build --release` wheel built from the post-fix `Cargo.toml` (Python 3.11 venv, `pip install maturin pytest pytest-cov pytest-asyncio numpy opencv-python`, then the built wheel force-installed). Confirms the pyo3 feature-gating fix above didn't regress the actual Python-facing extension module. |
| `actionlint .github/workflows/*.yml` | `actionlint` | **Already fixed as of commit `081c6ee`** (the prior pass) -- re-verified clean in this pass too, exit 0, no findings on either `ci.yml` or `publish.yml`. Both files already use current, non-deprecated actions (`actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `actions/setup-python@v5`, `actions/upload-artifact@v4`/`download-artifact@v4`, `codecov/codecov-action@v4`, `pypa/gh-action-pypi-publish@release/v1`, `softprops/action-gh-release@v2`). No changes needed this pass. |

### Known-failing Rust unit tests

**Updated 2026-09-22: now independently re-run and reconciled on macOS**,
which was not previously possible (see the pyo3 `extension-module`
feature-gating fix above). Authoritative result from
`cargo test --workspace --no-default-features --features database --release`:
typically **918-919 passed, 9-10 failed, 1 ignored** (one of the failures,
`advanced::streaming::tests::test_streaming_statistics`, is intermittent --
see item 10 below). The failures seen across repeated runs:

1. `adapters::pyroboframes_adapter::tests::test_temporal_metadata_preservation` -- previously known.
2. `adapters::pyrobovision_adapter::tests::test_select_best_model_rocky_night` -- previously known.
3. `exploration::gaussian_frontier_integration::tests::test_score_frontier_with_high_uncertainty` -- previously known.
4. `gaussian_splatting::fleet_learning::tests::test_fleet_learning_objects_near` -- previously known.
5. `gaussian_splatting::semantic::tests::test_mission_terrain_cost_delivery` -- previously known.
6. `slam::loop_closure::tests::test_loop_closure_detector` -- previously known (`test_loop_closure_detector` in README/VISION.md).
7. `temporal::quality_gates::tests::test_anomaly_detection_spike` -- previously known (README/VISION.md).
8. `fleet::consensus::tests::test_consensus_engine_majority` (`src/fleet/consensus.rs:397`, `assertion failed: consensus.is_some()`) -- **newly discovered this pass**, not previously documented anywhere, reproduces deterministically across repeated runs.
9. `fleet::learning::tests::test_learned_pattern` (`src/fleet/learning.rs:268`, `assertion failed: pattern.confidence > 0.5`) -- **newly discovered this pass**, not previously documented anywhere, reproduces deterministically across repeated runs.
10. `advanced::streaming::tests::test_streaming_statistics` (`src/advanced/streaming.rs:315`, `assert!(stats.throughput_points_per_sec > 0.0)`) -- **newly discovered this pass, and genuinely flaky/timing-dependent**, not previously documented anywhere. Root cause identified: `StreamingPipeline::calculate_throughput()` (`src/advanced/streaming.rs:232-237`) returns a hardcoded `0.0` whenever `self.current_batch.latency_us() == 0` -- on fast/optimized `--release` hardware, adding one point and immediately reading statistics back can complete in under 1 microsecond, hitting that guard and failing the `> 0.0` assertion. Reproduced 0/1 times in an initial multi-threaded run, then 1/1 times in two subsequent single-threaded reruns on the same machine -- timing-sensitive, not deterministic either way. Not fixed in this pass (would mean changing production throughput-calculation behavior, not just test/doc hygiene -- out of scope for a quick-fix pass); flagged for whoever next touches `src/advanced/streaming.rs`.

Not triaged or fixed in this pass beyond identifying root causes -- this was
primarily a tooling/build fix (making `cargo test` runnable on macOS at all),
not a full test-content audit. **Action item for a follow-up session**:
root-cause and fix (or intentionally skip/mark) the two newly-discovered
deterministic `fleet::` failures, decide whether `test_streaming_statistics`
should use a coarser clock/epsilon instead of a hard `> 0.0` assertion, and
reconcile this list against `docs/KNOWN_ISSUES.md`'s Linux/CI-sourced list
(they should now substantially match, since both are running the same
underlying suite, just on different platforms).

---

## 2. Not built / explicitly fake, by area (no hedging)

- **Terrain intelligence (`analyze_terrain`, `assess_mobility`)**: returns
  fixed/demo output regardless of input coordinates. `analyze_terrain()`
  does not query any real elevation/DEM source. Does not work for real
  terrain assessment. Tested only in the sense that its fixed-output shape
  is exercised by tests -- the *content* is not real analysis.
- **Photogrammetry** (`src/photogrammetry/mod.rs`, `src/reconstruction_3d/mod.rs`):
  bundle adjustment (`src/photogrammetry/mod.rs:347`), pose estimation
  (`src/reconstruction_3d/mod.rs:324`, `:792`), and point-cloud color
  estimation (`src/reconstruction_3d/mod.rs:833`) are explicit placeholders,
  not a real structure-from-motion solver. Camera pose initialization
  (`src/photogrammetry/mod.rs:288`) is also a placeholder. Untested beyond
  "the placeholder code runs and returns its fixed value."
- **Gaussian-splatting frontier/semantic scoring**: `strategic_value = 0.5`
  hardcoded (`src/gaussian_splatting/exploration.rs:47`), semantic terrain
  classification is a placeholder (`src/gaussian_splatting/semantic.rs:91`),
  and temporal decay (`apply_decay_to_store`) is a no-op
  (`src/gaussian_splatting/temporal.rs:49`). 3 related Rust unit tests
  currently fail on `main` per README/KNOWN_ISSUES.
  Does not work for real fleet-learning prioritization today.
- **Exploration path planning**: `graph_cost` is hardcoded to `0.0`
  (`src/exploration/path_planning.rs:64`, comment: "would query graph edges
  if available" -- it doesn't), and edge type is hardcoded to the string
  `"generic"` (`src/exploration/statistics.rs:92`). Not previously called
  out in README/KNOWN_ISSUES. Path planning that uses these will not
  reflect real graph-edge costs.
- **Shapefile export**: `src/export/mod.rs:234` is an explicit placeholder
  ("for future implementation with shapefile crate") -- not implemented at
  all, not partially implemented.
- **Neural-network prediction mode**: `src/analytics/prediction.rs:17`
  lists `NeuralNetwork` as a prediction-method variant with the comment
  "(placeholder)" -- no NN model exists behind it.
- **Persistent storage from Python**: Postgres persistence is real at the
  Rust level (`src/storage/postgres.rs`, wired via
  `ServerState::with_backend()`/`restore_from_backend()` in commit
  `64e119e`, 2026-08-24) but **not exposed through the Python API** --
  `pip install pyterrainMap` gives you in-memory-only behavior regardless of
  what the Rust core can do. SQLite and BigQuery backends are schema/config
  types only, with no live DB connection at either layer. The README's
  Features table previously called all three "not implemented," which was
  stale against the Postgres work; corrected in this pass.
- **`cargo audit`'s "0 vulnerabilities"** (2026-09-13): last verified with
  network access on that date, not re-verified today (see table above).

---

## 3. Documentation sprawl (flagged, not fixed this pass)

`docs/` contains 40+ files, several dated 2026-08-03 or earlier that read
as pre-implementation planning documents (week-by-week phase plans, "Status:
Production-Ready", "Tests: 120/120", multi-product "PyTerrainAI"/"PyNoramic"
ecosystem framing) that `docs/VISION.md` already explicitly disowns as
fictional ("neither PyTerrainAI nor PyNoramic exist as repositories").
Examples: `docs/PRODUCT_VISION.md`, `docs/READY_TO_BUILD.md`,
`docs/BUILD_PRIORITIES.md`, `docs/WEEK19_SUMMARY.md`,
`docs/SESSION_WEEK19_COMPLETE.md`, `docs/ARCHITECTURE_COMPLETE_WEEK19.md`,
`docs/TASKS.md`, and most of the `*_ARCHITECTURE.md`/`*_DESIGN.md` files
(bandwidth optimization, predictive caching, layered caching, production
storage, parallel inference, autonomous exploration, security auditability,
universal robot interoperability).

**`docs/ARCHITECTURE.md`'s PyNoramic contradiction: fixed 2026-09-22.** It
previously described a "Layer 3: Optional PyNoramic (Image stitching, SfM)"
in its top-of-file architecture diagram -- the exact fictional component
`docs/VISION.md` already disclaims elsewhere in the same `docs/` directory.
The diagram now shows the real 3-layer stack (Python API / Rust Core /
Persistent Storage) with a note pointing to `VISION.md`'s "Honest status"
section and to this file's photogrammetry/SfM entry for what does and
doesn't exist. The rest of that doc (data model, fusion algorithms,
concurrency model code samples) was not re-audited line-by-line in this
pass -- only the top-of-file contradiction was in scope. The broader
40+-file `docs/` sprawl below is still flagged, not fixed:

These were not individually re-verified
line-by-line in this pass (that would be its own multi-hour audit), but the
volume and dating pattern strongly suggest most describe unbuilt, aspirational
designs rather than the current codebase -- `docs/VISION.md` and the README
are the only docs in this tree that explicitly claim to be reconciled against
current `src/`. **Recommendation for a follow-up session**: either (a) move
the pre-implementation planning docs into a clearly labeled `docs/archive/`
or `docs/design-history/` directory with a one-line disclaimer banner, or
(b) audit each one and correct/delete. Leaving them in `docs/`'s main
namespace next to `VISION.md`/`KNOWN_ISSUES.md` risks a reader treating a
2026-08-03 aspirational design doc as current status.

---

## 4. Other technical debt (concrete, file:line)

- **363 `.unwrap()` calls** across `src/` (`grep -rn '\.unwrap()' src/ | wc -l`,
  includes test code -- not separated out in this pass). Any of these outside
  `#[cfg(test)]` blocks is a potential production panic on unexpected input.
  Not triaged file-by-file this pass; worth a dedicated audit for which ones
  sit on request-handling paths (`src/server.rs`, `src/api/mod.rs`) versus
  internal invariants that are genuinely infallible.
- ~~**`src/lib.rs:256`**: `// TODO: Implement in future weeks` -- vague,
  undated TODO with no tracking issue.~~ **Fixed 2026-09-22**: removed. All
  five modules the comment listed as not-yet-implemented (`storage`,
  `fusion`, `anomaly`, `query`, PyO3 `python` bindings) are already real,
  implemented modules declared earlier in the same file (`pub mod storage`,
  `pub mod fusion`, `pub mod anomaly`, `pub mod query`, `pub mod py`) -- the
  TODO was stale dead commentary, not an actual gap.
- **`src/api/mod.rs:144`**: `Ok(SensorValue::Camera { detections: vec![] }) // TODO: parse detections` --
  camera sensor detections are silently dropped/ignored rather than parsed.
- **`rustls-pemfile`**: flagged "unmaintained" by `cargo audit` regardless of
  version (upstream project archived) -- not a vulnerability, no fix
  available to adopt; documented already in `docs/KNOWN_ISSUES.md`.
- **`publish.yml`'s release-tagging logic**: `tag_name: v${{ github.run_id }}`
  produces a non-semver tag (a GitHub run ID, not a version) for the
  `workflow_dispatch` path. This predates this pass and wasn't restructured
  here (would change release semantics, out of scope for a docs/CI-hygiene
  pass) -- flagged for whoever next touches the release flow.
- **`codecov/codecov-action@v4`** (bumped from v3 in this pass) may require
  a `CODECOV_TOKEN` repository secret even for public repos, depending on
  Codecov's current policy -- `fail_ci_if_error: false` is already set so
  this won't block CI either way, but coverage upload may silently no-op
  without a token configured. Unverified in this sandbox (no network to
  Codecov).
- **`[tool.maturin]` in `pyproject.toml` has no `include = ["LICENSE"]`.**
  This is a recurring bug pattern elsewhere in this org (sdist omits
  LICENSE, PyPI rejects the upload with a 400) -- it is **not currently live**
  here because `publish.yml` builds with `maturin build --release --no-sdist`
  (no sdist is ever produced or uploaded, which the README already discloses
  as a limitation for a different reason: no source fallback on
  unsupported platforms). If sdist publishing is ever turned on for this
  repo, add `include = ["LICENSE"]` under `[tool.maturin]` *first* -- verified
  via direct inspection of `pyproject.toml`, not assumed from the pattern
  alone.

---

## 5. What's real and tested (for balance -- see README for full detail)

- Append-only H3-spatial + temporal-decay observation store: implemented,
  tested.
- Real HTTP/HTTPS API server (`start_server()`): implemented, tested
  end-to-end in both Rust and Python (a genuine TLS handshake among the
  Python tests).
- Traversability knowledge graph: implemented, tested.
- SLAM (loop closure, BoW) and 3D Tiles export: real implementations (one
  SLAM unit test currently failing, see above).
- Anomaly detection (z-score/IQR/rogue-bot/drift/spike): implemented (one
  unit test currently failing).
- H3 parent-cell rollup/compaction (`src/storage/rollup.rs`): real, not a
  stub -- reads a time window and produces genuine per-cell/per-bucket
  summaries. Not yet exposed to Python.
