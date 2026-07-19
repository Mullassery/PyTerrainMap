#!/usr/bin/env python3
"""
PyTerrainMap Setup Wizard

Interactive configuration for data warehouse selection and credentials management.
Runs during first-time setup or via `pytm configure`.
"""

import sys
import json
import os
from pathlib import Path
from typing import Dict, Any, Optional
from enum import Enum


class DataWarehouse(Enum):
    """Supported data warehouses."""
    POSTGRESQL = "postgresql"
    BIGQUERY = "bigquery"
    SNOWFLAKE = "snowflake"
    S3_ICEBERG = "s3_iceberg"
    DUCKDB = "duckdb"


class SetupWizard:
    """Interactive setup wizard for PyTerrainMap configuration."""

    def __init__(self, config_dir: Optional[Path] = None):
        """
        Args:
            config_dir: Directory for storing config files (~/.pyterrain by default)
        """
        self.config_dir = config_dir or Path.home() / ".pyterrain"
        self.config_dir.mkdir(exist_ok=True)
        self.config_file = self.config_dir / "config.json"
        self.creds_file = self.config_dir / "credentials.json"
        self.current_config = self._load_config()

    def _load_config(self) -> Dict[str, Any]:
        """Load existing config if available."""
        if self.config_file.exists():
            try:
                with open(self.config_file) as f:
                    return json.load(f)
            except json.JSONDecodeError:
                pass
        return {}

    def _save_config(self, config: Dict[str, Any]):
        """Save config to file (non-sensitive data only)."""
        with open(self.config_file, "w") as f:
            json.dump(config, f, indent=2)
        self.config_file.chmod(0o600)  # Restrict to user only

    def _save_credentials(self, creds: Dict[str, Any]):
        """Save credentials (sensitive data, encrypted in production)."""
        with open(self.creds_file, "w") as f:
            json.dump(creds, f, indent=2)
        self.creds_file.chmod(0o600)  # Restrict to user only

    def _print_header(self, text: str):
        """Print section header."""
        print(f"\n{'='*60}")
        print(f"  {text}")
        print(f"{'='*60}\n")

    def _print_option(self, num: int, title: str, description: str, cost: str = ""):
        """Print warehouse option."""
        print(f"{num}. {title}")
        print(f"   {description}")
        if cost:
            print(f"   💰 {cost}")
        print()

    def run(self):
        """Start the setup wizard."""
        self._print_header("PyTerrainMap Setup Wizard")

        print("Welcome! Let's configure PyTerrainMap for your deployment.\n")

        # Step 1: Select warehouse
        warehouse = self._select_warehouse()

        # Step 2: Collect warehouse-specific credentials
        credentials = self._collect_credentials(warehouse)

        # Step 3: Test connection
        if self._test_connection(warehouse, credentials):
            print("✅ Connection successful!\n")
        else:
            print("❌ Connection failed. Please verify credentials.\n")
            return False

        # Step 4: Configure optional features
        config = self._configure_optional_features(warehouse)

        # Step 5: Save configuration
        config["warehouse"] = warehouse.value
        self._save_config(config)
        self._save_credentials({"warehouse": warehouse.value, **credentials})

        self._print_header("Setup Complete!")
        print(f"Configuration saved to: {self.config_file}")
        print(f"Credentials saved to: {self.creds_file}\n")

        print("Next steps:")
        print("1. Start the PyTerrainMap server: pytm server")
        print("2. Set up ROS bridge: pytm ros-bridge --config config.yaml")
        print("3. View dashboard: open http://localhost:8080\n")

        return True

    def _select_warehouse(self) -> DataWarehouse:
        """Prompt user to select a data warehouse."""
        self._print_header("Step 1: Select Data Warehouse")

        print("Which data warehouse would you like to use for PyTerrainMap?\n")

        self._print_option(
            1,
            "PostgreSQL",
            "Open-source relational database with TimescaleDB for time-series data.\n"
            "   Best for: Small to medium deployments, already running on-premise.",
            "Free (self-hosted) or ~$0.30-1.00/hour (managed RDS)",
        )

        self._print_option(
            2,
            "BigQuery (Google Cloud)",
            "Serverless data warehouse with blazing-fast SQL queries.\n"
            "   Best for: Analytical workloads, cloud-native deployments, auto-scaling.",
            "~$7 per TB scanned (no storage fees for first 1GB/month)",
        )

        self._print_option(
            3,
            "Snowflake",
            "Multi-cloud analytics platform with native semi-structured data support.\n"
            "   Best for: Teams wanting cloud flexibility, complex JSON handling.",
            "~$2-4 per compute credit + storage (30-day free trial available)",
        )

        self._print_option(
            4,
            "S3 + Apache Iceberg (AWS)",
            "Object storage with ACID-compliant table format. Vendor-agnostic data lake.\n"
            "   Best for: Cost-sensitive, multi-tool ecosystems, portability.",
            "~$0.023/GB/month storage + compute costs",
        )

        self._print_option(
            5,
            "DuckDB (Embedded)",
            "Lightweight, in-process SQL database. Zero setup required.\n"
            "   Best for: Development, edge deployments, single-machine setups.",
            "Free (open source, no cloud costs)",
        )

        self._print_option(
            6,
            "All Five (Multi-Warehouse)",
            "Configure ALL warehouses with pluggable backends. Route observations\n"
            "   automatically based on data age, query patterns, and cost.",
            "Varies by usage; unified routing optimizes cost/latency",
        )

        while True:
            try:
                choice = input("Select warehouse (1-6): ").strip()
                choice_map = {
                    "1": DataWarehouse.POSTGRESQL,
                    "2": DataWarehouse.BIGQUERY,
                    "3": DataWarehouse.SNOWFLAKE,
                    "4": DataWarehouse.S3_ICEBERG,
                    "5": DataWarehouse.DUCKDB,
                    "6": "all",
                }
                result = choice_map.get(choice)
                if result:
                    if result == "all":
                        return self._select_multiple_warehouses()
                    return result
                print("Invalid choice. Please enter 1-6.\n")
            except KeyboardInterrupt:
                print("\n\nSetup cancelled.")
                sys.exit(0)

    def _select_multiple_warehouses(self) -> str:
        """Allow user to select multiple warehouses for federation."""
        print("\n📦 Multi-Warehouse Mode: Select which warehouses to enable\n")

        warehouses = {
            "1": (DataWarehouse.POSTGRESQL, "PostgreSQL (hot tier - primary ingestion)"),
            "2": (DataWarehouse.BIGQUERY, "BigQuery (warm tier - 1-90 days)"),
            "3": (DataWarehouse.SNOWFLAKE, "Snowflake (warm tier - alternative)"),
            "4": (DataWarehouse.S3_ICEBERG, "S3 + Iceberg (cold tier - >90 days)"),
            "5": (DataWarehouse.DUCKDB, "DuckDB (local cache/development)"),
        }

        selected = {}
        for key, (wh, desc) in warehouses.items():
            print(f"{key}. {desc}")

        print("\nEnter warehouse numbers separated by commas (e.g., '1,2,4' for PostgreSQL+BigQuery+S3):")
        print("Recommendation: '1,2,4' for production multi-tier setup\n")

        while True:
            try:
                choice = input("Select warehouses: ").strip()
                choices = [c.strip() for c in choice.split(",")]

                for c in choices:
                    if c not in warehouses:
                        print(f"Invalid choice: {c}. Please try again.\n")
                        break
                else:
                    selected = {warehouses[c][0].value: warehouses[c][0] for c in choices}
                    if selected:
                        return "multi", selected
                    print("Please select at least one warehouse.\n")
            except KeyboardInterrupt:
                print("\n\nSetup cancelled.")
                sys.exit(0)

    def _collect_credentials(self, warehouse: DataWarehouse) -> Dict[str, str]:
        """Collect warehouse-specific credentials."""
        self._print_header("Step 2: Configure Warehouse Credentials")

        credentials = {}

        if warehouse == DataWarehouse.POSTGRESQL:
            credentials = self._configure_postgresql()

        elif warehouse == DataWarehouse.BIGQUERY:
            credentials = self._configure_bigquery()

        elif warehouse == DataWarehouse.SNOWFLAKE:
            credentials = self._configure_snowflake()

        elif warehouse == DataWarehouse.S3_ICEBERG:
            credentials = self._configure_s3()

        elif warehouse == DataWarehouse.DUCKDB:
            credentials = self._configure_duckdb()

        return credentials

    def _configure_postgresql(self) -> Dict[str, str]:
        """Configure PostgreSQL connection."""
        print("PostgreSQL Configuration\n")

        host = input("Database host [localhost]: ").strip() or "localhost"
        port = input("Port [5432]: ").strip() or "5432"
        database = input("Database name [pyterrain]: ").strip() or "pyterrain"
        username = input("Username: ").strip()
        password = input("Password (will be stored securely): ").strip()

        connection_string = f"postgresql://{username}:{password}@{host}:{port}/{database}"

        return {
            "type": "postgresql",
            "connection_string": connection_string,
            "host": host,
            "port": port,
            "database": database,
            "username": username,
            "password": password,
        }

    def _configure_bigquery(self) -> Dict[str, str]:
        """Configure BigQuery connection."""
        print("BigQuery Configuration\n")
        print("You'll need a Google Cloud service account JSON key file.\n")
        print("Steps:")
        print("1. Go to: https://console.cloud.google.com/apis/credentials")
        print("2. Create a 'Service Account'")
        print("3. Create a JSON key file")
        print("4. Download and reference it here\n")

        key_file = input("Path to service account JSON key: ").strip()

        if not Path(key_file).exists():
            print(f"❌ File not found: {key_file}")
            return self._configure_bigquery()

        project_id = input("GCP Project ID: ").strip()
        dataset_id = input("BigQuery dataset ID [pyterrain]: ").strip() or "pyterrain"

        return {
            "type": "bigquery",
            "key_file": key_file,
            "project_id": project_id,
            "dataset_id": dataset_id,
        }

    def _configure_snowflake(self) -> Dict[str, str]:
        """Configure Snowflake connection."""
        print("Snowflake Configuration\n")

        account = input("Snowflake account (e.g., xy12345.us-east-1): ").strip()
        warehouse = input("Warehouse name: ").strip()
        database = input("Database name [PYTERRAIN]: ").strip() or "PYTERRAIN"
        schema = input("Schema name [PUBLIC]: ").strip() or "PUBLIC"
        username = input("Username: ").strip()
        password = input("Password (will be stored securely): ").strip()

        return {
            "type": "snowflake",
            "account": account,
            "warehouse": warehouse,
            "database": database,
            "schema": schema,
            "username": username,
            "password": password,
        }

    def _configure_s3(self) -> Dict[str, str]:
        """Configure S3 + Iceberg."""
        print("AWS S3 + Apache Iceberg Configuration\n")

        aws_access_key = input("AWS Access Key ID: ").strip()
        aws_secret_key = input("AWS Secret Access Key (will be stored securely): ").strip()
        region = input("AWS Region [us-east-1]: ").strip() or "us-east-1"
        bucket = input("S3 bucket name (pyterrain-data or similar): ").strip()
        prefix = input("S3 prefix [pyterrain/]: ").strip() or "pyterrain/"

        return {
            "type": "s3_iceberg",
            "aws_access_key": aws_access_key,
            "aws_secret_key": aws_secret_key,
            "region": region,
            "bucket": bucket,
            "prefix": prefix,
        }

    def _configure_duckdb(self) -> Dict[str, str]:
        """Configure DuckDB."""
        print("DuckDB Configuration\n")
        print("DuckDB stores data in a local file.\n")

        db_path = input("Database file path [~/.pyterrain/pyterrain.duckdb]: ").strip()
        if not db_path:
            db_path = str(self.config_dir / "pyterrain.duckdb")
        else:
            db_path = os.path.expanduser(db_path)

        return {
            "type": "duckdb",
            "db_path": db_path,
        }

    def _test_connection(self, warehouse: DataWarehouse, credentials: Dict[str, str]) -> bool:
        """Test connection to warehouse."""
        print("\n🔗 Testing connection...")

        try:
            if warehouse == DataWarehouse.POSTGRESQL:
                import psycopg2

                conn = psycopg2.connect(
                    host=credentials["host"],
                    port=int(credentials["port"]),
                    database=credentials["database"],
                    user=credentials["username"],
                    password=credentials["password"],
                )
                conn.close()
                return True

            elif warehouse == DataWarehouse.BIGQUERY:
                from google.cloud import bigquery

                bq = bigquery.Client.from_service_account_json(credentials["key_file"])
                list(bq.list_datasets(max_results=1))
                return True

            elif warehouse == DataWarehouse.SNOWFLAKE:
                import snowflake.connector

                conn = snowflake.connector.connect(
                    account=credentials["account"],
                    user=credentials["username"],
                    password=credentials["password"],
                    warehouse=credentials["warehouse"],
                    database=credentials["database"],
                    schema=credentials["schema"],
                )
                conn.close()
                return True

            elif warehouse == DataWarehouse.S3_ICEBERG:
                import boto3

                s3 = boto3.client(
                    "s3",
                    aws_access_key_id=credentials["aws_access_key"],
                    aws_secret_access_key=credentials["aws_secret_key"],
                    region_name=credentials["region"],
                )
                s3.head_bucket(Bucket=credentials["bucket"])
                return True

            elif warehouse == DataWarehouse.DUCKDB:
                import duckdb

                db = duckdb.connect(credentials["db_path"])
                db.execute("SELECT 1")
                db.close()
                return True

        except Exception as e:
            print(f"Connection error: {e}")
            return False

    def _configure_optional_features(self, warehouse: DataWarehouse) -> Dict[str, Any]:
        """Configure optional features."""
        self._print_header("Step 3: Optional Features")

        config = {
            "enable_monitoring": self._prompt_yes_no(
                "Enable Prometheus metrics monitoring? [Y/n]: ", True
            ),
            "enable_tracing": self._prompt_yes_no(
                "Enable distributed tracing (OpenTelemetry)? [y/N]: ", False
            ),
            "enable_auto_backup": self._prompt_yes_no(
                "Enable automatic daily backups? [Y/n]: ", True
            ),
        }

        if warehouse != DataWarehouse.DUCKDB:
            config["batch_size"] = int(
                input("Observation batch size for inserts [100]: ").strip() or "100"
            )
            config["batch_timeout_ms"] = int(
                input("Batch timeout in milliseconds [1000]: ").strip() or "1000"
            )

        return config

    def _prompt_yes_no(self, prompt: str, default: bool) -> bool:
        """Prompt for yes/no response."""
        response = input(prompt).strip().lower()
        if response in ("y", "yes"):
            return True
        elif response in ("n", "no"):
            return False
        return default


def main():
    """Entry point for setup wizard."""
    try:
        wizard = SetupWizard()
        success = wizard.run()
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\n\nSetup cancelled.")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Setup failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
