"""Tests for the allowlisted advanced-operation JSON bridge."""

import json
import math
import struct
from pathlib import Path

import pytest

from pyterrain_map import ops, run_op


def _synthetic_scene() -> dict:
    points = [
        [-1.0, 0.5, 5.0],
        [0.0, 1.0, 5.5],
        [1.0, -0.5, 6.0],
        [0.5, 0.0, 4.5],
        [-0.5, 0.5, 5.2],
        [0.2, -0.8, 6.1],
        [0.8, 0.9, 4.8],
        [-0.9, -0.4, 5.7],
        [0.1, 0.3, 5.1],
        [0.6, 0.6, 5.9],
    ]

    def keypoints(yaw_deg: float) -> list[dict]:
        yaw = math.radians(yaw_deg)
        result = []
        for index, (px, py, pz) in enumerate(points):
            x = px * math.cos(yaw) - pz * math.sin(yaw) + 0.2
            y = py
            z = px * math.sin(yaw) + pz * math.cos(yaw)
            result.append(
                {
                    "x": 640.0 + 1000.0 * x / z,
                    "y": 360.0 + 1000.0 * y / z,
                    "color": [100, 120, 140],
                    "descriptor": [index * 7.0, index * 3.0 + 1.0, index * 1.5],
                }
            )
        return result

    return {
        "images": [
            {
                "id": "cam1",
                "source": "cam1.jpg",
                "focal_length": 1000.0,
                "principal_point": [640.0, 360.0],
                "keypoints": keypoints(0.0),
            },
            {
                "id": "cam2",
                "source": "cam2.jpg",
                "focal_length": 1000.0,
                "principal_point": [640.0, 360.0],
                "keypoints": keypoints(10.0),
            },
        ],
        "bundle_adjustment_iterations": 2,
    }


def test_ops_lists_schemas_and_allowlist() -> None:
    specs = json.loads(ops())
    names = {spec["name"] for spec in specs}
    assert {
        "photogrammetry.reconstruct",
        "slam.track",
        "cesium.export_tileset",
        "elevation.srtm_lookup",
    } <= names
    for spec in specs:
        assert spec["description"]
        assert "type" in spec["input_schema"]
        assert "type" in spec["output_schema"]


def test_photogrammetry_reconstruct_success(tmp_path: Path) -> None:
    scene_path = tmp_path / "scene.json"
    scene_path.write_text(json.dumps(_synthetic_scene()))
    output_path = tmp_path / "cloud.json"

    result = json.loads(
        run_op(
            "photogrammetry.reconstruct",
            json.dumps(
                {
                    "input_path": str(scene_path),
                    "output_path": str(output_path),
                    "output_root": str(tmp_path),
                }
            ),
        )
    )
    assert result["status"] == "ok"
    assert result["image_count"] == 2
    assert result["point_count"] == 10

    cloud = json.loads(output_path.read_text())
    assert len(cloud["point_cloud"]["points"]) == 10
    assert len(cloud["camera_poses"]) == 2


def test_slam_track_success(tmp_path: Path) -> None:
    frames = [
        {
            "frame_id": index,
            "timestamp": 1000 + index * 100,
            "features": [
                {
                    "position": [100.0 + index, 200.0],
                    "descriptor": [255, 128, 64],
                    "point_3d": [float(index), 0.0, 5.0],
                }
            ],
        }
        for index in range(3)
    ]
    input_path = tmp_path / "frames.json"
    input_path.write_text(json.dumps({"frames": frames}))
    output_path = tmp_path / "trajectory.json"

    result = json.loads(
        run_op(
            "slam.track",
            json.dumps({"input_path": str(input_path), "output_path": str(output_path)}),
        )
    )
    assert result["status"] == "ok"
    assert result["keyframe_count"] == 3

    trajectory = json.loads(output_path.read_text())
    assert len(trajectory["keyframes"]) == 3
    assert trajectory["statistics"]["keyframe_count"] == 3


def test_cesium_export_writes_tileset_and_pnts(tmp_path: Path) -> None:
    input_path = tmp_path / "points.json"
    input_path.write_text(
        json.dumps(
            {
                "points": [
                    {"position": [0.0, 0.0, 0.0], "color": [255, 0, 0]},
                    {"position": [1.0, 2.0, 3.0], "color": [0, 255, 0]},
                    {"position": [-1.0, -2.0, -3.0], "color": [0, 0, 255]},
                ]
            }
        )
    )
    output_dir = tmp_path / "tiles"

    result = json.loads(
        run_op(
            "cesium.export_tileset",
            json.dumps({"input_path": str(input_path), "output_path": str(output_dir)}),
        )
    )
    assert result["status"] == "ok"
    assert result["point_count"] == 3

    tileset = json.loads((output_dir / "tileset.json").read_text())
    assert tileset["root"]["content"]["uri"] == "root.pnts"

    pnts = (output_dir / "root.pnts").read_bytes()
    assert pnts[:4] == b"pnts"
    byte_length = struct.unpack_from("<I", pnts, 8)[0]
    assert byte_length == len(pnts)


def test_srtm_lookup_success(tmp_path: Path) -> None:
    values = [30, 60, 90, 10, 20, 40, 0, 5, 15]
    (tmp_path / "N10E020.hgt").write_bytes(
        b"".join(struct.pack(">h", value) for value in values)
    )

    result = json.loads(
        run_op(
            "elevation.srtm_lookup",
            json.dumps({"tile_dir": str(tmp_path), "lat": 10.5, "lon": 20.5}),
        )
    )
    assert result["status"] == "ok"
    assert result["tile"] == "N10E020.hgt"
    assert result["elevation_m"] == 20.0


def test_unknown_op_raises_value_error() -> None:
    with pytest.raises(ValueError):
        run_op("not.an.op", "{}")


def test_malformed_json_raises_value_error() -> None:
    with pytest.raises(ValueError):
        run_op("photogrammetry.reconstruct", "{not json")


def test_missing_file_raises_file_not_found(tmp_path: Path) -> None:
    with pytest.raises(FileNotFoundError):
        run_op(
            "photogrammetry.reconstruct",
            json.dumps(
                {
                    "input_path": str(tmp_path / "missing.json"),
                    "output_path": str(tmp_path / "out.json"),
                }
            ),
        )


def test_output_root_rejects_escape(tmp_path: Path) -> None:
    scene_path = tmp_path / "scene.json"
    scene_path.write_text(json.dumps(_synthetic_scene()))
    root = tmp_path / "root"
    root.mkdir()

    with pytest.raises(PermissionError):
        run_op(
            "photogrammetry.reconstruct",
            json.dumps(
                {
                    "input_path": str(scene_path),
                    "output_path": str(tmp_path / "escaped.json"),
                    "output_root": str(root),
                }
            ),
        )
