"""Lightweight grid-map JSON export.

This is a documented JSON interchange for elevation/confidence grids, intended
for custom viewers and MapLibre overlays. It is **not** the ROS `grid_map`
message format; OctoMap/ROS binary export remains separate.
"""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


@dataclass
class GridCell:
    x: float
    y: float
    elevation_m: float | None
    confidence: float
    observations: int


def build_grid_map(
    observations: Iterable[dict[str, Any]],
    *,
    cell_size_m: float = 10.0,
) -> dict[str, Any]:
    """Aggregate lat/lon observations into a metric-ish grid.

    ``x``/``y`` are degrees; ``cell_size_m`` is converted to an approximate
    degree span using the mean latitude. Values are means within each cell.
    """
    if cell_size_m <= 0:
        raise ValueError("cell_size_m must be positive")
    obs = [o for o in observations if isinstance(o, dict)]
    if not obs:
        return {"format": "pyterrainmap-gridmap", "version": 1, "cells": []}
    mean_lat = sum(float(o.get("lat") or 0.0) for o in obs) / len(obs)
    deg_per_m_lat = 1.0 / 111_320.0
    deg_per_m_lon = 1.0 / (111_320.0 * max(0.1, math.cos(math.radians(mean_lat))))
    cell_lat = cell_size_m * deg_per_m_lat
    cell_lon = cell_size_m * deg_per_m_lon

    buckets: dict[tuple[int, int], list[dict[str, Any]]] = {}
    for o in obs:
        lat = float(o.get("lat") or 0.0)
        lon = float(o.get("lon") or 0.0)
        key = (int(math.floor(lat / cell_lat)), int(math.floor(lon / cell_lon)))
        buckets.setdefault(key, []).append(o)

    cells = []
    for (iy, ix), items in sorted(buckets.items()):
        elevations = [float(o["elevation_m"]) for o in items if o.get("elevation_m") is not None]
        confidences = [
            float(o.get("confidence", 1.0)) for o in items if o.get("confidence") is not None
        ]
        cells.append(
            GridCell(
                x=round(ix * cell_lon, 6),
                y=round(iy * cell_lat, 6),
                elevation_m=round(sum(elevations) / len(elevations), 2) if elevations else None,
                confidence=round(sum(confidences) / len(confidences), 4) if confidences else 1.0,
                observations=len(items),
            ).__dict__
        )
    return {
        "format": "pyterrainmap-gridmap",
        "version": 1,
        "cell_size_m": cell_size_m,
        "cells": cells,
    }


def write_grid_map(
    observations: Iterable[dict[str, Any]],
    path: str | Path,
    *,
    cell_size_m: float = 10.0,
) -> str:
    document = build_grid_map(observations, cell_size_m=cell_size_m)
    Path(path).write_text(json.dumps(document, indent=2), encoding="utf-8")
    return str(path)


__all__ = ["GridCell", "build_grid_map", "write_grid_map"]
