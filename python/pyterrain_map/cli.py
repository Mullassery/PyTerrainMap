"""PyTerrainMap CLI Interface

Interactive setup, configuration, and terrain analysis.
Accessible via: pytm <command>
"""

import argparse
import asyncio
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Optional

DEFAULT_OBSERVATIONS_DIR = Path.home() / ".pyterrain" / "observations"


def _resolve_observations_dir(config_dir: Optional[str], data_dir: Optional[str]) -> Path:
    """Where the observation store lives.

    Precedence: explicit `--data-dir`, then `config_dir`/config.json's
    `data_dir`/`observations_dir` key, then the default
    `~/.pyterrain/observations` that `LocalStorageBackend` uses.
    """
    if data_dir:
        return Path(data_dir).expanduser()

    config_path = Path(config_dir).expanduser() / "config.json" if config_dir else Path.home() / ".pyterrain" / "config.json"
    if config_path.exists():
        try:
            config = json.loads(config_path.read_text())
            configured = config.get("observations_dir") or config.get("data_dir")
            if configured:
                return Path(configured).expanduser()
        except (json.JSONDecodeError, OSError):
            pass
    return DEFAULT_OBSERVATIONS_DIR


def _parse_timestamp(value: Optional[str]) -> Optional[int]:
    """Accept either raw integer microseconds, an ISO-8601 string, or a
    `YYYY-MM-DD` date (interpreted as UTC midnight). Returns microseconds
    since the UNIX epoch, or None if not provided."""
    if value is None or value == "":
        return None
    value = value.strip()
    # Raw microseconds (all digits) -- passthrough.
    if value.isdigit():
        return int(value)
    # `Z` suffix -> +00:00 for fromisoformat on older Pythons.
    iso = value[:-1] + "+00:00" if value.endswith("Z") else value
    try:
        dt = datetime.fromisoformat(iso)
    except ValueError:
        # Bare date, e.g. 2026-10-11.
        dt = datetime.strptime(value, "%Y-%m-%d")
    if dt.tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    return int(dt.timestamp() * 1_000_000)


def _bounding_box(
    lat: float, lon: float, radius_km: float
) -> Dict[str, float]:
    """Approximate lat/lon box enclosing a circle of `radius_km` (a real
    equirectangular approximation, not a precise geodesic box -- adequate
    for an initial bounded scan, which the caller then optionally refines)."""
    import math

    lat_delta = radius_km / 111.32
    cos_lat = max(math.cos(math.radians(lat)), 1e-6)
    lon_delta = radius_km / (111.32 * cos_lat)
    return {
        "lat_min": lat - lat_delta,
        "lat_max": lat + lat_delta,
        "lon_min": lon - lon_delta,
        "lon_max": lon + lon_delta,
    }


async def _run_query(
    observations_dir: Path,
    *,
    robot_id: Optional[str],
    sensor_type: Optional[str],
    start_time: Optional[int],
    end_time: Optional[int],
    box: Optional[Dict[str, float]],
    limit: int,
):
    from .storage.local import LocalStorageBackend

    backend = LocalStorageBackend({"base_path": str(observations_dir)})
    await backend.connect()
    box = box or {}
    results = await backend.query(
        robot_id=robot_id,
        start_time=start_time,
        end_time=end_time,
        sensor_type=sensor_type,
        lat_min=box.get("lat_min"),
        lat_max=box.get("lat_max"),
        lon_min=box.get("lon_min"),
        lon_max=box.get("lon_max"),
        limit=limit,
    )
    return results


def query_command(args):
    """Query stored observations from the local NDJSON observation store."""
    observations_dir = _resolve_observations_dir(args.config_dir, getattr(args, "data_dir", None))

    try:
        start_time = _parse_timestamp(getattr(args, "start_time", None))
        end_time = _parse_timestamp(getattr(args, "end_time", None))
    except ValueError as exc:
        print(f"Invalid time value: {exc}", file=sys.stderr)
        return 2

    box = None
    lat = getattr(args, "lat", None)
    lon = getattr(args, "lon", None)
    radius_km = getattr(args, "radius_km", None)
    if lat is not None or lon is not None or radius_km is not None:
        if lat is None or lon is None:
            print("--lat and --lon must be provided together", file=sys.stderr)
            return 2
        box = _bounding_box(lat, lon, radius_km if radius_km is not None else 1.0)

    limit = getattr(args, "limit", 100) or 100
    store_existed = observations_dir.exists()

    try:
        results = asyncio.run(
            _run_query(
                observations_dir,
                robot_id=getattr(args, "robot_id", None),
                sensor_type=getattr(args, "sensor_type", None),
                start_time=start_time,
                end_time=end_time,
                box=box,
                limit=limit,
            )
        )
    except Exception as exc:  # pragma: no cover - surfaced to the user
        print(f"Query failed: {exc}", file=sys.stderr)
        return 1

    if getattr(args, "json", False):
        payload = {
            "observations_dir": str(observations_dir),
            "count": len(results),
            "observations": [obs.to_dict() for obs in results],
        }
        if results:
            payload["avg_confidence"] = sum(o.confidence for o in results) / len(results)
        print(json.dumps(payload, indent=2))
        return 0

    if not results:
        print(f"No observations found in {observations_dir}")
        if not store_existed:
            print(
                "  (store does not exist yet -- run the ROS bridge or write "
                "observations first; see `pytm setup`)"
            )
        return 0

    avg_conf = sum(o.confidence for o in results) / len(results)
    print(f"Found {len(results)} observation(s) in {observations_dir}")
    print(f"Average confidence: {avg_conf:.2f}\n")
    for i, obs in enumerate(results, start=1):
        print(
            f"{i:>4}. {obs.robot_id} @ ({obs.location_lat:.5f}, {obs.location_lon:.5f}) "
            f"[{obs.sensor_type}] conf={obs.confidence:.2f} t={obs.timestamp}"
        )
        if getattr(args, "verbose", False):
            print(f"        value: {obs.value_json}")
    return 0


def setup_command(args):
    """Run interactive setup wizard."""
    from .setup_wizard import SetupWizard

    config_dir = Path(args.config_dir) if args.config_dir else None
    wizard = SetupWizard(config_dir=config_dir)
    success = wizard.run()
    return 0 if success else 1


def configure_command(args):
    """Reconfigure warehouse (update existing setup)."""
    from .setup_wizard import SetupWizard

    config_dir = Path(args.config_dir) if args.config_dir else None
    wizard = SetupWizard(config_dir=config_dir)

    print("\n" + "=" * 60)
    print("  PyTerrainMap Configuration Update")
    print("=" * 60 + "\n")

    if wizard.current_config:
        print("Current configuration found:")
        print(f"  Warehouse: {wizard.current_config.get('warehouse', 'unknown')}")
        print()

    success = wizard.run()
    return 0 if success else 1


def version_command(args):
    """Print version information."""
    try:
        from . import __version__

        print(f"PyTerrainMap {__version__}")
    except ImportError:
        print("PyTerrainMap v0.1.0")
    return 0


def main() -> int:
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        prog="pytm",
        description="PyTerrainMap: Collaborative terrain mapping for multi-robot fleets",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  pytm setup                      # Initial configuration wizard
  pytm configure                  # Update warehouse configuration
  pytm version                    # Show version information
  pytm query --robot-id robot1    # Query stored observations
  pytm query --lat 40.1 --lon -74.5 --radius-km 0.5 --json
        """,
    )

    parser.add_argument(
        "--version",
        action="version",
        version="%(prog)s 0.1.0",
    )

    parser.add_argument(
        "--config-dir",
        default=None,
        help="Configuration directory (default: ~/.pyterrain)",
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Setup command
    setup_parser = subparsers.add_parser(
        "setup",
        help="Interactive setup wizard for PyTerrainMap (first-time configuration)",
    )
    setup_parser.set_defaults(func=setup_command)

    # Configure command
    configure_parser = subparsers.add_parser(
        "configure",
        help="Reconfigure PyTerrainMap warehouse settings",
    )
    configure_parser.set_defaults(func=configure_command)

    # Version command
    version_parser = subparsers.add_parser(
        "version",
        help="Print version information",
    )
    version_parser.set_defaults(func=version_command)

    # Query command
    query_parser = subparsers.add_parser(
        "query",
        help="Query stored observations from the local observation store",
    )
    query_parser.add_argument(
        "--data-dir",
        default=None,
        help="Observation store directory (default: ~/.pyterrain/observations)",
    )
    query_parser.add_argument(
        "--robot-id", default=None, help="Filter by robot id"
    )
    query_parser.add_argument(
        "--sensor-type", default=None, help="Filter by sensor type (e.g. thermal, lidar)"
    )
    query_parser.add_argument(
        "--start-time",
        default=None,
        help="Start time (inclusive): ISO-8601, YYYY-MM-DD, or microseconds",
    )
    query_parser.add_argument(
        "--end-time",
        default=None,
        help="End time (inclusive): ISO-8601, YYYY-MM-DD, or microseconds",
    )
    query_parser.add_argument(
        "--lat", type=float, default=None, help="Center latitude for a radius query"
    )
    query_parser.add_argument(
        "--lon", type=float, default=None, help="Center longitude for a radius query"
    )
    query_parser.add_argument(
        "--radius-km",
        type=float,
        default=None,
        help="Radius in km around --lat/--lon (default 1.0 when --lat/--lon given)",
    )
    query_parser.add_argument(
        "--limit", type=int, default=100, help="Maximum observations to return (default 100)"
    )
    query_parser.add_argument(
        "--json", action="store_true", help="Emit machine-readable JSON"
    )
    query_parser.add_argument(
        "--verbose", action="store_true", help="Show each observation's value payload"
    )
    query_parser.set_defaults(func=query_command)

    # If no args provided, show help
    if len(sys.argv) == 1:
        parser.print_help()
        return 0

    args = parser.parse_args()

    if hasattr(args, "func"):
        return args.func(args)
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
