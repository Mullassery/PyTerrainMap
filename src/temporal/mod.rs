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
}
