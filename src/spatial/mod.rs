//! Spatial indexing using H3 hierarchical hexagonal grids
//!
//! Organizes observations into H3 cells and elevation buckets for efficient
//! spatial-temporal queries. Supports radius-based searches and multi-resolution
//! operations.

use crate::types::{ElevationBucket, GeoPoint, Result, Error};
use xs_h3::{LatLng, H3Index, degs_to_rads};
use std::collections::HashMap;

/// H3 cell identifier
pub type H3Cell = H3Index;

/// Default H3 resolution (9 = ~174m hexagon size, good for robot navigation)
const DEFAULT_H3_RESOLUTION: i32 = 9;

/// Maximum k-ring distance for grid_disk queries
const MAX_K_RING: i32 = 15;

/// Spatial cell key: H3 cell + elevation bucket
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpatialKey {
    pub h3_cell: H3Cell,
    pub elevation_bucket: Option<ElevationBucket>,
}

impl SpatialKey {
    pub fn new(h3_cell: H3Cell, elevation_bucket: Option<ElevationBucket>) -> Self {
        SpatialKey {
            h3_cell,
            elevation_bucket,
        }
    }
}

/// Spatial index using H3 cells and elevation buckets
pub struct SpatialIndex {
    h3_resolution: i32,
    // Map from H3 cell + elevation to observation indices
    cells: HashMap<SpatialKey, Vec<usize>>,
}

impl SpatialIndex {
    /// Create new spatial index with default H3 resolution
    pub fn new() -> Self {
        SpatialIndex {
            h3_resolution: DEFAULT_H3_RESOLUTION,
            cells: HashMap::new(),
        }
    }

    /// Create spatial index with custom H3 resolution
    pub fn with_resolution(resolution: i32) -> Result<Self> {
        if resolution < 0 || resolution > 15 {
            return Err(Error::InvalidLocation);
        }
        Ok(SpatialIndex {
            h3_resolution: resolution,
            cells: HashMap::new(),
        })
    }

    /// Get H3 cell for a location
    pub fn location_to_h3(&self, location: GeoPoint) -> Result<H3Cell> {
        if !location.is_valid() {
            return Err(Error::InvalidLocation);
        }

        // Convert lat/lon (degrees) to H3 cell (needs radians)
        let lat_lng = LatLng {
            lat: degs_to_rads(location.lat),
            lng: degs_to_rads(location.lon),
        };
        let cell = xs_h3::lat_lng_to_cell(&lat_lng, self.h3_resolution)
            .map_err(|e| Error::InvalidObservation(format!("H3 conversion failed: {:?}", e)))?;

        Ok(cell)
    }

    /// Get elevation bucket for an elevation value
    pub fn elevation_to_bucket(&self, elevation: Option<f32>) -> Option<ElevationBucket> {
        elevation.map(ElevationBucket::from_elevation_1m)
    }

    /// Index an observation (adds to cell)
    pub fn insert(&mut self, obs_index: usize, location: GeoPoint, elevation: Option<f32>) -> Result<SpatialKey> {
        let h3_cell = self.location_to_h3(location)?;
        let elevation_bucket = self.elevation_to_bucket(elevation);
        let key = SpatialKey::new(h3_cell, elevation_bucket);

        self.cells
            .entry(key)
            .or_insert_with(Vec::new)
            .push(obs_index);

        Ok(key)
    }

    /// Get disk of H3 cells around a center (includes all cells within k distance)
    pub fn get_disk(&self, center: GeoPoint, radius_m: f32) -> Result<Vec<H3Cell>> {
        let center_cell = self.location_to_h3(center)?;

        // Convert meters to H3 ring radius (approximate)
        // H3 resolution 9 ~ 174m hexagon, so k ≈ radius_m / 174
        let k_ring = (radius_m / 174.0).ceil() as i32;
        let k_ring = k_ring.max(0).min(MAX_K_RING);

        // grid_disk returns all cells within k distance
        let max_cells = xs_h3::max_grid_disk_size(k_ring)
            .map_err(|_| Error::InvalidLocation)? as usize;

        let mut cells = vec![H3Index(0); max_cells];
        xs_h3::grid_disk(center_cell, k_ring, &mut cells[..])
            .map_err(|_| Error::InvalidLocation)?;

        // Filter out invalid cells (0 from unused buffer space)
        cells.retain(|&cell| cell.0 != 0);
        Ok(cells)
    }

    /// Query observations in radius around a point
    pub fn query_radius(
        &self,
        location: GeoPoint,
        radius_m: f32,
        elevation_filter: Option<ElevationBucket>,
    ) -> Result<Vec<usize>> {
        let cells = self.get_disk(location, radius_m)?;

        let mut results = Vec::new();
        for cell in cells {
            // Iterate through all elevation buckets if filter not specified
            if let Some(filter) = elevation_filter {
                let key = SpatialKey::new(cell, Some(filter));
                if let Some(obs_indices) = self.cells.get(&key) {
                    results.extend(obs_indices);
                }
            } else {
                // Include all elevation buckets for this cell
                for (key, obs_indices) in self.cells.iter() {
                    if key.h3_cell == cell {
                        results.extend(obs_indices);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get all cells that have observations
    pub fn get_populated_cells(&self) -> Vec<SpatialKey> {
        self.cells.keys().cloned().collect()
    }

    /// Get observation indices for a specific cell
    pub fn get_observations_in_cell(&self, key: SpatialKey) -> Option<&[usize]> {
        self.cells.get(&key).map(|v| v.as_slice())
    }

    /// Clear all indexed observations
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_index_creation() {
        let index = SpatialIndex::new();
        assert_eq!(index.h3_resolution, DEFAULT_H3_RESOLUTION);
        assert!(index.cells.is_empty());
    }

    #[test]
    fn test_location_to_h3() {
        let index = SpatialIndex::new();
        let location = GeoPoint::new(40.7128, -74.0060); // NYC

        let h3_cell = index.location_to_h3(location).unwrap();
        assert!(h3_cell.0 > 0);
    }

    #[test]
    fn test_invalid_location() {
        let index = SpatialIndex::new();
        let invalid = GeoPoint::new(95.0, -74.0060); // Invalid lat

        assert!(index.location_to_h3(invalid).is_err());
    }

    #[test]
    fn test_elevation_bucket_generation() {
        let index = SpatialIndex::new();

        let bucket1 = index.elevation_to_bucket(Some(42.5));
        assert_eq!(bucket1, Some(ElevationBucket { min_m: 42.0, max_m: 43.0 }));

        let bucket2 = index.elevation_to_bucket(Some(100.0));
        assert_eq!(bucket2, Some(ElevationBucket { min_m: 100.0, max_m: 101.0 }));

        let bucket_none = index.elevation_to_bucket(None);
        assert_eq!(bucket_none, None);
    }

    #[test]
    fn test_insert_and_query() {
        let mut index = SpatialIndex::new();
        let location = GeoPoint::new(40.7128, -74.0060);

        // Insert observation at index 0
        let key = index.insert(0, location, Some(100.0)).unwrap();
        assert!(key.h3_cell.0 > 0);
        assert_eq!(key.elevation_bucket, Some(ElevationBucket { min_m: 100.0, max_m: 101.0 }));

        // Query and verify it's there
        let results = index.query_radius(location, 100.0, None).unwrap();
        assert!(results.contains(&0));
    }

    #[test]
    fn test_multiple_observations_in_cell() {
        let mut index = SpatialIndex::new();
        let location = GeoPoint::new(40.7128, -74.0060);

        // Insert multiple observations at same location
        index.insert(0, location, Some(100.0)).unwrap();
        index.insert(1, location, Some(100.0)).unwrap();
        index.insert(2, location, Some(101.0)).unwrap(); // Different elevation

        // Query with no elevation filter
        let results = index.query_radius(location, 100.0, None).unwrap();
        assert_eq!(results.len(), 3);

        // Query with elevation filter
        let filter = ElevationBucket { min_m: 100.0, max_m: 101.0 };
        let filtered = index.query_radius(location, 100.0, Some(filter)).unwrap();
        assert_eq!(filtered.len(), 2); // Only obs 0 and 1
    }

    #[test]
    fn test_radius_query_multiple_cells() {
        let mut index = SpatialIndex::new();

        // NYC area
        let center = GeoPoint::new(40.7128, -74.0060);
        let nearby = GeoPoint::new(40.7260, -73.9897); // ~2km away

        index.insert(0, center, None).unwrap();
        index.insert(1, nearby, None).unwrap();

        // Query with large radius should include both
        let results = index.query_radius(center, 5000.0, None).unwrap();
        assert!(results.len() >= 1); // At least center cell
    }

    #[test]
    fn test_populated_cells() {
        let mut index = SpatialIndex::new();
        let loc1 = GeoPoint::new(40.7128, -74.0060);
        let loc2 = GeoPoint::new(40.7260, -73.9897);

        index.insert(0, loc1, Some(100.0)).unwrap();
        index.insert(1, loc2, Some(100.0)).unwrap();

        let cells = index.get_populated_cells();
        assert!(cells.len() >= 1);
    }

    #[test]
    fn test_clear() {
        let mut index = SpatialIndex::new();
        let location = GeoPoint::new(40.7128, -74.0060);

        index.insert(0, location, None).unwrap();
        assert!(!index.cells.is_empty());

        index.clear();
        assert!(index.cells.is_empty());
    }

    #[test]
    fn test_custom_resolution() {
        let index = SpatialIndex::with_resolution(6).unwrap();
        assert_eq!(index.h3_resolution, 6);

        let invalid = SpatialIndex::with_resolution(16);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_h3_consistency() {
        let index = SpatialIndex::new();
        let location = GeoPoint::new(40.7128, -74.0060);

        // Same location should produce same H3 cell
        let cell1 = index.location_to_h3(location).unwrap();
        let cell2 = index.location_to_h3(location).unwrap();
        assert_eq!(cell1, cell2);
    }
}
