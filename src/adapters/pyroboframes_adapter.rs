//! PyRoboFrames adapter for MCAP/ROS2 multi-sensor composition
//!
//! Converts aligned DataFrames from PyRoboFrames into PyTerrainMap Observations.
//! Handles multi-rate sensor composition, temporal alignment, and provenance tracking.

use crate::types::{Observation, Result, Error, GeoPoint, SensorType, SensorValue, TemporalMetadata, ClockSource};
use std::collections::HashMap;
use uuid::Uuid;

/// Aligned multi-sensor dataframe from PyRoboFrames
///
/// Represents a time-aligned slice of multi-rate sensor data.
#[derive(Clone, Debug)]
pub struct AlignedDataFrame {
    /// Unique episode identifier
    pub episode_id: String,
    /// Frame index within episode
    pub frame_index: u32,
    /// Source dataset name (e.g., "construction_site_2026_07_20")
    pub dataset_name: String,
    /// Reference timestamp for alignment (microseconds)
    pub reference_time_us: i64,
    /// Alignment window (±50-100ms typical)
    pub window_size_us: i64,
    /// Robot identifier
    pub robot_id: String,
    /// Sensor readings keyed by sensor_name
    pub sensor_readings: HashMap<String, SensorReading>,
}

/// Single sensor reading in dataframe
#[derive(Clone, Debug)]
pub struct SensorReading {
    /// Sensor type/model (e.g., "camera_rgb_01", "lidar_os1_64", "thermal_boson")
    pub sensor_name: String,
    /// When sensor captured this frame (microseconds)
    pub capture_time_us: i64,
    /// Sensor type
    pub sensor_type: SensorType,
    /// Raw sensor value
    pub value: SensorValue,
    /// Sensor-specific confidence (0.0-1.0)
    pub quality_score: f32,
}

/// RoboticsDataFrameAdapter: Converts PyRoboFrames DataFrames to Observations
///
/// Handles:
/// - Multi-rate sensor alignment (10fps camera + 1fps probe + 0.1Hz GPS)
/// - Temporal metadata preservation (capture times, quality per sensor)
/// - Provenance tracking (episode_id, frame_index, dataset_name)
/// - Clock source awareness (GPS, NavIC, IMU, etc.)
pub struct RoboticsDataFrameAdapter {
    /// Default clock source for this deployment (e.g., GPS)
    default_clock_source: ClockSource,
    /// Ingestion timestamp for all dataframes
    ingestion_time_us: i64,
}

impl RoboticsDataFrameAdapter {
    /// Create new adapter with default clock source
    pub fn new(clock_source: ClockSource) -> Self {
        RoboticsDataFrameAdapter {
            default_clock_source: clock_source,
            ingestion_time_us: chrono::Utc::now().timestamp_micros() as i64,
        }
    }

    /// Update ingestion timestamp (call before ingesting batch)
    pub fn set_ingestion_time(&mut self, time_us: i64) {
        self.ingestion_time_us = time_us;
    }

    /// Ingest aligned dataframe and convert to Observations
    ///
    /// Returns one Observation per sensor in the dataframe.
    /// All observations share temporal lineage and provenance.
    pub fn ingest_dataframe(&self, frame: &AlignedDataFrame) -> Result<Vec<Observation>> {
        if frame.sensor_readings.is_empty() {
            return Err(Error::InvalidObservation(
                "DataFr frame has no sensor readings".to_string(),
            ));
        }

        let mut observations = Vec::new();
        let processing_time_us = chrono::Utc::now().timestamp_micros() as i64;

        for (sensor_name, reading) in &frame.sensor_readings {
            // Create observation with full provenance
            let obs = Observation {
                id: Uuid::new_v4(),
                robot_id: frame.robot_id.clone(),
                timestamp: reading.capture_time_us,
                location: GeoPoint::new(0.0, 0.0), // Will be set from sensor data or external source
                elevation_asl: None,
                sensor_type: reading.sensor_type,
                value: reading.value.clone(),
                confidence: reading.quality_score,
                temporal: TemporalMetadata {
                    event_time_us: reading.capture_time_us,
                    capture_time_us: reading.capture_time_us,
                    transmission_time_us: frame.reference_time_us + (frame.window_size_us / 2),
                    ingestion_time_us: self.ingestion_time_us,
                    processing_time_us,
                    clock_source: self.default_clock_source,
                    precision_us: 1_000, // 1ms precision (typical for sensor timestamps)
                    estimated_latency_us: (self.ingestion_time_us - reading.capture_time_us) as u32,
                    sync_confidence: 0.9, // High confidence for PyRoboFrames-aligned data
                    is_late_arrival: false,
                    jitter_us: (frame.window_size_us / 4) as u32, // Jitter ~ 1/4 window size
                    temporal_confidence: 0.95,
                },
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("episode_id".to_string(), frame.episode_id.clone());
                    meta.insert("frame_index".to_string(), frame.frame_index.to_string());
                    meta.insert("dataset_name".to_string(), frame.dataset_name.clone());
                    meta.insert("sensor_name".to_string(), sensor_name.clone());
                    meta.insert("alignment_window_us".to_string(), frame.window_size_us.to_string());
                    meta
                },
            };

            observations.push(obs);
        }

        Ok(observations)
    }

    /// Ingest batch of dataframes
    pub fn ingest_batch(&self, frames: &[AlignedDataFrame]) -> Result<Vec<Observation>> {
        let mut all_observations = Vec::new();

        for frame in frames {
            let obs = self.ingest_dataframe(frame)?;
            all_observations.extend(obs);
        }

        Ok(all_observations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dataframe(episode_id: &str, frame_index: u32, num_sensors: usize) -> AlignedDataFrame {
        let mut sensor_readings = HashMap::new();

        // Add test sensor readings
        for i in 0..num_sensors {
            let sensor_name = format!("sensor_{}", i);
            sensor_readings.insert(
                sensor_name.clone(),
                SensorReading {
                    sensor_name,
                    capture_time_us: 1_000_000 + (i as i64 * 100_000),
                    sensor_type: SensorType::Camera,
                    value: SensorValue::Camera { detections: vec![] },
                    quality_score: 0.9,
                },
            );
        }

        AlignedDataFrame {
            episode_id: episode_id.to_string(),
            frame_index,
            dataset_name: "test_dataset".to_string(),
            reference_time_us: 1_000_000,
            window_size_us: 50_000, // 50ms window
            robot_id: "robot-1".to_string(),
            sensor_readings,
        }
    }

    #[test]
    fn test_adapter_creation() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);
        assert_eq!(adapter.ingestion_time_us > 0, true);
    }

    #[test]
    fn test_ingest_single_sensor_dataframe() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);
        let frame = create_test_dataframe("episode-1", 0, 1);

        let obs = adapter.ingest_dataframe(&frame).unwrap();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].robot_id, "robot-1");
        assert_eq!(obs[0].metadata.get("episode_id"), Some(&"episode-1".to_string()));
    }

    #[test]
    fn test_ingest_multi_sensor_dataframe() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);
        let frame = create_test_dataframe("episode-1", 0, 3);

        let obs = adapter.ingest_dataframe(&frame).unwrap();
        assert_eq!(obs.len(), 3);

        // All observations share same episode_id and dataset
        for o in &obs {
            assert_eq!(o.metadata.get("episode_id"), Some(&"episode-1".to_string()));
            assert_eq!(o.metadata.get("dataset_name"), Some(&"test_dataset".to_string()));
        }
    }

    #[test]
    fn test_ingest_batch() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);

        let mut frames = Vec::new();
        for i in 0..3 {
            frames.push(create_test_dataframe("episode-1", i, 2));
        }

        let obs = adapter.ingest_batch(&frames).unwrap();
        assert_eq!(obs.len(), 6); // 3 frames × 2 sensors
    }

    #[test]
    fn test_empty_dataframe_error() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);
        let frame = AlignedDataFrame {
            episode_id: "episode-1".to_string(),
            frame_index: 0,
            dataset_name: "test".to_string(),
            reference_time_us: 1_000_000,
            window_size_us: 50_000,
            robot_id: "robot-1".to_string(),
            sensor_readings: HashMap::new(),
        };

        let result = adapter.ingest_dataframe(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_temporal_metadata_preservation() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::NavIC);
        let frame = create_test_dataframe("episode-1", 0, 1);

        let obs = adapter.ingest_dataframe(&frame).unwrap();
        assert_eq!(obs[0].temporal.clock_source, ClockSource::NavIC);
        assert_eq!(obs[0].temporal.sync_confidence, 0.95);
        assert!(obs[0].temporal.ingestion_time_us >= adapter.ingestion_time_us);
    }

    #[test]
    fn test_provenance_tracking() {
        let adapter = RoboticsDataFrameAdapter::new(ClockSource::GPS);
        let frame = create_test_dataframe("episode-42", 100, 1);

        let obs = adapter.ingest_dataframe(&frame).unwrap();
        assert_eq!(obs[0].metadata.get("episode_id"), Some(&"episode-42".to_string()));
        assert_eq!(obs[0].metadata.get("frame_index"), Some(&"100".to_string()));
        assert_eq!(obs[0].metadata.get("alignment_window_us"), Some(&"50000".to_string()));
    }
}
