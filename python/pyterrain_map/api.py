"""
PyTerrainMap Python API

Programmatic interface for PyTerrainMap setup and configuration.
"""

from pathlib import Path
from typing import Dict, Any, Optional, List
from .setup_wizard import SetupWizard, DataWarehouse
import json


class PyTerrainMapSetup:
    """Programmatic setup interface for PyTerrainMap."""

    def __init__(self, config_dir: Optional[Path] = None):
        """
        Initialize setup interface.

        Args:
            config_dir: Configuration directory (~/.pyterrain by default)
        """
        self.wizard = SetupWizard(config_dir=config_dir)
        self.config_dir = self.wizard.config_dir

    def interactive_setup(self) -> bool:
        """Run interactive setup wizard."""
        return self.wizard.run()

    def configure_warehouse(
        self,
        warehouse: str,
        credentials: Dict[str, str],
        config: Optional[Dict[str, Any]] = None,
    ) -> bool:
        """
        Programmatically configure a warehouse without interactive prompts.

        Args:
            warehouse: "postgresql", "bigquery", "snowflake", "s3_iceberg", or "duckdb"
            credentials: Warehouse-specific credentials dict
            config: Optional configuration dict (batch_size, monitoring, etc.)

        Returns:
            True if configuration was successful

        Example:
            >>> setup = PyTerrainMapSetup()
            >>> setup.configure_warehouse(
            ...     warehouse="postgresql",
            ...     credentials={
            ...         "host": "localhost",
            ...         "port": "5432",
            ...         "database": "pyterrain",
            ...         "username": "user",
            ...         "password": "pass",
            ...     },
            ...     config={
            ...         "batch_size": 100,
            ...         "enable_monitoring": True,
            ...     }
            ... )
        """
        try:
            wh = DataWarehouse(warehouse)
        except ValueError:
            raise ValueError(
                f"Invalid warehouse: {warehouse}. "
                f"Must be one of: {', '.join([w.value for w in DataWarehouse])}"
            )

        # Test connection
        if not self.wizard._test_connection(wh, credentials):
            raise ConnectionError(f"Failed to connect to {warehouse}")

        # Build config
        full_config = config or {}
        full_config["warehouse"] = warehouse

        # Save configuration
        self.wizard._save_config(full_config)
        self.wizard._save_credentials({"warehouse": warehouse, **credentials})

        return True

    def configure_multi_warehouse(
        self,
        warehouses: Dict[str, Dict[str, str]],
        routing_policy: Optional[Dict[str, Any]] = None,
    ) -> bool:
        """
        Configure multiple warehouses with federation routing.

        Args:
            warehouses: Dict mapping warehouse names to credentials
                       {"postgresql": {...}, "bigquery": {...}, ...}
            routing_policy: Optional routing configuration
                           {"hot_tier": "postgresql", "warm_tier": "bigquery", ...}

        Returns:
            True if all warehouses configured successfully

        Example:
            >>> setup = PyTerrainMapSetup()
            >>> setup.configure_multi_warehouse(
            ...     warehouses={
            ...         "postgresql": {
            ...             "host": "localhost",
            ...             "port": "5432",
            ...             "database": "pyterrain",
            ...             "username": "user",
            ...             "password": "pass",
            ...         },
            ...         "bigquery": {
            ...             "key_file": "/path/to/service-account.json",
            ...             "project_id": "my-project",
            ...             "dataset_id": "pyterrain",
            ...         },
            ...         "s3_iceberg": {
            ...             "aws_access_key": "AKIA...",
            ...             "aws_secret_key": "...",
            ...             "region": "us-east-1",
            ...             "bucket": "my-bucket",
            ...             "prefix": "pyterrain/",
            ...         },
            ...     },
            ...     routing_policy={
            ...         "hot_tier": "postgresql",
            ...         "warm_tier": "bigquery",
            ...         "cold_tier": "s3_iceberg",
            ...     }
            ... )
        """
        config = {
            "warehouses": {},
            "routing_policy": routing_policy or {},
        }

        credentials_all = {}

        for warehouse_name, creds in warehouses.items():
            try:
                wh = DataWarehouse(warehouse_name)
            except ValueError:
                raise ValueError(f"Invalid warehouse: {warehouse_name}")

            # Test connection
            if not self.wizard._test_connection(wh, creds):
                raise ConnectionError(f"Failed to connect to {warehouse_name}")

            config["warehouses"][warehouse_name] = {"enabled": True}
            credentials_all[warehouse_name] = creds

        # Save configuration
        self.wizard._save_config(config)
        self.wizard._save_credentials(credentials_all)

        return True

    def get_config(self) -> Dict[str, Any]:
        """Get current configuration."""
        return self.wizard.current_config.copy()

    def get_credential_file(self) -> Path:
        """Get path to credentials file."""
        return self.wizard.creds_file

    def get_config_file(self) -> Path:
        """Get path to config file."""
        return self.wizard.config_file

    @classmethod
    def from_env(cls) -> Optional["PyTerrainMapSetup"]:
        """
        Create setup instance using environment variables.

        Supports:
        - PYTERRAIN_CONFIG_DIR: Configuration directory
        - PYTERRAIN_WAREHOUSE: Warehouse type
        - PYTERRAIN_HOST, PYTERRAIN_PORT, etc: Warehouse credentials

        Returns:
            PyTerrainMapSetup instance or None if env vars not set
        """
        import os

        config_dir = os.getenv("PYTERRAIN_CONFIG_DIR")
        warehouse = os.getenv("PYTERRAIN_WAREHOUSE")

        if not warehouse:
            return None

        instance = cls(config_dir=Path(config_dir) if config_dir else None)

        # Collect credentials from environment
        credentials = {}
        if warehouse == "postgresql":
            credentials = {
                "host": os.getenv("PYTERRAIN_HOST", "localhost"),
                "port": os.getenv("PYTERRAIN_PORT", "5432"),
                "database": os.getenv("PYTERRAIN_DATABASE", "pyterrain"),
                "username": os.getenv("PYTERRAIN_USERNAME", ""),
                "password": os.getenv("PYTERRAIN_PASSWORD", ""),
            }
        elif warehouse == "bigquery":
            credentials = {
                "key_file": os.getenv("PYTERRAIN_KEY_FILE", ""),
                "project_id": os.getenv("PYTERRAIN_PROJECT_ID", ""),
                "dataset_id": os.getenv("PYTERRAIN_DATASET_ID", "pyterrain"),
            }
        # ... add other warehouse types

        if credentials:
            instance.configure_warehouse(warehouse, credentials)

        return instance


class PyTerrainMapClient:
    """Client for interacting with PyTerrainMap."""

    def __init__(self, config_dir: Optional[Path] = None):
        """
        Initialize client.

        Loads configuration and connects to configured warehouse.

        Args:
            config_dir: Configuration directory (~/.pyterrain by default)
        """
        self.config_dir = config_dir or Path.home() / ".pyterrain"
        self.config_file = self.config_dir / "config.json"
        self.creds_file = self.config_dir / "credentials.json"

        if not self.config_file.exists():
            raise RuntimeError(
                "PyTerrainMap not configured. Run 'pytm setup' first."
            )

        self._load_config()

    def _load_config(self):
        """Load configuration from files."""
        with open(self.config_file) as f:
            self.config = json.load(f)

        with open(self.creds_file) as f:
            self.credentials = json.load(f)

    def connect(self):
        """Connect to configured warehouse."""
        warehouse = self.config.get("warehouse")
        if not warehouse:
            raise ValueError("Warehouse not configured")

        # Create backend based on warehouse type
        # (Implementation in actual backend factory)
        pass

    def push_observation(self, observation: Dict[str, Any]) -> str:
        """
        Push a single observation.

        Args:
            observation: Observation dict with robot_id, timestamp, location_lat,
                        location_lon, sensor_type, value_json, confidence

        Returns:
            Observation ID
        """
        pass

    def push_batch(self, observations: List[Dict[str, Any]]) -> List[str]:
        """Push multiple observations."""
        pass

    def query(
        self,
        location_lat: float,
        location_lon: float,
        radius_m: float,
        time_range_seconds: int,
    ) -> List[Dict[str, Any]]:
        """Query observations."""
        pass
