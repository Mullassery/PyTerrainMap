//! 3D Tiles format for streaming massive 3D geospatial datasets
//!
//! Implements point cloud (PNTS) and batched 3D model (B3DM) tiling
//! with hierarchical LOD structure for web-based 3D visualization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 3D Tiles tileset root
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tileset {
    /// Version of 3D Tiles specification
    pub asset: AssetMetadata,
    /// Bounding volume of entire tileset
    pub boundingVolume: BoundingVolume,
    /// Geometric error (max screen-space error)
    pub geometricError: f32,
    /// Root tile
    pub root: Tile,
    /// Optional properties schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertyDefinition>>,
}

/// Asset metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Specification version (e.g., "1.0", "1.1")
    pub version: String,
    /// Generator name
    pub generator: String,
    /// Creation timestamp
    pub created: String,
    /// Last modification timestamp
    pub modified: String,
}

/// Bounding volume (sphere, box, or geographic region)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BoundingVolume {
    /// Sphere [cx, cy, cz, radius]
    Sphere {
        sphere: [f32; 4],
    },
    /// Axis-aligned box [cx, cy, cz, sx, sy, sz, ...] (3x3 matrix)
    Box {
        #[serde(rename = "box")]
        box_data: [f32; 12],
    },
    /// Geographic region [west, south, east, north, min_height, max_height]
    Region {
        region: [f64; 6],
    },
}

impl BoundingVolume {
    /// Create sphere bounding volume
    pub fn sphere(cx: f32, cy: f32, cz: f32, radius: f32) -> Self {
        BoundingVolume::Sphere {
            sphere: [cx, cy, cz, radius],
        }
    }

    /// Create box bounding volume
    pub fn box_region(cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32) -> Self {
        BoundingVolume::Box {
            box_data: [
                cx, cy, cz,
                sx, 0.0, 0.0,
                0.0, sy, 0.0,
                0.0, 0.0, sz,
            ],
        }
    }

    /// Create geographic region (WGS84)
    pub fn region(west_rad: f64, south_rad: f64, east_rad: f64, north_rad: f64, min_h: f64, max_h: f64) -> Self {
        BoundingVolume::Region {
            region: [west_rad, south_rad, east_rad, north_rad, min_h, max_h],
        }
    }
}

/// Property definition for tile features
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDefinition {
    pub r#type: String, // "BOOLEAN", "UINT8", "INT32", "FLOAT32", "FLOAT64", "STRING"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<serde_json::Value>,
}

/// Single 3D Tile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    /// Bounding volume
    pub boundingVolume: BoundingVolume,
    /// Geometric error for this tile
    pub geometricError: f32,
    /// Content reference (URL to .pnts, .b3dm, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    /// Refinement strategy: "REPLACE" or "ADD"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refine: Option<String>,
    /// Child tiles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Tile>>,
    /// Matrix transform (4x4)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<[f32; 16]>,
}

impl Tile {
    /// Create tile with bounding volume and content
    pub fn new(bounding_volume: BoundingVolume, geometric_error: f32) -> Self {
        Tile {
            boundingVolume: bounding_volume,
            geometricError: geometric_error,
            content: None,
            refine: None,
            children: None,
            transform: None,
        }
    }

    /// Set content URL
    pub fn with_content(mut self, uri: String) -> Self {
        self.content = Some(Content { uri });
        self
    }

    /// Set refinement strategy
    pub fn with_refinement(mut self, refine: String) -> Self {
        self.refine = Some(refine);
        self
    }

    /// Add child tile
    pub fn add_child(&mut self, child: Tile) {
        if self.children.is_none() {
            self.children = Some(Vec::new());
        }
        if let Some(ref mut children) = self.children {
            children.push(child);
        }
    }

    /// Set transform matrix
    pub fn with_transform(mut self, transform: [f32; 16]) -> Self {
        self.transform = Some(transform);
        self
    }
}

/// Tile content reference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub uri: String,
}

/// Point Cloud Tile (PNTS) header
#[derive(Clone, Debug)]
pub struct PNTSHeader {
    /// Magic number: "pnts"
    pub magic: [u8; 4],
    /// Format version (1)
    pub version: u32,
    /// Total byte size
    pub byte_length: u32,
    /// Feature Table JSON byte length
    pub feature_table_json_byte_length: u32,
    /// Feature Table binary byte length
    pub feature_table_binary_byte_length: u32,
    /// Batch Table JSON byte length
    pub batch_table_json_byte_length: u32,
    /// Batch Table binary byte length
    pub batch_table_binary_byte_length: u32,
}

impl PNTSHeader {
    /// Create PNTS header
    pub fn new(
        feature_json_len: u32,
        feature_bin_len: u32,
        batch_json_len: u32,
        batch_bin_len: u32,
    ) -> Self {
        let byte_length = 28 + feature_json_len + feature_bin_len + batch_json_len + batch_bin_len;
        PNTSHeader {
            magic: *b"pnts",
            version: 1,
            byte_length,
            feature_table_json_byte_length: feature_json_len,
            feature_table_binary_byte_length: feature_bin_len,
            batch_table_json_byte_length: batch_json_len,
            batch_table_binary_byte_length: batch_bin_len,
        }
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(28);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.feature_table_json_byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.feature_table_binary_byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.batch_table_json_byte_length.to_le_bytes());
        bytes.extend_from_slice(&self.batch_table_binary_byte_length.to_le_bytes());
        bytes
    }
}

/// Point cloud feature data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointCloudFeatures {
    /// Points per tile (each tile has max points)
    pub POINTS_LENGTH: u32,
    /// Quantized position format: "POSITION_QUANTIZED" (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub POSITION_QUANTIZED: Option<String>,
    /// RGB color format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub RGB: Option<String>,
    /// Constant RGBA (if all points have same color)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub CONSTANT_RGBA: Option<u32>,
    /// Normals (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub NORMAL: Option<String>,
    /// Batch IDs (for per-point properties)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub BATCH_ID: Option<String>,
    /// Quantization matrix (if quantized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub QUANTIZED_VOLUME_OFFSET: Option<[f64; 3]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub QUANTIZED_VOLUME_SCALE: Option<[f64; 3]>,
}

/// 3D Tiles exporter
pub struct TilesExporter {
    /// Root bounding volume
    pub root_bounds: BoundingVolume,
    /// Tile tree
    pub root_tile: Option<Tile>,
    /// Maximum points per tile
    pub max_points_per_tile: u32,
    /// Geometric error for LOD
    pub geometric_error_base: f32,
}

impl TilesExporter {
    /// Create exporter
    pub fn new(root_bounds: BoundingVolume, max_points: u32) -> Self {
        TilesExporter {
            root_bounds,
            root_tile: None,
            max_points_per_tile: max_points,
            geometric_error_base: 10.0,
        }
    }

    /// Create tileset from point cloud
    pub fn create_tileset(&self, tileset_name: &str) -> Tileset {
        let asset = AssetMetadata {
            version: "1.0".to_string(),
            generator: "PyTerrainMap v0.0.1".to_string(),
            created: chrono::Utc::now().to_rfc3339(),
            modified: chrono::Utc::now().to_rfc3339(),
        };

        let root_tile = Tile::new(self.root_bounds.clone(), self.geometric_error_base)
            .with_content(format!("{}/root.pnts", tileset_name))
            .with_refinement("REPLACE".to_string());

        Tileset {
            asset,
            boundingVolume: self.root_bounds.clone(),
            geometricError: self.geometric_error_base,
            root: root_tile,
            properties: None,
        }
    }

    /// Add child tiles for LOD hierarchy
    pub fn add_lod_hierarchy(&mut self, level: usize, depth: usize) {
        if level >= depth || self.root_tile.is_none() {
            return;
        }

        let _points_per_child = self.max_points_per_tile / 8; // Octree split
        let error_multiplier = 2.0_f32.powi(level as i32 + 1);

        // Create 8 child tiles (octree subdivision)
        for i in 0..8 {
            let offset_x = if i & 1 == 0 { -0.25 } else { 0.25 };
            let offset_y = if i & 2 == 0 { -0.25 } else { 0.25 };
            let offset_z = if i & 4 == 0 { -0.25 } else { 0.25 };

            let child_bounds = BoundingVolume::sphere(offset_x, offset_y, offset_z, 0.25);
            let child_error = self.geometric_error_base / error_multiplier;
            let child = Tile::new(child_bounds, child_error)
                .with_content(format!("tile_{}.pnts", i))
                .with_refinement("ADD".to_string());

            if let Some(ref mut root) = self.root_tile {
                root.add_child(child);
            }
        }
    }

    /// Get tileset.json
    pub fn get_tileset_json(&self, tileset_name: &str) -> Result<String, serde_json::Error> {
        let tileset = self.create_tileset(tileset_name);
        serde_json::to_string_pretty(&tileset)
    }
}

impl Default for TilesExporter {
    fn default() -> Self {
        Self::new(
            BoundingVolume::sphere(0.0, 0.0, 0.0, 100.0),
            1_000_000,
        )
    }
}

/// Point cloud quantization for compression
pub struct QuantizedPointCloud {
    /// Quantized positions (16-bit)
    pub positions: Vec<u16>,
    /// RGB colors (8-bit each)
    pub colors: Vec<u8>,
    /// Batch IDs
    pub batch_ids: Vec<u32>,
    /// Bounding box for quantization
    pub bounds_min: (f32, f32, f32),
    pub bounds_max: (f32, f32, f32),
}

impl QuantizedPointCloud {
    /// Create quantized point cloud
    pub fn new(point_count: usize) -> Self {
        QuantizedPointCloud {
            positions: Vec::with_capacity(point_count * 3),
            colors: Vec::with_capacity(point_count * 3),
            batch_ids: Vec::with_capacity(point_count),
            bounds_min: (f32::MAX, f32::MAX, f32::MAX),
            bounds_max: (f32::MIN, f32::MIN, f32::MIN),
        }
    }

    /// Add quantized point
    pub fn add_point(&mut self, pos: (f32, f32, f32), color: (u8, u8, u8), batch_id: u32) {
        // Update bounds
        self.bounds_min = (
            self.bounds_min.0.min(pos.0),
            self.bounds_min.1.min(pos.1),
            self.bounds_min.2.min(pos.2),
        );
        self.bounds_max = (
            self.bounds_max.0.max(pos.0),
            self.bounds_max.1.max(pos.1),
            self.bounds_max.2.max(pos.2),
        );

        // Quantize position to 16-bit
        let scale_x = (pos.0 - self.bounds_min.0) / (self.bounds_max.0 - self.bounds_min.0).max(1e-6);
        let scale_y = (pos.1 - self.bounds_min.1) / (self.bounds_max.1 - self.bounds_min.1).max(1e-6);
        let scale_z = (pos.2 - self.bounds_min.2) / (self.bounds_max.2 - self.bounds_min.2).max(1e-6);

        self.positions.push((scale_x * 65535.0) as u16);
        self.positions.push((scale_y * 65535.0) as u16);
        self.positions.push((scale_z * 65535.0) as u16);

        // Add color
        self.colors.push(color.0);
        self.colors.push(color.1);
        self.colors.push(color.2);

        // Add batch ID
        self.batch_ids.push(batch_id);
    }

    /// Get serialized binary data
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Serialize quantized positions (16-bit each)
        for pos in &self.positions {
            data.extend_from_slice(&pos.to_le_bytes());
        }

        // Serialize colors (8-bit each)
        data.extend_from_slice(&self.colors);

        // Serialize batch IDs (32-bit each)
        for id in &self.batch_ids {
            data.extend_from_slice(&id.to_le_bytes());
        }

        data
    }

    /// Get point count
    pub fn point_count(&self) -> usize {
        self.batch_ids.len()
    }
}

/// 3D Tiles statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilesStats {
    pub tile_count: u32,
    pub total_points: u32,
    pub max_level: u32,
    pub tileset_byte_size: u64,
    pub compression_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_volume_sphere() {
        let bv = BoundingVolume::sphere(1.0, 2.0, 3.0, 5.0);
        match bv {
            BoundingVolume::Sphere { sphere } => {
                assert_eq!(sphere[0], 1.0);
                assert_eq!(sphere[3], 5.0);
            }
            _ => panic!("Expected sphere"),
        }
    }

    #[test]
    fn test_bounding_volume_box() {
        let bv = BoundingVolume::box_region(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        match bv {
            BoundingVolume::Box { box_data } => {
                assert_eq!(box_data[0], 1.0);
                assert_eq!(box_data[3], 4.0);
            }
            _ => panic!("Expected box"),
        }
    }

    #[test]
    fn test_bounding_volume_region() {
        let bv = BoundingVolume::region(-1.0, -0.5, 1.0, 0.5, 0.0, 100.0);
        match bv {
            BoundingVolume::Region { region } => {
                assert_eq!(region[0], -1.0);
                assert_eq!(region[5], 100.0);
            }
            _ => panic!("Expected region"),
        }
    }

    #[test]
    fn test_pnts_header() {
        let header = PNTSHeader::new(100, 200, 50, 100);
        assert_eq!(header.magic, *b"pnts");
        assert_eq!(header.version, 1);
        assert!(header.byte_length > 28);
    }

    #[test]
    fn test_pnts_header_to_bytes() {
        let header = PNTSHeader::new(100, 200, 50, 100);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 28);
        assert_eq!(&bytes[0..4], b"pnts");
    }

    #[test]
    fn test_tile_creation() {
        let bv = BoundingVolume::sphere(0.0, 0.0, 0.0, 10.0);
        let tile = Tile::new(bv, 5.0);
        assert_eq!(tile.geometricError, 5.0);
        assert!(tile.content.is_none());
    }

    #[test]
    fn test_tile_with_content() {
        let bv = BoundingVolume::sphere(0.0, 0.0, 0.0, 10.0);
        let tile = Tile::new(bv, 5.0).with_content("data.pnts".to_string());
        assert!(tile.content.is_some());
        assert_eq!(tile.content.unwrap().uri, "data.pnts");
    }

    #[test]
    fn test_tile_with_refinement() {
        let bv = BoundingVolume::sphere(0.0, 0.0, 0.0, 10.0);
        let tile = Tile::new(bv, 5.0).with_refinement("REPLACE".to_string());
        assert_eq!(tile.refine.unwrap(), "REPLACE");
    }

    #[test]
    fn test_tile_add_child() {
        let bv = BoundingVolume::sphere(0.0, 0.0, 0.0, 10.0);
        let mut parent = Tile::new(bv.clone(), 5.0);
        let child = Tile::new(bv, 2.5);
        parent.add_child(child);
        assert_eq!(parent.children.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_tiles_exporter_creation() {
        let exporter = TilesExporter::new(
            BoundingVolume::sphere(0.0, 0.0, 0.0, 100.0),
            1_000_000,
        );
        assert_eq!(exporter.max_points_per_tile, 1_000_000);
    }

    #[test]
    fn test_tiles_exporter_create_tileset() {
        let mut exporter = TilesExporter::new(
            BoundingVolume::sphere(0.0, 0.0, 0.0, 100.0),
            1_000_000,
        );
        let tileset = exporter.create_tileset("test");
        assert_eq!(tileset.asset.version, "1.0");
        assert!(tileset.root.content.is_some());
    }

    #[test]
    fn test_quantized_point_cloud() {
        let mut qpc = QuantizedPointCloud::new(10);
        qpc.add_point((1.0, 2.0, 3.0), (255, 128, 64), 0);
        assert_eq!(qpc.point_count(), 1);
        assert_eq!(qpc.positions.len(), 3);
        assert_eq!(qpc.colors.len(), 3);
    }

    #[test]
    fn test_quantized_point_cloud_binary() {
        let mut qpc = QuantizedPointCloud::new(2);
        qpc.add_point((1.0, 2.0, 3.0), (255, 128, 64), 0);
        qpc.add_point((4.0, 5.0, 6.0), (100, 150, 200), 1);
        let binary = qpc.to_binary();
        // 2 points × 3 positions × 2 bytes + 2 points × 3 colors × 1 byte + 2 points × 4 bytes
        assert!(binary.len() > 0);
    }

    #[test]
    fn test_asset_metadata() {
        let asset = AssetMetadata {
            version: "1.0".to_string(),
            generator: "PyTerrainMap".to_string(),
            created: "2025-01-01T00:00:00Z".to_string(),
            modified: "2025-01-01T00:00:00Z".to_string(),
        };
        assert_eq!(asset.version, "1.0");
    }

    #[test]
    fn test_tileset_serialization() {
        let mut exporter = TilesExporter::new(
            BoundingVolume::sphere(0.0, 0.0, 0.0, 100.0),
            1_000_000,
        );
        let json = exporter.get_tileset_json("test");
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("\"version\""));
        assert!(json_str.contains("PyTerrainMap"));
    }

    #[test]
    fn test_point_cloud_features() {
        let features = PointCloudFeatures {
            POINTS_LENGTH: 1000,
            POSITION_QUANTIZED: Some("POSITION_QUANTIZED".to_string()),
            RGB: Some("RGB".to_string()),
            CONSTANT_RGBA: None,
            NORMAL: None,
            BATCH_ID: Some("BATCH_ID".to_string()),
            QUANTIZED_VOLUME_OFFSET: Some([0.0, 0.0, 0.0]),
            QUANTIZED_VOLUME_SCALE: Some([1.0, 1.0, 1.0]),
        };
        assert_eq!(features.POINTS_LENGTH, 1000);
    }
}
