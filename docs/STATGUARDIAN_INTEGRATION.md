# StatGuardian integration

PyTerrainMap has two separate quality layers. They are deliberately not
conflated:

1. **Local heuristics** — `pyterrain_map.statguardian_integration` (in-process
   Python rules for calibration, multi-sensor consistency, temporal
   coordinates, and terrain anomalies). **These do not call StatGuardian.**
2. **Real HTTP client** — `pyterrain_map.statguardian_client.StatGuardianQualityClient`
   (calls a running `statguardian serve` at `/healthz` and `/validate`).

## Real client usage

```python
from pyterrain_map.statguardian_client import (
    StatGuardianQualityClient,
    StatGuardianUnavailable,
)

client = StatGuardianQualityClient("http://127.0.0.1:8000", timeout=10)
if not client.is_available():
    raise RuntimeError("StatGuardian service is not running")

result = client.validate_observation(observation)
if not result["passed"]:
    print(result["issues"])
```

`validate_observation` maps an `Observation` (or dict) to contract attributes
**without coercing nulls**, so `not_null` rules can reject incomplete
observations.

## Tested (real service)

`tests/test_statguardian_client_live.py` starts a real
`statguardian serve` process and verifies:

- a complete observation returns `passed: true`;
- an observation with a null required field returns `passed: false`;
- an unreachable service raises `StatGuardianUnavailable` (fail closed).

The test is skipped when the `statguardian` CLI is absent. **A skipped test is
not evidence the integration works** — it means it was not tested in that
environment.

## Not tested / known limitations

- **`enum` / `between` / `min` / `max` constraints were not enforced by the
  single-row `/validate` path in the observed 2.8.0 service** (a `bogus` enum
  value and an out-of-range `confidence` both returned `passed: true`).
  `not_null` and type checks did reject. Do not rely on range/enum enforcement
  through this endpoint until that is verified for your version.
- No authentication, TLS client certs, retries, or batching.
- The service validates one contract per process; multi-contract routing is not
  implemented.
- SyncFleet payloads (`pyterrain_map.syncfleet`) use a different, coercing
  mapping; do not reuse them for validation.

## Local heuristics

The in-process contracts are useful defaults and are unit-tested, but they are
**not** a StatGuardian integration and their thresholds are hard-coded. Use the
real client when you need a versioned `.sg` contract and a shared service.
