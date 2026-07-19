"""
Local filesystem storage backend for PyTerrainMap.

Stores observations as NDJSON files organized by date/robot/grid cell.
Useful for development, testing, and edge deployments.
"""

import os
import json
from pathlib import Path
from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta
import asyncio
from .base import StorageBackend, StorageObservation


class LocalStorageBackend(StorageBackend):
    """
    Local filesystem storage.

    File structure:
    base_path/
    ├── 2024/
    │   ├── 01/
    │   │   ├── 15/
    │   │   │   ├── robot-1/
    │   │   │   │   ├── grid_40.1_-74.0.ndjson
    │   │   │   │   └── grid_40.2_-73.9.ndjson
    │   │   │   └── robot-2/
    │   │   │       └── grid_40.1_-74.0.ndjson
    """

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize local storage backend.

        Args:
            config: Configuration dict with:
                - base_path: Directory to store observations (required)
                - max_file_size_mb: Max file size before rotation (default: 100)
        """
        super().__init__("local")
        self.base_path = Path(config.get("base_path", "~/.pyterrain/observations"))
        self.base_path = self.base_path.expanduser()
        self.max_file_size = config.get("max_file_size_mb", 100) * 1024 * 1024

    async def connect(self) -> bool:
        """Test connection by creating base directory."""
        try:
            self.base_path.mkdir(parents=True, exist_ok=True)
            # Write test file
            test_file = self.base_path / ".connectivity_test"
            test_file.write_text("ok")
            test_file.unlink()
            return True
        except Exception as e:
            print(f"Local storage connection failed: {e}")
            return False

    async def write_observation(self, obs: StorageObservation) -> bool:
        """Write single observation to local file."""
        try:
            file_path = self._get_file_path(obs)
            file_path.parent.mkdir(parents=True, exist_ok=True)

            # Append to file
            with open(file_path, "a") as f:
                f.write(obs.to_json() + "\n")

            self.stats["observations_written"] += 1
            return True
        except Exception as e:
            print(f"Failed to write observation: {e}")
            return False

    async def write_batch(self, observations: List[StorageObservation]) -> int:
        """Write batch of observations efficiently."""
        try:
            # Group by partition key for efficiency
            by_partition: Dict[str, List[StorageObservation]] = {}
            for obs in observations:
                key = self._partition_key(obs)
                if key not in by_partition:
                    by_partition[key] = []
                by_partition[key].append(obs)

            # Write each partition
            written = 0
            for partition, obs_list in by_partition.items():
                file_path = self.base_path / (partition + ".ndjson")
                file_path.parent.mkdir(parents=True, exist_ok=True)

                with open(file_path, "a") as f:
                    for obs in obs_list:
                        f.write(obs.to_json() + "\n")
                        written += 1

            self.stats["observations_written"] += written
            return written
        except Exception as e:
            print(f"Batch write failed: {e}")
            return 0

    async def query(
        self,
        robot_id: Optional[str] = None,
        start_time: Optional[int] = None,
        end_time: Optional[int] = None,
        sensor_type: Optional[str] = None,
        lat_min: Optional[float] = None,
        lat_max: Optional[float] = None,
        lon_min: Optional[float] = None,
        lon_max: Optional[float] = None,
        limit: int = 10000,
    ) -> List[StorageObservation]:
        """
        Query observations from local files.

        Scans NDJSON files matching time range and applies filters.
        """
        results = []
        count = 0

        # Find files in date range
        if start_time is None:
            start_date = datetime.utcnow() - timedelta(days=30)
        else:
            start_date = datetime.utcfromtimestamp(start_time / 1_000_000)

        if end_time is None:
            end_date = datetime.utcnow()
        else:
            end_date = datetime.utcfromtimestamp(end_time / 1_000_000)

        # Scan all NDJSON files in date range
        for ndjson_file in self.base_path.rglob("*.ndjson"):
            # Parse partition from path: YYYY/MM/DD/robot_id/grid_*.ndjson
            parts = ndjson_file.relative_to(self.base_path).parts

            if len(parts) < 4:
                continue

            file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))

            # Skip if file outside date range
            if file_date < start_date or file_date > end_date:
                continue

            file_robot_id = parts[3] if len(parts) > 3 else None

            # Skip if robot doesn't match
            if robot_id and file_robot_id != robot_id:
                continue

            # Read and filter observations
            try:
                with open(ndjson_file) as f:
                    for line in f:
                        if not line.strip():
                            continue

                        try:
                            obs = StorageObservation.from_json(line)

                            # Apply all filters
                            if not self._matches_filters(
                                obs,
                                robot_id=robot_id,
                                start_time=start_time,
                                end_time=end_time,
                                sensor_type=sensor_type,
                                lat_min=lat_min,
                                lat_max=lat_max,
                                lon_min=lon_min,
                                lon_max=lon_max,
                            ):
                                continue

                            results.append(obs)
                            count += 1

                            if count >= limit:
                                self.stats["observations_read"] += count
                                return results

                        except json.JSONDecodeError:
                            continue

            except Exception as e:
                print(f"Error reading {ndjson_file}: {e}")
                continue

        self.stats["observations_read"] += count
        return results

    async def get_stats(self) -> Dict[str, Any]:
        """Get storage statistics."""
        try:
            total_size = 0
            file_count = 0
            obs_count = 0

            for ndjson_file in self.base_path.rglob("*.ndjson"):
                file_count += 1
                total_size += ndjson_file.stat().st_size

                # Count lines (observations)
                try:
                    with open(ndjson_file) as f:
                        obs_count += sum(1 for line in f if line.strip())
                except:
                    pass

            return {
                "backend": "local",
                "base_path": str(self.base_path),
                "total_size_bytes": total_size,
                "total_size_mb": total_size / (1024 * 1024),
                "file_count": file_count,
                "observation_count": obs_count,
                "observations_written": self.stats["observations_written"],
                "observations_read": self.stats["observations_read"],
            }
        except Exception as e:
            return {"error": str(e)}

    async def delete_old(self, days: int) -> int:
        """Delete observations older than N days."""
        cutoff_date = datetime.utcnow() - timedelta(days=days)
        deleted = 0

        for ndjson_file in self.base_path.rglob("*.ndjson"):
            # Parse date from path: YYYY/MM/DD/...
            parts = ndjson_file.relative_to(self.base_path).parts
            if len(parts) >= 3:
                try:
                    file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                    if file_date < cutoff_date:
                        ndjson_file.unlink()
                        deleted += 1
                except (ValueError, IndexError):
                    pass

        return deleted

    def _get_file_path(self, obs: StorageObservation) -> Path:
        """Get file path for observation based on partition key."""
        partition = self._partition_key(obs)
        return self.base_path / (partition + ".ndjson")
