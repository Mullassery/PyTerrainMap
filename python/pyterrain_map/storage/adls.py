"""
Azure Data Lake Storage (ADLS) backend for PyTerrainMap.

Stores observations as NDJSON in ADLS containers.
"""

from typing import List, Dict, Any, Optional
from datetime import datetime, timedelta
from .base import StorageBackend, StorageObservation


class ADLSStorageBackend(StorageBackend):
    """
    Azure Data Lake Storage backend.

    Requires: azure-storage-file-datalake library
    Authentication: Connection string or service principal

    ADLS file structure:
    https://account.dfs.core.windows.net/container/
    ├── prefix/
    │   ├── 2024/01/15/robot-1/grid_40.1_-74.0.ndjson
    │   ├── 2024/01/15/robot-2/grid_40.1_-74.0.ndjson
    │   └── 2024/01/16/robot-1/grid_40.2_-73.9.ndjson
    """

    def __init__(self, config: Dict[str, Any]):
        """
        Initialize ADLS storage backend.

        Args:
            config: Configuration dict with one of:
                - connection_string: Azure storage connection string (easiest)
                - account_name: Storage account name
                - container_name: Container name
                - account_key: Storage account key (alternative to connection string)
                OR
                - account_name: Storage account name
                - container_name: Container name
                - service_principal: Dict with tenant_id, client_id, client_secret

            Additional optional:
                - prefix: Path prefix (default: "pyterrain/")
        """
        super().__init__("adls")
        self.connection_string = config.get("connection_string")
        self.account_name = config.get("account_name")
        self.container_name = config.get("container_name")
        self.account_key = config.get("account_key")
        self.prefix = config.get("prefix", "pyterrain/").rstrip("/")
        self.file_client = None
        self.client = None

        if self.connection_string:
            # Using connection string (simplest)
            self.auth_method = "connection_string"
        elif self.account_name and self.container_name:
            if self.account_key:
                self.auth_method = "account_key"
            else:
                self.auth_method = "service_principal"
        else:
            raise ValueError(
                "ADLS requires either: "
                "1) connection_string, or "
                "2) account_name + container_name + (account_key | service_principal)"
            )

        # Import azure storage
        try:
            from azure.storage.filedatalake import DataLakeServiceClient
            self.DataLakeServiceClient = DataLakeServiceClient
        except ImportError:
            raise ImportError(
                "azure-storage-file-datalake required for ADLS backend. "
                "Install: pip install azure-storage-file-datalake"
            )

    async def connect(self) -> bool:
        """Test connection to ADLS."""
        try:
            if self.auth_method == "connection_string":
                service_client = self.DataLakeServiceClient.from_connection_string(
                    self.connection_string
                )
            elif self.auth_method == "account_key":
                service_client = self.DataLakeServiceClient(
                    account_url=f"https://{self.account_name}.dfs.core.windows.net",
                    credential=self.account_key,
                )
            else:
                # Service principal (requires Azure SDK)
                raise NotImplementedError("Service principal auth not yet implemented")

            self.client = service_client
            self.file_client = service_client.get_file_system_client(self.container_name)

            # Test access
            list(self.file_client.get_paths(path=self.prefix, max_results=1))
            return True
        except Exception as e:
            print(f"ADLS connection failed: {e}")
            return False

    async def write_observation(self, obs: StorageObservation) -> bool:
        """Write single observation to ADLS."""
        try:
            if not self.client:
                await self.connect()

            file_path = self._get_file_path(obs)
            file_client = self.file_client.get_file_client(file_path)

            # Append to file
            body = obs.to_json() + "\n"
            file_client.append_data(body.encode("utf-8"), offset=file_client.get_file_properties().size)
            file_client.flush_data(file_client.get_file_properties().size)

            self.stats["observations_written"] += 1
            return True
        except Exception as e:
            print(f"Failed to write observation to ADLS: {e}")
            return False

    async def write_batch(self, observations: List[StorageObservation]) -> int:
        """Write batch of observations to ADLS."""
        try:
            if not self.client:
                await self.connect()

            # Group by file path
            by_path: Dict[str, List[StorageObservation]] = {}
            for obs in observations:
                file_path = self._get_file_path(obs)
                if file_path not in by_path:
                    by_path[file_path] = []
                by_path[file_path].append(obs)

            # Write each file
            written = 0
            for file_path, obs_list in by_path.items():
                file_client = self.file_client.get_file_client(file_path)

                # Get current file size
                try:
                    props = file_client.get_file_properties()
                    offset = props.size
                except:
                    offset = 0

                # Append data
                body = "".join(obs.to_json() + "\n" for obs in obs_list)
                file_client.append_data(body.encode("utf-8"), offset=offset)
                file_client.flush_data(offset + len(body.encode("utf-8")))
                written += len(obs_list)

            self.stats["observations_written"] += written
            return written
        except Exception as e:
            print(f"Batch write to ADLS failed: {e}")
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
        """Query observations from ADLS."""
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

            # List all files in prefix
            for path_properties in self.file_client.get_paths(path=self.prefix):
                if path_properties.is_directory:
                    continue

                file_path = path_properties.name

                # Parse date from path
                parts = file_path.replace(self.prefix + "/", "").split("/")
                if len(parts) < 4 or not file_path.endswith(".ndjson"):
                    continue

                try:
                    file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                    if file_date < start_date or file_date > end_date:
                        continue
                except (ValueError, IndexError):
                    continue

                # Read file from ADLS
                try:
                    file_client = self.file_client.get_file_client(file_path)
                    body = file_client.download_file().readall().decode("utf-8")
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
            print(f"Query from ADLS failed: {e}")
            return []

    async def get_stats(self) -> Dict[str, Any]:
        """Get ADLS storage statistics."""
        try:
            if not self.client:
                await self.connect()

            total_size = 0
            file_count = 0

            for path_properties in self.file_client.get_paths(path=self.prefix):
                if not path_properties.is_directory:
                    file_count += 1
                    total_size += path_properties.content_length or 0

            return {
                "backend": "adls",
                "account": self.account_name,
                "container": self.container_name,
                "prefix": self.prefix,
                "total_size_bytes": total_size,
                "total_size_gb": total_size / (1024 ** 3),
                "file_count": file_count,
                "observations_written": self.stats["observations_written"],
                "observations_read": self.stats["observations_read"],
            }
        except Exception as e:
            return {"error": str(e)}

    async def delete_old(self, days: int) -> int:
        """Delete observations older than N days from ADLS."""
        try:
            if not self.client:
                await self.connect()

            cutoff_date = datetime.utcnow() - timedelta(days=days)
            deleted = 0

            for path_properties in self.file_client.get_paths(path=self.prefix):
                if path_properties.is_directory:
                    continue

                file_path = path_properties.name
                parts = file_path.replace(self.prefix + "/", "").split("/")

                if len(parts) >= 3:
                    try:
                        file_date = datetime(int(parts[0]), int(parts[1]), int(parts[2]))
                        if file_date < cutoff_date:
                            file_client = self.file_client.get_file_client(file_path)
                            file_client.delete_file()
                            deleted += 1
                    except (ValueError, IndexError):
                        pass

            return deleted
        except Exception as e:
            print(f"Delete old from ADLS failed: {e}")
            return 0

    def _get_file_path(self, obs: StorageObservation) -> str:
        """Get ADLS file path for observation."""
        partition = self._partition_key(obs)
        return f"{self.prefix}/{partition}.ndjson"
