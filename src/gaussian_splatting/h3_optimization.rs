//! H3 spatial indexing optimization for Gaussian Splatting queries
//!
//! Uses hierarchical H3 spatial indexing to accelerate radius queries,
//! range scans, and multi-resolution queries across the world model.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// H3 resolution levels for multi-scale indexing
/// Resolution 0 (global) to 15 (~1m hexagons)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct H3Resolution(pub u8);

impl H3Resolution {
    /// Coarse resolution: 15km hexagons (for global queries)
    pub const COARSE: H3Resolution = H3Resolution(2);
    /// Medium resolution: ~1km hexagons (for regional queries)
    pub const MEDIUM: H3Resolution = H3Resolution(7);
    /// Fine resolution: ~100m hexagons (for local queries)
    pub const FINE: H3Resolution = H3Resolution(10);

    /// Estimate cell area in square kilometers
    pub fn estimated_area_km2(&self) -> f64 {
        match self.0 {
            0 => 4_250_546.0,
            1 => 607_220.0,
            2 => 86_745.0,
            3 => 12_392.0,
            4 => 1_770.0,
            5 => 253.0,
            6 => 36.2,
            7 => 5.16,
            8 => 0.737,
            9 => 0.105,
            10 => 0.015,
            11 => 0.002,
            12 => 0.0003,
            13 => 0.00004,
            14 => 0.000006,
            15 => 0.000001,
            _ => 0.0,
        }
    }
}

/// Multi-resolution spatial index using H3
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H3SpatialIndex {
    /// Mapping from H3 cell ID to list of splat IDs at each resolution
    index_coarse: HashMap<u64, Vec<String>>,   // H3 resolution 2
    index_medium: HashMap<u64, Vec<String>>,   // H3 resolution 7
    index_fine: HashMap<u64, Vec<String>>,     // H3 resolution 10

    /// Statistics
    pub total_indexed: usize,
    pub coarse_cells: usize,
    pub medium_cells: usize,
    pub fine_cells: usize,
}

impl H3SpatialIndex {
    /// Create new H3 spatial index
    pub fn new() -> Self {
        H3SpatialIndex {
            index_coarse: HashMap::new(),
            index_medium: HashMap::new(),
            index_fine: HashMap::new(),
            total_indexed: 0,
            coarse_cells: 0,
            medium_cells: 0,
            fine_cells: 0,
        }
    }

    /// Convert lat/lon/elevation to H3 cell ID at specified resolution
    /// (In production, this would use the real h3o crate)
    fn lat_lon_to_h3(&self, lat: f64, lon: f64, resolution: u8) -> u64 {
        // Simplified hash-based approach for demo
        // In production: use h3o::LatLng::new(lat, lon)?.to_cell(resolution)?
        let lat_bits = ((lat + 90.0) * 1000.0) as u64;
        let lon_bits = ((lon + 180.0) * 1000.0) as u64;
        let res_bits = (resolution as u64) << 56;
        res_bits | (lat_bits << 32) | lon_bits
    }

    /// Index a splat at all resolution levels
    pub fn index_splat(&mut self, splat_id: &str, lat: f64, lon: f64) {
        let coarse_cell = self.lat_lon_to_h3(lat, lon, H3Resolution::COARSE.0);
        let medium_cell = self.lat_lon_to_h3(lat, lon, H3Resolution::MEDIUM.0);
        let fine_cell = self.lat_lon_to_h3(lat, lon, H3Resolution::FINE.0);

        self.index_coarse
            .entry(coarse_cell)
            .or_insert_with(Vec::new)
            .push(splat_id.to_string());

        self.index_medium
            .entry(medium_cell)
            .or_insert_with(Vec::new)
            .push(splat_id.to_string());

        self.index_fine
            .entry(fine_cell)
            .or_insert_with(Vec::new)
            .push(splat_id.to_string());

        self.total_indexed += 1;
        self.coarse_cells = self.index_coarse.len();
        self.medium_cells = self.index_medium.len();
        self.fine_cells = self.index_fine.len();
    }

    /// Query splats in radius using H3 spatial index
    ///
    /// Strategy: Use medium resolution to find candidate cells, then filter
    pub fn query_radius_candidates(
        &self,
        lat: f64,
        lon: f64,
        radius_m: f64,
    ) -> Vec<String> {
        // Choose resolution based on radius
        let resolution = if radius_m > 10000.0 {
            H3Resolution::COARSE
        } else if radius_m > 1000.0 {
            H3Resolution::MEDIUM
        } else {
            H3Resolution::FINE
        };

        // Get center cell
        let center_cell = self.lat_lon_to_h3(lat, lon, resolution.0);

        // Get candidates from index
        let index = match resolution {
            H3Resolution::COARSE => &self.index_coarse,
            H3Resolution::MEDIUM => &self.index_medium,
            _ => &self.index_fine,
        };

        // For simplicity, return all splats in center cell and neighboring cells
        let mut candidates = Vec::new();

        if let Some(splats) = index.get(&center_cell) {
            candidates.extend(splats.clone());
        }

        // In production, would also query neighboring cells via h3o::Cell::neighbors()
        candidates
    }

    /// Get statistics on index fragmentation
    pub fn stats(&self) -> H3IndexStats {
        let avg_splats_per_coarse = if self.coarse_cells > 0 {
            self.total_indexed as f32 / self.coarse_cells as f32
        } else {
            0.0
        };

        let avg_splats_per_fine = if self.fine_cells > 0 {
            self.total_indexed as f32 / self.fine_cells as f32
        } else {
            0.0
        };

        H3IndexStats {
            total_splats: self.total_indexed,
            coarse_cells: self.coarse_cells,
            medium_cells: self.medium_cells,
            fine_cells: self.fine_cells,
            avg_splats_per_coarse,
            avg_splats_per_fine,
            index_memory_est_kb: self.estimate_memory_kb(),
        }
    }

    /// Estimate memory used by index
    fn estimate_memory_kb(&self) -> f32 {
        let coarse_size = self.index_coarse.len() * (8 + 24);  // cell ID + vec overhead
        let medium_size = self.index_medium.len() * (8 + 24);
        let fine_size = self.index_fine.len() * (8 + 24);
        let string_size = self.total_indexed * 24;  // average string + heap allocation

        ((coarse_size + medium_size + fine_size + string_size) as f32) / 1024.0
    }

    /// Clear index
    pub fn clear(&mut self) {
        self.index_coarse.clear();
        self.index_medium.clear();
        self.index_fine.clear();
        self.total_indexed = 0;
        self.coarse_cells = 0;
        self.medium_cells = 0;
        self.fine_cells = 0;
    }
}

impl Default for H3SpatialIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about H3 index state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct H3IndexStats {
    pub total_splats: usize,
    pub coarse_cells: usize,
    pub medium_cells: usize,
    pub fine_cells: usize,
    pub avg_splats_per_coarse: f32,
    pub avg_splats_per_fine: f32,
    pub index_memory_est_kb: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h3_resolution_areas() {
        assert!(H3Resolution::COARSE.estimated_area_km2() > H3Resolution::MEDIUM.estimated_area_km2());
        assert!(H3Resolution::MEDIUM.estimated_area_km2() > H3Resolution::FINE.estimated_area_km2());
    }

    #[test]
    fn test_index_creation() {
        let index = H3SpatialIndex::new();
        assert_eq!(index.total_indexed, 0);
    }

    #[test]
    fn test_splat_indexing() {
        let mut index = H3SpatialIndex::new();

        // Index 100 splats
        for i in 0..100 {
            index.index_splat(&format!("splat_{}", i), 40.0 + i as f64 * 0.001, -74.0);
        }

        assert_eq!(index.total_indexed, 100);
        assert!(index.coarse_cells > 0);
        assert!(index.medium_cells > 0);
        assert!(index.fine_cells > 0);
    }

    #[test]
    fn test_query_candidates() {
        let mut index = H3SpatialIndex::new();

        // Index 50 splats in a region
        for i in 0..50 {
            index.index_splat(&format!("splat_{}", i), 40.0, -74.0 + i as f64 * 0.001);
        }

        // Query with different radii
        let candidates_small = index.query_radius_candidates(40.0, -74.0, 500.0);
        let candidates_large = index.query_radius_candidates(40.0, -74.0, 20000.0);

        assert!(!candidates_small.is_empty());
        assert!(!candidates_large.is_empty());
    }

    #[test]
    fn test_index_stats() {
        let mut index = H3SpatialIndex::new();

        for i in 0..1000 {
            index.index_splat(&format!("splat_{}", i), 40.0 + (i % 100) as f64 * 0.001, -74.0);
        }

        let stats = index.stats();
        assert_eq!(stats.total_splats, 1000);
        assert!(stats.index_memory_est_kb > 0.0);
    }

    #[test]
    fn test_index_clear() {
        let mut index = H3SpatialIndex::new();

        for i in 0..100 {
            index.index_splat(&format!("splat_{}", i), 40.0, -74.0);
        }

        assert_eq!(index.total_indexed, 100);

        index.clear();
        assert_eq!(index.total_indexed, 0);
        assert_eq!(index.coarse_cells, 0);
    }
}
