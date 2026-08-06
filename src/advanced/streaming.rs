//! Real-time Observation Streaming
//!
//! Phase 5.3: Efficient streaming of terrain observations with
//! windowing, batching, and backpressure handling.

use crate::temporal::TemporalPoint;
use serde::{Deserialize, Serialize};

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
}

/// Stub: Streaming pipeline (to be implemented in Phase 5.3)
pub struct StreamingPipeline {
    pub window: StreamWindow,
    pub backpressure_enabled: bool,
}

impl StreamingPipeline {
    /// Create new streaming pipeline
    pub fn new(window: StreamWindow) -> Self {
        StreamingPipeline {
            window,
            backpressure_enabled: true,
        }
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
