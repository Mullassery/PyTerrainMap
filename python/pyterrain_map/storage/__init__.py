"""
PyTerrainMap Storage Backends

Simple, pluggable storage adapters for observations.
Supports: Local disk, S3, GCS, ADLS
"""

from .adls import ADLSStorageBackend
from .base import StorageBackend, StorageObservation
from .gcs import GCSStorageBackend
from .local import LocalStorageBackend
from .s3 import S3StorageBackend

__all__ = [
    "StorageBackend",
    "StorageObservation",
    "LocalStorageBackend",
    "S3StorageBackend",
    "GCSStorageBackend",
    "ADLSStorageBackend",
]
