//! Data contracts and lineage tracking for ecosystem integration
//!
//! Ensures consistency across PyRoboFrames → PyRoboVision → PyTerrainMap pipeline.
//! Maintains immutable audit trail of data provenance.

use crate::types::{Observation, Result, Error};
use std::collections::HashMap;
use chrono::Utc;

/// Unified observation schema for ecosystem boundaries
#[derive(Clone, Debug)]
pub struct UnifiedObservationSchema {
    /// Contract version (e.g., "1.0")
    pub version: String,
    /// Required fields that must be present
    pub required_fields: Vec<String>,
    /// Field descriptions (field_name -> description)
    pub field_descriptions: HashMap<String, String>,
}

impl UnifiedObservationSchema {
    /// Create v1.0 schema with standard requirements
    pub fn v1_0() -> Self {
        let mut descriptions = HashMap::new();
        descriptions.insert("id".to_string(), "Unique observation UUID".to_string());
        descriptions.insert("robot_id".to_string(), "Robot that made this observation".to_string());
        descriptions.insert("location".to_string(), "Geographic position".to_string());
        descriptions.insert("temporal".to_string(), "Full temporal metadata (5 dimensions)".to_string());
        descriptions.insert("sensor_type".to_string(), "Type of sensor (Camera, LiDAR, etc)".to_string());
        descriptions.insert("value".to_string(), "Sensor reading (type-specific)".to_string());
        descriptions.insert("confidence".to_string(), "Sensor confidence 0.0-1.0".to_string());
        descriptions.insert("metadata".to_string(), "Provenance (episode_id, frame_index, etc)".to_string());

        UnifiedObservationSchema {
            version: "1.0".to_string(),
            required_fields: vec![
                "id".to_string(),
                "robot_id".to_string(),
                "temporal.event_time_us".to_string(),
                "temporal.ingestion_time_us".to_string(),
                "sensor_type".to_string(),
                "confidence".to_string(),
            ],
            field_descriptions: descriptions,
        }
    }

    /// Validate observation against schema
    pub fn validate(&self, obs: &Observation) -> Result<()> {
        // Check required temporal fields
        if obs.temporal.event_time_us == 0 {
            return Err(Error::InvalidObservation(
                "Missing event_time_us".to_string(),
            ));
        }

        if obs.temporal.ingestion_time_us == 0 {
            return Err(Error::InvalidObservation(
                "Missing ingestion_time_us".to_string(),
            ));
        }

        // Check required provenance fields
        if !obs.metadata.contains_key("episode_id") {
            return Err(Error::InvalidObservation(
                "Missing episode_id in metadata".to_string(),
            ));
        }

        // Check temporal consistency
        if obs.temporal.event_time_us > obs.temporal.ingestion_time_us {
            return Err(Error::TimeError(
                "event_time cannot be after ingestion_time".to_string(),
            ));
        }

        Ok(())
    }
}

/// Lineage entry tracking data provenance
#[derive(Clone, Debug)]
pub struct LineageEntry {
    /// Observation UUID
    pub observation_id: String,
    /// Pipeline stage that processed this observation
    pub stage: String,
    /// Timestamp of processing (microseconds)
    pub timestamp_us: i64,
    /// Version of processing component
    pub version: String,
    /// Processor ID (robot, service, etc)
    pub processor_id: String,
}

/// Lineage tracker for full provenance chain
pub struct LineageTracker {
    /// All lineage entries: observation_id -> Vec<LineageEntry>
    lineage: HashMap<String, Vec<LineageEntry>>,
}

impl LineageTracker {
    /// Create new tracker
    pub fn new() -> Self {
        LineageTracker {
            lineage: HashMap::new(),
        }
    }

    /// Record lineage entry for observation
    pub fn record_entry(&mut self, observation_id: String, stage: String, version: String, processor_id: String) {
        let entry = LineageEntry {
            observation_id: observation_id.clone(),
            stage,
            timestamp_us: Utc::now().timestamp_micros(),
            version,
            processor_id,
        };

        self.lineage
            .entry(observation_id)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Get full lineage for observation
    pub fn get_lineage(&self, observation_id: &str) -> Option<Vec<LineageEntry>> {
        self.lineage.get(observation_id).cloned()
    }

    /// Reconstruct processing history
    pub fn get_history(&self, observation_id: &str) -> Result<Vec<String>> {
        let entries = self.get_lineage(observation_id)
            .ok_or_else(|| Error::QueryError(
                format!("No lineage for observation {}", observation_id),
            ))?;

        Ok(entries
            .into_iter()
            .map(|e| format!("{} (v{}) @ {} by {}", e.stage, e.version, e.timestamp_us, e.processor_id))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType, SensorValue, TemporalMetadata, ClockSource};
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_valid_observation() -> Observation {
        Observation {
            id: Uuid::new_v4(),
            robot_id: "robot-1".to_string(),
            timestamp: 1_000_000,
            location: GeoPoint::new(40.71, -74.00),
            elevation_asl: None,
            sensor_type: SensorType::Camera,
            value: SensorValue::Camera { detections: vec![] },
            confidence: 0.9,
            temporal: TemporalMetadata {
                event_time_us: 1_000_000,
                capture_time_us: 1_000_000,
                transmission_time_us: 1_100_000,
                ingestion_time_us: 1_500_000,
                processing_time_us: 1_600_000,
                clock_source: ClockSource::GPS,
                precision_us: 1_000,
                estimated_latency_us: 500_000,
                sync_confidence: 0.95,
                is_late_arrival: false,
                jitter_us: 10_000,
                temporal_confidence: 0.95,
            },
            metadata: {
                let mut m = HashMap::new();
                m.insert("episode_id".to_string(), "episode-1".to_string());
                m.insert("frame_index".to_string(), "0".to_string());
                m
            },
        }
    }

    #[test]
    fn test_schema_creation() {
        let schema = UnifiedObservationSchema::v1_0();
        assert_eq!(schema.version, "1.0");
        assert!(schema.required_fields.contains(&"id".to_string()));
    }

    #[test]
    fn test_schema_validation_valid() {
        let schema = UnifiedObservationSchema::v1_0();
        let obs = create_valid_observation();
        assert!(schema.validate(&obs).is_ok());
    }

    #[test]
    fn test_schema_validation_missing_episode_id() {
        let schema = UnifiedObservationSchema::v1_0();
        let mut obs = create_valid_observation();
        obs.metadata.clear();

        let result = schema.validate(&obs);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_validation_temporal_consistency() {
        let schema = UnifiedObservationSchema::v1_0();
        let mut obs = create_valid_observation();

        // Make event_time after ingestion_time (invalid)
        obs.temporal.event_time_us = 2_000_000;
        obs.temporal.ingestion_time_us = 1_500_000;

        let result = schema.validate(&obs);
        assert!(result.is_err());
    }

    #[test]
    fn test_lineage_tracker_creation() {
        let tracker = LineageTracker::new();
        assert_eq!(tracker.lineage.len(), 0);
    }

    #[test]
    fn test_lineage_recording() {
        let mut tracker = LineageTracker::new();
        let obs_id = "obs-123".to_string();

        tracker.record_entry(
            obs_id.clone(),
            "PyRoboFrames".to_string(),
            "1.0".to_string(),
            "robot-1".to_string(),
        );

        let lineage = tracker.get_lineage(&obs_id);
        assert!(lineage.is_some());
        assert_eq!(lineage.unwrap().len(), 1);
    }

    #[test]
    fn test_lineage_history() {
        let mut tracker = LineageTracker::new();
        let obs_id = "obs-123".to_string();

        tracker.record_entry(obs_id.clone(), "PyRoboFrames".to_string(), "1.0".to_string(), "robot-1".to_string());
        tracker.record_entry(obs_id.clone(), "PyRoboVision".to_string(), "2.1".to_string(), "model-selector".to_string());
        tracker.record_entry(obs_id.clone(), "PyTerrainMap".to_string(), "0.2".to_string(), "fusion-engine".to_string());

        let history = tracker.get_history(&obs_id).unwrap();
        assert_eq!(history.len(), 3);
        assert!(history[0].contains("PyRoboFrames"));
        assert!(history[1].contains("PyRoboVision"));
        assert!(history[2].contains("PyTerrainMap"));
    }

    #[test]
    fn test_lineage_missing_observation() {
        let tracker = LineageTracker::new();
        let result = tracker.get_history("nonexistent");
        assert!(result.is_err());
    }
}
