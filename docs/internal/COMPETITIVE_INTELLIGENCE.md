# Competitive intelligence in roadmap and upgrade work

**Policy date:** 2026-10-19

Competitive findings are an input to product decisions, not a one-off report.
Every pending roadmap item, new feature, and routine dependency upgrade should
check the repository's [`AUDIT_FINDINGS.md`](AUDIT_FINDINGS.md) before work
starts, and record the decision in the PR or changelog.

## Required check before building

1. **Read `AUDIT_FINDINGS.md`.** Find the relevant competitor row and the
   ranked next actions. If the finding is stale, update it with a dated source.
2. **Name the competitor and the user problem.** A feature is not justified by
   "competitor has it". State the workflow it unlocks, who needs it, and how
   often. Mark unverified demand as a hypothesis.
3. **Prefer reuse over new code.** Check whether an existing Mullassery repo
   already provides the capability (see `BUNDLES.md` in the profile repo and
   the bundle role file in this repo, when present). Typical reuse:
   - crawling/fetching/robots/SSRF → **MudFish**
   - architecture maps / PR diffs → **CodeAtlas**
   - data-quality contracts / validation → **StatGuardian**
   - token counting / cost tables → **PyTokenCalc**
   - dependency graph / advisories / SBOM → **PyDeepMesh**
   - segmentation / CLV / churn → **ClusterAudienceKit**
   - activation / lineage / PII → **PyReverseETL**
   - fleet state sync → **SyncFleet** (only after conflict resolution)
4. **Choose one disposition:** build, integrate, defer, or reject. Write it
   down. A rejection with a reason is a successful outcome.
5. **For routine upgrades**, check whether the upgrade closes a known
   competitor gap or breaks a differentiated capability. Run the tests that
   cover the affected path; do not merge a green build that silently changes a
   user-visible contract.

## PR / changelog checklist

- [ ] Competitor and source URL/date recorded (or "no direct competitor").
- [ ] User impact and frequency stated; unverified demand labelled.
- [ ] Reuse check done against sibling repos/bundles.
- [ ] Failure mode and rollback considered (especially auth, PII, cost).
- [ ] Docs/roadmap status updated honestly.
- [ ] Tests or a manual verification command included.

## Review cadence

- Refresh the competitor row when a release changes the capability.
- Re-check the full `AUDIT_FINDINGS.md` at least quarterly and before a major
  release.
- If a finding is wrong or stale, correct it in the same PR that relies on it.

## Non-goals

- Copying features for parity.
- Adding a dependency just because a competitor uses it.
- Treating star counts or download counts as proof of user need.

## OSS gap analysis — 2026-10-24

**What matters most here:** a native ROS 2 node and occupancy semantics. Duck-typed adapters and no free-space query keep it outside standard ROS 2 planning stacks; ship a real node/launch path and evaluate OctoMap-style occupancy.

Scope: PyTerrainMap 1.11.0 (Rust core, PyO3 bindings) against OSS terrain, occupancy, and H3 indexing projects, judged on the capabilities a team selects a terrain store for. "No" means not found in the reviewed public docs, not proof of absence. Star and download counts were deliberately not used.

### Comparable OSS projects

| Project | GitHub | License | Last activity | Why it is the comparison |
|---|---|---|---|---|
| OctoMap | https://github.com/OctoMap/octomap | BSD-3-Clause (core library; octovis is GPL) | v1.10.1 released 2026-09-18; last push 2026-09-26 | De facto OSS 3D probabilistic occupancy mapping. Chosen when a team needs volumetric terrain/obstacle mapping from robot sensors instead of an observation log. |
| grid_map | https://github.com/ANYbotics/grid_map | BSD-3-Clause | Last push 2026-01-13; latest tagged release 1.6.4 (2021-01-06); ROS 2 branches maintained | 2.5D multi-layer grid for elevation, variance, normals, and traversability. Closest data model to our elevation/confidence grid JSON export. |
| H3 | https://github.com/uber/h3 | Apache-2.0 | v4.5.0 released 2026-05-21; last push 2026-10-23 | The indexing primitive PyTerrainMap builds on. Chosen by teams that need hexagonal indexing only, without a store, decay, or API. |

### Capability comparison

| Capability | Ours | OctoMap | grid_map | H3 | Gap |
|---|---|---|---|---|---|
| Hexagonal hierarchical spatial index | Yes | No | No | Yes | None; we use H3 via `xs_h3`. |
| Elevation-bucketed 3D keys | Yes | Partial | Partial | No | Ours is explicit `(H3Cell, ElevationBucket)`; OctoMap is 3D occupancy, grid_map is one height cell. |
| Append-only observation history | Yes | No | No | No | Ours is ahead; competitors update or prune map state in place. |
| Per-observation temporal confidence decay | Yes | No | No | No | Ours is ahead; no reviewed competitor ships time-based confidence decay. |
| Spatial plus temporal query in one call | Yes | No | No | No | Ours is ahead; no reviewed competitor exposes a time-window query. |
| HTTP/HTTPS query server | Yes | No | No | No | Ours is ahead; competitors are libraries/ROS nodes. |
| Python-first API | Yes | No | No | Partial | Ours is ahead; H3 has separate `h3-py` bindings, the others are C++. |
| Multi-robot ID-tagged fused store | Yes | No | No | No | Ours is ahead; no reviewed competitor provides cross-robot identity in the map. |
| PostgreSQL persistence | Yes | No | No | No | Ours is ahead; OctoMap offers file serialization, grid_map is in-memory. |
| ROS 2 ingestion | Partial | Partial | Yes | No | We are weaker: duck-typed adapters, not a ROS 2 node or message-compatible package. |
| Volumetric/free-space occupancy query | No | Yes | No | No | Absent; we store observations, not ray-traced occupied/free space. |
| Built-in map pruning or bounded retention | No | Yes | Yes | No | Absent; our decay changes confidence, it does not reclaim storage. |
| Terrain slope and robot mobility assessment | Yes | No | Partial | No | Ours is ahead on mobility decisions; grid_map filters compute normals, not traversability per robot type. |
| Cesium 3D Tiles export | Partial | No | No | No | Weak but unique; README documents one root point-cloud tile, no LOD. |
| Prebuilt wheels for Linux and Windows | No | Partial | Partial | Yes | Weaker distribution: PyPI is macOS arm64 only; Linux/Windows builds are CI artifacts. |

### Gaps that matter (ranked)

1. **ROS 2-native integration is weaker, not absent.** Evidence: our README describes "duck-typed" LaserScan/PointCloud2/odometry adapters "without importing `rclpy`/`rospy`"; grid_map ships maintained ROS 2 branches (humble/iron/jazzy/rolling) and native message conversions. Hurts ROS 2 teams that expect a node, launch file, and standard messages. Effort: M.
2. **No volumetric occupancy or free-space semantics.** Evidence: OctoMap's core is probabilistic occupancy with ray casting and pruning; our store fuses typed sensor values and confidence, and exposes no occupancy/free-space query. Hurts obstacle-avoidance and planner users who compare us to OctoMap. Effort: L.
3. **Append-only store has no compaction or retention policy.** Evidence: README says observations can be added and queried but never deleted; decay affects confidence only. OctoMap prunes octree nodes; grid_map uses a circular buffer that repositions without copying. Hurts long-running fleets with unbounded disk growth. Effort: M.
4. **Distribution excludes the main fleet platforms.** Evidence: README and STATUS.md state PyPI wheels are macOS arm64 only; Linux x86_64/aarch64 and Windows wheels are CI artifacts. Hurts Linux/ROS fleet deployments that will not build Rust from source. Effort: S.
5. **No native C++ consumer API.** Evidence: the public surface is PyO3 plus the allowlisted JSON `run_op()` bridge; OctoMap and grid_map are C++ libraries with ROS integration. Hurts C++ planners that cannot embed a Python runtime. Effort: L.

### Where we are differentiated

- Append-only, `robot_id`-tagged observation history with query-time exponential/linear decay is not offered by OctoMap, grid_map, or H3.
- A single store combines H3 spatial indexing, elevation buckets, temporal windows, and confidence in one query (`TerrainMap.query`).
- Built-in HTTP/HTTPS server (`/health`, `/stats`, `/observations`, `/query/spatial`) removes a service layer competitors require users to write.
- PostgreSQL write-through and restore-on-startup are shipped; none of the three comparables provide a database backend.
- Terrain intelligence (`analyze_terrain()` with real DEM lookup and measured slope, `assess_mobility()` per robot type) is a product-level capability, not just a map data structure.
- Python-first packaging via PyO3/maturin versus C++/ROS (OctoMap, grid_map) or a C library plus separate bindings (H3).

### Disposition

- **Build:** Linux/Windows wheel publication and a ROS 2-native adapter; bounded retention/compaction for long-running fleets. Do not rebuild occupancy mapping.
- **Integrate/reuse:** H3 via `xs_h3` (already in use); OctoMap or Voxblox behind an adapter if volumetric occupancy becomes a paid requirement; retain the custom grid-map JSON export and do not claim ROS `grid_map` compatibility.
- **Defer/reject:** full SLAM and hosted service remain rejected per README/AUDIT_FINDINGS. Defer a C++ SDK and RViz plugin until a concrete consumer exists.

### Sources

- https://github.com/OctoMap/octomap (accessed 2026-10-24)
- https://github.com/ANYbotics/grid_map (accessed 2026-10-24)
- https://github.com/uber/h3 (accessed 2026-10-24)
- Repo files inspected: README.md, AUDIT_FINDINGS.md, COMPETITIVE_INTELLIGENCE.md, Cargo.toml, pyproject.toml, docs/ARCHITECTURE.md, docs/COMPARISON.md, docs/STATUS.md
