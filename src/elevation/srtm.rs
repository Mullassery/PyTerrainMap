//! Local SRTM `.hgt` tile lookup.
//!
//! SRTM height tiles are square, big-endian `i16` rasters named
//! `N40W075.hgt` / `S01E036.hgt`, covering one degree of latitude and
//! longitude. Row 0 is the north edge, column 0 is the west edge, and the
//! void sentinel is `-32768`. Both 1 arc-second (3601x3601) and 3
//! arc-second (1201x1201) tiles are supported; any square tile size is
//! accepted so tests can use tiny synthetic fixtures.

use super::{ElevationError, ElevationProvider};
use std::fs;
use std::path::{Path, PathBuf};

const VOID_VALUE: i16 = -32768;

/// Local SRTM tile directory provider.
#[derive(Clone, Debug)]
pub struct SrtmTileProvider {
    tile_dir: PathBuf,
}

impl SrtmTileProvider {
    /// Create a provider rooted at a directory containing `.hgt` tiles.
    pub fn new(tile_dir: impl Into<PathBuf>) -> Self {
        SrtmTileProvider {
            tile_dir: tile_dir.into(),
        }
    }

    pub fn tile_dir(&self) -> &Path {
        &self.tile_dir
    }

    /// SRTM tile filename for a coordinate (floor to the south-west corner).
    pub fn tile_file_name(lat: f64, lon: f64) -> Result<String, ElevationError> {
        if !lat.is_finite() || !lon.is_finite() {
            return Err(ElevationError(
                "latitude/longitude must be finite".to_string(),
            ));
        }
        if !(-90.0..90.0).contains(&lat) {
            return Err(ElevationError(format!(
                "latitude {lat} outside SRTM range [-90, 90)"
            )));
        }
        if !(-180.0..180.0).contains(&lon) {
            return Err(ElevationError(format!(
                "longitude {lon} outside SRTM range [-180, 180)"
            )));
        }

        let lat_floor = lat.floor() as i32;
        let lon_floor = lon.floor() as i32;
        let ns = if lat_floor >= 0 { 'N' } else { 'S' };
        let ew = if lon_floor >= 0 { 'E' } else { 'W' };

        Ok(format!(
            "{}{:02}{}{:03}.hgt",
            ns,
            lat_floor.abs(),
            ew,
            lon_floor.abs()
        ))
    }

    /// Read the interpolated elevation (meters) at `(lat, lon)`.
    pub fn elevation(&self, lat: f64, lon: f64) -> Result<f64, ElevationError> {
        let file_name = Self::tile_file_name(lat, lon)?;
        let path = self.tile_dir.join(&file_name);
        let data = fs::read(&path).map_err(|e| {
            ElevationError(format!("failed to read SRTM tile {}: {e}", path.display()))
        })?;
        let size = tile_size(&data, &path)?;

        let lat_floor = lat.floor();
        let lon_floor = lon.floor();
        let lat_frac = lat - lat_floor;
        let lon_frac = lon - lon_floor;

        let row = (1.0 - lat_frac) * (size - 1) as f64;
        let col = lon_frac * (size - 1) as f64;
        sample_bilinear(&data, size, row, col)
    }
}

impl ElevationProvider for SrtmTileProvider {
    fn elevations(&self, points: &[(f64, f64)]) -> Result<Vec<f64>, ElevationError> {
        points
            .iter()
            .map(|&(lat, lon)| self.elevation(lat, lon))
            .collect()
    }
}

fn tile_size(data: &[u8], path: &Path) -> Result<usize, ElevationError> {
    if data.len() < 8 || !data.len().is_multiple_of(2) {
        return Err(ElevationError(format!(
            "SRTM tile {} is not a valid big-endian i16 raster ({} bytes)",
            path.display(),
            data.len()
        )));
    }
    let samples = data.len() / 2;
    let size = (samples as f64).sqrt().round() as usize;
    if size < 2 || size * size != samples {
        return Err(ElevationError(format!(
            "SRTM tile {} has {samples} samples, which is not a square raster",
            path.display()
        )));
    }
    Ok(size)
}

fn raw_sample(data: &[u8], size: usize, row: usize, col: usize) -> i16 {
    let idx = (row * size + col) * 2;
    i16::from_be_bytes([data[idx], data[idx + 1]])
}

fn sample_bilinear(data: &[u8], size: usize, row: f64, col: f64) -> Result<f64, ElevationError> {
    let max_index = size - 1;
    let row = row.clamp(0.0, max_index as f64);
    let col = col.clamp(0.0, max_index as f64);

    let r0 = row.floor() as usize;
    let c0 = col.floor() as usize;
    let r1 = (r0 + 1).min(max_index);
    let c1 = (c0 + 1).min(max_index);
    let row_weight = row - r0 as f64;
    let col_weight = col - c0 as f64;

    let corners = [
        (r0, c0, (1.0 - row_weight) * (1.0 - col_weight)),
        (r0, c1, (1.0 - row_weight) * col_weight),
        (r1, c0, row_weight * (1.0 - col_weight)),
        (r1, c1, row_weight * col_weight),
    ];

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for (r, c, weight) in corners {
        if weight <= 0.0 {
            continue;
        }
        let value = raw_sample(data, size, r, c);
        if value == VOID_VALUE {
            continue;
        }
        weighted_sum += value as f64 * weight;
        total_weight += weight;
    }

    if total_weight <= 0.0 {
        return Err(ElevationError(
            "SRTM sample is a void value (-32768)".to_string(),
        ));
    }
    Ok(weighted_sum / total_weight)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ptm_srtm_{name}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_tile(dir: &Path, name: &str, size: usize, values: &[i16]) {
        assert_eq!(values.len(), size * size);
        let mut bytes = Vec::with_capacity(values.len() * 2);
        for value in values {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        fs::write(dir.join(name), bytes).unwrap();
    }

    #[test]
    fn tile_names_follow_srtm_convention() {
        assert_eq!(
            SrtmTileProvider::tile_file_name(40.7, -74.006).unwrap(),
            "N40W075.hgt"
        );
        assert_eq!(
            SrtmTileProvider::tile_file_name(40.7, -74.0).unwrap(),
            "N40W074.hgt"
        );
        assert_eq!(
            SrtmTileProvider::tile_file_name(-0.5, 36.2).unwrap(),
            "S01E036.hgt"
        );
        assert_eq!(
            SrtmTileProvider::tile_file_name(0.0, 0.0).unwrap(),
            "N00E000.hgt"
        );
    }

    #[test]
    fn tile_names_reject_out_of_range_coordinates() {
        assert!(SrtmTileProvider::tile_file_name(90.0, 0.0).is_err());
        assert!(SrtmTileProvider::tile_file_name(0.0, 180.0).is_err());
        assert!(SrtmTileProvider::tile_file_name(f64::NAN, 0.0).is_err());
    }

    #[test]
    fn reads_synthetic_tile_and_interpolates() {
        let dir = TempDir::new("read");
        // North row first: 3x3 tile spanning 10..11N, 20..21E.
        let values = [
            30, 60, 90, //
            10, 20, 40, //
            0, 5, 15,
        ];
        write_tile(dir.path(), "N10E020.hgt", 3, &values);
        let provider = SrtmTileProvider::new(dir.path());

        // Exact south-west corner (row 2, col 0).
        assert_eq!(provider.elevation(10.0, 20.0).unwrap(), 0.0);
        // Exact center (row 1, col 1).
        assert_eq!(provider.elevation(10.5, 20.5).unwrap(), 20.0);
        // Interpolate among row 0/1 and column 1/2 (60, 90, 20, 40).
        let expected = (60.0 + 90.0 + 20.0 + 40.0) / 4.0;
        let actual = provider.elevation(10.75, 20.75).unwrap();
        assert!((actual - expected).abs() < 1e-9, "got {actual}");
    }

    #[test]
    fn missing_tile_is_a_real_error() {
        let dir = TempDir::new("missing");
        let provider = SrtmTileProvider::new(dir.path());
        let err = provider.elevation(10.5, 20.5).unwrap_err();
        assert!(err.0.contains("N10E020.hgt"));
    }

    #[test]
    fn void_samples_are_not_returned_as_elevation() {
        let dir = TempDir::new("void");
        let values = [VOID_VALUE; 4];
        write_tile(dir.path(), "N10E020.hgt", 2, &values);
        let provider = SrtmTileProvider::new(dir.path());
        assert!(provider.elevation(10.5, 20.5).is_err());
    }
}
