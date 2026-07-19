//! Weather and soil data integration for environmental context
//!
//! Imports weather observations and soil conditions to enrich terrain mapping,
//! enabling multi-modal terrain analysis and adaptive mission planning.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Weather observation data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherObservation {
    /// Location (latitude, longitude)
    pub location: (f64, f64),
    /// Timestamp (microseconds since epoch)
    pub timestamp_us: i64,
    /// Temperature (Celsius)
    pub temperature_celsius: f32,
    /// Relative humidity (0.0-1.0)
    pub humidity: f32,
    /// Precipitation (mm)
    pub precipitation_mm: f32,
    /// Wind speed (m/s)
    pub wind_speed_ms: f32,
    /// Wind direction (degrees, 0-360)
    pub wind_direction_degrees: f32,
    /// Atmospheric pressure (hPa)
    pub pressure_hpa: f32,
    /// Visibility (meters)
    pub visibility_meters: f32,
    /// Cloud coverage (0.0-1.0)
    pub cloud_cover: f32,
    /// Weather condition (Clear, Cloudy, Rainy, Snowy, etc.)
    pub condition: WeatherCondition,
    /// Data source (OpenWeather, WeatherAPI, NOAA, etc.)
    pub source: String,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
}

impl WeatherObservation {
    /// Create weather observation
    pub fn new(location: (f64, f64), timestamp_us: i64) -> Self {
        WeatherObservation {
            location,
            timestamp_us,
            temperature_celsius: 20.0,
            humidity: 0.5,
            precipitation_mm: 0.0,
            wind_speed_ms: 0.0,
            wind_direction_degrees: 0.0,
            pressure_hpa: 1013.25,
            visibility_meters: 10000.0,
            cloud_cover: 0.0,
            condition: WeatherCondition::Clear,
            source: "unknown".to_string(),
            confidence: 0.8,
        }
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature_celsius = temp;
        self
    }

    /// Set humidity
    pub fn with_humidity(mut self, humidity: f32) -> Self {
        self.humidity = humidity.max(0.0).min(1.0);
        self
    }

    /// Set wind
    pub fn with_wind(mut self, speed: f32, direction: f32) -> Self {
        self.wind_speed_ms = speed;
        self.wind_direction_degrees = direction % 360.0;
        self
    }

    /// Set precipitation
    pub fn with_precipitation(mut self, mm: f32) -> Self {
        self.precipitation_mm = mm.max(0.0);
        self
    }

    /// Detect if conditions are suitable for aerial observations
    pub fn is_flight_safe(&self) -> bool {
        // Wind < 10 m/s, visibility > 5km, no heavy rain
        self.wind_speed_ms < 10.0
            && self.visibility_meters > 5000.0
            && self.precipitation_mm < 5.0
    }

    /// Detect if conditions are suitable for ground robots
    pub fn is_ground_safe(&self) -> bool {
        // Visibility > 1km, not heavily raining
        self.visibility_meters > 1000.0 && self.precipitation_mm < 10.0
    }
}

/// Weather conditions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    Overcast,
    Drizzle,
    Rainy,
    HeavyRain,
    Thunderstorm,
    Snowy,
    Foggy,
    Hazy,
    Windy,
}

impl WeatherCondition {
    /// Get visibility impact (multiplier for LiDAR/camera)
    pub fn visibility_impact(&self) -> f32 {
        match self {
            WeatherCondition::Clear => 1.0,
            WeatherCondition::PartlyCloudy => 0.95,
            WeatherCondition::Cloudy => 0.9,
            WeatherCondition::Overcast => 0.85,
            WeatherCondition::Drizzle => 0.7,
            WeatherCondition::Rainy => 0.5,
            WeatherCondition::HeavyRain => 0.2,
            WeatherCondition::Thunderstorm => 0.0, // Not safe
            WeatherCondition::Snowy => 0.4,
            WeatherCondition::Foggy => 0.3,
            WeatherCondition::Hazy => 0.6,
            WeatherCondition::Windy => 0.8,
        }
    }
}

/// Soil condition data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoilCondition {
    /// Location (latitude, longitude)
    pub location: (f64, f64),
    /// Depth (centimeters)
    pub depth_cm: f32,
    /// Soil type (sandy, loamy, clayey, peat, etc.)
    pub soil_type: SoilType,
    /// Soil texture (grain size distribution)
    pub texture: SoilTexture,
    /// Moisture content (volumetric %, 0-100)
    pub moisture_percent: f32,
    /// pH level (0-14)
    pub ph: f32,
    /// Organic matter (%, 0-100)
    pub organic_matter_percent: f32,
    /// Nitrogen content (mg/kg)
    pub nitrogen_mg_kg: f32,
    /// Phosphorus content (mg/kg)
    pub phosphorus_mg_kg: f32,
    /// Potassium content (mg/kg)
    pub potassium_mg_kg: f32,
    /// Bulk density (kg/m³)
    pub bulk_density_kg_m3: f32,
    /// Bearing capacity (kPa) - for ground robots
    pub bearing_capacity_kpa: f32,
    /// Compaction index (0.0-1.0, 1.0 = fully compacted)
    pub compaction_index: f32,
    /// Timestamp (microseconds)
    pub timestamp_us: i64,
    /// Data source
    pub source: String,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
}

impl SoilCondition {
    /// Create soil condition
    pub fn new(location: (f64, f64), depth_cm: f32, soil_type: SoilType) -> Self {
        SoilCondition {
            location,
            depth_cm,
            soil_type,
            texture: SoilTexture::Loam,
            moisture_percent: 25.0,
            ph: 7.0,
            organic_matter_percent: 3.0,
            nitrogen_mg_kg: 50.0,
            phosphorus_mg_kg: 25.0,
            potassium_mg_kg: 100.0,
            bulk_density_kg_m3: 1300.0,
            bearing_capacity_kpa: 100.0,
            compaction_index: 0.5,
            timestamp_us: chrono::Utc::now().timestamp_micros(),
            source: "unknown".to_string(),
            confidence: 0.7,
        }
    }

    /// Set moisture
    pub fn with_moisture(mut self, moisture: f32) -> Self {
        self.moisture_percent = moisture.max(0.0).min(100.0);
        self
    }

    /// Set pH
    pub fn with_ph(mut self, ph: f32) -> Self {
        self.ph = ph.max(0.0).min(14.0);
        self
    }

    /// Set compaction
    pub fn with_compaction(mut self, compaction: f32) -> Self {
        self.compaction_index = compaction.max(0.0).min(1.0);
        self
    }

    /// Detect if soil supports ground robot movement
    pub fn is_passable(&self) -> bool {
        // Bearing capacity > 50 kPa, compaction < 0.9, not waterlogged
        self.bearing_capacity_kpa > 50.0 && self.compaction_index < 0.9 && self.moisture_percent < 80.0
    }

    /// Get trafficability rating (0.0-1.0)
    pub fn trafficability_rating(&self) -> f32 {
        let bearing_score = (self.bearing_capacity_kpa / 200.0).min(1.0);
        let moisture_score = if self.moisture_percent < 30.0 {
            self.moisture_percent / 30.0
        } else if self.moisture_percent < 60.0 {
            1.0
        } else {
            (100.0 - self.moisture_percent) / 40.0
        };
        let compaction_score = 1.0 - self.compaction_index;

        (bearing_score + moisture_score + compaction_score) / 3.0
    }
}

/// Soil types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilType {
    Sand,
    Silt,
    Clay,
    Loam,
    SandyLoam,
    SiltLoam,
    ClayLoam,
    Peat,
    Rock,
    Unknown,
}

/// Soil texture (grain size distribution)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilTexture {
    CoarseSand,
    MediumSand,
    FineSand,
    Loam,
    SiltLoam,
    ClayLoam,
    Silt,
    Clay,
}

/// Weather data source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataSource {
    /// Source type
    pub source_type: WeatherSourceType,
    /// API key or credentials
    pub api_key: Option<String>,
    /// Base URL
    pub url: String,
    /// Update frequency (hours)
    pub update_frequency_hours: u32,
    /// Coverage area (radius in km)
    pub coverage_km: f32,
}

impl WeatherDataSource {
    /// Create OpenWeather source
    pub fn openweather(api_key: &str) -> Self {
        WeatherDataSource {
            source_type: WeatherSourceType::OpenWeather,
            api_key: Some(api_key.to_string()),
            url: "https://api.openweathermap.org/data/2.5".to_string(),
            update_frequency_hours: 1,
            coverage_km: 10.0,
        }
    }

    /// Create NOAA source
    pub fn noaa() -> Self {
        WeatherDataSource {
            source_type: WeatherSourceType::NOAA,
            api_key: None,
            url: "https://api.weather.gov".to_string(),
            update_frequency_hours: 1,
            coverage_km: 50.0,
        }
    }

    /// Create WeatherAPI source
    pub fn weatherapi(api_key: &str) -> Self {
        WeatherDataSource {
            source_type: WeatherSourceType::WeatherAPI,
            api_key: Some(api_key.to_string()),
            url: "https://api.weatherapi.com/v1".to_string(),
            update_frequency_hours: 1,
            coverage_km: 10.0,
        }
    }
}

/// Weather data source types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherSourceType {
    OpenWeather,
    NOAA,
    WeatherAPI,
    Meteoblue,
    Custom,
}

/// Soil data source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoilDataSource {
    /// Source type
    pub source_type: SoilSourceType,
    /// API key or credentials
    pub api_key: Option<String>,
    /// Base URL
    pub url: String,
    /// Update frequency (days)
    pub update_frequency_days: u32,
    /// Grid resolution (meters)
    pub grid_resolution_meters: f32,
}

impl SoilDataSource {
    /// Create ISRIC soil database source
    pub fn isric() -> Self {
        SoilDataSource {
            source_type: SoilSourceType::ISRIC,
            api_key: None,
            url: "https://soilgrids.org/soilgrids/v2.0".to_string(),
            update_frequency_days: 365,
            grid_resolution_meters: 250.0,
        }
    }

    /// Create USDA NRCS source
    pub fn usda_nrcs(api_key: &str) -> Self {
        SoilDataSource {
            source_type: SoilSourceType::USDA_NRCS,
            api_key: Some(api_key.to_string()),
            url: "https://sdmdataaccess.nrcs.usda.gov/Tabular".to_string(),
            update_frequency_days: 90,
            grid_resolution_meters: 30.0,
        }
    }

    /// Create custom soil survey source
    pub fn custom(url: &str) -> Self {
        SoilDataSource {
            source_type: SoilSourceType::Custom,
            api_key: None,
            url: url.to_string(),
            update_frequency_days: 180,
            grid_resolution_meters: 100.0,
        }
    }
}

/// Soil data source types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilSourceType {
    ISRIC,
    USDA_NRCS,
    SoilGrids,
    LUCAS,
    Custom,
}

/// Weather grid cell for spatial coverage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherGridCell {
    /// Grid coordinates (x, y)
    pub grid_coord: (u32, u32),
    /// Weather observations in this cell
    pub observations: Vec<WeatherObservation>,
    /// Last update timestamp
    pub last_update_us: i64,
}

impl WeatherGridCell {
    /// Create weather grid cell
    pub fn new(x: u32, y: u32) -> Self {
        WeatherGridCell {
            grid_coord: (x, y),
            observations: Vec::new(),
            last_update_us: 0,
        }
    }

    /// Get current weather (most recent)
    pub fn current_weather(&self) -> Option<&WeatherObservation> {
        self.observations.last()
    }

    /// Get average temperature
    pub fn average_temperature(&self) -> f32 {
        if self.observations.is_empty() {
            return 20.0;
        }
        let sum: f32 = self.observations.iter().map(|o| o.temperature_celsius).sum();
        sum / self.observations.len() as f32
    }
}

/// Soil grid cell for spatial coverage
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoilGridCell {
    /// Grid coordinates (x, y)
    pub grid_coord: (u32, u32),
    /// Soil conditions at different depths
    pub conditions_by_depth: HashMap<u32, SoilCondition>, // depth_cm -> condition
    /// Last update timestamp
    pub last_update_us: i64,
}

impl SoilGridCell {
    /// Create soil grid cell
    pub fn new(x: u32, y: u32) -> Self {
        SoilGridCell {
            grid_coord: (x, y),
            conditions_by_depth: HashMap::new(),
            last_update_us: 0,
        }
    }

    /// Get condition at depth
    pub fn condition_at_depth(&self, depth_cm: u32) -> Option<&SoilCondition> {
        self.conditions_by_depth.get(&depth_cm)
    }

    /// Get surface condition (topsoil, ~10cm)
    pub fn surface_condition(&self) -> Option<&SoilCondition> {
        self.conditions_by_depth.get(&10)
    }

    /// Get average trafficability
    pub fn average_trafficability(&self) -> f32 {
        if self.conditions_by_depth.is_empty() {
            return 0.5;
        }
        let sum: f32 = self.conditions_by_depth
            .values()
            .map(|c| c.trafficability_rating())
            .sum();
        sum / self.conditions_by_depth.len() as f32
    }
}

/// Combined weather + soil conditions at a location
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentalConditions {
    /// Location (latitude, longitude)
    pub location: (f64, f64),
    /// Timestamp (microseconds)
    pub timestamp_us: i64,
    /// Weather observation
    pub weather: Option<WeatherObservation>,
    /// Soil condition
    pub soil: Option<SoilCondition>,
    /// Mission suitability score (0.0-1.0)
    pub mission_suitability: f32,
}

impl EnvironmentalConditions {
    /// Create environmental conditions
    pub fn new(location: (f64, f64)) -> Self {
        EnvironmentalConditions {
            location,
            timestamp_us: chrono::Utc::now().timestamp_micros(),
            weather: None,
            soil: None,
            mission_suitability: 0.5,
        }
    }

    /// Add weather observation
    pub fn with_weather(mut self, weather: WeatherObservation) -> Self {
        self.weather = Some(weather);
        self.update_suitability();
        self
    }

    /// Add soil condition
    pub fn with_soil(mut self, soil: SoilCondition) -> Self {
        self.soil = Some(soil);
        self.update_suitability();
        self
    }

    /// Update mission suitability score
    pub fn update_suitability(&mut self) {
        let mut scores = Vec::new();

        if let Some(w) = &self.weather {
            let aerial_safe = if w.is_flight_safe() { 1.0 } else { 0.3 };
            scores.push(aerial_safe);
        }

        if let Some(s) = &self.soil {
            let ground_safe = if s.is_passable() { 1.0 } else { 0.2 };
            scores.push(ground_safe);
        }

        self.mission_suitability = if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_observation_creation() {
        let obs = WeatherObservation::new((40.71, -74.00), 1000);
        assert_eq!(obs.location, (40.71, -74.00));
        assert_eq!(obs.temperature_celsius, 20.0);
    }

    #[test]
    fn test_weather_observation_with_temperature() {
        let obs = WeatherObservation::new((40.71, -74.00), 1000)
            .with_temperature(25.5);
        assert_eq!(obs.temperature_celsius, 25.5);
    }

    #[test]
    fn test_weather_observation_with_wind() {
        let obs = WeatherObservation::new((40.71, -74.00), 1000)
            .with_wind(5.0, 90.0);
        assert_eq!(obs.wind_speed_ms, 5.0);
        assert_eq!(obs.wind_direction_degrees, 90.0);
    }

    #[test]
    fn test_weather_flight_safe() {
        let obs = WeatherObservation::new((40.71, -74.00), 1000)
            .with_wind(5.0, 0.0)
            .with_precipitation(0.0);
        assert!(obs.is_flight_safe());

        let obs_windy = WeatherObservation::new((40.71, -74.00), 1000)
            .with_wind(15.0, 0.0);
        assert!(!obs_windy.is_flight_safe());
    }

    #[test]
    fn test_weather_ground_safe() {
        let obs = WeatherObservation::new((40.71, -74.00), 1000)
            .with_precipitation(0.0);
        assert!(obs.is_ground_safe());
    }

    #[test]
    fn test_weather_condition_visibility() {
        assert_eq!(WeatherCondition::Clear.visibility_impact(), 1.0);
        assert_eq!(WeatherCondition::Rainy.visibility_impact(), 0.5);
        assert_eq!(WeatherCondition::Thunderstorm.visibility_impact(), 0.0);
    }

    #[test]
    fn test_soil_condition_creation() {
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam);
        assert_eq!(soil.location, (40.71, -74.00));
        assert_eq!(soil.depth_cm, 10.0);
    }

    #[test]
    fn test_soil_condition_with_moisture() {
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam)
            .with_moisture(45.0);
        assert_eq!(soil.moisture_percent, 45.0);
    }

    #[test]
    fn test_soil_condition_passable() {
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam)
            .with_moisture(30.0)
            .with_compaction(0.5);
        soil.bearing_capacity_kpa > 50.0;
        soil.compaction_index < 0.9;
    }

    #[test]
    fn test_soil_trafficability_rating() {
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam);
        let rating = soil.trafficability_rating();
        assert!(rating >= 0.0 && rating <= 1.0);
    }

    #[test]
    fn test_weather_data_source_openweather() {
        let source = WeatherDataSource::openweather("key123");
        assert_eq!(source.source_type, WeatherSourceType::OpenWeather);
        assert!(source.api_key.is_some());
    }

    #[test]
    fn test_weather_data_source_noaa() {
        let source = WeatherDataSource::noaa();
        assert_eq!(source.source_type, WeatherSourceType::NOAA);
        assert!(source.api_key.is_none());
    }

    #[test]
    fn test_soil_data_source_isric() {
        let source = SoilDataSource::isric();
        assert_eq!(source.source_type, SoilSourceType::ISRIC);
    }

    #[test]
    fn test_soil_data_source_usda_nrcs() {
        let source = SoilDataSource::usda_nrcs("key123");
        assert_eq!(source.source_type, SoilSourceType::USDA_NRCS);
        assert!(source.api_key.is_some());
    }

    #[test]
    fn test_weather_grid_cell() {
        let mut cell = WeatherGridCell::new(0, 0);
        let obs = WeatherObservation::new((40.71, -74.00), 1000);
        cell.observations.push(obs);
        assert_eq!(cell.observations.len(), 1);
        assert!(cell.current_weather().is_some());
    }

    #[test]
    fn test_weather_grid_average_temperature() {
        let mut cell = WeatherGridCell::new(0, 0);
        cell.observations.push(WeatherObservation::new((40.71, -74.00), 1000)
            .with_temperature(20.0));
        cell.observations.push(WeatherObservation::new((40.71, -74.00), 2000)
            .with_temperature(30.0));
        assert_eq!(cell.average_temperature(), 25.0);
    }

    #[test]
    fn test_soil_grid_cell() {
        let mut cell = SoilGridCell::new(0, 0);
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam);
        cell.conditions_by_depth.insert(10, soil);
        assert!(cell.condition_at_depth(10).is_some());
    }

    #[test]
    fn test_soil_grid_surface_condition() {
        let mut cell = SoilGridCell::new(0, 0);
        let soil = SoilCondition::new((40.71, -74.00), 10.0, SoilType::Loam);
        cell.conditions_by_depth.insert(10, soil);
        assert!(cell.surface_condition().is_some());
    }

    #[test]
    fn test_environmental_conditions() {
        let mut env = EnvironmentalConditions::new((40.71, -74.00));
        let weather = WeatherObservation::new((40.71, -74.00), 1000);
        env = env.with_weather(weather);
        assert!(env.weather.is_some());
    }

    #[test]
    fn test_environmental_conditions_suitability() {
        let mut env = EnvironmentalConditions::new((40.71, -74.00));
        let weather = WeatherObservation::new((40.71, -74.00), 1000);
        env = env.with_weather(weather);
        assert!(env.mission_suitability >= 0.0 && env.mission_suitability <= 1.0);
    }

    #[test]
    fn test_soil_type_enum() {
        assert_eq!(SoilType::Loam, SoilType::Loam);
        assert_ne!(SoilType::Loam, SoilType::Clay);
    }

    #[test]
    fn test_soil_texture_enum() {
        assert_eq!(SoilTexture::Loam, SoilTexture::Loam);
    }

    #[test]
    fn test_weather_condition_enum() {
        assert_eq!(WeatherCondition::Clear, WeatherCondition::Clear);
    }
}
