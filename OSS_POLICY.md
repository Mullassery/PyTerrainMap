# Open Source Software (OSS) Policy

## Core Commitment

**PyTerrainMap and PyTerrainAI use ONLY open-source components.**

- ❌ No proprietary libraries
- ❌ No cloud services (AWS, Google Cloud, Azure)
- ❌ No commercial APIs
- ❌ No closed-source dependencies
- ✅ All dependencies must have permissive licenses (MIT, Apache 2.0, BSD)

This ensures:
- Users own their data
- Self-hosted deployment
- No vendor lock-in
- Community-friendly
- MIT licensed projects can use PyTerrain

---

## Approved Dependencies

### Rust Core (PyTerrainMap)

**Essential:**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| tokio | 1.35+ | MIT | Async runtime |
| uuid | 1.6+ | MIT/Apache 2.0 | Unique IDs |
| serde | 1.0+ | MIT/Apache 2.0 | Serialization |
| serde_json | 1.0+ | MIT/Apache 2.0 | JSON support |
| parking_lot | 0.12+ | MIT/Apache 2.0 | Faster locks |
| pyo3 | 0.20+ | Apache 2.0 | Python bindings |
| pyo3-asyncio | 0.20+ | Apache 2.0 | Async Python support |

**Geospatial:**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| h3 | 0.11+ | Apache 2.0 | H3 hierarchical indexing |
| geo | 0.27+ | Apache 2.0 | Geometric operations |

**Math/Stats:**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| statrs | 0.16+ | Apache 2.0 | Statistical functions |
| ndarray | 0.15+ | MIT/Apache 2.0 | N-dimensional arrays |
| nalgebra | 0.33+ | MIT/Apache 2.0 | Linear algebra (optional) |

**Utilities:**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| chrono | 0.4+ | MIT/Apache 2.0 | Time handling |
| thiserror | 1.0+ | MIT/Apache 2.0 | Error types |
| anyhow | 1.0+ | MIT/Apache 2.0 | Error handling |
| async-trait | 0.1+ | MIT/Apache 2.0 | Async traits |
| futures | 0.3+ | MIT/Apache 2.0 | Async utilities |
| tracing | 0.1+ | MIT | Structured logging |
| tracing-subscriber | 0.3+ | MIT | Logging implementation |

**Web (Optional, for HTTP API):**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| hyper | 0.14+ | MIT | HTTP server |
| tokio-util | 0.7+ | MIT | Tokio utilities |

**Persistence (Optional, if added):**
| Crate | Version | License | Purpose |
|-------|---------|---------|---------|
| rusqlite | 0.30+ | MIT | SQLite bindings |
| sqlx | 0.7+ | MIT/Apache 2.0 | SQL query builder |
| sled | 0.34+ | MIT/Apache 2.0 | Embedded DB |
| rocksdb | 0.21+ | Apache 2.0 | RocksDB bindings |

**NO:**
- ❌ Google Cloud libraries
- ❌ AWS SDK
- ❌ Azure SDK
- ❌ Proprietary geospatial libs
- ❌ Commercial ML frameworks

---

### Python (PyTerrainAI)

**Essential:**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| pydantic | 2.0+ | MIT | Data validation |
| fastapi | 0.104+ | MIT | HTTP framework |
| httpx | 0.24+ | BSD | HTTP client |
| uvicorn | 0.24+ | BSD | ASGI server |

**Core Scientific:**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| numpy | 1.24+ | BSD | Numerical computing |
| scipy | 1.10+ | BSD | Scientific computing |
| scikit-learn | 1.3+ | BSD | ML algorithms (light) |

**Geospatial:**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| shapely | 2.0+ | BSD | Geometric operations |
| pyproj | 3.6+ | MIT | Coordinate transforms |

**Image Processing (for PyNoramic):**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| opencv-python | 4.8+ | Apache 2.0 | Image processing |
| Pillow | 10.0+ | HPND | Image manipulation |
| scikit-image | 0.21+ | BSD | Image algorithms |

**Database (Optional):**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| SQLAlchemy | 2.0+ | MIT | ORM (OSS-only) |
| psycopg | 3.1+ | LGPL 3.0 | PostgreSQL driver |
| duckdb | 0.9+ | MIT | Embedded analytics DB |

**Development/Testing:**
| Package | Version | License | Purpose |
|---------|---------|---------|---------|
| pytest | 7.0+ | MIT | Testing framework |
| pytest-asyncio | 0.21+ | Apache 2.0 | Async testing |
| black | 23.0+ | MIT | Code formatter |
| ruff | 0.1+ | MIT | Linter |
| mypy | 1.0+ | MIT | Type checker |

**NO:**
- ❌ TensorFlow (too heavyweight)
- ❌ PyTorch (too heavyweight)
- ❌ AWS boto3 (cloud-specific)
- ❌ Google Cloud libraries
- ❌ Azure SDK
- ❌ Proprietary APIs
- ❌ Commercial ML services

---

## License Compliance

All dependencies must have **permissive licenses only:**

✅ **Allowed:**
- MIT
- Apache 2.0
- Apache 2.0 + MIT (dual)
- BSD (2-clause, 3-clause, 0-clause)
- ISC
- LGPL 3.0 (for optional dependencies only)

❌ **Not Allowed:**
- GPL v2 / v3 (copyleft, restrictive)
- AGPL (too restrictive)
- Commercial licenses
- Proprietary/closed source
- Any license not OSI-approved

---

## Dependency Audit

Run these commands regularly to ensure compliance:

```bash
# Rust: Check licenses
cargo license

# Python: Check licenses
pip-licenses --format csv > licenses.csv
grep -v "MIT\|Apache\|BSD\|ISC" licenses.csv

# Both: No proprietary code
grep -r "import google\|import aws\|import azure" src/ pyterrain_ai/
# Should return: (nothing)
```

---

## Storage Backends (No Proprietary Databases)

If persistent storage is added:

✅ **Approved:**
- SQLite (built-in, no dependency)
- PostgreSQL (open-source relational)
- DuckDB (open-source OLAP)
- RocksDB (open-source key-value)
- Parquet/Arrow (open-source columnar)

❌ **Not Approved:**
- DynamoDB (AWS-specific)
- Firebase (Google proprietary)
- CosmosDB (Azure proprietary)
- Any proprietary database

---

## ML/Analytics (Keep Lightweight)

If ML features are added:

✅ **Approved (Lightweight):**
- scikit-learn (OSS, small models)
- XGBoost (OSS)
- LightGBM (OSS)
- ONNX Runtime (OSS inference)
- OpenCV (OSS computer vision)

❌ **Not Approved (Too Heavy/Proprietary):**
- TensorFlow (too heavy for mapping layer)
- PyTorch (too heavy for mapping layer)
- Hugging Face Transformers (only for OSS models)
- AWS SageMaker (proprietary)
- Google Cloud ML (proprietary)
- Azure ML (proprietary)

---

## CI/CD & DevOps (OSS Only)

✅ **Approved:**
- GitHub Actions (GitHub-native, free tier)
- GitLab CI/CD (open-source)
- Jenkins (open-source)
- Drone CI (open-source)
- Act (GitHub Actions local runner)

❌ **Not Approved:**
- Travis CI (closed-source)
- CircleCI (proprietary)
- Datadog CI (proprietary)
- AWS CodePipeline (proprietary)

---

## Hosting & Deployment (User's Choice)

PyTerrain is self-hosted. Users can deploy to:

✅ **Approved:**
- Linux/Docker (user's infrastructure)
- Kubernetes (OSS, user's cluster)
- Raspberry Pi (OSS OS)
- Local machines
- Self-hosted servers
- Open-source cloud platforms

⚠️ **User's Choice (not required by PyTerrain):**
- AWS, Google Cloud, Azure (users can deploy there, PyTerrain doesn't require it)
- Proprietary PaaS (users can use, but not required)

**Key:** PyTerrain doesn't mandate or integrate with proprietary cloud services.

---

## Monitoring/Observability (OSS Preferred)

If monitoring is added:

✅ **Recommended (OSS):**
- Prometheus (OSS metrics)
- Grafana (OSS dashboards)
- OpenTelemetry (OSS observability)
- ELK Stack (OSS logging)
- Jaeger (OSS tracing)

❌ **Not Recommended:**
- Datadog (proprietary)
- New Relic (proprietary)
- Splunk (proprietary)
- AWS CloudWatch (AWS-specific)

---

## Policy Enforcement

### In Cargo.toml:
```toml
# All dependencies must be OSS
# Check: cargo license
# Ensure no "Proprietary" or "Unknown" licenses
```

### In pyproject.toml:
```toml
# All dependencies must be OSS
# Check: pip-licenses --format csv
# Ensure no proprietary licenses
```

### In CI/CD:
```yaml
# Before release, verify:
- cargo license | grep -i proprietary  # Should fail if any found
- pip-licenses | grep -i proprietary    # Should fail if any found
```

---

## MIT License Commitment

Both PyTerrainMap and PyTerrainAI are **MIT licensed**, meaning:
- Users can use freely
- Users can modify
- Users can distribute
- No restrictions on proprietary use
- Only requirement: include license

This is compatible ONLY with OSS dependencies (permissive licenses).

---

## User Data & Privacy

Because PyTerrain is self-hosted OSS:
- Users own all their data
- No data sent to cloud
- No vendor lock-in
- No proprietary tracking
- No subscription required
- No API keys to proprietary services

---

## Review Process

Before adding ANY new dependency:

1. **License check:** Must be MIT, Apache 2.0, or BSD
2. **Size check:** Must not significantly bloat installation
3. **Alternatives:** Is there an OSS alternative?
4. **Necessity:** Is it really needed?
5. **GitHub PR:** Link to license documentation

---

## Summary

| Component | Policy |
|-----------|--------|
| **Core Libraries** | OSS only (MIT, Apache 2.0, BSD) |
| **Storage** | Self-hosted, OSS databases |
| **Deployment** | User's choice, no cloud requirement |
| **ML/Analytics** | Lightweight OSS, no heavyweight proprietary |
| **CI/CD** | OSS or GitHub-native |
| **License** | MIT only (compatible with OSS) |

**PyTerrain = Fully open source, zero proprietary components.**
