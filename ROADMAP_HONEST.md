# PyTerrainMap: Honest Roadmap and Technical Debt

Last verified: 2026-09-20, against a clean local checkout of `main`
(`af55465`). Every number below was re-run in this pass; where the
sandbox couldn't run something (network-gated), that's stated explicitly
rather than repeating an old number as if it were re-verified today.

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
| `cargo clippy --release -- -D warnings` | same | **FAILS: 231 errors** (`grep -c '^error:'` on the run output, minus the summary line). Previously documented as "227" -- the real, current count is 231, a small drift upward since that note was written, not the same number. Overwhelmingly `non_snake_case` on fields that mirror the real Cesium 3D Tiles JSON field names (`src/tiles_3d/mod.rs`, e.g. `QUANTIZED_VOLUME_OFFSET` line 238, `QUANTIZED_VOLUME_SCALE` line 240) -- these are spec-mandated names, not naming mistakes; the correct fix is `#[allow(non_snake_case)]` per struct, not a rename. A few are real: e.g. `src/caching/mod.rs:344` `if start_layer <= 0 && end_layer >= 0` is flagged `unused_comparisons` (likely `<= 0`/`>= 0` on an unsigned type -- worth a look, it's a logic smell not just a style lint), and `src/adapters/pyrobovision_adapter.rs:106` (`let mAP = ...`) is a real non-snake-case variable, not a spec-mirroring field. CI runs this with `continue-on-error: true`, so it doesn't block merges. **Not fixed in this pass** -- 231 individual annotations/renames is a large, mechanical change that deserves its own reviewed PR, not a drive-by inside a docs pass. |
| `cargo test --workspace --release` | plain invocation | **Cannot be exercised locally on macOS at all, re-confirmed in this pass.** Plain `cargo test --workspace --release` fails to *link* with "symbol(s) not found for architecture arm64" (`_Py_IsInitialized`, `_Py_NoneStruct`, etc.). Retrying with `RUSTFLAGS="-C link-args=-undefined -C link-args=dynamic_lookup"` (the documented workaround) gets past linking (1m35s release build, 134 lib warnings + 153 test-target warnings from clippy-adjacent rustc lints) but the resulting test binary then **crashes at runtime** on launch: `dyld[...]: symbol not found in flat namespace '_PyBaseObject_Type'`, `SIGABRT`. So the RUSTFLAGS workaround is necessary but not sufficient on this platform -- no per-test pass/fail results could be obtained in this sandbox by any means. This matches the repo's own pre-existing explanation (PyO3 extension-module builds defer Python C-API symbol resolution to a real embedding Python process; a standalone test binary run outside Python has nothing to resolve those symbols against) and is consistent with CI only ever running this on Linux. The known-failing-test list below is therefore still sourced from `docs/KNOWN_ISSUES.md`/README, not independently re-run this pass. |
| `cargo audit` | `cargo audit` | **Could not run at all in this sandbox** -- `cargo-audit` is installed, but fetching the RustSec advisory database (`https://github.com/RustSec/advisory-db.git`) fails outright (no network egress in this environment: `error sending request for url ...`). The "0 vulnerabilities" figure from the 2026-09-13 pass (commit `af55465`) is **not re-verified here** -- it reflects that pass's real, network-connected run, not a claim re-checked today. Treat it as "last known good as of 2026-09-13," not "currently confirmed clean." Run `cargo audit` on a machine with network access before relying on it. |
| `black --check python/` | same | **Passes** -- re-run in this pass in a fresh Python 3.11 venv (25 files, "would be left unchanged"). |
| `ruff check python/` | same | **Passes** -- re-run in this pass, "All checks passed!". |
| `pytest tests/` | `pytest tests/ -q --no-cov` | **Passes: 205/205**, 8.93s -- re-run for real in this pass. Required building the extension first: `python3.11 -m venv`, `pip install maturin pytest pytest-cov pytest-asyncio numpy opencv-python`, then `RUSTFLAGS="-C link-args=-undefined -C link-args=dynamic_lookup" maturin develop --release` (the RUSTFLAGS *are* needed and sufficient for the real `maturin`-built cdylib extension module -- unlike bare `cargo test` above, this is the actual, supported way to run this project locally on macOS, and it works). Confirms the README's "Implemented, tested" claims for the observation store, HTTP/HTTPS server, and traversability graph are not just asserted -- they have a real, currently-passing automated suite behind them. |
| `actionlint .github/workflows/*.yml` | `actionlint` | **Failed before this pass** (10 findings: `actions-rs/toolchain@v1`, `actions/setup-python@v4`, `codecov/codecov-action@v3`, `actions/upload-artifact@v3`, `actions/download-artifact@v3` all flagged as actions GitHub will refuse to run). **Fixed in this pass** -- see CHANGELOG `[Unreleased]`. `actionlint` now exits clean. `publish.yml` specifically was broken enough (deprecated `upload-artifact@v3`/`download-artifact@v3`, archived `actions/create-release@v1`) that any real release-workflow run before this fix likely would have failed outright; this was never exercised because no release has been cut recently enough to hit it. |

### Known-failing Rust unit tests

`docs/KNOWN_ISSUES.md` already tracks this in more detail (two overlapping
but not identical lists from CI vs. the README) -- not re-litigated here.
Net: at least 7 distinct named failing tests as of the last time someone
actually ran and reconciled the full suite. This pass **could not** re-run
and reconcile them -- as established above, `cargo test` cannot be executed
at all on macOS in this sandbox (or apparently on any macOS machine, RUSTFLAGS
workaround included; the failure is a runtime `dyld` symbol-resolution crash,
not a link-time or environment-config problem). **Action item for a
follow-up session**: run `cargo test --workspace --release -- --nocapture`
end to end **on Linux** (matching what CI actually does) and produce one
authoritative failing-test list; macOS-native `cargo test` for this crate may
simply not be viable given the extension-module architecture, which is worth
confirming/documenting explicitly rather than re-discovering each time.

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
universal robot interoperability). **Confirmed directly, not just inferred
from dating**: `docs/ARCHITECTURE.md` (the file a stranger would open first
for "how is this built") still describes a "Layer 3: Optional PyNoramic
(Image stitching, SfM)" in its top-of-file architecture diagram -- the exact
fictional component `docs/VISION.md` already disclaims elsewhere in the same
`docs/` directory. This wasn't rewritten in this pass (fixing one sentence
here would still leave the rest of that 14KB doc unaudited, and per this
project's own past practice a `docs/architecture/README.md` rewrite deserves
its own dedicated pass, not a docs-hygiene-pass drive-by) -- flagged here so
it's not mistaken for already handled. These were not individually re-verified
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
- **`src/lib.rs:256`**: `// TODO: Implement in future weeks` -- vague,
  undated TODO with no tracking issue.
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
