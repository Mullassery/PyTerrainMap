"""Real StatGuardian HTTP client for PyTerrainMap.

Calls a running StatGuardian service (``statguardian serve``) at
``/healthz`` and ``/validate``. This is the cross-repo integration; the
in-process rules in :mod:`pyterrain_map.statguardian_integration` are local
heuristics and are deliberately separate.

Example::

    from pyterrain_map.statguardian_client import StatGuardianQualityClient

    client = StatGuardianQualityClient("http://127.0.0.1:8000")
    result = client.validate_observation(observation)
    if not result["passed"]:
        ...
"""

from __future__ import annotations

import json
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Any

from .syncfleet import _get, _location


class StatGuardianUnavailable(RuntimeError):
    """Raised when the StatGuardian service is unreachable or returns an error."""


@dataclass
class StatGuardianQualityClient:
    """Minimal client for StatGuardian's ``/validate`` endpoint."""

    base_url: str
    timeout: float = 10.0

    def __post_init__(self) -> None:
        self.base_url = self.base_url.rstrip("/")

    def _request(self, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        url = f"{self.base_url}{path}"
        data = None if payload is None else json.dumps(payload).encode("utf-8")
        request = urllib.request.Request(
            url,
            data=data,
            headers={"Content-Type": "application/json"},
            method="POST" if data is not None else "GET",
        )
        try:
            with urllib.request.urlopen(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode("utf-8"))
        except (urllib.error.URLError, OSError, ValueError) as exc:
            raise StatGuardianUnavailable(f"StatGuardian request to {url} failed: {exc}") from exc

    def is_available(self) -> bool:
        try:
            self._request("/healthz")
            return True
        except StatGuardianUnavailable:
            return False

    def validate(self, attributes: dict[str, Any]) -> dict[str, Any]:
        """Validate arbitrary attributes against the service's configured contract."""
        return self._request("/validate", {"entity": {"attributes": attributes}})

    def validate_observation(self, observation: Any) -> dict[str, Any]:
        """Validate a PyTerrainMap ``Observation`` (or dict) against the service.

        Unlike the SyncFleet payload, this preserves ``None`` values so the
        contract's ``not_null`` checks can reject incomplete observations.
        """
        return self.validate(observation_attributes(observation))


def observation_attributes(observation: Any) -> dict[str, Any]:
    """Map an ``Observation``/dict to contract attributes without coercing nulls."""
    lat, lon = _location(observation)
    if _get(observation, "location") is None:
        lat = _get(observation, "lat", "location_lat")
        lon = _get(observation, "lon", "location_lon")
    value = _get(observation, "value", "value_json", "sensor_value")
    if isinstance(value, str):
        try:
            value = json.loads(value)
        except ValueError:
            pass
    return {
        "robot_id": _get(observation, "robot_id"),
        "timestamp": _get(observation, "timestamp"),
        "latitude": lat,
        "longitude": lon,
        "sensor_type": _get(observation, "sensor_type"),
        "sensor_value": value,
        "confidence": _get(observation, "confidence"),
    }


__all__ = [
    "StatGuardianQualityClient",
    "StatGuardianUnavailable",
    "observation_attributes",
]
