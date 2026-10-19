# Advanced operations (`run_op` / `ops`)

`pyterrain_map.run_op(op, payload_json)` is a narrow JSON bridge to advanced
capabilities that are implemented in the Rust core but are not part of the
typed Python API. It is deliberately allowlisted: passing any name not returned
by `ops()` raises `ValueError`.

```python
import json
from pyterrain_map import ops, run_op

specs = json.loads(ops())          # names, descriptions, input/output schemas
result = json.loads(run_op("elevation.srtm_lookup", json.dumps({
    "tile_dir": "/data/srtm",
    "lat": 40.7128,
    "lon": -74.0060,
})))
```

`run_op` accepts and returns JSON strings. The existing typed APIs
(`TerrainMap`, `analyze_terrain`, `start_server`, etc.) are unchanged.

## File paths and `output_root`

File-based operations take `input_path` and `output_path` in the payload. If
you also pass `output_root`, the resolved output path must stay inside that
existing directory; attempts to escape it raise `PermissionError`. Input files
must exist and be regular files (otherwise `FileNotFoundError`). Output parent
directories are created when needed after the `output_root` check. Output
paths that are themselves symlinks are rejected.

Exceptions:

| Failure | Python exception |
|---|---|
| Unknown op, malformed JSON, missing/invalid fields, bad geometry | `ValueError` |
| Missing input file, missing SRTM tile, inaccessible `output_root` | `FileNotFoundError` |
| Output path escapes `output_root` | `PermissionError` |
| Unexpected filesystem/operation failure | `RuntimeError` |

## `photogrammetry.reconstruct`

Runs the real multi-view geometry pipeline: descriptor matching with Lowe's
ratio test, essential-matrix pose recovery, DLT triangulation, and optional
structure-only bundle adjustment. Caller-supplied keypoints are required
because image/keypoint detection is deliberately Python-side.

Payload:

```json
{
  "input_path": "/data/scene.json",
  "output_path": "/data/cloud.json",
  "output_root": "/data"
}
```

Scene file:

```json
{
  "images": [
    {
      "id": "cam1",
      "source": "cam1.jpg",
      "focal_length": 1000.0,
      "principal_point": [640.0, 360.0],
      "keypoints": [
        {"x": 512.0, "y": 300.0, "color": [120, 130, 140],
         "descriptor": [0.1, 0.2, 0.3]}
      ]
    },
    {
      "id": "cam2",
      "source": "cam2.jpg",
      "focal_length": 1000.0,
      "principal_point": [640.0, 360.0],
      "keypoints": [
        {"x": 500.0, "y": 305.0, "color": [120, 130, 140],
         "descriptor": [0.1, 0.2, 0.3]}
      ]
    }
  ],
  "bundle_adjustment_iterations": 10
}
```

Optional scene fields:

- `pairs`: explicit `[["cam1", "cam2"], ...]` pairs. Default: consecutive
  images in input order.
- `bundle_adjustment_iterations`: default `10`; set `0` to skip.
- Per-image `focal_length` / `principal_point`: default `1000.0` and
  `[640, 360]`.

Each matched pair needs at least 8 keypoints. The output file contains the
dense point cloud (`points`, `bounds`, `statistics`), recovered camera poses,
and any per-pair errors. Two-view reconstructions retain the inherent
scale ambiguity of monocular SfM; treat the result as a consistent shape up to
an unknown scale unless you add external scale/odometry constraints. If no
points can be triangulated, the op raises `ValueError` rather than writing
fabricated output.

## `slam.track`

Feeds caller-supplied visual features, optional IMU readings, and optional
depth measurements through the Rust `SLAMTracker` pose graph, including the
real loop-closure detector. This is keyframe/pose-graph tracking, not a full
camera-to-map pipeline: it does not run feature detection or visual odometry
from raw images.

Payload:

```json
{
  "input_path": "/data/frames.json",
  "output_path": "/data/trajectory.json"
}
```

Frame file:

```json
{
  "frames": [
    {
      "frame_id": 0,
      "timestamp": 1000,
      "features": [
        {"position": [100.0, 200.0], "descriptor": [255, 128, 64],
         "point_3d": [0.0, 0.0, 5.0]}
      ]
    }
  ],
  "imu": [
    {"timestamp": 1000, "acceleration": [0.0, 0.0, 9.8],
     "angular_velocity": [0.01, 0.02, 0.03], "magnetic_field": null}
  ],
  "depth": [],
  "enable_loop_closure": true
}
```

The output contains the ordered keyframes (pose, covariance, feature count),
loop closures, and tracker statistics. Frames with no features raise
`ValueError`.

## `cesium.export_tileset`

Writes a 3D Tiles 1.0 point-cloud tileset:

- `tileset.json` — a root tile using a box bounding volume.
- `root.pnts` — a real quantized point-cloud tile with
  `POSITION_QUANTIZED`, per-point `RGB`, 8-byte-aligned JSON/binary sections,
  and a valid `pnts` header.

Payload:

```json
{
  "input_path": "/data/cloud.json",
  "output_path": "/data/cesium-tiles",
  "tileset_name": "Field scan"
}
```

Input points can be:

```json
{"points": [
  {"position": [0.0, 0.0, 0.0], "color": [255, 0, 0]},
  {"position": [1.0, 2.0, 3.0], "color": [0, 255, 0]}
]}
```

The full output from `photogrammetry.reconstruct` is also accepted (the op
reads `point_cloud.points`). Positions are local Cartesian meters; this
exporter does not georeference them into Earth-fixed coordinates. Apply your
own Cesium model transform if the reconstruction is georeferenced.

## `elevation.srtm_lookup`

Reads a local SRTM `.hgt` tile and returns a bilinearly interpolated elevation
for a WGS84 latitude/longitude.

```python
result = json.loads(run_op("elevation.srtm_lookup", json.dumps({
    "tile_dir": "/data/srtm",
    "lat": 40.7128,
    "lon": -74.0060,
})))
print(result["tile"], result["elevation_m"])
# N40W075.hgt 10.0 (example)
```

Tile rules:

- Filenames use the south-west corner: `N40W075.hgt`, `S01E036.hgt`, etc.
- Row 0 is the north edge, column 0 is the west edge.
- Samples are big-endian signed 16-bit; `-32768` is void and is skipped during
  interpolation (all-void samples raise `RuntimeError`).
- Real SRTM 1-arc-second (3601×3601) and 3-arc-second (1201×1201) tiles work;
  any square tile is accepted so synthetic test fixtures can be tiny.
- Coordinates must be finite and inside `[-90, 90)` latitude,
  `[-180, 180)` longitude.
- Missing tile files raise `FileNotFoundError`.

This op does not download tiles and does not call a network elevation service.
The existing `analyze_terrain()` path continues to use its Open-Meteo /
Copernicus DEM lookup.

## Limits

- `photogrammetry.reconstruct` needs caller-supplied keypoints and at least
  8 matches per pair.
- `slam.track` tracks a pose graph; it is not a full camera-to-map SLAM stack.
- `cesium.export_tileset` produces one root point-cloud tile (no LOD
  subdivision) in local coordinates.
- `elevation.srtm_lookup` only reads local `.hgt` tiles.
- Ops are synchronous and file-based; there is no streaming or remote-job
  variant yet.
