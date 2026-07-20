# PyTerrainMap & Ecosystem - Pending Tasks

**Last Updated:** 2026-07-20 (Phase 7 Complete)
**Status:** SfM + SLAM Core Pipeline COMPLETE | Traversability Intelligence Next  
**Total P0 Tasks:** 8 | **P1 Tasks:** 12 | **P2 Tasks:** 15 | **P3 Tasks:** 8

---

## 🔴 P0: CRITICAL PATH (Ship Next)

### PyTerrainMap: Complete SfM + SLAM Pipeline ✅
- [x] **Phase 6.3: Loop Closure Detection** (COMPLETE)
  - Place recognition with feature-based descriptors ✅
  - Epipolar geometry verification ✅
  - Confidence-weighted loop closure matching ✅
  - 12 comprehensive tests ✅

- [x] **Phase 6.4: Pose Graph Refinement** (COMPLETE)
  - Global pose graph optimization ✅
  - Loop closure edge integration ✅
  - Information matrix weighting ✅
  - 15 tests (total 120 passing) ✅

- [x] **Phase 7: Real-Time SLAM Integration** (COMPLETE - v1.0.4)
  - Visual odometry tracker (frame-to-frame motion) ✅
  - IMU pre-integration (sensor fusion) ✅
  - Real-time pose graph with incremental optimization ✅
  - 17 SLAM tests ✅
  - **Published:** https://pypi.org/project/pyterrainMap/1.0.4/ ✅

### PyTerrainMap: Traversability Intelligence (Phase 8 Complete ✅)
- [x] **Phase 8: Spatial Knowledge Graph** (COMPLETE - 2026-07-20)
  - Node types (rooms, landmarks, regions, terrain cells) ✅
  - Edge types (doors, corridors, stairs, elevators, paths) ✅
  - Core metadata schema ✅
  - PostgreSQL schema design ✅
  - 34 tests (exceeds 15-20 target) ✅
  - **Enables:** Distance models, robot profiles, fleet learning

- [ ] **Phase B: Distance Models** (4-5 days)
  - Geometric layer (Euclidean, 2D, 3D, elevation)
  - Topological layer (navigable connections)
  - Multi-distance caching
  - Add 12-15 tests
  - **Depends on:** Phase A

---

## 🟡 P1: HIGH PRIORITY (Next 2 Weeks)

### PyTerrainMap: Traversability Intelligence
- [ ] **Phase C: Traversability Observations** (5-6 days)
  - Observation types (success, failure, difficulty)
  - Confidence scoring (0.0-1.0)
  - Historical records with timestamps
  - Query interface
  - Add 15-18 tests
  - **Depends on:** Phase B

- [ ] **Phase D: Connector Intelligence** (4-5 days)
  - Connector entity model (doors, corridors, bridges, elevators)
  - Physical dimensions (width, height, clearance, slope)
  - Accessibility scoring
  - State management (open/closed/blocked/inaccessible)
  - Add 12-15 tests
  - **Depends on:** Phase A, B

- [ ] **Phase E: Robot Profiles** (4-5 days)
  - Robot capability profiles (dimensions, weight, mobility type)
  - Constraint checking engine
  - Route filtering by robot type
  - Compatibility matrices
  - Add 12-15 tests
  - **Depends on:** Phase D

- [ ] **Phase F: Fleet Learning** (5-6 days)
  - Cross-robot observation sharing
  - Conflict resolution
  - Knowledge propagation
  - Consensus mechanisms
  - Add 15-18 tests
  - **Depends on:** Phase C, E

### TinyBridge: Phase 1 Week 4 (Final)
- [ ] **Complete Boot Tier Verification** (1-2 days)
  - Run full 4-tier boot sequence
  - Measure timing at each tier
  - Verify kernel optimizations
  - Stress test with concurrent VMs
  - **Status:** 95% complete

- [ ] **Phase 1 Week 4: Sign-Off** (1 day)
  - All 8 boot tier tests passing
  - Performance benchmarks documented
  - Ready for Phase 2 kickoff

### TinyBridge: Phase 2 (OTel Export)
- [ ] **Prometheus Backend Implementation** (3-4 days)
  - Metrics scrape endpoint
  - Boot timing metrics
  - CPU/memory profiling
  - Add integration tests

- [ ] **Jaeger Tracing Backend** (3-4 days)
  - Distributed trace export
  - Span generation for boot phases
  - Trace visualization support
  - Add integration tests

---

## 🟠 P2: MEDIUM PRIORITY (Weeks 3-4)

### PyTerrainMap: Traversability Intelligence (Continued)
- [ ] **Phase G: Dynamic Environments** (4-5 days)
  - Time-based state changes (doors opening/closing)
  - Temporal queries
  - Confidence decay over time
  - Environment versioning
  - Add 12-15 tests
  - **Depends on:** Phase C

- [ ] **Phase H: Route Planning Integration** (5-7 days)
  - Multi-objective pathfinding
  - Cost-aware routing (minimize: distance/time/risk)
  - Safety weighting
  - Traversability validation
  - Add 15-20 tests
  - **Depends on:** Phase D, E, F, G

- [ ] **Phase I: Query API** (3-4 days)
  - Traversability queries
  - Route planning API
  - Historical analytics queries
  - Add 10-12 tests
  - **Depends on:** Phase H

### PyTerrainMap: Phase 8 (3D Tiles Export)
- [ ] **3D Tiles Point Cloud Export** (4-5 days)
  - Convert point clouds to PNTS format
  - LOD hierarchy generation
  - Georeferencing
  - Add 12-15 tests

- [ ] **Cesium Integration** (3-4 days)
  - Web viewer configuration
  - Tileset metadata
  - Cesium Ion compatibility
  - Add 8-10 tests

### StatGuardian: v2.2 Lineage Extraction
- [ ] **Complete Lineage Extraction Layer** (5-6 days)
  - Data flow tracking
  - Transformation recording
  - Source attribution
  - Add 15-20 tests
  - **Status:** Week 1 complete, extraction layer pending

- [ ] **Quality Gate Validation** (3-4 days)
  - Contract validation
  - Lineage-aware validation
  - Drift detection with lineage context
  - Add 10-12 tests

---

## 🔵 P3: LOWER PRIORITY (Weeks 4-6)

### PyTerrainMap: Extended Features
- [ ] **Bundle Adjustment with Robust Estimators** (3-4 days)
  - M-estimators (Huber, Tukey)
  - Outlier downweighting
  - Convergence improvements
  - Add 8-10 tests

- [ ] **Incremental SfM Pipeline** (4-5 days)
  - Add frames one-at-a-time
  - Selective pose refinement
  - Memory-efficient processing
  - Add 12-15 tests

- [ ] **Camera Initialization** (3-4 days)
  - Multiple pose hypotheses
  - Cheirality checking
  - Best hypothesis selection
  - Add 8-10 tests

### PrismNote: v1.4 Release
- [ ] **Complete Browser Execution Tests** (3-4 days)
  - Keyboard navigation (149 test cases)
  - Tab switching (verified fixed in v1.3)
  - Code execution (SQL/Spark)
  - Add coverage for all 5 deployment targets

- [ ] **Production Deployment** (2-3 days)
  - Docker image build & test
  - AWS deployment validation
  - GCP deployment validation
  - Azure deployment validation
  - Kubernetes deployment validation

### OpenAnchor: v0.1→v0.2
- [ ] **Governance Layer** (5-6 days)
  - Data quality contracts
  - Schema validation
  - Policy enforcement
  - Add 12-15 tests
  - **Depends on:** StatGuardian v2.2

- [ ] **RAG Integration** (4-5 days)
  - Langfuse integration
  - Token attribution
  - Cost tracking per query
  - Add 10-12 tests

### PyStreamMCP: v0.3→v0.4
- [ ] **StatGuardian Gate Integration** (3-4 days)
  - Quality gate validation
  - Automatic quality checks
  - Policy enforcement
  - Add 8-10 tests
  - **Depends on:** StatGuardian v2.2

- [ ] **Cost Optimization Heuristics** (3-4 days)
  - Context window optimization
  - Token budget management
  - Dynamic sampling
  - Add 10-12 tests

---

## 📊 Dependencies & Critical Path

```
TinyBridge Phase 1.4 (COMPLETE) ──→ Phase 2 (OTel)
                                 ──→ Phase 3 (OKF)

PyTerrainMap Phase 6.3 (Loop) ──→ 6.4 (Pose Graph) ──→ 7 (SLAM)
                              ──→ 8 (3D Tiles)

Traversability A (Graph) ──→ B (Distance) ──→ C (Obs) + D (Connector)
                         ├──→ E (Robot) ───┐
                         └──→ F (Fleet) ───┤
                                        └──→ G (Dynamic) ──→ H (Routing) ──→ I (API)

StatGuardian 2.2 (Lineage) ──→ 2.3 (Quality Gates) ──→ 3.0 (LLM)
                            ├──→ OpenAnchor 0.2 (Governance)
                            └──→ PyStreamMCP 0.4 (Gates)
```

---

## 🎯 Success Criteria

### Phase 6 Completion (Loop Closure + Pose Graph)
- ✅ 77 tests passing (49 SfM + 10 RANSAC + 18 Keyframe)
- ⏳ 100+ tests passing (add 23+ for enhancements 3-4)
- ✅ Commit: `4ee76b4` (Phase 6.2 complete)
- ⏳ Commit: Phase 6.3 (Loop Closure)
- ⏳ Commit: Phase 6.4 (Pose Graph)

### Traversability Intelligence Completion (Phase A-I)
- ⏳ Spatial knowledge graph operational
- ⏳ All 4 distance models implemented
- ⏳ Fleet learning functional
- ⏳ Route planning integrated
- ⏳ 100+ traversability tests passing

### TinyBridge Phase 1 Completion
- ✅ Boot tier 1: <1.5s SSH
- ✅ Boot tier 2: <5s usable
- ⏳ Boot tier 3: <120s complete
- ⏳ Boot tier 4: on-demand
- ✅ 19 core tests passing
- ⏳ Phase 2: OTel export implementation

---

## 📈 Metrics & Tracking

**PyTerrainMap:**
- Tests: 154/180 (86% complete) ✅ Phases 1-8 Complete
- LOC: 5,000+/6,000 (83% complete)
- Phase 8: Spatial Knowledge Graph (34 tests, 2366 LOC)

**TinyBridge:**
- Phase 1: 100% complete
- Phase 2: 0% complete
- Total: 20% (1/5 phases)

**Traversability:**
- Status: 0% (pending start)
- Planned: 8 phases, ~3-4K LOC, ~200+ tests

**Three-Project Integration (OpenAnchor + StatGuardian + PyStreamMCP):**
- Status: 30% complete
- Next: StatGuardian 2.2 lineage extraction

---

## 🚀 Next Steps (Phase 9+)

**PHASE 8 COMPLETE** — Spatial Knowledge Graph foundation built. 154/180 tests passing.

1. **Immediate (Week 2):**
   - Start Phase 9: Multi-Layer Distance Models
   - Implement 4 distance types: geometric, topological, cost, semantic
   - Add 12-15 tests for distance computation
   - PostgreSQL queries for spatial operations

2. **Week 3:**
   - Phase 10: Robot Capability Profiles
   - Define dimensions, weight, locomotion type, traction models
   - Accessibility filtering by robot type
   - 12-15 tests

3. **Week 4:**
   - Phase 11: Fleet Learning Engine
   - Observation aggregation across robots
   - Conflict resolution (different robots, different outcomes)
   - Consensus mechanisms

---

**Questions for Prioritization:**
1. Continue Phase 6 enhancements (Loop Closure → Pose Graph) first?
2. Start Traversability Intelligence Phase A in parallel?
3. Pause Phase 6 to focus exclusively on Traversability (strategic pivot)?
4. Keep TinyBridge Phase 2 as parallel track?

