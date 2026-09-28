//! Real elevation/DEM data for terrain analysis.
//!
//! `analyze_terrain()`/`assess_mobility()` previously returned fixed,
//! hardcoded output regardless of the coordinates passed in (a slope risk
//! severity of exactly `0.4` every time, a summary claiming "Elevation data
//! retrieved from SRTM" while never actually querying any elevation
//! source). This module provides a real elevation lookup against Open-Meteo's
//! free, no-API-key-required elevation API (Copernicus DEM GLO-90, ~90m
//! resolution) and derives a real slope estimate from it.

use serde::Deserialize;
use std::time::Duration;

const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_PER_DEGREE_LAT: f64 = 111.32;

#[derive(Debug)]
pub struct ElevationError(pub String);

impl std::fmt::Display for ElevationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "elevation lookup failed: {}", self.0)
    }
}

impl std::error::Error for ElevationError {}

#[derive(Deserialize)]
struct OpenMeteoElevationResponse {
    elevation: Vec<f64>,
}

/// A real elevation data source, queried by (lat, lon) points.
pub trait ElevationProvider {
    /// Fetch elevation in meters above sea level for each `(lat, lon)` point,
    /// in the same order as `points`. Real network call -- returns a real
    /// error (never fabricated data) if the request fails.
    fn elevations(&self, points: &[(f64, f64)]) -> Result<Vec<f64>, ElevationError>;
}

/// Open-Meteo elevation API client (https://open-meteo.com/en/docs/elevation-api,
/// CC-BY 4.0, backed by Copernicus DEM GLO-90). Free, no API key required.
pub struct OpenMeteoElevationProvider {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl OpenMeteoElevationProvider {
    pub fn new() -> Self {
        Self::with_base_url("https://api.open-meteo.com/v1/elevation".to_string())
    }

    /// Test/self-hosted-mirror hook: point at any server implementing the
    /// same `?latitude=..&longitude=..` -> `{"elevation": [...]}` contract.
    pub fn with_base_url(base_url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest blocking client builder should not fail with no custom TLS config");
        OpenMeteoElevationProvider { base_url, client }
    }
}

impl Default for OpenMeteoElevationProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ElevationProvider for OpenMeteoElevationProvider {
    fn elevations(&self, points: &[(f64, f64)]) -> Result<Vec<f64>, ElevationError> {
        if points.is_empty() {
            return Ok(Vec::new());
        }

        let lat_csv = points
            .iter()
            .map(|(lat, _)| format!("{lat}"))
            .collect::<Vec<_>>()
            .join(",");
        let lon_csv = points
            .iter()
            .map(|(_, lon)| format!("{lon}"))
            .collect::<Vec<_>>()
            .join(",");

        let response = self
            .client
            .get(&self.base_url)
            .query(&[("latitude", lat_csv), ("longitude", lon_csv)])
            .send()
            .map_err(|e| ElevationError(format!("request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(ElevationError(format!(
                "elevation API returned HTTP {}",
                response.status()
            )));
        }

        let parsed: OpenMeteoElevationResponse = response
            .json()
            .map_err(|e| ElevationError(format!("failed to parse response: {e}")))?;

        if parsed.elevation.len() != points.len() {
            return Err(ElevationError(format!(
                "expected {} elevation values, got {}",
                points.len(),
                parsed.elevation.len()
            )));
        }

        Ok(parsed.elevation)
    }
}

/// Offset a `(lat, lon)` point by `distance_km` due north (bearing 0),
/// east (90), south (180), or west (270). Uses the standard flat-earth
/// approximation (111.32 km/degree latitude, corrected by cos(lat) for
/// longitude) -- accurate to well under 1% at the small radii (<= a few km)
/// this is used for.
pub fn offset_point(lat: f64, lon: f64, distance_km: f64, bearing_deg: f64) -> (f64, f64) {
    let bearing_rad = bearing_deg.to_radians();
    let delta_lat_deg = (distance_km * bearing_rad.cos()) / KM_PER_DEGREE_LAT;
    let km_per_degree_lon = KM_PER_DEGREE_LAT * lat.to_radians().cos();
    let delta_lon_deg = if km_per_degree_lon.abs() < 1e-9 {
        0.0 // at the poles, longitude is undefined; avoid divide-by-near-zero
    } else {
        (distance_km * bearing_rad.sin()) / km_per_degree_lon
    };
    (lat + delta_lat_deg, lon + delta_lon_deg)
}

/// Slope in degrees from a rise (meters, can be negative) over a horizontal
/// run (meters, must be positive). `atan(rise/run)`, the standard slope
/// angle definition.
pub fn slope_degrees(rise_m: f64, run_m: f64) -> f64 {
    if run_m <= 0.0 {
        return 0.0;
    }
    (rise_m / run_m).atan().to_degrees()
}

/// Real terrain sample: center elevation plus the steepest of the 4
/// cardinal-direction slopes measured at `radius_km`.
pub struct TerrainSample {
    pub center_elevation_m: f64,
    pub min_elevation_m: f64,
    pub max_elevation_m: f64,
    pub max_slope_degrees: f64,
}

/// Sample real elevation at the center and 4 cardinal points `radius_km`
/// away, and derive a real max-slope estimate. This is the function
/// `analyze_terrain()` calls; kept generic over `ElevationProvider` so it's
/// unit-testable without a real network call (see tests below).
pub fn sample_terrain(
    provider: &dyn ElevationProvider,
    lat: f64,
    lon: f64,
    radius_km: f64,
) -> Result<TerrainSample, ElevationError> {
    let radius_km = radius_km.max(0.01); // avoid a degenerate 0-radius query
    let points = [
        (lat, lon),
        offset_point(lat, lon, radius_km, 0.0),   // N
        offset_point(lat, lon, radius_km, 90.0),  // E
        offset_point(lat, lon, radius_km, 180.0), // S
        offset_point(lat, lon, radius_km, 270.0), // W
    ];

    let elevations = provider.elevations(&points)?;
    let center = elevations[0];
    let run_m = radius_km * 1000.0;

    let max_slope = elevations[1..]
        .iter()
        .map(|&e| slope_degrees((e - center).abs(), run_m))
        .fold(0.0_f64, f64::max);

    let min_elevation_m = elevations.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_elevation_m = elevations
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);

    Ok(TerrainSample {
        center_elevation_m: center,
        min_elevation_m,
        max_elevation_m,
        max_slope_degrees: max_slope,
    })
}

/// Map a real measured slope to a risk severity in `[0.0, 1.0]`.
/// 0 degrees (flat) -> 0.0. 45+ degrees (near-cliff) -> 1.0. Linear in
/// between, which matches how the rest of this codebase treats slope-based
/// hazard severity (e.g. `TerrainCostMap`'s 0..1 traversal costs).
pub fn slope_to_risk_severity(slope_degrees: f64) -> f32 {
    (slope_degrees / 45.0).clamp(0.0, 1.0) as f32
}

#[allow(dead_code)]
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeElevationProvider {
        by_point: std::collections::HashMap<(i64, i64), f64>,
    }

    impl FakeElevationProvider {
        fn new(entries: &[((f64, f64), f64)]) -> Self {
            let mut by_point = std::collections::HashMap::new();
            for &((lat, lon), elevation) in entries {
                by_point.insert(Self::key(lat, lon), elevation);
            }
            FakeElevationProvider { by_point }
        }

        fn key(lat: f64, lon: f64) -> (i64, i64) {
            ((lat * 1e6).round() as i64, (lon * 1e6).round() as i64)
        }
    }

    impl ElevationProvider for FakeElevationProvider {
        fn elevations(&self, points: &[(f64, f64)]) -> Result<Vec<f64>, ElevationError> {
            points
                .iter()
                .map(|&(lat, lon)| {
                    self.by_point
                        .get(&Self::key(lat, lon))
                        .copied()
                        .ok_or_else(|| ElevationError(format!("no fixture for ({lat}, {lon})")))
                })
                .collect()
        }
    }

    #[test]
    fn slope_degrees_matches_known_closed_form_values() {
        // 45 degrees: rise == run
        assert!((slope_degrees(100.0, 100.0) - 45.0).abs() < 1e-9);
        // flat
        assert_eq!(slope_degrees(0.0, 100.0), 0.0);
        // ~5.71 degrees: tan(5.71deg) ~= 0.1
        assert!((slope_degrees(10.0, 100.0) - 5.7106).abs() < 1e-3);
    }

    #[test]
    fn slope_degrees_zero_run_does_not_divide_by_zero() {
        assert_eq!(slope_degrees(50.0, 0.0), 0.0);
    }

    #[test]
    fn offset_point_north_moves_only_latitude() {
        let (lat, lon) = offset_point(10.0, 20.0, KM_PER_DEGREE_LAT, 0.0); // exactly 1 degree
        assert!((lat - 11.0).abs() < 1e-3);
        assert!((lon - 20.0).abs() < 1e-9);
    }

    #[test]
    fn offset_point_east_moves_only_longitude_and_widens_near_equator() {
        let (lat, lon) = offset_point(0.0, 20.0, KM_PER_DEGREE_LAT, 90.0);
        assert!((lat - 0.0).abs() < 1e-9);
        assert!((lon - 21.0).abs() < 1e-3);
    }

    #[test]
    fn offset_point_east_narrows_at_high_latitude() {
        // At 60 degrees latitude, a degree of longitude is only ~half as
        // wide (cos(60deg) = 0.5) as at the equator -- moving the same
        // physical distance east should cover roughly double the degrees.
        let (_, lon_at_equator) = offset_point(0.0, 0.0, 50.0, 90.0);
        let (_, lon_at_60) = offset_point(60.0, 0.0, 50.0, 90.0);
        assert!(lon_at_60 > lon_at_equator * 1.8);
    }

    #[test]
    fn sample_terrain_computes_real_slope_from_elevation_samples() {
        // Center at sea level, due north rises 100m over 1km -> atan(100/1000)
        // ~= 5.71 degrees; the other 3 directions are flat.
        let provider = FakeElevationProvider::new(&[
            ((10.0, 20.0), 0.0),
            (offset_point(10.0, 20.0, 1.0, 0.0), 100.0),
            (offset_point(10.0, 20.0, 1.0, 90.0), 0.0),
            (offset_point(10.0, 20.0, 1.0, 180.0), 0.0),
            (offset_point(10.0, 20.0, 1.0, 270.0), 0.0),
        ]);

        let sample = sample_terrain(&provider, 10.0, 20.0, 1.0).unwrap();
        assert_eq!(sample.center_elevation_m, 0.0);
        assert_eq!(sample.max_elevation_m, 100.0);
        assert_eq!(sample.min_elevation_m, 0.0);
        assert!((sample.max_slope_degrees - 5.7106).abs() < 1e-3);
    }

    #[test]
    fn sample_terrain_propagates_real_provider_errors() {
        let provider = FakeElevationProvider::new(&[]); // no fixtures at all
        let result = sample_terrain(&provider, 10.0, 20.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn slope_to_risk_severity_matches_expected_boundaries() {
        assert_eq!(slope_to_risk_severity(0.0), 0.0);
        assert_eq!(slope_to_risk_severity(45.0), 1.0);
        assert_eq!(slope_to_risk_severity(90.0), 1.0); // clamped
        assert!((slope_to_risk_severity(22.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    #[ignore] // real network call -- run explicitly with `cargo test -- --ignored`
    fn open_meteo_provider_returns_real_elevation_data() {
        let provider = OpenMeteoElevationProvider::new();
        // Zermatt, Switzerland -- real, well-known high elevation (~1600m+).
        let elevations = provider.elevations(&[(46.0207, 7.7491)]).unwrap();
        assert_eq!(elevations.len(), 1);
        assert!(elevations[0] > 1000.0, "got {}", elevations[0]);
    }
}
