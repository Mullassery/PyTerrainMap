//! Temporal indexing and decay functions for observation aging
//!
//! Manages time-series data, observation freshness, and temporal decay.
//! Observations maintain immutable original confidence; decay applied on read.

use crate::types::{Result, Error};

/// Time-based decay function type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecayFunction {
    /// Linear decay: confidence = base - (age_ms / half_life_ms) * base
    Linear { half_life_ms: i64 },
    /// Exponential decay: confidence = base * (0.5 ^ (age_ms / half_life_ms))
    Exponential { half_life_ms: i64 },
    /// No decay (observations stay at original confidence forever)
    None,
}

impl DecayFunction {
    /// Apply decay function to a confidence value given age
    pub fn apply(&self, original_confidence: f32, age_ms: i64) -> f32 {
        if age_ms < 0 {
            return original_confidence;
        }

        match self {
            DecayFunction::None => original_confidence,
            DecayFunction::Linear { half_life_ms } => {
                if *half_life_ms <= 0 {
                    return 0.0;
                }
                // Linear decay reaches 0 at 2 * half_life_ms
                let decay_ratio = age_ms as f32 / (2.0 * *half_life_ms as f32);
                (original_confidence * (1.0 - decay_ratio)).max(0.0)
            }
            DecayFunction::Exponential { half_life_ms } => {
                if *half_life_ms <= 0 {
                    return 0.0;
                }
                let exponent = age_ms as f32 / *half_life_ms as f32;
                original_confidence * 0.5_f32.powf(exponent)
            }
        }
    }

    /// Get confidence at a specific time offset
    pub fn confidence_at(&self, original_confidence: f32, age_ms: i64) -> f32 {
        self.apply(original_confidence, age_ms)
    }
}

/// Temporal index for time-based queries
pub struct TemporalIndex {
    /// Timestamps of indexed observations (sorted)
    timestamps: Vec<i64>,
    /// Decay function to apply
    decay: DecayFunction,
}

impl TemporalIndex {
    /// Create new temporal index with decay function
    pub fn new(decay: DecayFunction) -> Self {
        TemporalIndex {
            timestamps: Vec::new(),
            decay,
        }
    }

    /// Add an observation timestamp
    pub fn insert(&mut self, timestamp_us: i64) -> Result<()> {
        if timestamp_us < 0 {
            return Err(Error::TimeError("Timestamp must be non-negative".to_string()));
        }

        // Insert in sorted order (binary search)
        match self.timestamps.binary_search(&timestamp_us) {
            Ok(_) => {
                // Duplicate timestamp - allow it (multiple observations at same time)
                self.timestamps.push(timestamp_us);
            }
            Err(pos) => {
                self.timestamps.insert(pos, timestamp_us);
            }
        }

        Ok(())
    }

    /// Get observations in time range [from_us, to_us]
    pub fn range_query(&self, from_us: i64, to_us: i64) -> Result<Vec<usize>> {
        if from_us < 0 || to_us < 0 {
            return Err(Error::TimeError("Timestamps must be non-negative".to_string()));
        }
        if from_us > to_us {
            return Err(Error::TimeError("from_us must be <= to_us".to_string()));
        }

        let start_idx = self
            .timestamps
            .binary_search(&from_us)
            .unwrap_or_else(|x| x);
        let end_idx = self
            .timestamps
            .binary_search(&to_us)
            .map(|x| x + 1)
            .unwrap_or_else(|x| x);

        Ok((start_idx..end_idx).collect())
    }

    /// Get observations newer than timestamp
    pub fn since(&self, timestamp_us: i64) -> Result<Vec<usize>> {
        if timestamp_us < 0 {
            return Err(Error::TimeError("Timestamp must be non-negative".to_string()));
        }

        let start_idx = self
            .timestamps
            .binary_search(&timestamp_us)
            .unwrap_or_else(|x| x);

        Ok((start_idx..self.timestamps.len()).collect())
    }

    /// Get oldest N observations
    pub fn oldest_n(&self, n: usize) -> Vec<usize> {
        let count = n.min(self.timestamps.len());
        (0..count).collect()
    }

    /// Get newest N observations
    pub fn newest_n(&self, n: usize) -> Vec<usize> {
        let len = self.timestamps.len();
        let count = n.min(len);
        ((len - count)..len).collect()
    }

    /// Get decayed confidence for an observation at current time
    pub fn decayed_confidence(&self, obs_index: usize, current_time_us: i64, original_confidence: f32) -> Result<f32> {
        if obs_index >= self.timestamps.len() {
            return Err(Error::QueryError(format!("Observation index {} out of range", obs_index)));
        }

        let obs_timestamp = self.timestamps[obs_index];
        let age_us = current_time_us - obs_timestamp;
        let age_ms = age_us / 1000; // Convert microseconds to milliseconds

        Ok(self.decay.apply(original_confidence, age_ms))
    }

    /// Get all timestamps (for testing/debugging)
    pub fn all_timestamps(&self) -> &[i64] {
        &self.timestamps
    }

    /// Get current decay function
    pub fn decay_function(&self) -> DecayFunction {
        self.decay
    }

    /// Clear all indexed timestamps
    pub fn clear(&mut self) {
        self.timestamps.clear();
    }

    /// Get number of observations indexed
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }
}

/// Enhanced temporal index with event-time ordering and watermarking
///
/// Tracks both event_time (when observation occurred) and arrival_time (when we received it).
/// Maintains watermark to detect and handle late arrivals.
pub struct TemporalIndexEnhanced {
    /// Event timestamps (sorted by event time, not arrival)
    event_times: Vec<i64>,
    /// Arrival timestamps (parallel to event_times)
    arrival_times: Vec<i64>,
    /// Maximum event time seen so far (watermark)
    watermark_us: i64,
    /// Late arrivals detected (event_time < watermark but arrival after)
    late_arrivals: Vec<usize>,
    /// Decay function to apply
    decay: DecayFunction,
}

impl TemporalIndexEnhanced {
    /// Create new enhanced temporal index
    pub fn new(decay: DecayFunction) -> Self {
        TemporalIndexEnhanced {
            event_times: Vec::new(),
            arrival_times: Vec::new(),
            watermark_us: 0,
            late_arrivals: Vec::new(),
            decay,
        }
    }

    /// Insert observation with event_time and arrival_time
    ///
    /// Returns true if this is a late arrival (event_time < watermark)
    pub fn insert(&mut self, event_time_us: i64, arrival_time_us: i64) -> Result<bool> {
        if event_time_us < 0 || arrival_time_us < 0 {
            return Err(Error::TimeError("Timestamps must be non-negative".to_string()));
        }

        // Check if this is a late arrival
        let is_late = event_time_us < self.watermark_us;

        // Update watermark if this observation is newer
        if event_time_us > self.watermark_us {
            self.watermark_us = event_time_us;
        }

        // Insert in sorted order by event_time
        match self.event_times.binary_search(&event_time_us) {
            Ok(pos) => {
                // Duplicate event time - insert after existing
                self.event_times.insert(pos + 1, event_time_us);
                self.arrival_times.insert(pos + 1, arrival_time_us);
            }
            Err(pos) => {
                self.event_times.insert(pos, event_time_us);
                self.arrival_times.insert(pos, arrival_time_us);
            }
        }

        // Track late arrivals
        if is_late {
            let idx = self.event_times.iter().position(|&t| t == event_time_us)
                .unwrap_or(self.event_times.len() - 1);
            self.late_arrivals.push(idx);
        }

        Ok(is_late)
    }

    /// Get watermark (maximum event_time seen)
    pub fn watermark(&self) -> i64 {
        self.watermark_us
    }

    /// Get observations that arrived late
    pub fn late_arrivals(&self) -> &[usize] {
        &self.late_arrivals
    }

    /// Check if observation is late arrival
    pub fn is_late_arrival(&self, index: usize) -> bool {
        self.late_arrivals.contains(&index)
    }

    /// Get latency (arrival_time - event_time) in milliseconds
    pub fn latency_ms(&self, index: usize) -> Result<i64> {
        if index >= self.event_times.len() {
            return Err(Error::QueryError(format!("Index {} out of range", index)));
        }
        // arrival and event times are in microseconds, convert to milliseconds
        let latency_us = self.arrival_times[index] - self.event_times[index];
        Ok(latency_us / 1_000)
    }

    /// Get all event times (for debugging)
    pub fn event_times(&self) -> &[i64] {
        &self.event_times
    }

    /// Get all arrival times (for debugging)
    pub fn arrival_times(&self) -> &[i64] {
        &self.arrival_times
    }

    /// Query by event time range [from_us, to_us]
    pub fn range_query(&self, from_us: i64, to_us: i64) -> Result<Vec<usize>> {
        if from_us < 0 || to_us < 0 {
            return Err(Error::TimeError("Timestamps must be non-negative".to_string()));
        }
        if from_us > to_us {
            return Err(Error::TimeError("from_us must be <= to_us".to_string()));
        }

        let start_idx = self.event_times
            .binary_search(&from_us)
            .unwrap_or_else(|x| x);
        let end_idx = self.event_times
            .binary_search(&to_us)
            .map(|x| x + 1)
            .unwrap_or_else(|x| x);

        Ok((start_idx..end_idx).collect())
    }

    /// Get observations newer than event_time
    pub fn since(&self, event_time_us: i64) -> Result<Vec<usize>> {
        if event_time_us < 0 {
            return Err(Error::TimeError("Timestamp must be non-negative".to_string()));
        }

        let start_idx = self.event_times
            .binary_search(&event_time_us)
            .unwrap_or_else(|x| x);

        Ok((start_idx..self.event_times.len()).collect())
    }

    /// Calculate temporal quality factor (0.0-1.0) based on latency
    ///
    /// Quality decreases with latency:
    /// - <100ms latency: quality 1.0
    /// - >5s latency: quality 0.3
    pub fn temporal_quality(&self, index: usize) -> Result<f32> {
        let latency_ms = self.latency_ms(index)?;

        if latency_ms <= 100 {
            Ok(1.0)
        } else if latency_ms >= 5000 {
            Ok(0.3)
        } else {
            // Linear interpolation between 100ms (1.0) and 5000ms (0.3)
            let ratio = (latency_ms - 100) as f32 / (5000.0 - 100.0);
            Ok(1.0 - (ratio * 0.7))
        }
    }

    /// Get number of observations
    pub fn len(&self) -> usize {
        self.event_times.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.event_times.is_empty()
    }

    /// Clear all observations
    pub fn clear(&mut self) {
        self.event_times.clear();
        self.arrival_times.clear();
        self.late_arrivals.clear();
        self.watermark_us = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_function_none() {
        let decay = DecayFunction::None;
        assert_eq!(decay.apply(0.9, 0), 0.9);
        assert_eq!(decay.apply(0.9, 10000), 0.9);
        assert_eq!(decay.apply(0.9, 1000000), 0.9);
    }

    #[test]
    fn test_decay_function_linear() {
        let decay = DecayFunction::Linear { half_life_ms: 1000 };

        // At creation (age=0)
        assert_eq!(decay.apply(1.0, 0), 1.0);

        // At half-life, should be 0.5 (linear from 1.0 at t=0 to 0.0 at t=2000)
        assert_eq!(decay.apply(1.0, 1000), 0.5);

        // At 2x half-life, should be 0.0 (clamped)
        assert_eq!(decay.apply(1.0, 2000), 0.0);

        // Linear decay at 1/4 of the zero time (500ms out of 2000ms)
        // Remaining = 1 - (500/2000) = 0.75
        assert!((decay.apply(0.8, 500) - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_decay_function_exponential() {
        let decay = DecayFunction::Exponential { half_life_ms: 1000 };

        // At creation (age=0)
        assert!((decay.apply(1.0, 0) - 1.0).abs() < 0.001);

        // At half-life, should be 0.5
        assert!((decay.apply(1.0, 1000) - 0.5).abs() < 0.001);

        // At 2x half-life, should be 0.25
        assert!((decay.apply(1.0, 2000) - 0.25).abs() < 0.001);

        // At 3x half-life, should be 0.125
        assert!((decay.apply(1.0, 3000) - 0.125).abs() < 0.001);
    }

    #[test]
    fn test_temporal_index_creation() {
        let index = TemporalIndex::new(DecayFunction::None);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_temporal_index_insert() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        index.insert(1000).unwrap();
        assert_eq!(index.len(), 1);

        index.insert(2000).unwrap();
        assert_eq!(index.len(), 2);

        // Duplicates are allowed
        index.insert(2000).unwrap();
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_temporal_index_sorted() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        // Insert out of order
        index.insert(3000).unwrap();
        index.insert(1000).unwrap();
        index.insert(2000).unwrap();

        let timestamps = index.all_timestamps();
        assert_eq!(timestamps, &[1000, 2000, 3000]);
    }

    #[test]
    fn test_temporal_range_query() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        for t in &[1000, 2000, 3000, 4000, 5000] {
            index.insert(*t).unwrap();
        }

        // Query [2000, 4000]
        let results = index.range_query(2000, 4000).unwrap();
        assert_eq!(results, vec![1, 2, 3]); // Indices for 2000, 3000, 4000

        // Query exact boundaries
        let results = index.range_query(1000, 5000).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_temporal_since() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        for t in &[1000, 2000, 3000, 4000] {
            index.insert(*t).unwrap();
        }

        // Since 3000
        let results = index.since(3000).unwrap();
        assert_eq!(results, vec![2, 3]); // Indices for 3000, 4000
    }

    #[test]
    fn test_temporal_oldest_newest() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        for t in &[1000, 2000, 3000, 4000, 5000] {
            index.insert(*t).unwrap();
        }

        let oldest = index.oldest_n(2);
        assert_eq!(oldest, vec![0, 1]); // First two

        let newest = index.newest_n(2);
        assert_eq!(newest, vec![3, 4]); // Last two
    }

    #[test]
    fn test_decayed_confidence() {
        let mut index = TemporalIndex::new(DecayFunction::Exponential { half_life_ms: 1000 });

        index.insert(0).unwrap(); // At t=0

        // At current time = 1000ms later (1_000_000 microseconds)
        let decayed = index.decayed_confidence(0, 1_000_000, 1.0).unwrap();
        assert!((decayed - 0.5).abs() < 0.001); // Should be ~0.5
    }

    #[test]
    fn test_temporal_invalid_operations() {
        let mut index = TemporalIndex::new(DecayFunction::None);

        // Negative timestamp should fail
        assert!(index.insert(-1000).is_err());

        // Zero timestamp is valid (start of epoch)
        assert!(index.insert(0).is_ok());

        // Add another timestamp and test range queries
        index.insert(1000).unwrap();

        // Range query with invalid times
        assert!(index.range_query(-1, 1000).is_err());
        assert!(index.range_query(1000, 500).is_err());

        // Out of range index
        assert!(index.decayed_confidence(10, 5000, 0.9).is_err());
    }

    #[test]
    fn test_decay_zero_half_life() {
        let decay_linear = DecayFunction::Linear { half_life_ms: 0 };
        assert_eq!(decay_linear.apply(1.0, 100), 0.0);

        let decay_exp = DecayFunction::Exponential { half_life_ms: 0 };
        assert_eq!(decay_exp.apply(1.0, 100), 0.0);
    }

    #[test]
    fn test_decay_negative_age() {
        let decay = DecayFunction::Exponential { half_life_ms: 1000 };
        // Negative age means future timestamp - should return original
        assert_eq!(decay.apply(0.9, -1000), 0.9);
    }

    // ========== TemporalIndexEnhanced Tests ==========

    #[test]
    fn test_enhanced_index_creation() {
        let index = TemporalIndexEnhanced::new(DecayFunction::None);
        assert_eq!(index.watermark(), 0);
        assert!(index.is_empty());
        assert_eq!(index.late_arrivals().len(), 0);
    }

    #[test]
    fn test_enhanced_index_in_order_insertion() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Events in order
        assert!(!index.insert(1000, 1100).unwrap());  // event_time=1000, arrival_time=1100
        assert!(!index.insert(2000, 2100).unwrap());  // event_time=2000, arrival_time=2100
        assert!(!index.insert(3000, 3100).unwrap());  // event_time=3000, arrival_time=3100

        assert_eq!(index.len(), 3);
        assert_eq!(index.watermark(), 3000);
        assert_eq!(index.late_arrivals().len(), 0);
    }

    #[test]
    fn test_enhanced_index_late_arrival() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Insert in-order events to establish watermark
        index.insert(1000, 1100).unwrap();
        index.insert(3000, 3100).unwrap();
        assert_eq!(index.watermark(), 3000);

        // Now insert an event with earlier event_time (late arrival)
        assert!(index.insert(2000, 5000).unwrap());
        assert_eq!(index.late_arrivals().len(), 1);
    }

    #[test]
    fn test_enhanced_index_latency_calculation() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Event at 1_000_000µs (1s), arrives at 1_100_000µs (1.1s) -> latency 100ms
        index.insert(1_000_000, 1_100_000).unwrap();
        assert_eq!(index.latency_ms(0).unwrap(), 100);

        // Event at 2_000_000µs (2s), arrives at 7_000_000µs (7s) -> latency 5000ms
        index.insert(2_000_000, 7_000_000).unwrap();
        assert_eq!(index.latency_ms(1).unwrap(), 5000);
    }

    #[test]
    fn test_enhanced_index_temporal_quality() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Fast arrival (100ms latency) -> quality 1.0
        index.insert(1_000_000, 1_100_000).unwrap();
        assert!((index.temporal_quality(0).unwrap() - 1.0).abs() < 0.001);

        // Slow arrival (5000ms latency) -> quality 0.3
        index.insert(2_000_000, 7_000_000).unwrap();
        assert!((index.temporal_quality(1).unwrap() - 0.3).abs() < 0.001);

        // Medium latency (2550ms = midpoint) -> quality ~0.65
        index.insert(3_000_000, 5_550_000).unwrap();
        let quality = index.temporal_quality(2).unwrap();
        assert!(quality > 0.6 && quality < 0.7);
    }

    #[test]
    fn test_enhanced_index_out_of_order_events() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Insert completely out of order
        // First event: event_time=5000, arrival=5100, watermark becomes 5000
        index.insert(5_000_000, 5_100_000).unwrap();

        // Subsequent events have event_time < 5000, so they're late arrivals
        index.insert(1_000_000, 1_100_000).unwrap();  // Late arrival (1000 < 5000)
        index.insert(3_000_000, 3_100_000).unwrap();  // Late arrival (3000 < 5000)
        index.insert(2_000_000, 2_100_000).unwrap();  // Late arrival (2000 < 5000)

        // Should be sorted by event_time
        assert_eq!(index.event_times(), &[1_000_000, 2_000_000, 3_000_000, 5_000_000]);
        assert_eq!(index.watermark(), 5_000_000);

        // Three late arrivals (indices 0, 1, 2 all have event_time < watermark)
        assert_eq!(index.late_arrivals().len(), 3);
    }

    #[test]
    fn test_enhanced_index_range_query() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        for i in 0..10 {
            index.insert(i * 1000, i * 1000 + 100).unwrap();
        }

        // Query [2000, 6000]
        let results = index.range_query(2000, 6000).unwrap();
        assert_eq!(results, vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_enhanced_index_since_query() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        for i in 0..5 {
            index.insert(i * 1000, i * 1000 + 100).unwrap();
        }

        // Query since 2000 (inclusive)
        let results = index.since(2000).unwrap();
        assert_eq!(results, vec![2, 3, 4]);
    }

    #[test]
    fn test_enhanced_index_clear() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        index.insert(1000, 1100).unwrap();
        index.insert(2000, 2100).unwrap();
        assert_eq!(index.len(), 2);

        index.clear();
        assert!(index.is_empty());
        assert_eq!(index.watermark(), 0);
        assert_eq!(index.late_arrivals().len(), 0);
    }

    #[test]
    fn test_enhanced_index_watermark_invariant() {
        let mut index = TemporalIndexEnhanced::new(DecayFunction::None);

        // Watermark should never go backwards
        index.insert(1000, 1100).unwrap();
        assert_eq!(index.watermark(), 1000);

        index.insert(500, 1200).unwrap();  // Earlier event_time
        assert_eq!(index.watermark(), 1000);  // Watermark unchanged

        index.insert(1500, 1300).unwrap();
        assert_eq!(index.watermark(), 1500);  // Watermark moves forward
    }
}
