//! Export layer for spatial data visualization tools
//!
//! Supports popular GIS and web mapping formats:
//! - GeoJSON (universal vector, web mapping)
//! - KML (Google Earth, 3D visualization)
//! - GeoTIFF (raster, satellite imagery)
//! - Shapefile (ESRI, desktop GIS)
//! - 3D Tiles (Cesium, web 3D)

use crate::types::{Observation, FusedData, GeoPoint, SensorType, SensorValue};

/// Supported export formats
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    /// GeoJSON (RFC 7946) - universal vector format
    GeoJSON,
    /// KML (OGC standard) - Google Earth, 3D visualization
    KML,
    /// GeoTIFF - georeferenced raster format
    GeoTIFF,
    /// Shapefile (ESRI) - classic GIS format
    Shapefile,
    /// 3D Tiles - Cesium Web 3D format
    ThreeDTiles,
    /// OBJ - simple 3D mesh format
    OBJ,
    /// WKT - Well-Known Text (GIS interchange)
    WKT,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::GeoJSON => write!(f, "geojson"),
            ExportFormat::KML => write!(f, "kml"),
            ExportFormat::GeoTIFF => write!(f, "geotiff"),
            ExportFormat::Shapefile => write!(f, "shapefile"),
            ExportFormat::ThreeDTiles => write!(f, "3dtiles"),
            ExportFormat::OBJ => write!(f, "obj"),
            ExportFormat::WKT => write!(f, "wkt"),
        }
    }
}

/// GeoJSON FeatureCollection exporter
pub struct GeoJSONExporter;

impl GeoJSONExporter {
    /// Export observations as GeoJSON FeatureCollection
    pub fn export_observations(observations: &[&Observation]) -> String {
        let features: Vec<serde_json::Value> = observations
            .iter()
            .map(|obs| Self::observation_to_feature(obs))
            .collect();

        let feature_collection = serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
            "metadata": {
                "count": observations.len(),
                "format": "GeoJSON",
                "generated": chrono::Utc::now().to_rfc3339(),
            }
        });

        serde_json::to_string_pretty(&feature_collection).unwrap_or_default()
    }

    /// Export fused data as GeoJSON Point feature
    pub fn export_fused_data(location: GeoPoint, fused: &FusedData) -> String {
        let mut properties = serde_json::json!({
            "type": "fused_estimate",
            "activity_level": fused.activity_level,
        });

        if let Some(temp) = &fused.temperature {
            properties["temperature"] = serde_json::json!({
                "celsius": temp.celsius,
                "variance": temp.variance,
                "readings": temp.num_readings,
            });
        }

        properties["detections_count"] = serde_json::json!(fused.object_detections.len());

        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [location.lon, location.lat]
            },
            "properties": properties,
        });

        serde_json::to_string_pretty(&feature).unwrap_or_default()
    }

    fn observation_to_feature(obs: &Observation) -> serde_json::Value {
        let mut properties = serde_json::json!({
            "robot_id": obs.robot_id,
            "timestamp": obs.timestamp,
            "sensor_type": obs.sensor_type.to_string(),
            "confidence": obs.confidence,
        });

        // Add sensor-specific data
        match &obs.value {
            SensorValue::Temperature { celsius } => {
                properties["value_celsius"] = serde_json::json!(celsius);
            }
            SensorValue::LiDAR { distances_cm } => {
                properties["distance_count"] = serde_json::json!(distances_cm.len());
                properties["distance_min_cm"] = serde_json::json!(distances_cm.iter().min().unwrap_or(&0));
                properties["distance_max_cm"] = serde_json::json!(distances_cm.iter().max().unwrap_or(&0));
            }
            SensorValue::Ultrasonic { distance_cm } => {
                properties["distance_cm"] = serde_json::json!(distance_cm);
            }
            SensorValue::Camera { detections } => {
                properties["detections_count"] = serde_json::json!(detections.len());
                let classes: Vec<String> = detections.iter().map(|d| d.class_label.clone()).collect();
                properties["detected_classes"] = serde_json::json!(classes);
            }
            SensorValue::Movement { velocity, heading } => {
                properties["velocity"] = serde_json::json!(velocity);
                properties["heading"] = serde_json::json!(heading);
            }
        }

        // Add metadata
        for (key, value) in &obs.metadata {
            properties[key] = serde_json::json!(value);
        }

        serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [obs.location.lon, obs.location.lat]
            },
            "properties": properties,
        })
    }
}

/// KML exporter for Google Earth and 3D visualization
pub struct KMLExporter;

impl KMLExporter {
    /// Export observations as KML with placemarks
    pub fn export_observations(observations: &[&Observation]) -> String {
        let mut kml = String::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <name>PyTerrainMap Observations</name>
    <description>Exported observations from multi-robot terrain mapping</description>
"#,
        );

        for obs in observations {
            kml.push_str(&Self::observation_to_placemark(obs));
        }

        kml.push_str(
            r#"  </Document>
</kml>"#,
        );

        kml
    }

    fn observation_to_placemark(obs: &Observation) -> String {
        let sensor_name = obs.sensor_type.to_string();
        let icon = match obs.sensor_type {
            SensorType::Thermal => "https://maps.google.com/mapfiles/ms/icons/red-dot.png",
            SensorType::LiDAR => "https://maps.google.com/mapfiles/ms/icons/blue-dot.png",
            SensorType::Camera => "https://maps.google.com/mapfiles/ms/icons/yellow-dot.png",
            SensorType::Movement => "https://maps.google.com/mapfiles/ms/icons/green-dot.png",
            SensorType::Ultrasonic => "https://maps.google.com/mapfiles/ms/icons/purple-dot.png",
        };

        let description = match &obs.value {
            SensorValue::Temperature { celsius } => format!("Temperature: {:.1}°C", celsius),
            SensorValue::LiDAR { distances_cm } => {
                format!("LiDAR readings: {} points", distances_cm.len())
            }
            SensorValue::Ultrasonic { distance_cm } => format!("Distance: {}cm", distance_cm),
            SensorValue::Camera { detections } => {
                format!("Detections: {}", detections.len())
            }
            SensorValue::Movement { velocity, heading } => {
                format!("Velocity: {:.1} m/s, Heading: {:.1}°", velocity, heading)
            }
        };

        format!(
            r#"    <Placemark>
      <name>{} - {}</name>
      <description>{}</description>
      <TimeStamp>
        <when>{}</when>
      </TimeStamp>
      <Style>
        <IconStyle>
          <Icon>
            <href>{}</href>
          </Icon>
          <scale>{}</scale>
        </IconStyle>
      </Style>
      <Point>
        <coordinates>{},{}</coordinates>
      </Point>
    </Placemark>
"#,
            obs.robot_id,
            sensor_name,
            description,
            format_timestamp_iso(obs.timestamp),
            icon,
            obs.confidence,
            obs.location.lon,
            obs.location.lat
        )
    }
}

/// Shapefile export skeleton (requires external library)
pub struct ShapefileExporter;

impl ShapefileExporter {
    /// Shapefile export requires external dependency
    /// This is a placeholder for future implementation with shapefile crate
    pub fn export_observations(_observations: &[&Observation]) -> Result<String, String> {
        Err("Shapefile export requires external 'shapefile' crate dependency".to_string())
    }
}

/// 3D Tiles exporter skeleton
pub struct ThreeDTilesExporter;

impl ThreeDTilesExporter {
    /// 3D Tiles export for Cesium web viewer
    /// Skeleton implementation - requires point cloud to 3D tiles conversion
    pub fn export_tileset(observations: &[&Observation]) -> serde_json::Value {
        serde_json::json!({
            "asset": {
                "version": "1.0",
                "tilesetVersion": "1.0.0"
            },
            "geometricError": 100.0,
            "root": {
                "boundingVolume": {
                    "sphere": [0, 0, 0, 1000.0]
                },
                "geometricError": 50.0,
                "refine": "ADD",
                "children": [],
                "metadata": {
                    "observation_count": observations.len()
                }
            }
        })
    }
}

/// OBJ format exporter for 3D mesh visualization
pub struct OBJExporter;

impl OBJExporter {
    /// Export observations as OBJ point cloud
    pub fn export_observations(observations: &[&Observation]) -> String {
        let mut obj = String::from("# PyTerrainMap Point Cloud Export\n");
        obj.push_str(&format!("# {} observations\n", observations.len()));
        obj.push_str("# Format: vertex (v) x y z\n\n");

        for obs in observations {
            let z = obs.elevation_asl.unwrap_or(0.0);
            obj.push_str(&format!("v {} {} {}\n", obs.location.lat, obs.location.lon, z));
        }

        obj
    }
}

/// WKT (Well-Known Text) exporter for GIS interchange
pub struct WKTExporter;

impl WKTExporter {
    /// Export observation as WKT Point
    pub fn observation_to_wkt(obs: &Observation) -> String {
        format!(
            "POINT ({} {})",
            obs.location.lon, obs.location.lat
        )
    }

    /// Export observations as WKT MultiPoint
    pub fn export_observations(observations: &[&Observation]) -> String {
        let points: Vec<String> = observations
            .iter()
            .map(|obs| format!("({} {})", obs.location.lon, obs.location.lat))
            .collect();

        format!("MULTIPOINT ({})", points.join(", "))
    }
}

/// Main exporter interface
pub struct SpatialExporter;

impl SpatialExporter {
    /// Export observations to specified format
    pub fn export(
        observations: &[&Observation],
        format: ExportFormat,
    ) -> Result<String, String> {
        match format {
            ExportFormat::GeoJSON => Ok(GeoJSONExporter::export_observations(observations)),
            ExportFormat::KML => Ok(KMLExporter::export_observations(observations)),
            ExportFormat::WKT => Ok(WKTExporter::export_observations(observations)),
            ExportFormat::OBJ => Ok(OBJExporter::export_observations(observations)),
            ExportFormat::Shapefile => ShapefileExporter::export_observations(observations),
            ExportFormat::ThreeDTiles => {
                Ok(serde_json::to_string_pretty(
                    &ThreeDTilesExporter::export_tileset(observations),
                )
                .unwrap_or_default())
            }
            ExportFormat::GeoTIFF => Err("GeoTIFF export requires external 'gdal' binding".to_string()),
        }
    }
}

/// Convert microsecond timestamp to ISO 8601 format
fn format_timestamp_iso(timestamp_us: i64) -> String {
    use chrono::{DateTime, Utc};

    let secs = timestamp_us / 1_000_000;
    let nanos = ((timestamp_us % 1_000_000) * 1000) as u32;

    match DateTime::<Utc>::from_timestamp(secs, nanos) {
        Some(dt) => dt.to_rfc3339(),
        None => "1970-01-01T00:00:00Z".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ObjectDetection;

    fn create_test_observation(lat: f64, lon: f64, sensor: SensorType, value: SensorValue) -> Observation {
        Observation::new(
            "test_bot".to_string(),
            1000000,
            GeoPoint::new(lat, lon),
            Some(100.0),
            sensor,
            value,
            0.95,
        )
    }

    #[test]
    fn test_geojson_export_single() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let geojson = GeoJSONExporter::export_observations(&[&obs]);
        assert!(geojson.contains("FeatureCollection"));
        assert!(geojson.contains("40.71")); // Partial match for coordinate
        assert!(geojson.contains("-74.")); // Partial match for coordinate
        assert!(geojson.contains("22.5"));
    }

    #[test]
    fn test_geojson_export_multiple() {
        let obs1 = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );
        let obs2 = create_test_observation(
            40.7260,
            -73.9897,
            SensorType::Movement,
            SensorValue::Movement {
                velocity: 2.5,
                heading: 45.0,
            },
        );

        let geojson = GeoJSONExporter::export_observations(&[&obs1, &obs2]);
        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        assert_eq!(parsed["features"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_geojson_with_camera_detections() {
        let detections = vec![ObjectDetection {
            class_label: "person".to_string(),
            confidence: 0.95,
            bbox: [10.0, 20.0, 50.0, 100.0],
        }];

        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Camera,
            SensorValue::Camera { detections },
        );

        let geojson = GeoJSONExporter::export_observations(&[&obs]);
        assert!(geojson.contains("person"));
        assert!(geojson.contains("camera"));
    }

    #[test]
    fn test_kml_export() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let kml = KMLExporter::export_observations(&[&obs]);
        assert!(kml.contains("<?xml"));
        assert!(kml.contains("kml"));
        assert!(kml.contains("Placemark"));
        assert!(kml.contains("40.7128"));
        assert!(kml.contains("Point"));
    }

    #[test]
    fn test_kml_multiple_sensors() {
        let thermal = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );
        let lidar = create_test_observation(
            40.7260,
            -73.9897,
            SensorType::LiDAR,
            SensorValue::LiDAR {
                distances_cm: vec![100, 200, 300],
            },
        );

        let kml = KMLExporter::export_observations(&[&thermal, &lidar]);
        assert!(kml.contains("Placemark"));
        assert!(kml.contains("red-dot")); // Thermal
        assert!(kml.contains("blue-dot")); // LiDAR
    }

    #[test]
    fn test_wkt_export_single() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let wkt = WKTExporter::observation_to_wkt(&obs);
        assert_eq!(wkt, "POINT (-74.006 40.7128)");
    }

    #[test]
    fn test_wkt_export_multipoint() {
        let obs1 = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );
        let obs2 = create_test_observation(
            40.7260,
            -73.9897,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 23.5 },
        );

        let wkt = WKTExporter::export_observations(&[&obs1, &obs2]);
        assert!(wkt.contains("MULTIPOINT"));
        assert!(wkt.contains("40.71")); // Partial match
        assert!(wkt.contains("40.72")); // Partial match
    }

    #[test]
    fn test_obj_export() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let obj = OBJExporter::export_observations(&[&obs]);
        assert!(obj.contains("v 40.7128 -74.006 100"));
        assert!(obj.contains("# PyTerrainMap Point Cloud Export"));
    }

    #[test]
    fn test_spatial_exporter_geojson() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::GeoJSON);
        assert!(result.is_ok());
        let geojson = result.unwrap();
        assert!(geojson.contains("FeatureCollection"));
    }

    #[test]
    fn test_spatial_exporter_kml() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::KML);
        assert!(result.is_ok());
        let kml = result.unwrap();
        assert!(kml.contains("<?xml"));
    }

    #[test]
    fn test_spatial_exporter_wkt() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::WKT);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_format_display() {
        assert_eq!(ExportFormat::GeoJSON.to_string(), "geojson");
        assert_eq!(ExportFormat::KML.to_string(), "kml");
        assert_eq!(ExportFormat::OBJ.to_string(), "obj");
    }

    #[test]
    fn test_geojson_fused_data() {
        use crate::types::{TemperatureEstimate, FusedDetection};

        let fused = FusedData {
            temperature: Some(TemperatureEstimate {
                celsius: 22.5,
                variance: 0.5,
                num_readings: 3,
            }),
            obstacle_map: None,
            object_detections: vec![FusedDetection {
                class_label: "person".to_string(),
                avg_confidence: 0.9,
                num_detections: 2,
                bbox_mean: [10.0, 20.0, 50.0, 100.0],
            }],
            activity_level: 0.75,
        };

        let geojson = GeoJSONExporter::export_fused_data(
            GeoPoint::new(40.7128, -74.0060),
            &fused,
        );

        assert!(geojson.contains("fused_estimate"));
        assert!(geojson.contains("22.5"));
        assert!(geojson.contains("0.75"));
    }

    #[test]
    fn test_3dtiles_export() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::ThreeDTiles);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["asset"]["version"], "1.0");
    }

    #[test]
    fn test_shapefile_not_implemented() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::Shapefile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("shapefile"));
    }

    #[test]
    fn test_geotiff_not_implemented() {
        let obs = create_test_observation(
            40.7128,
            -74.0060,
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
        );

        let result = SpatialExporter::export(&[&obs], ExportFormat::GeoTIFF);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("gdal"));
    }
}
