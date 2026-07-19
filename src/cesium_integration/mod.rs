//! Cesium.js integration for web-based 3D visualization
//!
//! Provides configuration, styling, and API support for rendering
//! PyTerrainMap data in Cesium web viewers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cesium viewer configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CesiumConfig {
    /// Viewer element ID (HTML div to mount Cesium)
    pub element_id: String,
    /// Initial camera position
    pub camera: CameraPosition,
    /// Terrain provider (Cesium Ion or custom)
    pub terrain_provider: TerrainProvider,
    /// Image base layer (satellite, street map, etc.)
    pub base_layer: BaseLayer,
    /// 3D Tiles layers to load
    pub tileset_layers: Vec<TilesetLayer>,
    /// Feature layers (GeoJSON, KML)
    pub feature_layers: Vec<FeatureLayer>,
    /// Viewer options
    pub options: ViewerOptions,
}

impl CesiumConfig {
    /// Create default Cesium configuration
    pub fn new(element_id: &str) -> Self {
        CesiumConfig {
            element_id: element_id.to_string(),
            camera: CameraPosition::default(),
            terrain_provider: TerrainProvider::default(),
            base_layer: BaseLayer::Bing,
            tileset_layers: Vec::new(),
            feature_layers: Vec::new(),
            options: ViewerOptions::default(),
        }
    }

    /// Add tileset layer
    pub fn add_tileset(&mut self, tileset: TilesetLayer) {
        self.tileset_layers.push(tileset);
    }

    /// Add feature layer (GeoJSON/KML)
    pub fn add_feature_layer(&mut self, layer: FeatureLayer) {
        self.feature_layers.push(layer);
    }

    /// Set base layer (background imagery)
    pub fn set_base_layer(&mut self, layer: BaseLayer) {
        self.base_layer = layer;
    }

    /// Set terrain provider
    pub fn set_terrain(&mut self, provider: TerrainProvider) {
        self.terrain_provider = provider;
    }

    /// Generate JavaScript initialization code
    pub fn generate_viewer_script(&self) -> String {
        let config_json = serde_json::to_string_pretty(self).unwrap_or_default();
        format!(
            r#"
// Initialize Cesium viewer
var viewer = new Cesium.Viewer('{}', {{
    terrainProvider: Cesium.Cesium3DTileset.fromUrl('{}'),
    imageryProvider: new Cesium.BingMapsImageryProvider({{
        url: 'https://dev.virtualearth.net/',
        key: 'YOUR_BING_MAPS_KEY'
    }})
}});

// Set camera position
viewer.camera.setView({{
    destination: Cesium.Cartesian3.fromDegrees({}, {}, {}),
    orientation: {{
        heading: Cesium.Math.toRadians({}),
        pitch: Cesium.Math.toRadians({}),
        roll: 0.0
    }}
}});

// Load tilesets
var tilesets = {};

// Configuration
var config = {};
"#,
            self.element_id,
            self.terrain_provider.url_or_default(),
            self.camera.longitude,
            self.camera.latitude,
            self.camera.altitude,
            self.camera.heading,
            self.camera.pitch,
            serde_json::to_string(&self.tileset_layers).unwrap_or_default(),
            config_json
        )
    }
}

impl Default for CesiumConfig {
    fn default() -> Self {
        Self::new("cesium-container")
    }
}

/// Camera position in geographic coordinates
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CameraPosition {
    /// Longitude (degrees)
    pub longitude: f64,
    /// Latitude (degrees)
    pub latitude: f64,
    /// Altitude above ground (meters)
    pub altitude: f64,
    /// Heading/azimuth (degrees, 0-360)
    pub heading: f64,
    /// Pitch/elevation angle (degrees, -90 to 90)
    pub pitch: f64,
}

impl CameraPosition {
    /// Create camera position
    pub fn new(lon: f64, lat: f64, alt: f64) -> Self {
        CameraPosition {
            longitude: lon,
            latitude: lat,
            altitude: alt,
            heading: 0.0,
            pitch: -45.0,
        }
    }

    /// Set heading (direction looking)
    pub fn with_heading(mut self, heading: f64) -> Self {
        self.heading = heading % 360.0;
        self
    }

    /// Set pitch (angle down from horizontal)
    pub fn with_pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch.max(-90.0).min(90.0);
        self
    }

    /// Create for a bounding box (fit all in view)
    pub fn fit_bounds(west: f64, south: f64, east: f64, north: f64) -> Self {
        let center_lon = (west + east) / 2.0;
        let center_lat = (south + north) / 2.0;
        let diag_degrees = ((east - west).powi(2) + (north - south).powi(2)).sqrt();
        let altitude = (diag_degrees * 111_000.0) / 2.0; // Rough conversion degrees to meters

        CameraPosition::new(center_lon, center_lat, altitude)
    }
}

impl Default for CameraPosition {
    fn default() -> Self {
        // Default to New York City
        CameraPosition::new(-74.0060, 40.7128, 5000.0)
    }
}

/// Terrain provider options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainProvider {
    /// Cesium Ion asset (requires API key)
    CesiumIon {
        asset_id: u32,
        access_token: String,
    },
    /// USGS elevation data
    USGS,
    /// Mapbox Terrain-RGB
    Mapbox {
        access_token: String,
    },
    /// Custom URL
    Custom {
        url: String,
    },
    /// No terrain (flat Earth)
    None,
}

impl TerrainProvider {
    /// Get URL for terrain provider
    pub fn url_or_default(&self) -> String {
        match self {
            TerrainProvider::CesiumIon { asset_id, access_token } => {
                format!(
                    "https://assets.cesium.com/1/{}?access_token={}",
                    asset_id, access_token
                )
            }
            TerrainProvider::USGS => {
                "https://elevation3d.arcgis.com/arcgis/rest/services/WorldElevation3D/WorldElevation3D/ImageServer".to_string()
            }
            TerrainProvider::Mapbox { access_token } => {
                format!(
                    "https://api.mapbox.com/raster/v1/mapbox.mapbox-terrain-v2/tilesets/mapbox.mapbox-terrain-v2{{z}}-{{x}}-{{y}}.webp?access_token={}",
                    access_token
                )
            }
            TerrainProvider::Custom { url } => url.clone(),
            TerrainProvider::None => String::new(),
        }
    }
}

impl Default for TerrainProvider {
    fn default() -> Self {
        TerrainProvider::USGS
    }
}

/// Base imagery layer
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BaseLayer {
    /// Bing Maps (satellite, street, etc.)
    Bing,
    /// OpenStreetMap
    OpenStreetMap,
    /// USGS Imagery
    USGSImagery,
    /// Mapbox (requires API key)
    Mapbox,
    /// Sentinel-2 satellite
    Sentinel2,
}

impl BaseLayer {
    /// Get provider name for Cesium
    pub fn provider_name(&self) -> &str {
        match self {
            BaseLayer::Bing => "BingMapsImageryProvider",
            BaseLayer::OpenStreetMap => "OpenStreetMapImageryProvider",
            BaseLayer::USGSImagery => "USGSImageryProvider",
            BaseLayer::Mapbox => "MapboxImageryProvider",
            BaseLayer::Sentinel2 => "IonImageryProvider",
        }
    }
}

/// 3D Tileset layer configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilesetLayer {
    /// Display name
    pub name: String,
    /// URL to tileset.json
    pub tileset_url: String,
    /// Display visibility
    pub visible: bool,
    /// Opacity (0.0-1.0)
    pub opacity: f32,
    /// Maximum screen-space error for LOD
    pub maximum_screen_space_error: f32,
    /// Properties to display in inspector
    pub properties: Option<HashMap<String, String>>,
}

impl TilesetLayer {
    /// Create tileset layer
    pub fn new(name: &str, url: &str) -> Self {
        TilesetLayer {
            name: name.to_string(),
            tileset_url: url.to_string(),
            visible: true,
            opacity: 1.0,
            maximum_screen_space_error: 16.0,
            properties: None,
        }
    }

    /// Set visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.max(0.0).min(1.0);
        self
    }

    /// Set LOD threshold
    pub fn with_screen_space_error(mut self, sse: f32) -> Self {
        self.maximum_screen_space_error = sse;
        self
    }

    /// Add property metadata
    pub fn add_property(&mut self, key: &str, value: &str) {
        if self.properties.is_none() {
            self.properties = Some(HashMap::new());
        }
        if let Some(ref mut props) = self.properties {
            props.insert(key.to_string(), value.to_string());
        }
    }
}

/// Feature layer (GeoJSON, KML)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureLayer {
    /// Display name
    pub name: String,
    /// Data source URL or inline GeoJSON
    pub data_source: DataSource,
    /// Display visibility
    pub visible: bool,
    /// Feature styling
    pub style: FeatureStyle,
}

impl FeatureLayer {
    /// Create feature layer
    pub fn new(name: &str, data_source: DataSource) -> Self {
        FeatureLayer {
            name: name.to_string(),
            data_source,
            visible: true,
            style: FeatureStyle::default(),
        }
    }

    /// Set visibility
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set style
    pub fn with_style(mut self, style: FeatureStyle) -> Self {
        self.style = style;
        self
    }
}

/// Feature data source type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataSource {
    /// GeoJSON URL
    GeoJSON {
        url: String,
    },
    /// KML URL
    KML {
        url: String,
    },
    /// Inline GeoJSON
    GeoJSONInline {
        data: serde_json::Value,
    },
    /// WMS (Web Map Service)
    WMS {
        url: String,
        layers: String,
    },
}

/// Feature styling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureStyle {
    /// Point/marker color (hex or CSS color)
    pub point_color: String,
    /// Point size (pixels)
    pub point_size: f32,
    /// Line color
    pub line_color: String,
    /// Line width (pixels)
    pub line_width: f32,
    /// Fill color for polygons
    pub fill_color: String,
    /// Fill opacity (0.0-1.0)
    pub fill_opacity: f32,
}

impl FeatureStyle {
    /// Create style
    pub fn new() -> Self {
        FeatureStyle {
            point_color: "#FF0000".to_string(),
            point_size: 8.0,
            line_color: "#00FF00".to_string(),
            line_width: 2.0,
            fill_color: "#0000FF".to_string(),
            fill_opacity: 0.3,
        }
    }

    /// Set point color (hex)
    pub fn with_point_color(mut self, color: &str) -> Self {
        self.point_color = color.to_string();
        self
    }

    /// Set line color (hex)
    pub fn with_line_color(mut self, color: &str) -> Self {
        self.line_color = color.to_string();
        self
    }

    /// Set fill color (hex)
    pub fn with_fill_color(mut self, color: &str) -> Self {
        self.fill_color = color.to_string();
        self
    }
}

impl Default for FeatureStyle {
    fn default() -> Self {
        Self::new()
    }
}

/// Viewer display options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewerOptions {
    /// Enable scene mode selector
    pub scene_mode_picker_enabled: bool,
    /// Enable geocoder (search)
    pub geocoder_enabled: bool,
    /// Enable home button
    pub home_button_enabled: bool,
    /// Enable fullscreen button
    pub fullscreen_button_enabled: bool,
    /// Enable info box (property inspector)
    pub info_box_enabled: bool,
    /// Enable timeline
    pub timeline_enabled: bool,
    /// Enable animation controls
    pub animation_enabled: bool,
    /// Background color (hex)
    pub background_color: String,
    /// Enable shadows
    pub shadows_enabled: bool,
    /// Enable fog
    pub fog_enabled: bool,
}

impl ViewerOptions {
    /// Create with all features enabled
    pub fn new() -> Self {
        ViewerOptions {
            scene_mode_picker_enabled: true,
            geocoder_enabled: true,
            home_button_enabled: true,
            fullscreen_button_enabled: true,
            info_box_enabled: true,
            timeline_enabled: false,
            animation_enabled: true,
            background_color: "#000000".to_string(),
            shadows_enabled: true,
            fog_enabled: true,
        }
    }

    /// Minimal UI
    pub fn minimal() -> Self {
        ViewerOptions {
            scene_mode_picker_enabled: false,
            geocoder_enabled: false,
            home_button_enabled: false,
            fullscreen_button_enabled: false,
            info_box_enabled: false,
            timeline_enabled: false,
            animation_enabled: false,
            background_color: "#000000".to_string(),
            shadows_enabled: false,
            fog_enabled: false,
        }
    }
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Cesium API response for tileset metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TilesetMetadata {
    pub name: String,
    pub description: String,
    pub bounds: BoundingBox,
    pub point_count: u32,
    pub last_updated: String,
    pub properties: HashMap<String, String>,
}

/// Bounding box for tileset
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BoundingBox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
    pub min_height: f64,
    pub max_height: f64,
}

impl BoundingBox {
    /// Create bounding box from geographic bounds
    pub fn new(west: f64, south: f64, east: f64, north: f64, min_h: f64, max_h: f64) -> Self {
        BoundingBox {
            west,
            south,
            east,
            north,
            min_height: min_h,
            max_height: max_h,
        }
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        ((self.west + self.east) / 2.0, (self.south + self.north) / 2.0)
    }

    /// Get size in degrees
    pub fn size(&self) -> (f64, f64) {
        (self.east - self.west, self.north - self.south)
    }
}

/// Cesium Ion asset reference
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CesiumIonAsset {
    /// Asset ID (numeric identifier)
    pub asset_id: u32,
    /// Asset name
    pub name: String,
    /// Asset type (TILESET, IMAGERY, TERRAIN, etc.)
    pub asset_type: String,
    /// Cesium Ion URL
    pub url: String,
    /// Access token
    pub access_token: String,
}

impl CesiumIonAsset {
    /// Create Cesium Ion asset reference
    pub fn new(asset_id: u32, name: &str, access_token: &str) -> Self {
        CesiumIonAsset {
            asset_id,
            name: name.to_string(),
            asset_type: "TILESET".to_string(),
            url: format!("https://assets.cesium.com/1/{}", asset_id),
            access_token: access_token.to_string(),
        }
    }

    /// Get full URL with token
    pub fn full_url(&self) -> String {
        format!("{}?access_token={}", self.url, self.access_token)
    }
}

/// Measurement tool configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasurementTool {
    /// Tool enabled
    pub enabled: bool,
    /// Measurement mode: "distance", "area", "angle"
    pub mode: String,
    /// Point marker color
    pub marker_color: String,
    /// Label color
    pub label_color: String,
}

impl MeasurementTool {
    /// Create distance measurement tool
    pub fn distance() -> Self {
        MeasurementTool {
            enabled: true,
            mode: "distance".to_string(),
            marker_color: "#00FF00".to_string(),
            label_color: "#FFFFFF".to_string(),
        }
    }

    /// Create area measurement tool
    pub fn area() -> Self {
        MeasurementTool {
            enabled: true,
            mode: "area".to_string(),
            marker_color: "#0000FF".to_string(),
            label_color: "#FFFFFF".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cesium_config_creation() {
        let config = CesiumConfig::new("viewer");
        assert_eq!(config.element_id, "viewer");
        assert_eq!(config.tileset_layers.len(), 0);
    }

    #[test]
    fn test_cesium_config_add_tileset() {
        let mut config = CesiumConfig::new("viewer");
        config.add_tileset(TilesetLayer::new("terrain", "http://example.com/tileset.json"));
        assert_eq!(config.tileset_layers.len(), 1);
    }

    #[test]
    fn test_cesium_config_base_layer() {
        let mut config = CesiumConfig::new("viewer");
        config.set_base_layer(BaseLayer::OpenStreetMap);
        assert_eq!(config.base_layer, BaseLayer::OpenStreetMap);
    }

    #[test]
    fn test_camera_position_creation() {
        let cam = CameraPosition::new(-74.0, 40.0, 5000.0);
        assert_eq!(cam.longitude, -74.0);
        assert_eq!(cam.latitude, 40.0);
    }

    #[test]
    fn test_camera_position_with_heading() {
        let cam = CameraPosition::new(-74.0, 40.0, 5000.0).with_heading(45.0);
        assert_eq!(cam.heading, 45.0);
    }

    #[test]
    fn test_camera_position_fit_bounds() {
        let cam = CameraPosition::fit_bounds(-74.0, 40.0, -73.0, 41.0);
        assert_eq!(cam.longitude, -73.5);
        assert_eq!(cam.latitude, 40.5);
        assert!(cam.altitude > 0.0);
    }

    #[test]
    fn test_camera_position_pitch_clamping() {
        let cam = CameraPosition::new(-74.0, 40.0, 5000.0).with_pitch(100.0);
        assert_eq!(cam.pitch, 90.0); // Clamped to max
    }

    #[test]
    fn test_terrain_provider_cesium_ion() {
        let provider = TerrainProvider::CesiumIon {
            asset_id: 1,
            access_token: "token123".to_string(),
        };
        let url = provider.url_or_default();
        assert!(url.contains("assets.cesium.com"));
        assert!(url.contains("token123"));
    }

    #[test]
    fn test_terrain_provider_usgs() {
        let provider = TerrainProvider::USGS;
        let url = provider.url_or_default();
        assert!(url.contains("arcgis"));
    }

    #[test]
    fn test_base_layer_provider_name() {
        assert_eq!(BaseLayer::Bing.provider_name(), "BingMapsImageryProvider");
        assert_eq!(BaseLayer::OpenStreetMap.provider_name(), "OpenStreetMapImageryProvider");
    }

    #[test]
    fn test_tileset_layer_creation() {
        let layer = TilesetLayer::new("terrain", "http://example.com/tileset.json");
        assert_eq!(layer.name, "terrain");
        assert!(layer.visible);
        assert_eq!(layer.opacity, 1.0);
    }

    #[test]
    fn test_tileset_layer_with_opacity() {
        let layer = TilesetLayer::new("terrain", "url").with_opacity(0.5);
        assert_eq!(layer.opacity, 0.5);
    }

    #[test]
    fn test_tileset_layer_opacity_clamping() {
        let layer = TilesetLayer::new("terrain", "url").with_opacity(1.5);
        assert_eq!(layer.opacity, 1.0);
    }

    #[test]
    fn test_feature_layer_creation() {
        let ds = DataSource::GeoJSON {
            url: "http://example.com/data.geojson".to_string(),
        };
        let layer = FeatureLayer::new("features", ds);
        assert_eq!(layer.name, "features");
        assert!(layer.visible);
    }

    #[test]
    fn test_feature_style_creation() {
        let style = FeatureStyle::new();
        assert_eq!(style.point_color, "#FF0000");
        assert_eq!(style.line_width, 2.0);
    }

    #[test]
    fn test_feature_style_with_colors() {
        let style = FeatureStyle::new()
            .with_point_color("#00FF00")
            .with_line_color("#0000FF");
        assert_eq!(style.point_color, "#00FF00");
        assert_eq!(style.line_color, "#0000FF");
    }

    #[test]
    fn test_viewer_options_default() {
        let opts = ViewerOptions::default();
        assert!(opts.geocoder_enabled);
        assert!(opts.info_box_enabled);
    }

    #[test]
    fn test_viewer_options_minimal() {
        let opts = ViewerOptions::minimal();
        assert!(!opts.geocoder_enabled);
        assert!(!opts.info_box_enabled);
    }

    #[test]
    fn test_bounding_box_creation() {
        let bbox = BoundingBox::new(-74.0, 40.0, -73.0, 41.0, 0.0, 100.0);
        assert_eq!(bbox.west, -74.0);
        assert_eq!(bbox.max_height, 100.0);
    }

    #[test]
    fn test_bounding_box_center() {
        let bbox = BoundingBox::new(-74.0, 40.0, -73.0, 41.0, 0.0, 100.0);
        let (lon, lat) = bbox.center();
        assert_eq!(lon, -73.5);
        assert_eq!(lat, 40.5);
    }

    #[test]
    fn test_bounding_box_size() {
        let bbox = BoundingBox::new(-74.0, 40.0, -73.0, 41.0, 0.0, 100.0);
        let (width, height) = bbox.size();
        assert_eq!(width, 1.0);
        assert_eq!(height, 1.0);
    }

    #[test]
    fn test_cesium_ion_asset() {
        let asset = CesiumIonAsset::new(123, "MyTerrain", "my-token");
        assert_eq!(asset.asset_id, 123);
        assert_eq!(asset.name, "MyTerrain");
        let url = asset.full_url();
        assert!(url.contains("my-token"));
    }

    #[test]
    fn test_measurement_tool_distance() {
        let tool = MeasurementTool::distance();
        assert_eq!(tool.mode, "distance");
        assert!(tool.enabled);
    }

    #[test]
    fn test_measurement_tool_area() {
        let tool = MeasurementTool::area();
        assert_eq!(tool.mode, "area");
    }

    #[test]
    fn test_cesium_config_serialization() {
        let config = CesiumConfig::default();
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());
    }

    #[test]
    fn test_tileset_layer_add_property() {
        let mut layer = TilesetLayer::new("test", "url");
        layer.add_property("source", "radar");
        assert!(layer.properties.is_some());
        let props = layer.properties.unwrap();
        assert_eq!(props.get("source").unwrap(), "radar");
    }

    #[test]
    fn test_data_source_geojson() {
        let ds = DataSource::GeoJSON {
            url: "data.geojson".to_string(),
        };
        match ds {
            DataSource::GeoJSON { url } => {
                assert_eq!(url, "data.geojson");
            }
            _ => panic!("Expected GeoJSON"),
        }
    }

    #[test]
    fn test_data_source_wms() {
        let ds = DataSource::WMS {
            url: "http://example.com/wms".to_string(),
            layers: "layer1,layer2".to_string(),
        };
        match ds {
            DataSource::WMS { url, layers } => {
                assert!(url.contains("example.com"));
                assert_eq!(layers, "layer1,layer2");
            }
            _ => panic!("Expected WMS"),
        }
    }
}
