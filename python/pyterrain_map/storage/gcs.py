"""
Google Cloud Storage (GCS) backend for PyTerrainMap.

Stores observations as NDJSON in GCS buckets.
"""

from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta
from .base import StorageBackend, StorageObservation


class GCSStorageBackend(StorageBackend):
    """
    Google Cloud Storage backend.

    Requires: google-cloud-storage library
    Authentication: Service account key file or Application Default Credentials

    GCS bucket structure:
    gs://bucket/prefix/
    ├── 2024/01/15/robot-1/grid_40.1_-74.0.ndjson
    ├── 2024/01/15/robot-2/grid_40.1_-74.0.ndjson
    └── 2024/01/16/robot-1/grid_40.2_-73.9.ndjson
    """

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize GCS storage backend.

        Args:
            config: Configuration dict with:
                - bucket: GCS bucket name (required)
                - prefix: GCS object prefix (default: "pyterrain/")
                - project_id: GCP project ID (optional, auto-detected from credentials)
                - credentials_file: Path to service account JSON (optional)
        """
        super().__init__("gcs")
        self.bucket_name = config.get("bucket")
        self.prefix = config.get("prefix", "pyterrain/").rstrip("/")
        self.project_id = config.get("project_id")
        self.credentials_file = config.get("credentials_file")
        self.bucket = None
        self.client = None

        if not self.bucket_name:
            raise ValueError("GCS bucket name required in config")

        # Import google cloud
        try:
            from google.cloud import storage
            self.storage = storage
        except ImportError:
            raise ImportError(
                "google-cloud-storage required for GCS backend. "
                "Install: pip install google-cloud-storage"
            )

    async def connect(self) -> bool:
        """Test connection to GCS."""
        try:
            if self.credentials_file:
                self.client = self.storage.Client.from_service_account_json(
                    self.credentials_file,
                    project=self.project_id,
                )
            else:
                self.client = self.storage.Client(project=self.project_id)

            self.bucket = self.client.bucket(self.bucket_name)
            # Test access
            self.bucket.reload()
            return True
        except Exception as e:
            print(f"GCS connection failed: {e}")
            return False

    async def write_observation(self, obs: StorageObservation) -> bool:
        """Write single observation to GCS."""
        try:
            if not self.client:
                await self.connect()

            blob_name = self._get_blob_name(obs)
            blob = self.bucket.blob(blob_name)

            # Append to blob (in practice, use resumable upload for large files)
            body = obs.to_json() + "\n"
            blob.upload_from_string(body, content_type="text/plain")

            self.stats["observations_written"] += 1
            return True
        except Exception as e:
            print(f"Failed to write observation to GCS: {e}")
            return False

    async def write_batch(self, observations: List[StorageObservation]) -> int:
        """Write batch of observations to GCS."""
        try:
            if not self.client:
                await self.connect()

            # Group by blob name
            by_blob: Dict[str, List[StorageObservation]] = {}
            for obs in observations:
                blob_name = self._get_blob_name(obs)
                if blob_name not in by_blob:
                    by_blob[blob_name] = []
                by_blob[blob_name].append(obs)

            # Write each blob
            written = 0
            for blob_name, obs_list in by_blob.items():
                blob = self.bucket.blob(blob_name)
                body = "".join(obs.to_json() + "\n" for obs in obs_list)
                blob.upload_from_string(body, content_type="text/plain")
                written += len(obs_list)

            self.stats["observations_written"] += written
            return written
        except Exception as e:
            print(f"Batch write to GCS failed: {e}")
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
        """Query observations from GCS."""
        try:
            if not self.client:
                await self.connect()

            results = []
            count = 0

            # Determine date range
            if start_time is None:
                start_date = datetime.utcnow() - timedelta(days=30)
            else:
                start_date = datetime.utcfromtimestamp(start_time / 1_000_000)

            if end_time is None:
                end_date = datetime.utcnow()
            else:
                end_date = datetime.utcfromtimestamp(end_time / 1_000_000)

            # List all blobs in prefix
            prefix = f"{self.prefix}/"
            for blob in self.client.list_blobs(self.bucket_name, prefix=prefix):
                blob_name = blob.name

                # Parse date from blob name
                parts = blob_name.replace(self.prefix + "/", "").split("/")
                if len(parts) < 4:
                    continue

                try:
                    file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                    if file_date < start_date or file_date > end_date:
                        continue
                except (ValueError, IndexError):
                    continue

                # Read blob from GCS
                try:
                    body = blob.download_as_string().decode("utf-8")
                except:
                    continue

                # Process lines
                for line in body.split("\n"):
                    if not line.strip():
                        continue

                    try:
                        obs = StorageObservation.from_json(line)

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

                    except Exception:
                        continue

            self.stats["observations_read"] += count
            return results
        except Exception as e:
            print(f"Query from GCS failed: {e}")
            return []

    async def get_stats(self) -> Dict[str, Any]:
        """Get GCS storage statistics."""
        try:
            if not self.client:
                await self.connect()

            total_size = 0
            blob_count = 0

            for blob in self.client.list_blobs(self.bucket_name, prefix=f"{self.prefix}/"):
                blob_count += 1
                total_size += blob.size or 0

            return {
                "backend": "gcs",
                "bucket": self.bucket_name,
                "prefix": self.prefix,
                "total_size_bytes": total_size,
                "total_size_gb": total_size / (1024 ** 3),
                "blob_count": blob_count,
                "observations_written": self.stats["observations_written"],
                "observations_read": self.stats["observations_read"],
            }
        except Exception as e:
            return {"error": str(e)}

    async def delete_old(self, days: int) -> int:
        """Delete observations older than N days from GCS."""
        try:
            if not self.client:
                await self.connect()

            cutoff_date = datetime.utcnow() - timedelta(days=days)
            deleted = 0

            for blob in self.client.list_blobs(self.bucket_name, prefix=f"{self.prefix}/"):
                blob_name = blob.name
                parts = blob_name.replace(self.prefix + "/", "").split("/")

                if len(parts) >= 3:
                    try:
                        file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                        if file_date < cutoff_date:
                            blob.delete()
                            deleted += 1
                    except (ValueError, IndexError):
                        pass

            return deleted
        except Exception as e:
            print(f"Delete old from GCS failed: {e}")
            return 0

    def _get_blob_name(self, obs: StorageObservation) -> str:
        """Get GCS blob name for observation."""
        partition = self._partition_key(obs)
        return f"{self.prefix}/{partition}.ndjson"
