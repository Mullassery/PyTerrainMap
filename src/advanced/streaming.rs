//! Real-time Observation Streaming
//!
//! Phase 5.3: Efficient streaming of terrain observations with
//! windowing, batching, and backpressure handling.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Stream window configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamWindow {
    pub window_size_us: i64,
    pub slide_interval_us: i64,
    pub max_batch_size: u32,
}

impl StreamWindow {
    /// Create new stream window
    pub fn new(window_size_us: i64, slide_interval_us: i64, max_batch_size: u32) -> Self {
        StreamWindow {
            window_size_us,
            slide_interval_us,
            max_batch_size,
        }
    }

    /// Get window overlap ratio
    pub fn overlap_ratio(&self) -> f32 {
        1.0 - (self.slide_interval_us as f32 / self.window_size_us as f32)
    }
}

/// Stream batch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamBatch {
    pub batch_id: u64,
    pub points: Vec<TemporalPoint>,
    pub created_at_us: i64,
    pub received_at_us: i64,
    pub is_complete: bool,
}

impl StreamBatch {
    /// Create new stream batch
    pub fn new(batch_id: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        StreamBatch {
            batch_id,
            points: Vec::new(),
            created_at_us: now,
            received_at_us: now,
            is_complete: false,
        }
    }

    /// Add point to batch
    pub fn add_point(&mut self, point: TemporalPoint) -> bool {
        if self.points.len() >= 1000 {
            return false; // Batch full
        }
        self.points.push(point);
        true
    }

    /// Get batch size
    pub fn size(&self) -> u32 {
        self.points.len() as u32
    }

    /// Get batch latency in microseconds
    pub fn latency_us(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        now - self.created_at_us
    }

    /// Mark batch complete
    pub fn complete(&mut self) {
        self.is_complete = true;
    }
}

/// Stream window state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WindowState {
    pub window_start_us: i64,
    pub window_end_us: i64,
    pub points_in_window: Vec<TemporalPoint>,
    pub is_closed: bool,
}

impl WindowState {
    /// Create new window
    pub fn new(start_us: i64, size_us: i64) -> Self {
        WindowState {
            window_start_us: start_us,
            window_end_us: start_us + size_us,
            points_in_window: Vec::new(),
            is_closed: false,
        }
    }

    /// Add point if in window
    pub fn try_add_point(&mut self, point: &TemporalPoint) -> bool {
        if point.timestamp >= self.window_start_us && point.timestamp < self.window_end_us {
            self.points_in_window.push(*point);
            true
        } else {
            false
        }
    }

    /// Check if point is after window
    pub fn is_point_after_window(&self, point: &TemporalPoint) -> bool {
        point.timestamp >= self.window_end_us
    }
}

/// Real-time streaming pipeline
pub struct StreamingPipeline {
    pub window: StreamWindow,
    pub backpressure_enabled: bool,
    pub current_batch: StreamBatch,
    pub windows: VecDeque<WindowState>,
    pub batches_processed: u64,
    pub points_processed: u64,
}

impl StreamingPipeline {
    /// Create new streaming pipeline
    pub fn new(window: StreamWindow) -> Self {
        StreamingPipeline {
            window,
            backpressure_enabled: true,
            current_batch: StreamBatch::new(0),
            windows: VecDeque::new(),
            batches_processed: 0,
            points_processed: 0,
        }
    }

    /// Add point to stream
    pub fn add_point(&mut self, point: TemporalPoint) -> StreamResult {
        // Try to add to current batch
        if !self.current_batch.add_point(point) {
            // Batch is full, flush and create new
            return StreamResult::BatchFull;
        }

        self.points_processed += 1;

        // Check windowing
        self.update_windows(&point);

        StreamResult::Success
    }

    /// Update active windows
    fn update_windows(&mut self, point: &TemporalPoint) {
        // Create new window if needed
        if self.windows.is_empty() {
            let new_window = WindowState::new(point.timestamp, self.window.window_size_us);
            self.windows.push_back(new_window);
        }

        // Add point to all applicable windows
        for window in self.windows.iter_mut() {
            if !window.is_closed {
                window.try_add_point(point);
            }
        }

        // Check if we need new windows (sliding window)
        if let Some(last_window) = self.windows.back() {
            if point.timestamp > last_window.window_end_us {
                // Create new window
                let new_start = last_window.window_start_us + self.window.slide_interval_us;
                let new_window = WindowState::new(new_start, self.window.window_size_us);
                self.windows.push_back(new_window);
            }
        }

        // Remove closed windows
        while self.windows.len() > 0 && self.windows[0].is_closed {
            self.windows.pop_front();
        }
    }

    /// Flush current batch
    pub fn flush_batch(&mut self) -> StreamBatch {
        self.current_batch.complete();
        let batch = self.current_batch.clone();
        self.batches_processed += 1;

        // Create new batch
        let new_batch_id = self.batches_processed;
        self.current_batch = StreamBatch::new(new_batch_id);

        batch
    }

    /// Close window explicitly
    pub fn close_window(&mut self) -> Option<WindowState> {
        if let Some(mut window) = self.windows.pop_front() {
            window.is_closed = true;
            Some(window)
        } else {
            None
        }
    }

    /// Get streaming statistics
    pub fn statistics(&self) -> StreamingStatistics {
        StreamingStatistics {
            batches_processed: self.batches_processed,
            current_batch_size: self.current_batch.size(),
            points_processed: self.points_processed,
            active_windows: self.windows.len() as u32,
            throughput_points_per_sec: self.calculate_throughput(),
        }
    }

    /// Calculate throughput
    fn calculate_throughput(&self) -> f32 {
        if self.current_batch.latency_us() == 0 {
            return 0.0;
        }
        (self.points_processed as f32 * 1_000_000.0) / self.current_batch.latency_us() as f32
    }
}

impl Default for StreamingPipeline {
    fn default() -> Self {
        Self::new(StreamWindow::new(
            1_000_000, // 1 second window
            100_000,   // 100ms slide
            1000,      // 1000 points max batch
        ))
    }
}

/// Stream result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StreamResult {
    Success,
    BatchFull,
    BackpressureApplied,
    WindowClosed,
}

/// Streaming statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamingStatistics {
    pub batches_processed: u64,
    pub current_batch_size: u32,
    pub points_processed: u64,
    pub active_windows: u32,
    pub throughput_points_per_sec: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_window() {
        let window = StreamWindow::new(1_000_000, 100_000, 1000);
        assert_eq!(window.overlap_ratio(), 0.9);
    }

    #[test]
    fn test_stream_batch() {
        let mut batch = StreamBatch::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);

        assert!(batch.add_point(point));
        assert_eq!(batch.size(), 1);
    }

    #[test]
    fn test_window_state() {
        let mut window = WindowState::new(0, 1_000_000);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 500_000, 0.8);

        assert!(window.try_add_point(&point));
        assert_eq!(window.points_in_window.len(), 1);
    }

    #[test]
    fn test_streaming_pipeline() {
        let mut pipeline = StreamingPipeline::default();
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);

        pipeline.add_point(point);
        assert_eq!(pipeline.points_processed, 1);
    }

    #[test]
    fn test_streaming_statistics() {
        let mut pipeline = StreamingPipeline::default();
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);

        pipeline.add_point(point);
        let stats = pipeline.statistics();

        assert_eq!(stats.points_processed, 1);
        assert!(stats.throughput_points_per_sec > 0.0);
    }
}
