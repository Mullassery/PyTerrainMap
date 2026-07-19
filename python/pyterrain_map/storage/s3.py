"""
AWS S3 storage backend for PyTerrainMap.

Stores observations as NDJSON in S3 buckets with partitioning by date/robot/grid cell.
"""

from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta
from .base import StorageBackend, StorageObservation


class S3StorageBackend(StorageBackend):
    """
    AWS S3 storage backend.

    Requires: boto3 library and AWS credentials
    Environment variables or config:
    - AWS_ACCESS_KEY_ID
    - AWS_SECRET_ACCESS_KEY
    - AWS_REGION

    S3 object structure:
    s3://bucket/prefix/
    ├── 2024/01/15/robot-1/grid_40.1_-74.0.ndjson
    ├── 2024/01/15/robot-2/grid_40.1_-74.0.ndjson
    └── 2024/01/16/robot-1/grid_40.2_-73.9.ndjson
    """

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize S3 storage backend.

        Args:
            config: Configuration dict with:
                - bucket: S3 bucket name (required)
                - prefix: S3 object prefix (default: "pyterrain/")
                - region: AWS region (default: "us-east-1")
                - aws_access_key: AWS access key (optional, uses env vars if not provided)
                - aws_secret_key: AWS secret key (optional, uses env vars if not provided)
        """
        super().__init__("s3")
        self.bucket = config.get("bucket")
        self.prefix = config.get("prefix", "pyterrain/").rstrip("/")
        self.region = config.get("region", "us-east-1")
        self.client = None

        if not self.bucket:
            raise ValueError("S3 bucket name required in config")

        # Import boto3
        try:
            import boto3
            self.boto3 = boto3
        except ImportError:
            raise ImportError("boto3 required for S3 backend. Install: pip install boto3")

    async def connect(self) -> bool:
        """Test connection to S3."""
        try:
            self.client = self.boto3.client("s3", region_name=self.region)
            # Test access
            self.client.head_bucket(Bucket=self.bucket)
            return True
        except Exception as e:
            print(f"S3 connection failed: {e}")
            return False

    async def write_observation(self, obs: StorageObservation) -> bool:
        """Write single observation to S3."""
        try:
            if not self.client:
                await self.connect()

            key = self._get_object_key(obs)
            body = obs.to_json() + "\n"

            # Append to existing object (in practice, use multipart upload)
            self.client.put_object(
                Bucket=self.bucket,
                Key=key,
                Body=body,
            )

            self.stats["observations_written"] += 1
            return True
        except Exception as e:
            print(f"Failed to write observation to S3: {e}")
            return False

    async def write_batch(self, observations: List[StorageObservation]) -> int:
        """Write batch of observations to S3."""
        try:
            if not self.client:
                await self.connect()

            # Group by partition key
            by_key: Dict[str, List[StorageObservation]] = {}
            for obs in observations:
                key = self._get_object_key(obs)
                if key not in by_key:
                    by_key[key] = []
                by_key[key].append(obs)

            # Write each partition
            written = 0
            for key, obs_list in by_key.items():
                body = "".join(obs.to_json() + "\n" for obs in obs_list)
                self.client.put_object(
                    Bucket=self.bucket,
                    Key=key,
                    Body=body,
                )
                written += len(obs_list)

            self.stats["observations_written"] += written
            return written
        except Exception as e:
            print(f"Batch write to S3 failed: {e}")
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
        """Query observations from S3."""
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

            # List all objects in prefix
            paginator = self.client.get_paginator("list_objects_v2")
            prefix = f"{self.prefix}/"
            if robot_id:
                # Can further filter by robot_id in prefix
                pass

            for page in paginator.paginate(Bucket=self.bucket, Prefix=prefix):
                if "Contents" not in page:
                    continue

                for obj in page["Contents"]:
                    key = obj["Key"]

                    # Parse date from key: prefix/YYYY/MM/DD/robot_id/grid_*.ndjson
                    parts = key.replace(self.prefix + "/", "").split("/")
                    if len(parts) < 4:
                        continue

                    try:
                        file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                        if file_date < start_date or file_date > end_date:
                            continue
                    except (ValueError, IndexError):
                        continue

                    # Read object from S3
                    response = self.client.get_object(Bucket=self.bucket, Key=key)
                    body = response["Body"].read().decode("utf-8")

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
            print(f"Query from S3 failed: {e}")
            return []

    async def get_stats(self) -> Dict[str, Any]:
        """Get S3 storage statistics."""
        try:
            if not self.client:
                await self.connect()

            total_size = 0
            object_count = 0
            obs_count = 0

            paginator = self.client.get_paginator("list_objects_v2")
            for page in paginator.paginate(Bucket=self.bucket, Prefix=f"{self.prefix}/"):
                if "Contents" not in page:
                    continue

                for obj in page["Contents"]:
                    object_count += 1
                    total_size += obj["Size"]

            return {
                "backend": "s3",
                "bucket": self.bucket,
                "prefix": self.prefix,
                "region": self.region,
                "total_size_bytes": total_size,
                "total_size_gb": total_size / (1024 ** 3),
                "object_count": object_count,
                "observations_written": self.stats["observations_written"],
                "observations_read": self.stats["observations_read"],
            }
        except Exception as e:
            return {"error": str(e)}

    async def delete_old(self, days: int) -> int:
        """Delete observations older than N days from S3."""
        try:
            if not self.client:
                await self.connect()

            cutoff_date = datetime.utcnow() - timedelta(days=days)
            deleted = 0

            paginator = self.client.get_paginator("list_objects_v2")
            for page in paginator.paginate(Bucket=self.bucket, Prefix=f"{self.prefix}/"):
                if "Contents" not in page:
                    continue

                for obj in page["Contents"]:
                    key = obj["Key"]
                    parts = key.replace(self.prefix + "/", "").split("/")

                    if len(parts) >= 3:
                        try:
                            file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                            if file_date < cutoff_date:
                                self.client.delete_object(Bucket=self.bucket, Key=key)
                                deleted += 1
                        except (ValueError, IndexError):
                            pass

            return deleted
        except Exception as e:
            print(f"Delete old from S3 failed: {e}")
            return 0

    def _get_object_key(self, obs: StorageObservation) -> str:
        """Get S3 object key for observation."""
        partition = self._partition_key(obs)
        return f"{self.prefix}/{partition}.ndjson"
