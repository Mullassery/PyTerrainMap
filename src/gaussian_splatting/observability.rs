//! OpenTelemetry observability for Gaussian Splatting operations
//!
//! Provides distributed tracing, metrics, and structured logging for:
//! - Observation ingestion and fusion
//! - Temporal decay and pruning
//! - Spatial queries and uncertainty computation
//! - Multi-bot fleet coordination
//! - Change event detection

use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Observability context for a Gaussian Splatting operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObsContext {
    /// Trace ID (correlates operations across microservices)
    pub trace_id: String,
    /// Span ID (unique to this operation)
    pub span_id: String,
    /// Operation name (e.g., "fusion", "decay", "query_radius")
    pub operation: String,
    /// Bot ID performing the operation
    pub bot_id: Option<String>,
    /// Timestamp when operation started (microseconds since epoch)
    pub start_time_us: i64,
}

impl ObsContext {
    /// Create new observability context
    pub fn new(operation: &str, bot_id: Option<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_secs() * 1_000_000 + d.subsec_micros() as u64) as i64)
            .unwrap_or(0);

        ObsContext {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
            operation: operation.to_string(),
            bot_id,
            start_time_us: now,
        }
    }
}

/// Metrics for Gaussian Splatting operations
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GaussianMetrics {
    /// Total observations ingested
    pub observations_ingested: u64,
    /// Successful fusions (observations matched existing splats)
    pub fusions_successful: u64,
    /// Failed fusions (no match found, new splat created)
    pub fusions_failed: u64,
    /// Average fusion latency in microseconds
    pub fusion_latency_us_avg: f32,
    /// Maximum fusion latency observed
    pub fusion_latency_us_max: u64,

    /// Temporal decay operations
    pub decay_operations: u64,
    /// Average decay latency
    pub decay_latency_us_avg: f32,
    /// Splats pruned (confidence fell below threshold)
    pub splats_pruned: u64,

    /// Radius queries executed
    pub queries_radius: u64,
    /// Average query latency
    pub query_latency_us_avg: f32,
    /// Average splats returned per query
    pub query_result_avg: f32,

    /// Uncertainty calculations
    pub uncertainty_calcs: u64,
    /// Average uncertainty calc latency
    pub uncertainty_latency_us_avg: f32,

    /// Change events detected
    pub change_events: u64,
    /// Object moved events
    pub events_object_moved: u64,
    /// Object appeared events
    pub events_object_appeared: u64,
    /// Object disappeared events
    pub events_object_disappeared: u64,
}

impl GaussianMetrics {
    /// Record observation ingestion
    pub fn record_observation(&mut self) {
        self.observations_ingested += 1;
    }

    /// Record successful fusion
    pub fn record_fusion(&mut self, latency_us: u64) {
        self.fusions_successful += 1;
        self.fusion_latency_us_avg =
            (self.fusion_latency_us_avg * (self.fusions_successful - 1) as f32 + latency_us as f32)
            / self.fusions_successful as f32;
        if latency_us > self.fusion_latency_us_max {
            self.fusion_latency_us_max = latency_us;
        }
    }

    /// Record new splat creation
    pub fn record_new_splat(&mut self) {
        self.fusions_failed += 1;
    }

    /// Record decay operation
    pub fn record_decay(&mut self, latency_us: u64, pruned: u32) {
        self.decay_operations += 1;
        self.decay_latency_us_avg =
            (self.decay_latency_us_avg * (self.decay_operations - 1) as f32 + latency_us as f32)
            / self.decay_operations as f32;
        self.splats_pruned += pruned as u64;
    }

    /// Record query
    pub fn record_query(&mut self, latency_us: u64, result_count: usize) {
        self.queries_radius += 1;
        self.query_latency_us_avg =
            (self.query_latency_us_avg * (self.queries_radius - 1) as f32 + latency_us as f32)
            / self.queries_radius as f32;
        self.query_result_avg =
            (self.query_result_avg * (self.queries_radius - 1) as f32 + result_count as f32)
            / self.queries_radius as f32;
    }

    /// Record change event
    pub fn record_change_event(&mut self, event_type: &str) {
        self.change_events += 1;
        match event_type {
            "object_moved" => self.events_object_moved += 1,
            "object_appeared" => self.events_object_appeared += 1,
            "object_disappeared" => self.events_object_disappeared += 1,
            _ => {}
        }
    }
}

/// Observability event (for structured logging)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObsEvent {
    pub timestamp_us: i64,
    pub trace_id: String,
    pub span_id: String,
    pub level: String,  // "trace", "debug", "info", "warn", "error"
    pub message: String,
    pub operation: String,
    pub bot_id: Option<String>,
    pub details: std::collections::HashMap<String, String>,
}

impl ObsEvent {
    /// Create new observability event
    pub fn new(
        ctx: &ObsContext,
        level: &str,
        message: &str,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_secs() * 1_000_000 + d.subsec_micros() as u64) as i64)
            .unwrap_or(0);

        ObsEvent {
            timestamp_us: now,
            trace_id: ctx.trace_id.clone(),
            span_id: ctx.span_id.clone(),
            level: level.to_string(),
            message: message.to_string(),
            operation: ctx.operation.clone(),
            bot_id: ctx.bot_id.clone(),
            details: std::collections::HashMap::new(),
        }
    }
}

/// Observability tracer for Gaussian Splatting
pub struct GaussianSplattingTracer {
    metrics: Arc<parking_lot::Mutex<GaussianMetrics>>,
    events: Arc<parking_lot::Mutex<Vec<ObsEvent>>>,
    max_events: usize,
}

impl GaussianSplattingTracer {
    /// Create new tracer
    pub fn new(max_events: usize) -> Self {
        GaussianSplattingTracer {
            metrics: Arc::new(parking_lot::Mutex::new(GaussianMetrics::default())),
            events: Arc::new(parking_lot::Mutex::new(Vec::new())),
            max_events,
        }
    }

    /// Record a fusion operation
    pub fn record_fusion(&self, ctx: &ObsContext, success: bool, latency_us: u64) {
        let mut metrics = self.metrics.lock();
        if success {
            metrics.record_fusion(latency_us);
        } else {
            metrics.record_new_splat();
        }

        let mut event = ObsEvent::new(ctx, "debug",
            if success { "Fusion successful" } else { "New splat created" });
        event.details.insert("latency_us".to_string(), latency_us.to_string());
        event.details.insert("success".to_string(), success.to_string());
        self.record_event(event);
    }

    /// Record a decay operation
    pub fn record_decay(&self, ctx: &ObsContext, latency_us: u64, pruned: u32) {
        let mut metrics = self.metrics.lock();
        metrics.record_decay(latency_us, pruned);

        let mut event = ObsEvent::new(ctx, "info", "Temporal decay applied");
        event.details.insert("latency_us".to_string(), latency_us.to_string());
        event.details.insert("splats_pruned".to_string(), pruned.to_string());
        self.record_event(event);
    }

    /// Record a query operation
    pub fn record_query(&self, ctx: &ObsContext, latency_us: u64, result_count: usize) {
        let mut metrics = self.metrics.lock();
        metrics.record_query(latency_us, result_count);

        let mut event = ObsEvent::new(ctx, "debug", "Radius query completed");
        event.details.insert("latency_us".to_string(), latency_us.to_string());
        event.details.insert("results".to_string(), result_count.to_string());
        self.record_event(event);
    }

    /// Record a change event
    pub fn record_change_event(&self, ctx: &ObsContext, event_type: &str, details: &str) {
        let mut metrics = self.metrics.lock();
        metrics.record_change_event(event_type);

        let mut event = ObsEvent::new(ctx, "info", &format!("Change detected: {}", event_type));
        event.details.insert("event_type".to_string(), event_type.to_string());
        event.details.insert("details".to_string(), details.to_string());
        self.record_event(event);
    }

    /// Record generic event
    pub fn record_event(&self, event: ObsEvent) {
        let mut events = self.events.lock();
        events.push(event);

        // Keep only recent events
        if events.len() > self.max_events {
            events.remove(0);
        }
    }

    /// Get current metrics
    pub fn metrics(&self) -> GaussianMetrics {
        self.metrics.lock().clone()
    }

    /// Get recent events
    pub fn events(&self, limit: usize) -> Vec<ObsEvent> {
        let events = self.events.lock();
        events.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear all events
    pub fn clear_events(&self) {
        self.events.lock().clear();
    }

    /// Export metrics as OpenMetrics format
    pub fn export_metrics(&self) -> String {
        let metrics = self.metrics.lock();
        format!(
            "# HELP gaussian_observations_ingested Total observations ingested\n\
             # TYPE gaussian_observations_ingested counter\n\
             gaussian_observations_ingested {}\n\
             # HELP gaussian_fusions_successful Successful fusions\n\
             # TYPE gaussian_fusions_successful counter\n\
             gaussian_fusions_successful {}\n\
             # HELP gaussian_fusion_latency_us Average fusion latency\n\
             # TYPE gaussian_fusion_latency_us gauge\n\
             gaussian_fusion_latency_us {}\n\
             # HELP gaussian_queries_radius Radius queries\n\
             # TYPE gaussian_queries_radius counter\n\
             gaussian_queries_radius {}\n\
             # HELP gaussian_query_latency_us Average query latency\n\
             # TYPE gaussian_query_latency_us gauge\n\
             gaussian_query_latency_us {}\n\
             # HELP gaussian_change_events Change events detected\n\
             # TYPE gaussian_change_events counter\n\
             gaussian_change_events {}\n",
            metrics.observations_ingested,
            metrics.fusions_successful,
            metrics.fusion_latency_us_avg,
            metrics.queries_radius,
            metrics.query_latency_us_avg,
            metrics.change_events,
        )
    }
}

impl Default for GaussianSplattingTracer {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obs_context_creation() {
        let ctx = ObsContext::new("fusion", Some("bot_01".to_string()));
        assert_eq!(ctx.operation, "fusion");
        assert_eq!(ctx.bot_id, Some("bot_01".to_string()));
        assert!(!ctx.trace_id.is_empty());
        assert!(!ctx.span_id.is_empty());
    }

    #[test]
    fn test_metrics_recording() {
        let mut metrics = GaussianMetrics::default();

        metrics.record_observation();
        assert_eq!(metrics.observations_ingested, 1);

        metrics.record_fusion(100);
        assert_eq!(metrics.fusions_successful, 1);
        assert!(metrics.fusion_latency_us_avg > 0.0);

        metrics.record_fusion(200);
        assert_eq!(metrics.fusions_successful, 2);
        assert_eq!(metrics.fusion_latency_us_avg, 150.0);
    }

    #[test]
    fn test_tracer_recording() {
        let tracer = GaussianSplattingTracer::new(100);
        let ctx = ObsContext::new("test", None);

        tracer.record_fusion(&ctx, true, 50);
        tracer.record_query(&ctx, 30, 10);
        tracer.record_change_event(&ctx, "object_moved", "pallet moved 0.5m");

        let metrics = tracer.metrics();
        assert_eq!(metrics.fusions_successful, 1);
        assert_eq!(metrics.queries_radius, 1);
        assert_eq!(metrics.change_events, 1);

        let events = tracer.events(10);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_metrics_export() {
        let tracer = GaussianSplattingTracer::new(100);
        let ctx = ObsContext::new("fusion", None);

        tracer.record_fusion(&ctx, true, 75);
        tracer.record_query(&ctx, 45, 5);

        let exported = tracer.export_metrics();
        assert!(exported.contains("gaussian_observations_ingested"));
        assert!(exported.contains("gaussian_fusions_successful 1"));
        assert!(exported.contains("gaussian_queries_radius 1"));
    }

    #[test]
    fn test_event_ring_buffer() {
        let tracer = GaussianSplattingTracer::new(5);
        let ctx = ObsContext::new("test", None);

        // Record more events than max
        for i in 0..10 {
            let mut event = ObsEvent::new(&ctx, "info", &format!("Event {}", i));
            event.details.insert("index".to_string(), i.to_string());
            tracer.record_event(event);
        }

        let events = tracer.events(100);
        assert_eq!(events.len(), 5);  // Should be capped at max_events

        // Most recent events should be preserved
        assert_eq!(events[0].details["index"], "9");
        assert_eq!(events[4].details["index"], "5");
    }
}
