# PyTerrainMap Build Priorities & Roadmap

**Last Updated:** 2026-07-20  
**Total Duration:** 26 weeks (MVP + Ecosystem Integration)  
**Status:** Ready to begin Phase 1

---

## Executive Summary

PyTerrainMap is a **multi-phase project** combining:
1. **Weeks 1-10:** Core spatial intelligence platform (MVP)
2. **Weeks 11-18:** Advanced features (storage, image stitching)
3. **Weeks 19-26:** Ecosystem integration (PyRoboFrames, PyRoboVision)

**Critical Path:** Phases 1-2 (weeks 1-10) must complete before Phase 5 can start. Phase 3-4 can run in parallel.

---

## P0 Priorities (Blocking Releases)

### MVP Release (Week 10 — Weeks 1-10)

| Week | Component | Deliverable | Owner | Status |
|------|-----------|-------------|-------|--------|
| 1-2 | PyTerrainMap Core | Types, structs, serialization | Rust | Not started |
| 2-3 | Spatial Indexing | H3 cells, elevation buckets, k-ring queries | Rust | Not started |
| 3-4 | Storage Engine | In-memory storage, RwLock concurrency | Rust | Not started |
| 4-5 | Query API | Spatial + temporal queries, batch operations | Rust | Not started |
| 5-6 | Python Bindings | PyO3 extension, async integration, maturin | Rust/Python | Not started |
| 5-8 | PyTerrainAI | RBAC, temporal decay, anomaly detection, HTTP API | Python | Not started |
| 7-10 | Integration & Testing | End-to-end scenarios, performance benchmarks, examples | Integration | Not started |

**Success Criteria:**
- ✅ Store observations at >1000/sec
- ✅ Query location+time in <50ms
- ✅ RBAC enforced per mission
- ✅ Temporal decay applied
- ✅ Anomaly detection working (z-score)
- ✅ HTTP API responding
- ✅ All tests passing
- ✅ 3 example scenarios working

**Git Milestone:** `v1.0.0-mvp` (Week 10)

---

### Phase 5 Release — Ecosystem Integration (Week 26 — Weeks 19-26)

| Week | Component | Deliverable | Dependency | Status |
|------|-----------|-------------|-----------|--------|
| 19-20 | PyRoboFrames Adapter | Sensor ingest, temporal alignment, composition tracking | PyTerrainMap v1.0.0 | Planned |
| 21-22 | PyRoboVision Adapter | Model registry lookup, terrain-aware weighting | PyTerrainMap v1.0.0 | Planned |
| 23-24 | Data Contracts | Observation schema, provenance tracking, lineage | Both adapters | Planned |
| 25-26 | Integration Tests | 3+ end-to-end scenarios, architecture docs | All above | Planned |

**Success Criteria:**
- ✅ Can ingest multi-robot MCAP streams
- ✅ Model registry lookups working (mAP by terrain)
- ✅ Fusion weights computed correctly
- ✅ Full provenance chain intact
- ✅ 3+ use cases validated (construction, agriculture, disaster)
- ✅ Integration tests passing
- ✅ Architecture docs complete

**Git Milestone:** `v2.0.0-ecosystem` (Week 26)

---

## P1 Priorities (After MVP, Parallel to Phase 5)

| Week | Component | Deliverable | Notes |
|------|-----------|-------------|-------|
| 11-12 | Persistent Storage | SQLite + PostgreSQL backends | Pluggable storage layer |
| 13-15 | Image Stitching | PyNoramic (Structure from Motion) | GeoTIFF output |
| 16-18 | Production Hardening | Logging, monitoring, deployment guides | Docker, K8s configs |

**Rationale:** These don't block MVP release. Can begin Week 11 while Phase 5 planning continues.

---

## P2 Priorities (Future, Post-Release)

- [ ] ML-based anomaly detection (isolation forests, autoencoders)
- [ ] Real-time model retraining from observations
- [ ] Federated terrain mapping (multi-site coordination)
- [ ] Historical terrain change detection
- [ ] Cost optimization (multi-cloud deployment)

---

## Critical Dependencies & Blockers

### Must Have Before Phase 1 Starts
- ✅ Rust 1.97+ (for edition2024)
- ✅ Cargo + maturin
- ✅ Python 3.10+
- ✅ GitHub repo with CI/CD

### Must Have Before Phase 5 Starts
- ✅ PyTerrainMap v1.0.0 published to PyPI
- ✅ All tests passing
- ✅ PyRoboFrames 1.2.1+ available
- ✅ PyRoboVision 1.2.1+ available
- ✅ Integration analysis complete (✅ DONE 2026-07-20)

---

## Build Order (Dependency Graph)

```
Phase 1: Core (Weeks 1-10)
  ├─ Types (weeks 1-2)
  ├─ Spatial Indexing (weeks 2-3)
  ├─ Storage (weeks 3-4)
  ├─ Query API (weeks 4-5)
  ├─ Python Bindings (weeks 5-6)
  └─ Integration & Testing (weeks 7-10)

Phase 2: PyTerrainAI (Weeks 5-8, parallel to Phase 1)
  ├─ RBAC (weeks 5-6)
  ├─ Temporal Decay (weeks 6-7)
  ├─ Anomaly Detection (weeks 7-8)
  └─ HTTP API (weeks 7-8)

Phase 3: Testing (Weeks 7-10, depends on Phase 1+2)
  ├─ End-to-End Tests (week 7)
  ├─ Performance Tests (weeks 8-9)
  └─ Documentation (week 10)

Phase 4: Advanced (Weeks 11-18, parallel to Phase 5 prep)
  ├─ Persistent Storage (weeks 11-12)
  ├─ Image Stitching (weeks 13-15)
  └─ Production Hardening (weeks 16-18)

Phase 5: Ecosystem (Weeks 19-26, depends on Phase 1 v1.0.0)
  ├─ PyRoboFrames Adapter (weeks 19-20) ← P0
  ├─ PyRoboVision Adapter (weeks 21-22) ← P0
  ├─ Data Contracts (weeks 23-24) ← P0
  └─ Integration Tests (weeks 25-26) ← P0
```

---

## Resource Allocation

### Weeks 1-10 (MVP)
- **Primary:** 1 Rust engineer (core + bindings)
- **Secondary:** 1 Python engineer (PyTerrainAI + API)
- **Support:** 1 engineer (testing + examples)
- **Total:** ~2.5 FTE

### Weeks 11-18 (Advanced)
- Can reduce to 1 FTE (less critical path)
- Storage engineer part-time
- Image processing engineer part-time

### Weeks 19-26 (Ecosystem)
- **Primary:** 2 integration engineers
- **Secondary:** 1 testing engineer
- **Total:** ~2.5 FTE

---

## Key Milestones & Git Tags

| Date | Milestone | Git Tag | Checklist |
|------|-----------|---------|-----------|
| Week 6 | Core Implementation | v0.1.0-alpha | Types, spatial, storage working |
| Week 10 | MVP Release | v1.0.0 | All P0 criteria met, tests passing |
| Week 18 | Advanced Features | v1.5.0 | Storage + images + hardening done |
| Week 26 | Ecosystem Ready | v2.0.0 | All integrations working, 3+ use cases |

---

## Testing Strategy

### Phase 1 Tests (MVP)
```
Unit Tests:
  - Spatial indexing (H3 operations)
  - Storage (insert, query, update)
  - Query API (multi-criteria)
  - Python bindings (async calling)

Integration Tests:
  - End-to-end storage + query + fusion
  - HTTP API (RBAC, decay, anomaly)
  - Performance benchmarks (<1ms insert, <50ms query)

Example Scenarios:
  - Police surveillance (stationary drones)
  - Factory inspection (multi-robot)
  - Agricultural monitoring (rover with multi-rate sensors)

Target:** 200+ tests passing
```

### Phase 5 Tests (Ecosystem)
```
Integration Tests:
  - PyRoboFrames MCAP ingest (3-robot scenario)
  - PyRoboVision model registry lookup
  - Multi-model ensemble fusion
  - Provenance tracing (end-to-end lineage)

Use Case Validation:
  - Construction Site Inspection (3 drones, 4K+thermal+LiDAR)
  - Agricultural Yield (multi-rate sensors, seasonal adaptation)
  - Disaster Response (3-model ensemble, multi-robot agreement)

Target:** 50+ integration tests passing
```

---

## Communication & Approval Gates

### Before Week 1 Starts
- [ ] Confirm Rust 1.97, Python 3.10+, cargo, maturin installed
- [ ] Create GitHub repo with CI/CD pipeline
- [ ] Approve 18-week timeline

### Before Phase 2 Kicks Off (End of Week 4)
- [ ] Core types compile without errors
- [ ] Spatial indexing working (k-ring queries passing)
- [ ] Approve PyTerrainAI RBAC design

### Before MVP Release (Week 10)
- [ ] All P0 tests passing (200+ tests)
- [ ] Performance benchmarks met (<1ms insert, <50ms query)
- [ ] Examples working
- [ ] Approve v1.0.0 release to PyPI

### Before Phase 5 Starts (Week 19)
- [ ] PyTerrainMap v1.0.0 published & stable
- [ ] PyRoboFrames 1.2.1+ available
- [ ] PyRoboVision 1.2.1+ available
- [ ] Approve ecosystem integration phase

### Before v2.0.0 Release (Week 26)
- [ ] All P0 integration tests passing
- [ ] 3+ use cases validated
- [ ] Architecture docs complete
- [ ] Approve v2.0.0 release to PyPI

---

## Success Metrics

| Metric | MVP Target | Post-Integration Target |
|--------|-----------|------------------------|
| Observations/sec | 1,000+ | 10,000+ |
| Query latency | <50ms | <50ms (with fusion) |
| Sensor streams ingested | Manual | 3+ concurrent MCAP streams |
| Vision models integrated | 0 | 3+ (RGB, thermal, LiDAR) |
| Tests passing | 200+ | 250+ |
| Example scenarios | 3 | 6+ (includes multi-robot) |
| Documentation pages | 5 | 12+ (ecosystem guides) |

---

## Known Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Rust H3 library issues | High | Early spike (week 1-2); fallback to postgis |
| PyO3 async complexity | Medium | Use pyo3-asyncio; test early (week 5) |
| Multi-rate sensor alignment | High | PyRoboFrames handles; verify schema match (week 19) |
| Model registry latency | Medium | Cache model perf; test with 100+ models (week 21) |
| Provenance tracking overhead | Low | Use string interning; benchmark (week 24) |

---

## First Steps (This Week)

1. **Create Cargo.toml** with h3, uuid, tokio, pyo3
2. **Write src/types.rs** (Observation, GeoPoint, SensorType)
3. **Write src/spatial/mod.rs** (H3 indexing)
4. **Get first test passing** (`cargo test`)
5. **Create GitHub CI/CD** (.github/workflows/rust.yml)

**Target:** By end of week 1, types should compile and spatial indexing should work.

---

## Questions & Escalations

- **Timing:** Is 18-week timeline realistic with current team? (Decide before week 1)
- **Scope:** Should Phase 4 (image stitching) be in MVP or deferred to v1.5? (P1 candidate)
- **Dependencies:** Are PyRoboFrames 1.2.1+ and PyRoboVision 1.2.1+ stable enough? (Check week 18)

---

**Status:** Ready to begin Phase 1  
**Last Updated:** 2026-07-20  
**Next Review:** End of Week 2 (timebox update, dependency check)
