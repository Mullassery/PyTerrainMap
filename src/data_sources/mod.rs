//! External geospatial data source integration
//!
//! Enables layering of external data from OSM, elevation, satellite imagery,
//! street-view imagery, addresses, and other sources to enrich terrain maps.

use crate::types::{GeoPoint, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// External data source type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSourceType {
    /// OpenStreetMap: roads, buildings, POIs, land use
    OpenStreetMap,
    /// SRTM/USGS elevation data (30m resolution)
    ElevationSRTM,
    /// Copernicus Sentinel-2 satellite imagery (10-60m resolution)
    SatelliteSentinel2,
    /// Landsat satellite imagery (30m resolution)
    SatelliteLandsat,
    /// Planet Labs high-resolution satellite (3m resolution)
    SatellitePlanet,
    /// Mapillary street-view imagery (crowdsourced)
    StreetViewMapillary,
    /// OpenAerialMap drone imagery
    DroneImageryOAM,
    /// GeoNames: place names and geographic features
    GeoNames,
    /// OpenAddresses: address datasets
    OpenAddresses,
    /// Natural Earth: boundaries, rivers, physical geography
    NaturalEarth,
    /// NASA Earthdata climate/terrain datasets
    NASAEarthdata,
    /// Custom local dataset (LiDAR, orthophoto, etc.)
    Custom,
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::OpenStreetMap => write!(f, "OpenStreetMap"),
            DataSourceType::ElevationSRTM => write!(f, "SRTM/USGS Elevation"),
            DataSourceType::SatelliteSentinel2 => write!(f, "Sentinel-2"),
            DataSourceType::SatelliteLandsat => write!(f, "Landsat"),
            DataSourceType::SatellitePlanet => write!(f, "Planet Labs"),
            DataSourceType::StreetViewMapillary => write!(f, "Mapillary"),
            DataSourceType::DroneImageryOAM => write!(f, "OpenAerialMap"),
            DataSourceType::GeoNames => write!(f, "GeoNames"),
            DataSourceType::OpenAddresses => write!(f, "OpenAddresses"),
            DataSourceType::NaturalEarth => write!(f, "Natural Earth"),
            DataSourceType::NASAEarthdata => write!(f, "NASA Earthdata"),
            DataSourceType::Custom => write!(f, "Custom"),
        }
    }
}

/// Data geometry type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GeometryType {
    /// Point feature
    Point { lat: f64, lon: f64 },
    /// LineString (e.g., road)
    LineString { coordinates: Vec<(f64, f64)> },
    /// Polygon (e.g., building, land use)
    Polygon { coordinates: Vec<Vec<(f64, f64)>> },
    /// Raster grid (satellite imagery, elevation)
    Raster {
        bounds: (f64, f64, f64, f64), // (min_lat, min_lon, max_lat, max_lon)
        resolution_m: f32,
    },
}

/// External feature from a data source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalFeature {
    /// Source of this feature
    pub source: DataSourceType,
    /// Feature type (e.g., "road", "building", "POI", "elevation")
    pub feature_type: String,
    /// Geometry
    pub geometry: GeometryType,
    /// Feature properties
    pub properties: HashMap<String, String>,
    /// Confidence in this data (0.0-1.0)
    pub confidence: f32,
    /// Data timestamp (when source was last updated)
    pub timestamp: Option<i64>,
    /// Resolution in meters (for raster data)
    pub resolution_m: Option<f32>,
}

impl ExternalFeature {
    /// Create point feature
    pub fn point(
        source: DataSourceType,
        feature_type: &str,
        location: GeoPoint,
    ) -> Self {
        ExternalFeature {
            source,
            feature_type: feature_type.to_string(),
            geometry: GeometryType::Point {
                lat: location.lat,
                lon: location.lon,
            },
            properties: HashMap::new(),
            confidence: 0.95,
            timestamp: None,
            resolution_m: None,
        }
    }

    /// Create linestring feature (e.g., road)
    pub fn linestring(
        source: DataSourceType,
        feature_type: &str,
        coordinates: Vec<(f64, f64)>,
    ) -> Self {
        ExternalFeature {
            source,
            feature_type: feature_type.to_string(),
            geometry: GeometryType::LineString { coordinates },
            properties: HashMap::new(),
            confidence: 0.95,
            timestamp: None,
            resolution_m: None,
        }
    }

    /// Create polygon feature (e.g., building)
    pub fn polygon(
        source: DataSourceType,
        feature_type: &str,
        coordinates: Vec<Vec<(f64, f64)>>,
    ) -> Self {
        ExternalFeature {
            source,
            feature_type: feature_type.to_string(),
            geometry: GeometryType::Polygon { coordinates },
            properties: HashMap::new(),
            confidence: 0.95,
            timestamp: None,
            resolution_m: None,
        }
    }

    /// Create raster feature (e.g., satellite imagery, elevation)
    pub fn raster(
        source: DataSourceType,
        feature_type: &str,
        bounds: (f64, f64, f64, f64),
        resolution_m: f32,
    ) -> Self {
        ExternalFeature {
            source,
            feature_type: feature_type.to_string(),
            geometry: GeometryType::Raster {
                bounds,
                resolution_m,
            },
            properties: HashMap::new(),
            confidence: 0.90,
            timestamp: None,
            resolution_m: Some(resolution_m),
        }
    }

    /// Add property
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.max(0.0).min(1.0);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Intersects with location (point in feature bounds)
    pub fn intersects(&self, location: GeoPoint) -> bool {
        match &self.geometry {
            GeometryType::Point { lat, lon } => {
                (location.lat - lat).abs() < 0.001 && (location.lon - lon).abs() < 0.001
            }
            GeometryType::Raster { bounds, .. } => {
                location.lat >= bounds.0
                    && location.lat <= bounds.2
                    && location.lon >= bounds.1
                    && location.lon <= bounds.3
            }
            _ => false, // Complex geometry intersection would need proper library
        }
    }
}

/// Data source configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// Source type
    pub source_type: DataSourceType,
    /// API endpoint or file path
    pub endpoint: String,
    /// API key (if required)
    pub api_key: Option<String>,
    /// Confidence weight (0.0-1.0) for fusion
    pub weight: f32,
    /// Enabled for queries
    pub enabled: bool,
    /// Cache size (number of features)
    pub cache_size: usize,
    /// Refresh interval (seconds)
    pub refresh_interval_secs: u64,
}

impl DataSourceConfig {
    /// Create config for OSM (free, no API key)
    pub fn openstreetmap() -> Self {
        DataSourceConfig {
            source_type: DataSourceType::OpenStreetMap,
            endpoint: "https://api.openstreetmap.org/api/0.6".to_string(),
            api_key: None,
            weight: 0.9,
            enabled: true,
            cache_size: 10000,
            refresh_interval_secs: 86400, // Daily
        }
    }

    /// Create config for SRTM elevation
    pub fn srtm() -> Self {
        DataSourceConfig {
            source_type: DataSourceType::ElevationSRTM,
            endpoint: "https://raster.nationalmap.gov/arcgis/rest/services/elevation".to_string(),
            api_key: None,
            weight: 0.95,
            enabled: true,
            cache_size: 5000,
            refresh_interval_secs: 604800, // Weekly
        }
    }

    /// Create config for Sentinel-2 (free)
    pub fn sentinel2() -> Self {
        DataSourceConfig {
            source_type: DataSourceType::SatelliteSentinel2,
            endpoint: "https://scihub.copernicus.eu/dhus".to_string(),
            api_key: None,
            weight: 0.85,
            enabled: true,
            cache_size: 1000,
            refresh_interval_secs: 432000, // 5 days (revisit time)
        }
    }

    /// Create config for Mapillary (free)
    pub fn mapillary(api_key: String) -> Self {
        DataSourceConfig {
            source_type: DataSourceType::StreetViewMapillary,
            endpoint: "https://api.mapillary.com/v4".to_string(),
            api_key: Some(api_key),
            weight: 0.75,
            enabled: true,
            cache_size: 2000,
            refresh_interval_secs: 604800, // Weekly
        }
    }

    /// Create config for GeoNames (free)
    pub fn geonames() -> Self {
        DataSourceConfig {
            source_type: DataSourceType::GeoNames,
            endpoint: "http://api.geonames.org/".to_string(),
            api_key: None,
            weight: 0.8,
            enabled: true,
            cache_size: 5000,
            refresh_interval_secs: 2592000, // Monthly
        }
    }

    /// Set as disabled
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Data source registry
pub struct DataSourceRegistry {
    sources: HashMap<DataSourceType, DataSourceConfig>,
    cache: HashMap<String, Vec<ExternalFeature>>,
}

impl DataSourceRegistry {
    /// Create new registry
    pub fn new() -> Self {
        DataSourceRegistry {
            sources: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    /// Add data source
    pub fn add_source(&mut self, config: DataSourceConfig) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }

        self.sources.insert(config.source_type, config);
        Ok(())
    }

    /// Get features near location from all sources
    pub fn get_features_at(
        &self,
        location: GeoPoint,
        radius_m: f32,
    ) -> Result<Vec<ExternalFeature>> {
        let mut features = Vec::new();

        for (_, config) in &self.sources {
            if !config.enabled {
                continue;
            }

            let cache_key = format!("{:.4}_{:.4}_{}", location.lat, location.lon, radius_m);
            if let Some(cached) = self.cache.get(&cache_key) {
                features.extend(cached.clone());
            }
        }

        Ok(features)
    }

    /// Get features of specific type
    pub fn get_features_by_type(&self, feature_type: &str) -> Vec<ExternalFeature> {
        self.cache
            .values()
            .flat_map(|features| {
                features
                    .iter()
                    .filter(|f| f.feature_type == feature_type)
                    .cloned()
            })
            .collect()
    }

    /// Get all enabled sources
    pub fn enabled_sources(&self) -> Vec<DataSourceType> {
        self.sources
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(source_type, _)| *source_type)
            .collect()
    }

    /// Get source config
    pub fn get_config(&self, source_type: DataSourceType) -> Option<&DataSourceConfig> {
        self.sources.get(&source_type)
    }
}

impl Default for DataSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Context enrichment from external data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextEnrichment {
    /// Nearby roads (OSM)
    pub nearby_roads: Vec<String>,
    /// Nearby buildings (OSM)
    pub nearby_buildings: Vec<String>,
    /// Nearby POIs (OSM + GeoNames)
    pub nearby_pois: Vec<String>,
    /// Terrain type (from satellite imagery or Natural Earth)
    pub terrain_type: Option<String>,
    /// Land use classification (OSM)
    pub land_use: Option<String>,
    /// Elevation from SRTM (meters)
    pub elevation_srtm: Option<f32>,
    /// Recent satellite imagery available
    pub satellite_coverage: Option<String>, // "Sentinel-2", "Landsat", etc.
    /// Street-view coverage (Mapillary)
    pub street_view_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_feature_point() {
        let feature = ExternalFeature::point(
            DataSourceType::GeoNames,
            "city",
            GeoPoint::new(40.7128, -74.0060),
        );

        assert_eq!(feature.source, DataSourceType::GeoNames);
        assert_eq!(feature.feature_type, "city");
        assert_eq!(feature.confidence, 0.95);
    }

    #[test]
    fn test_external_feature_linestring() {
        let coords = vec![(40.0, -74.0), (41.0, -73.0)];
        let feature =
            ExternalFeature::linestring(DataSourceType::OpenStreetMap, "road", coords.clone());

        assert_eq!(feature.source, DataSourceType::OpenStreetMap);
        assert_eq!(feature.feature_type, "road");
        assert!(matches!(feature.geometry, GeometryType::LineString { .. }));
    }

    #[test]
    fn test_external_feature_polygon() {
        let coords = vec![vec![(40.0, -74.0), (40.1, -74.0), (40.1, -73.9), (40.0, -74.0)]];
        let feature = ExternalFeature::polygon(
            DataSourceType::OpenStreetMap,
            "building",
            coords.clone(),
        );

        assert_eq!(feature.feature_type, "building");
        assert!(matches!(feature.geometry, GeometryType::Polygon { .. }));
    }

    #[test]
    fn test_external_feature_raster() {
        let bounds = (40.0, -74.0, 41.0, -73.0);
        let feature = ExternalFeature::raster(
            DataSourceType::SatelliteSentinel2,
            "satellite_image",
            bounds,
            10.0,
        );

        assert_eq!(feature.resolution_m, Some(10.0));
        assert!(matches!(feature.geometry, GeometryType::Raster { .. }));
    }

    #[test]
    fn test_feature_with_properties() {
        let feature = ExternalFeature::point(
            DataSourceType::OpenStreetMap,
            "POI",
            GeoPoint::new(40.7128, -74.0060),
        )
        .with_property("name", "New York")
        .with_property("type", "city")
        .with_confidence(0.98)
        .with_timestamp(1000000);

        assert_eq!(feature.properties.get("name"), Some(&"New York".to_string()));
        assert_eq!(feature.confidence, 0.98);
        assert_eq!(feature.timestamp, Some(1000000));
    }

    #[test]
    fn test_feature_intersects() {
        let location = GeoPoint::new(40.7128, -74.0060);
        let feature = ExternalFeature::point(DataSourceType::GeoNames, "city", location);

        assert!(feature.intersects(location));
        assert!(!feature.intersects(GeoPoint::new(50.0, -80.0)));
    }

    #[test]
    fn test_raster_intersects() {
        let bounds = (40.0, -74.0, 41.0, -73.0);
        let feature =
            ExternalFeature::raster(DataSourceType::SatelliteSentinel2, "image", bounds, 10.0);

        assert!(feature.intersects(GeoPoint::new(40.5, -73.5)));
        assert!(!feature.intersects(GeoPoint::new(42.0, -72.0)));
    }

    #[test]
    fn test_datasource_config_openstreetmap() {
        let config = DataSourceConfig::openstreetmap();
        assert_eq!(config.source_type, DataSourceType::OpenStreetMap);
        assert!(config.enabled);
        assert_eq!(config.weight, 0.9);
    }

    #[test]
    fn test_datasource_config_srtm() {
        let config = DataSourceConfig::srtm();
        assert_eq!(config.source_type, DataSourceType::ElevationSRTM);
        assert_eq!(config.weight, 0.95);
    }

    #[test]
    fn test_datasource_config_sentinel2() {
        let config = DataSourceConfig::sentinel2();
        assert_eq!(config.source_type, DataSourceType::SatelliteSentinel2);
        assert_eq!(config.weight, 0.85);
    }

    #[test]
    fn test_datasource_config_mapillary() {
        let config = DataSourceConfig::mapillary("test_key".to_string());
        assert_eq!(config.source_type, DataSourceType::StreetViewMapillary);
        assert_eq!(config.api_key, Some("test_key".to_string()));
    }

    #[test]
    fn test_datasource_registry_add() {
        let mut registry = DataSourceRegistry::new();
        let config = DataSourceConfig::openstreetmap();

        registry.add_source(config).unwrap();
        assert_eq!(registry.enabled_sources().len(), 1);
    }

    #[test]
    fn test_datasource_registry_disabled_source() {
        let mut registry = DataSourceRegistry::new();
        let config = DataSourceConfig::openstreetmap().disabled();

        registry.add_source(config).unwrap();
        assert_eq!(registry.enabled_sources().len(), 0);
    }

    #[test]
    fn test_datasource_type_display() {
        assert_eq!(
            DataSourceType::OpenStreetMap.to_string(),
            "OpenStreetMap"
        );
        assert_eq!(
            DataSourceType::ElevationSRTM.to_string(),
            "SRTM/USGS Elevation"
        );
        assert_eq!(
            DataSourceType::SatelliteSentinel2.to_string(),
            "Sentinel-2"
        );
    }

    #[test]
    fn test_context_enrichment() {
        let context = ContextEnrichment {
            nearby_roads: vec!["Main St".to_string(), "5th Ave".to_string()],
            nearby_buildings: vec!["Building A".to_string()],
            nearby_pois: vec!["Park".to_string()],
            terrain_type: Some("urban".to_string()),
            land_use: Some("commercial".to_string()),
            elevation_srtm: Some(42.5),
            satellite_coverage: Some("Sentinel-2".to_string()),
            street_view_available: true,
        };

        assert_eq!(context.nearby_roads.len(), 2);
        assert_eq!(context.elevation_srtm, Some(42.5));
    }
}
