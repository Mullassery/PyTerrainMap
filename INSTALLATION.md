# PyTerrainMap Installation & Setup Guide

## Quick Start

### 1. Install PyTerrainMap

**Via pip (recommended):**
```bash
pip install pyterrainMap
```

**Via uv:**
```bash
uv pip install pyterrainMap
```

**From source:**
```bash
git clone https://github.com/Mullassery/pyterrain-map.git
cd pyterrain-map
pip install -e .
```

### 2. Run Setup Wizard

After installation, configure your data warehouse:

```bash
pytm setup
```

This will launch an interactive wizard that:
- ✅ Shows all supported data warehouses with pros/cons
- ✅ Collects warehouse-specific credentials
- ✅ Tests your connection
- ✅ Configures optional features (monitoring, backups)
- ✅ Saves configuration to `~/.pyterrain/`

## Setup Wizard: Step-by-Step

### Step 1: Select Data Warehouse

When you run `pytm setup`, you'll see this menu:

```
============================================================
  Step 1: Select Data Warehouse
============================================================

Which data warehouse would you like to use for PyTerrainMap?

1. PostgreSQL
   Open-source relational database with TimescaleDB for time-series data.
   Best for: Small to medium deployments, already running on-premise.
   💰 Free (self-hosted) or ~$0.30-1.00/hour (managed RDS)

2. BigQuery (Google Cloud)
   Serverless data warehouse with blazing-fast SQL queries.
   Best for: Analytical workloads, cloud-native deployments, auto-scaling.
   💰 ~$7 per TB scanned (no storage fees for first 1GB/month)

3. Snowflake
   Multi-cloud analytics platform with native semi-structured data support.
   Best for: Teams wanting cloud flexibility, complex JSON handling.
   💰 ~$2-4 per compute credit + storage (30-day free trial available)

4. S3 + Apache Iceberg (AWS)
   Object storage with ACID-compliant table format. Vendor-agnostic data lake.
   Best for: Cost-sensitive, multi-tool ecosystems, portability.
   💰 ~$0.023/GB/month storage + compute costs

5. DuckDB (Embedded)
   Lightweight, in-process SQL database. Zero setup required.
   Best for: Development, edge deployments, single-machine setups.
   💰 Free (open source, no cloud costs)

6. All Five (Multi-Warehouse)
   Configure ALL warehouses with pluggable backends. Route observations
   automatically based on data age, query patterns, and cost.
   💰 Varies by usage; unified routing optimizes cost/latency

Select warehouse (1-6):
```

### Step 2: Provide Credentials

Credentials vary by warehouse:

**PostgreSQL:**
```
Database host [localhost]: db.example.com
Port [5432]: 5432
Database name [pyterrain]: pyterrain
Username: postgres
Password (will be stored securely): ***
```

**BigQuery:**
- Download service account JSON from Google Cloud Console
- Path to service account JSON key: /path/to/service-account.json
- GCP Project ID: my-project
- BigQuery dataset ID [pyterrain]: pyterrain

**Snowflake:**
```
Snowflake account (e.g., xy12345.us-east-1): xy12345.us-east-1
Warehouse name: compute_wh
Database name [PYTERRAIN]: PYTERRAIN
Schema name [PUBLIC]: PUBLIC
Username: my_user
Password (will be stored securely): ***
```

**S3 + Iceberg:**
```
AWS Access Key ID: AKIA...
AWS Secret Access Key (will be stored securely): ***
AWS Region [us-east-1]: us-east-1
S3 bucket name (pyterrain-data or similar): pyterrain-prod
S3 prefix [pyterrain/]: pyterrain/
```

**DuckDB:**
```
Database file path [~/.pyterrain/pyterrain.duckdb]: ~/.pyterrain/pyterrain.duckdb
```

### Step 3: Connection Test

The wizard automatically tests your connection:
```
🔗 Testing connection...
✅ Connection successful!
```

### Step 4: Configure Optional Features

```
============================================================
  Step 3: Optional Features
============================================================

Enable Prometheus metrics monitoring? [Y/n]: y
Enable distributed tracing (OpenTelemetry)? [y/N]: n
Enable automatic daily backups? [Y/n]: y
Observation batch size for inserts [100]: 500
Batch timeout in milliseconds [1000]: 2000
```

### Step 5: Save & Done

```
============================================================
Setup Complete!
============================================================

Configuration saved to: /Users/you/.pyterrain/config.json
Credentials saved to: /Users/you/.pyterrain/credentials.json

Next steps:
1. Start the PyTerrainMap server: pytm server
2. Set up ROS bridge: pytm ros-bridge --config config.yaml
3. View dashboard: open http://localhost:8080
```

## Programmatic Setup (Python)

If you prefer to configure PyTerrainMap programmatically without the interactive wizard:

```python
from pyterrain_map import PyTerrainMapSetup

# Single warehouse setup
setup = PyTerrainMapSetup()
setup.configure_warehouse(
    warehouse="postgresql",
    credentials={
        "host": "db.example.com",
        "port": "5432",
        "database": "pyterrain",
        "username": "postgres",
        "password": "secret",
    },
    config={
        "batch_size": 500,
        "enable_monitoring": True,
    }
)
```

### Multi-Warehouse Programmatic Setup

Configure multiple warehouses with federation:

```python
from pyterrain_map import PyTerrainMapSetup

setup = PyTerrainMapSetup()
setup.configure_multi_warehouse(
    warehouses={
        "postgresql": {
            "host": "localhost",
            "port": "5432",
            "database": "pyterrain",
            "username": "user",
            "password": "pass",
        },
        "bigquery": {
            "key_file": "/path/to/service-account.json",
            "project_id": "my-project",
            "dataset_id": "pyterrain",
        },
        "s3_iceberg": {
            "aws_access_key": "AKIA...",
            "aws_secret_key": "...",
            "region": "us-east-1",
            "bucket": "my-bucket",
            "prefix": "pyterrain/",
        },
    },
    routing_policy={
        "hot_tier": "postgresql",
        "warm_tier": "bigquery",
        "cold_tier": "s3_iceberg",
    }
)
```

### Environment Variable Setup

Configure via environment variables (useful for Docker/Kubernetes):

```bash
# PostgreSQL
export PYTERRAIN_WAREHOUSE=postgresql
export PYTERRAIN_HOST=db.example.com
export PYTERRAIN_PORT=5432
export PYTERRAIN_DATABASE=pyterrain
export PYTERRAIN_USERNAME=user
export PYTERRAIN_PASSWORD=secret

# Then initialize
pytm setup

# Or programmatically
from pyterrain_map import PyTerrainMapSetup
setup = PyTerrainMapSetup.from_env()
```

## Configuration Files

After setup, PyTerrainMap creates these files in `~/.pyterrain/`:

### `config.json` (Non-sensitive)
```json
{
  "warehouse": "postgresql",
  "batch_size": 100,
  "batch_timeout_ms": 1000,
  "enable_monitoring": true,
  "enable_tracing": false,
  "enable_auto_backup": true
}
```

### `credentials.json` (Sensitive - encrypted in production)
```json
{
  "warehouse": "postgresql",
  "host": "localhost",
  "port": "5432",
  "database": "pyterrain",
  "username": "postgres",
  "password": "***"
}
```

**⚠️ Security Note:** Credentials are stored with restricted permissions (0600). In production, use:
- AWS Secrets Manager for AWS credentials
- Google Cloud Secret Manager for GCP credentials
- Hashicorp Vault for general secrets
- Kubernetes Secrets for containerized deployments

## Update Configuration

To change your warehouse or configuration later:

```bash
pytm configure
```

This will prompt you to update warehouse settings without affecting existing data.

## Warehouse Recommendations

### For Development
```bash
# Option 1: Local SQLite/DuckDB (zero setup)
Option 5: DuckDB
# Option 2: PostgreSQL with free managed option
Option 1: PostgreSQL
```

### For Production (Single Warehouse)
```bash
# AWS: S3 + Iceberg (cheapest long-term)
Option 4: S3 + Iceberg

# Google Cloud: BigQuery (fastest queries)
Option 2: BigQuery

# Multi-cloud: Snowflake (portability)
Option 3: Snowflake
```

### For Production (Multi-Warehouse Federation)
```bash
# Recommended: Option 6
# Hot tier (immediate queries): PostgreSQL
# Warm tier (1-90 days): BigQuery or Snowflake
# Cold tier (>90 days): S3 + Iceberg
# Automatic routing based on data age & query patterns
```

## Next Steps After Setup

### 1. Start the Server
```bash
pytm server
```

### 2. Configure ROS Bridge (for robots)
```bash
pytm ros-bridge --config robot_config.yaml
```

See [ROS_BRIDGE_ARCHITECTURE.md](ROS_BRIDGE_ARCHITECTURE.md) for details.

### 3. Test Your Setup
```python
from pyterrain_map import PyTerrainMapClient

client = PyTerrainMapClient()
# Push observations
result = client.push_observation({
    "robot_id": "robot-1",
    "timestamp": 1721683200000000,  # microseconds
    "location_lat": 40.7128,
    "location_lon": -74.0060,
    "sensor_type": "lidar",
    "value_json": '{"intensity": 128, "range_m": 15.3}',
    "confidence": 0.92,
})
print(f"Observation ID: {result}")
```

## Troubleshooting

### "PyTerrainMap not configured" error
```bash
# Run setup wizard
pytm setup
```

### "Connection failed" during setup
- Verify credentials are correct
- Check firewall/network access
- For PostgreSQL: `psql -h host -U user -d database`
- For BigQuery: `gcloud auth application-default print-access-token`
- For S3: `aws s3 ls`

### "Module not found" error
```bash
# Reinstall
pip install --upgrade pyterrainMap
```

### Port already in use
```bash
# Use different port
pytm server --port 8081
```

## Support

- Documentation: https://github.com/Mullassery/pyterrain-map
- Issues: https://github.com/Mullassery/pyterrain-map/issues
- Email: mullassery@gmail.com
