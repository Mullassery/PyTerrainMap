# Comparison

PyTerrainMap is a Python/Rust, H3-indexed **observation store** with temporal
decay, an HTTP API, and terrain/mobility helpers. The projects below are
building blocks or adjacent tools, not drop-in replacements.

Sources verified on **2026-10-19**:

- H3 — <https://github.com/uber/h3>
- grid_map — <https://github.com/ANYbotics/grid_map>
- PostGIS — <https://postgis.net/>

Not assessed in this pass: Tile38, OctoMap, Cesium, MapLibre. This is not a
benchmark.

## Pick by problem

| Your problem | Start with | Why |
|---|---|---|
| Fuse multi-robot terrain observations with decay | **PyTerrainMap** | Append-only H3 store + temporal confidence |
| Serve terrain queries over HTTP | **PyTerrainMap** | Built-in `/query/spatial` API |
| H3 hexagonal indexing primitives | H3 | Canonical indexing library |
| ROS 2.5D grid layers, filters, RViz | grid_map | Mature ROS C++ library |
| Geospatial SQL, raster, arbitrary GIS queries | PostGIS | PostgreSQL extension |
| Full camera-to-map SLAM pipeline | none of these | PyTerrainMap is not a full SLAM pipeline |

## Matrix

| Capability | PyTerrainMap | H3 | grid_map | PostGIS |
|---|---|---|---|---|
| Append-only observation store | ✅ | — | — | via schema |
| H3 spatial indexing | ✅ | ✅ (primitive) | — | — |
| Temporal confidence decay | ✅ | — | — | app-defined |
| HTTP query API | ✅ | — | — | — |
| Multi-robot fleet sync | ✅ (planned with SyncFleet) | — | — | — |
| ROS 2.5D grid maps | grid JSON only | — | ✅ | raster |
| SQL geospatial queries | PostgreSQL backend | — | — | ✅ |
| Terrain slope / mobility helpers | ✅ | — | filters | SQL functions |
| Python-first | ✅ | bindings | — | — |

Legend: ✅ stated in reviewed official docs; — not stated (not proof of absence).

## How they fit together

- Use **H3** for indexing primitives (PyTerrainMap builds on H3-style cells).
- Use **grid_map** when you need ROS-native 2.5D layers and RViz.
- Use **PostGIS** when your source of truth is PostgreSQL and you need SQL/GIS.
- Use **PyTerrainMap** when you want a Python-first append-only store with
  decay, an HTTP API, and terrain/mobility helpers.

See [Project status](STATUS.md) and
[grid-map JSON export](../README.md#grid-map-json-export) for the custom grid-map
JSON interchange (not ROS `grid_map` binary).
