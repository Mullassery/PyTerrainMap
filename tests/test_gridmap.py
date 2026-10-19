"""Grid-map JSON export tests."""

from __future__ import annotations

import json

from pyterrain_map.gridmap import build_grid_map, write_grid_map


def test_gridmap_aggregates_cells():
    observations = [
        {"lat": 51.5, "lon": -0.1, "elevation_m": 10, "confidence": 0.9},
        {"lat": 51.5001, "lon": -0.1001, "elevation_m": 20, "confidence": 0.7},
        {"lat": 51.6, "lon": -0.2, "elevation_m": 5, "confidence": 1.0},
    ]
    doc = build_grid_map(observations, cell_size_m=100)
    assert doc["format"] == "pyterrainmap-gridmap"
    cells = doc["cells"]
    assert any(c["observations"] == 2 for c in cells)
    first = next(c for c in cells if c["observations"] == 2)
    assert first["elevation_m"] == 15.0
    assert first["confidence"] == 0.8


def test_empty_and_write(tmp_path):
    assert build_grid_map([])["cells"] == []
    out = tmp_path / "grid.json"
    write_grid_map([{"lat": 1, "lon": 2, "elevation_m": 3}], out)
    assert json.loads(out.read_text())["cells"]
