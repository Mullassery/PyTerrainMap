"""SyncFleet integration: capture PyTerrainMap observations as SyncFleet operations.

SyncFleet replicates *operations* stored in its journal, not arbitrary SQLite
tables. This module turns PyTerrainMap observations into ``dataset apply``
operations so a fleet can replicate them over SyncFleet's existing mTLS
transport.

What this does (tested against the real ``syncfleet`` binary):

- serialises an ``Observation`` into a stable JSON payload;
- derives a deterministic entity id (retries are idempotent);
- registers the dataset/table allowlist if it does not exist;
- records one ``insert`` operation per observation via ``syncfleet dataset apply``.

What this does **not** do:

- hook ``TerrainMap.push_observation`` automatically (call the bridge);
- materialise operations into a typed SQLite table;
- capture writes from PyTerrainMap's PostgreSQL backend;
- replace SyncFleet's transport/peer enrollment (use SyncFleet's own commands).

Example::

    from pyterrain_map.syncfleet import SyncFleetBridge

    bridge = SyncFleetBridge(
        binary="syncfleet",
        config="~/.config/syncfleet/config.toml",
    )
    bridge.apply_observations(terrain.observations())
"""

from __future__ import annotations

import json
import subprocess
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

DEFAULT_DATASET = "terrain"
DEFAULT_TABLE = "terrain_observations"


class SyncFleetError(RuntimeError):
    """Raised when the SyncFleet CLI rejects or cannot run a command."""


def _get(obs: Any, *names: str, default: Any = None) -> Any:
    for name in names:
        if isinstance(obs, dict) and name in obs:
            return obs[name]
        if hasattr(obs, name):
            return getattr(obs, name)
    return default


def _location(obs: Any) -> tuple[float, float]:
    loc = _get(obs, "location")
    if loc is not None:
        return float(_get(loc, "lat", default=0.0)), float(_get(loc, "lon", default=0.0))
    lat = _get(obs, "lat", "location_lat", default=0.0)
    lon = _get(obs, "lon", "location_lon", default=0.0)
    return float(lat), float(lon)


def _sensor_value(obs: Any) -> Any:
    raw = _get(obs, "value", "value_json", "sensor_value", default=None)
    if isinstance(raw, str):
        try:
            return json.loads(raw)
        except ValueError:
            return raw
    return raw


def observation_payload(obs: Any) -> dict[str, Any]:
    """Return the stable JSON payload stored as a SyncFleet operation."""
    lat, lon = _location(obs)
    return {
        "robot_id": str(_get(obs, "robot_id", default="")),
        "timestamp": int(_get(obs, "timestamp", default=0)),
        "latitude": lat,
        "longitude": lon,
        "sensor_type": str(_get(obs, "sensor_type", default="")),
        "sensor_value": _sensor_value(obs),
        "confidence": float(_get(obs, "confidence", default=1.0)),
    }


def observation_entity_id(obs: Any) -> str:
    """Deterministic entity id so retrying the same observation is idempotent."""
    payload = observation_payload(obs)
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return str(uuid.uuid5(uuid.NAMESPACE_URL, f"pyterrainmap:{canonical}"))


@dataclass
class SyncFleetBridge:
    """Record PyTerrainMap observations in a SyncFleet node's journal."""

    binary: str = "syncfleet"
    config: str | Path = "~/.config/syncfleet/config.toml"
    dataset: str = DEFAULT_DATASET
    table: str = DEFAULT_TABLE
    timeout: float = 30.0

    def _run(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
        command = [self.binary, *args]
        completed = subprocess.run(
            command, capture_output=True, text=True, timeout=self.timeout, check=False
        )
        if check and completed.returncode != 0:
            detail = (completed.stderr or completed.stdout or "").strip()
            raise SyncFleetError(f"`{' '.join(command)}` failed: {detail}")
        return completed

    def _config_args(self) -> tuple[str, str]:
        return ("--config", str(Path(self.config).expanduser()))

    def dataset_exists(self) -> bool:
        completed = self._run(
            "dataset", "show", *self._config_args(), self.dataset, check=False
        )
        return completed.returncode == 0

    def ensure_dataset(self) -> None:
        """Register the dataset/table allowlist if it is not already present."""
        if self.dataset_exists():
            return
        self._run(
            "dataset",
            "register",
            *self._config_args(),
            "--name",
            self.dataset,
            "--tables",
            self.table,
        )

    def apply_observation(self, obs: Any) -> str:
        """Record one observation as an ``insert`` operation; return its entity id."""
        entity = observation_entity_id(obs)
        payload = observation_payload(obs)
        self._run(
            "dataset",
            "apply",
            *self._config_args(),
            "--dataset",
            self.dataset,
            "--entity",
            entity,
            "--op",
            "insert",
            "--data",
            json.dumps(payload, sort_keys=True),
        )
        return entity

    def apply_observations(self, observations: Iterable[Any]) -> int:
        """Register the dataset and record every observation. Returns the count."""
        if hasattr(observations, "observations") and callable(observations.observations):
            observations = observations.observations()
        items = list(observations)
        self.ensure_dataset()
        for obs in items:
            self.apply_observation(obs)
        return len(items)


__all__ = [
    "DEFAULT_DATASET",
    "DEFAULT_TABLE",
    "SyncFleetBridge",
    "SyncFleetError",
    "observation_entity_id",
    "observation_payload",
]
