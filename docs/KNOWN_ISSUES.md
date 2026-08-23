# PyTerrainMap - Known Issues

**Last Updated:** 2026-08-23  
**Version:** 1.5.0  
**Status:** Published to PyPI (macOS arm64 wheel only — see Platform Support below)

---

## Build Warnings (Non-Critical)

### PyO3 Linker Warning

**Severity:** 🟡 Warning (non-blocking)  
**Message:** Cannot link Python symbols from Anaconda Python 3.13  
**Status:** Environment-specific (not a code issue)  
**Affected:** Local development with Anaconda Python

#### Details

```
error: failed to build native library through cargo
Caused by: unable to link Python symbols from Anaconda Python 3.13
```

**Root Cause:** Maturin/PyO3 has issues linking system Python from Anaconda distribution on macOS ARM64.

**Workarounds:**

1. **Use System Python** (Recommended)
   ```bash
   # On macOS with Homebrew
   brew install python@3.10
   /opt/homebrew/bin/python3.10 -m pip install pyterrainmap
   ```

2. **Use CI/CD** (GitHub Actions)
   - Builds successfully on standard CI runners
   - Python 3.10+ from ubuntu-latest works fine

3. **Use Pre-built Wheel** (macOS arm64 only — see Platform Support below)
   ```bash
   pip install pyterrainMap
   # Downloads the pre-built macOS arm64 wheel; no compilation needed.
   # Other platforms fall through to a source build, which this
   # workaround doesn't avoid.
   ```

4. **Update Maturin** (May help)
   ```bash
   pip install --upgrade maturin
   ```

---

## Quality Validation

**Embedded Validation Contracts:** 4  
- Sensor Calibration (drift, age, accuracy, confidence)
- Multi-Sensor Consistency (agreement, sync, variance, outliers)
- Temporal Coordinates (bounds, ordering, gaps, quality)
- Terrain Mapping Anomalies (gradients, density, coherence)

**Validation Logging:** Enabled  
- Location: `terrain_validations/` JSONL files
- Audit Trail: Full lineage for every validation
- Compliance Scoring: 0-100% per operation

**Status:** ✅ Working correctly  
**No issues reported** with validation layer

---

## Known Limitations

### 1. Multi-Robot Consensus
- Requires clock synchronization < 100ms
- Handles up to 10 concurrent robots (tested)
- Untested: 100+ robot scenarios

### 2. Terrain Density
- Minimum 100 points/m² recommended
- Lower density may trigger anomaly warnings
- Higher density (>1000 pts/m²) may increase memory usage

### 3. Temporal Gaps
- Detects gaps > 60 seconds as warnings
- Larger gaps may indicate sensor dropout
- Consider sensor recalibration if frequent

### 4. Python Version Support
- Python 3.10+ required (`pyproject.toml`)
- CI (`test` job, ubuntu-latest) runs the full pytest suite on 3.10, 3.11,
  3.12, and 3.13 via a source build. `pyproject.toml`'s trove classifiers
  currently only list 3.10–3.12 — 3.13 is exercised by CI but not yet
  declared as a classifier.
- The published macOS wheel is built `abi3` for cp310+, so it should work
  on 3.13 at runtime too, but that's not independently verified against
  the actual wheel (only against source builds in CI).
- Untested: PyPy, other Python implementations

---

## Performance Notes

| Operation | Latency | Throughput | Notes |
|-----------|---------|-----------|-------|
| Add sensor data | <1ms | 10K readings/sec | Per reading |
| Finalize terrain | 50-500ms | 1 map/sec | Depends on point count |
| Query by region | <100ms | Instant | Spatial indexing |
| Validate consistency | <10ms | Fast | 3 sensor nominal |

---

## Dependency Issues

**Python Dependencies:** All stable  
**Rust Dependencies:** Checked via Cargo.lock  
**External:** None (SQLite optional)

**Status:** ✅ No known dependency conflicts

---

## Platform Support

**Corrected 2026-08-23** against the README's verified wheel audit and
`.github/workflows/ci.yml` (this table previously claimed macOS Intel and
Linux x86_64 were "fully tested," which was never true of the *published
package* — corrected below to distinguish "PyPI wheel available" from
"CI builds/tests it, but doesn't publish it"):

| Platform | PyPI wheel | CI coverage | Notes |
|----------|------------|-------------|-------|
| macOS ARM64 (Apple Silicon) | ✅ Yes | `build-wheels` (macos-latest) | Only wheel published on PyPI (`cp310-abi3-macosx_11_0_arm64`), verified against published files through v1.5.0 |
| Linux x86_64 | ❌ No wheel, no sdist | `build-wheels` (ubuntu-latest) builds a wheel; separate `test` job runs the full pytest suite via `maturin develop` on Python 3.10–3.13 | Build-from-source is exercised and passes in CI; PyPI install still fails without building locally |
| Windows | ❌ No wheel | `build-wheels` (windows-latest) builds a wheel | Build succeeds in CI, but no CI job runs the Python test suite on Windows — only the build step is verified |
| macOS Intel | ❌ No wheel | Not separately covered — `macos-latest` GitHub runners are Apple Silicon | Not verified either way |
| Docker/K8s | — | Not covered by any workflow | Would depend on the base image's platform above |

CI-built Linux/Windows wheels (`build-wheels` job) are uploaded only as
CI artifacts, not published to PyPI — `pip install` on those platforms
still requires a local source build (see Installation in the README).

---

## Testing Status

**Corrected 2026-08-23** — the "20+ passing" figure below was from the
1.0.0 release and had drifted badly from the current codebase.

**Unit Tests:** ~924 Rust (`grep -rc '#\[test\]' src/`), ~205 Python
(`grep -rc 'def test_' tests/`) as of this revision.

**Known failing tests** — two sources list overlapping but not identical
sets, and this pass didn't run the full suite to reconcile them:
- `.github/workflows/ci.yml`'s `cargo-check` job (`continue-on-error`)
  names 5: `adapters::pyroboframes_adapter`, `adapters::pyrobovision_adapter`,
  `exploration::gaussian_frontier_integration`,
  `gaussian_splatting::fleet_learning`, `gaussian_splatting::semantic`.
- The README's Features table separately names `test_anomaly_detection_spike`
  and `test_loop_closure_detector` as failing, plus the same frontier/
  fleet-learning/semantic-terrain-cost trio referenced above by module path.
- Net: at least 7 distinct known-failing Rust tests across both sources
  (the two adapter tests and the two individually-named tests don't
  appear in the other source's list). Treat both docs as partial views
  until someone runs `cargo test --workspace --release` and reconciles
  them into one list.

**Integration Tests:** Full validation layer tested  
**End-to-End:** Multi-robot scenarios tested (up to 10 robots)  
**Load Testing:** Tested with 50M+ observations  

**Status:** Core observation storage, spatial/temporal querying, and the
HTTP/HTTPS server are real and tested. Terrain intelligence
(`analyze_terrain`/`assess_mobility`) is fixed/demo output, not real
terrain analysis, and Gaussian-splatting/3D-reconstruction are partially
implemented — see the README's Features table before calling this
"production ready" for those areas.

---

## Reporting Issues

If you encounter issues:

1. **Check PyPI Installation First**
   ```bash
   pip install pyterrainMap --force-reinstall
   ```

2. **Python Version Check**
   ```bash
   python --version  # Should be 3.10+
   ```

3. **Validation Logs**
   ```bash
   ls -la terrain_validations/  # Check for validation records
   ```

4. **GitHub Issues**
   https://github.com/Mullassery/PyTerrainMap/issues

---

## Version History

Synced against `CHANGELOG.md`, which only documents these two releases —
the previously listed 0.2.0/0.1.0 entries don't exist in the changelog
and have been removed.

| Version | Status | Notes |
|---------|--------|-------|
| 1.5.0 | ✅ Current (2026-08-17) | Fixed the immutable-store `clear()` contradiction, a broken `query()` (degrees passed to `.cos()` instead of radians), and most Rust-registered classes/functions being unimportable from `pyterrain_map` |
| 1.0.0 | Previous (2024-07-26) | First stable release; quality validation embedded |

---

**Status:** Operational; linker warning is environment-specific. See
Testing Status above for current, non-aspirational feature-readiness.  
**Last Review:** 2026-08-23
