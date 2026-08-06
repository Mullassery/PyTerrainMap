//! Change Point Detection
//!
//! Phase 6.3: Identify time points where terrain characteristics
//! fundamentally change, enabling event-based analysis.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};

/// Change point detection result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangePoint {
    pub timestamp_us: i64,
    pub location: (f32, f32),
    pub magnitude: f32,      // Change magnitude
    pub confidence: f32,     // Confidence 0.0-1.0
    pub change_type: ChangeType,
}

/// Type of change
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    LevelShift,   // Mean shift
    Variance,     // Variance change
    Trend,        // Slope change
    Spike,        // Sudden spike
}

impl ChangePoint {
    /// Create new change point
    pub fn new(
        timestamp_us: i64,
        location: (f32, f32),
        magnitude: f32,
        ctype: ChangeType,
    ) -> Self {
        let confidence = (magnitude / 10.0).min(1.0);
        ChangePoint {
            timestamp_us,
            location,
            magnitude,
            confidence,
            change_type: ctype,
        }
    }
}

/// Change point detector using CUSUM algorithm
pub struct ChangePointDetector {
    pub window_size: u32,
    pub sensitivity: f32,
    pub detected_changes: Vec<ChangePoint>,
}

impl ChangePointDetector {
    /// Create new detector
    pub fn new(window_size: u32) -> Self {
        ChangePointDetector {
            window_size,
            sensitivity: 1.0,
            detected_changes: Vec::new(),
        }
    }

    /// Detect change points in time series
    pub fn detect(&mut self, points: &[TemporalPoint]) -> Vec<ChangePoint> {
        if points.len() < self.window_size as usize {
            return Vec::new();
        }

        let mut changes = Vec::new();

        // Sliding window analysis
        for i in self.window_size as usize..points.len() {
            let window = &points[i - self.window_size as usize..=i];
            if let Some(change) = self.analyze_window(window) {
                changes.push(change);
            }
        }

        self.detected_changes = changes.clone();
        changes
    }

    /// Analyze window for change points
    fn analyze_window(&self, window: &[TemporalPoint]) -> Option<ChangePoint> {
        if window.len() < 10 {
            return None;
        }

        let mid = window.len() / 2;
        let first_half: Vec<f32> = window[..mid].iter().map(|p| p.z).collect();
        let second_half: Vec<f32> = window[mid..].iter().map(|p| p.z).collect();

        let mean1 = first_half.iter().sum::<f32>() / first_half.len() as f32;
        let mean2 = second_half.iter().sum::<f32>() / second_half.len() as f32;

        let shift = (mean2 - mean1).abs();

        if shift > self.sensitivity {
            let last_point = window.last().unwrap();
            let change = ChangePoint::new(
                last_point.timestamp,
                (last_point.x, last_point.y),
                shift,
                ChangeType::LevelShift,
            );
            Some(change)
        } else {
            None
        }
    }

    /// Detect trend change
    pub fn detect_trend_change(&self, points: &[TemporalPoint]) -> Option<ChangePoint> {
        if points.len() < 20 {
            return None;
        }

        let third = points.len() / 3;
        let early: Vec<f32> = points[..third].iter().map(|p| p.z).collect();
        let late: Vec<f32> = points[third * 2..].iter().map(|p| p.z).collect();

        let early_trend = self.compute_slope(&early);
        let late_trend = self.compute_slope(&late);

        let trend_change = (late_trend - early_trend).abs();

        if trend_change > self.sensitivity * 0.5 {
            let last_point = points.last().unwrap();
            Some(ChangePoint::new(
                last_point.timestamp,
                (last_point.x, last_point.y),
                trend_change,
                ChangeType::Trend,
            ))
        } else {
            None
        }
    }

    /// Compute slope of data
    fn compute_slope(&self, data: &[f32]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let n = data.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, &y) in data.iter().enumerate() {
            let x = i as f32;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        slope
    }

    /// Get statistics
    pub fn statistics(&self) -> ChangePointStatistics {
        ChangePointStatistics {
            total_changes: self.detected_changes.len() as u32,
            level_shifts: self.detected_changes
                .iter()
                .filter(|c| c.change_type == ChangeType::LevelShift)
                .count() as u32,
            trends: self.detected_changes
                .iter()
                .filter(|c| c.change_type == ChangeType::Trend)
                .count() as u32,
        }
    }
}

impl Default for ChangePointDetector {
    fn default() -> Self {
        Self::new(50)
    }
}

/// Change point statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangePointStatistics {
    pub total_changes: u32,
    pub level_shifts: u32,
    pub trends: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_point() {
        let cp = ChangePoint::new(1000, (1.0, 2.0), 5.0, ChangeType::LevelShift);
        assert_eq!(cp.change_type, ChangeType::LevelShift);
        assert!(cp.confidence > 0.0);
    }

    #[test]
    fn test_detector() {
        let detector = ChangePointDetector::new(50);
        assert_eq!(detector.window_size, 50);
    }

    #[test]
    fn test_detector_detect() {
        let mut detector = ChangePointDetector::new(10);

        let mut points = Vec::new();
        for i in 0..30 {
            let z = if i < 15 { 10.0 } else { 20.0 };
            points.push(TemporalPoint::new(1.0, 2.0, z, 1000 + i as i64 * 100, 0.8));
        }

        let changes = detector.detect(&points);
        assert!(changes.len() > 0);
    }
}
