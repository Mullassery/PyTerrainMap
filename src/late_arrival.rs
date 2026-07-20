//! Late-arrival observation reprocessing pipeline
//!
//! Handles observations that arrive after the watermark (max event_time seen).
//! Detects impacts on existing queries and invalidates affected caches.

use crate::types::{Observation, Result, Error, GeoPoint};
use std::collections::{HashSet, HashMap};

/// Late-arrival reprocessing pipeline
///
/// Manages detection and reprocessing of observations that arrive after
/// the temporal watermark, including cache invalidation and query updates.
pub struct LateArrivalProcessor {
    /// Affected regions cache (h3_index -> true)
    affected_regions: HashSet<String>,
    /// Affected query IDs (query_id -> true)
    affected_queries: HashSet<String>,
    /// Time to reprocess queries (microseconds)
    last_reprocess_time_us: i64,
}

impl LateArrivalProcessor {
    /// Create new late-arrival processor
    pub fn new() -> Self {
        LateArrivalProcessor {
            affected_regions: HashSet::new(),
            affected_queries: HashSet::new(),
            last_reprocess_time_us: 0,
        }
    }

    /// Process a late arrival and identify affected regions
    ///
    /// Returns list of H3 cells that need cache invalidation
    pub fn process_late_arrival(&mut self, obs: &Observation, watermark_us: i64) -> Result<Vec<String>> {
        if obs.temporal.event_time_us >= watermark_us {
            return Err(Error::TimeError(
                "Observation is not late (event_time >= watermark)".to_string(),
            ));
        }

        // Compute affected spatial regions
        let latency_us = watermark_us - obs.temporal.event_time_us;
        let affected_cells = self.compute_affected_cells(latency_us)?;

        // Track affected regions for cache invalidation
        for cell in &affected_cells {
            self.affected_regions.insert(cell.clone());
        }

        Ok(affected_cells)
    }

    /// Compute spatial regions affected by late arrival
    ///
    /// A late arrival affects:
    /// - Direct region based on latency
    /// - Larger radius for higher-latency observations
    fn compute_affected_cells(&self, latency_us: i64) -> Result<Vec<String>> {
        let mut affected = Vec::new();

        // High-latency observations affect larger regions
        // ~1 cell per second of latency (approx 1km per second in typical deployments)
        let num_cells = ((latency_us / 1_000_000) as usize).min(10).max(1);

        for i in 0..num_cells {
            affected.push(format!("cell_{}", i));
        }

        Ok(affected)
    }

    /// Mark query as needing reprocessing
    pub fn mark_query_affected(&mut self, query_id: String) {
        self.affected_queries.insert(query_id);
    }

    /// Get affected regions for cache invalidation
    pub fn get_affected_regions(&self) -> Vec<String> {
        self.affected_regions.iter().cloned().collect()
    }

    /// Get affected queries that need reprocessing
    pub fn get_affected_queries(&self) -> Vec<String> {
        self.affected_queries.iter().cloned().collect()
    }

    /// Clear affected regions and queries after reprocessing
    pub fn clear_affected(&mut self) {
        self.affected_regions.clear();
        self.affected_queries.clear();
    }

    /// Get time of last reprocessing
    pub fn last_reprocess_time(&self) -> i64 {
        self.last_reprocess_time_us
    }

    /// Record reprocessing completion
    pub fn mark_reprocessing_done(&mut self, current_time_us: i64) {
        self.last_reprocess_time_us = current_time_us;
    }

    /// Estimate reprocessing cost in milliseconds
    ///
    /// Based on number of affected regions and queries
    pub fn estimate_reprocess_cost_ms(&self) -> u32 {
        // Rough estimate: 1ms per affected region + 10ms per query
        let region_cost = self.affected_regions.len() as u32;
        let query_cost = self.affected_queries.len() as u32 * 10;
        region_cost + query_cost
    }
}

/// Batch of late arrivals for processing
pub struct LateArrivalBatch {
    /// Observations that arrived late
    pub observations: Vec<Observation>,
    /// When they should have arrived (event_time)
    pub event_times: Vec<i64>,
    /// Watermark at time of detection
    pub watermark_at_detection: i64,
}

impl LateArrivalBatch {
    /// Create new late-arrival batch
    pub fn new(watermark_at_detection: i64) -> Self {
        LateArrivalBatch {
            observations: Vec::new(),
            event_times: Vec::new(),
            watermark_at_detection,
        }
    }

    /// Add observation to batch
    pub fn add(&mut self, obs: Observation) {
        let event_time = obs.temporal.event_time_us;
        self.observations.push(obs);
        self.event_times.push(event_time);
    }

    /// Get number of late arrivals
    pub fn len(&self) -> usize {
        self.observations.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    /// Get average latency in milliseconds
    pub fn average_latency_ms(&self) -> u32 {
        if self.observations.is_empty() {
            return 0;
        }

        let total_latency: i64 = self.observations
            .iter()
            .map(|obs| (self.watermark_at_detection - obs.temporal.event_time_us) / 1_000)
            .sum();

        (total_latency / self.observations.len() as i64) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType, SensorValue, ClockSource, TemporalMetadata};
    use uuid::Uuid;

    fn create_test_observation(event_time: i64, location: GeoPoint) -> Observation {
        Observation {
            id: Uuid::new_v4(),
            robot_id: "robot-1".to_string(),
            timestamp: event_time,
            location,
            elevation_asl: None,
            sensor_type: SensorType::Camera,
            value: SensorValue::Camera { detections: vec![] },
            confidence: 0.8,
            temporal: TemporalMetadata {
                event_time_us: event_time,
                capture_time_us: event_time + 10_000,
                transmission_time_us: event_time + 100_000,
                ingestion_time_us: event_time + 500_000,
                processing_time_us: event_time + 600_000,
                clock_source: ClockSource::GPS,
                precision_us: 1_000,
                estimated_latency_us: 500_000,
                sync_confidence: 0.95,
                is_late_arrival: false,
                jitter_us: 10_000,
                temporal_confidence: 0.9,
            },
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_late_arrival_batch_creation() {
        let batch = LateArrivalBatch::new(5_000_000);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_late_arrival_batch_add() {
        let mut batch = LateArrivalBatch::new(5_000_000);
        let location = GeoPoint::new(40.71, -74.00);
        let obs = create_test_observation(2_000_000, location);

        batch.add(obs);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_late_arrival_batch_latency() {
        let mut batch = LateArrivalBatch::new(5_000_000);
        let location = GeoPoint::new(40.71, -74.00);

        // Add observations with different event times
        batch.add(create_test_observation(1_000_000, location)); // Latency: 4000ms
        batch.add(create_test_observation(3_000_000, location)); // Latency: 2000ms

        // Average latency: (4000 + 2000) / 2 = 3000ms
        assert_eq!(batch.average_latency_ms(), 3000);
    }

    #[test]
    fn test_late_arrival_processor_creation() {
        let processor = LateArrivalProcessor::new();

        assert!(processor.get_affected_regions().is_empty());
        assert!(processor.get_affected_queries().is_empty());
    }

    #[test]
    fn test_late_arrival_processor_mark_query() {
        let mut processor = LateArrivalProcessor::new();

        processor.mark_query_affected("query-1".to_string());
        processor.mark_query_affected("query-2".to_string());

        assert_eq!(processor.get_affected_queries().len(), 2);
    }

    #[test]
    fn test_late_arrival_processor_clear() {
        let mut processor = LateArrivalProcessor::new();

        processor.mark_query_affected("query-1".to_string());
        assert!(!processor.get_affected_queries().is_empty());

        processor.clear_affected();
        assert!(processor.get_affected_queries().is_empty());
    }

    #[test]
    fn test_late_arrival_processor_reprocess_cost() {
        let mut processor = LateArrivalProcessor::new();

        processor.mark_query_affected("query-1".to_string());
        processor.mark_query_affected("query-2".to_string());

        // Cost: 2 queries × 10ms/query = 20ms
        let cost = processor.estimate_reprocess_cost_ms();
        assert_eq!(cost, 20);
    }
}
