"""
PyTerrainMap Storage Backends

Simple, pluggable storage adapters for observations.
Supports: Local disk, S3, GCS, ADLS
"""

from .base import StorageBackend, StorageObservation
from .local import LocalStorageBackend
from .s3 import S3StorageBackend
from .gcs import GCSStorageBackend
from .adls import ADLSStorageBackend

__all__ = [
    "StorageBackend",
    "StorageObservation",
    "LocalStorageBackend",
    "S3StorageBackend",
    "GCSStorageBackend",
    "ADLSStorageBackend",
]
