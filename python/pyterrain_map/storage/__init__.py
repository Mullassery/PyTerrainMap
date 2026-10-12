"""
PyTerrainMap Storage Backends

Simple, pluggable storage adapters for observations.
Supports: Local disk, S3, GCS, ADLS
"""

from .adls import ADLSStorageBackend
from .base import StorageBackend, StorageFactory, StorageObservation
from .gcs import GCSStorageBackend
from .local import LocalStorageBackend
from .s3 import S3StorageBackend

# Register built-in backends with the factory. This used to happen at
# import time inside base.py, which caused a circular import (base ->
# adls -> base) that made the whole `pyterrain_map.storage` package
# unimportable. Registering here, after every backend module is imported,
# avoids the cycle.
StorageFactory.register("local", LocalStorageBackend)
StorageFactory.register("s3", S3StorageBackend)
StorageFactory.register("gcs", GCSStorageBackend)
StorageFactory.register("adls", ADLSStorageBackend)

__all__ = [
    "StorageBackend",
    "StorageObservation",
    "StorageFactory",
    "LocalStorageBackend",
    "S3StorageBackend",
    "GCSStorageBackend",
    "ADLSStorageBackend",
]
