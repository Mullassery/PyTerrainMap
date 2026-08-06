//! Performance Profiling & Metrics
//!
//! Phase 5.5: Comprehensive performance monitoring, metrics collection,
//! and optimization recommendations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Performance metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_us: u64,
    pub memory_used_bytes: usize,
    pub throughput_ops_per_sec: f32,
    pub timestamp_us: i64,
}

impl PerformanceMetrics {
    /// Create new metric
    pub fn new(operation: String, duration_us: u64, memory_bytes: usize, ops: f32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        PerformanceMetrics {
            operation,
            duration_us,
            memory_used_bytes: memory_bytes,
            throughput_ops_per_sec: ops,
            timestamp_us: now,
        }
    }
}

/// Operation timer
pub struct OperationTimer {
    start_time: Instant,
    operation_name: String,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start(operation_name: &str) -> Self {
        OperationTimer {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
        }
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Finish timing and get metrics
    pub fn finish(self, memory_bytes: usize, ops: f32) -> PerformanceMetrics {
        let duration_us = self.elapsed_us();
        PerformanceMetrics::new(
            self.operation_name,
            duration_us,
            memory_bytes,
            ops,
        )
    }
}

/// Performance statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub operation: String,
    pub call_count: u64,
    pub total_duration_us: u64,
    pub avg_duration_us: f32,
    pub min_duration_us: u64,
    pub max_duration_us: u64,
    pub p95_duration_us: u64, // 95th percentile
    pub p99_duration_us: u64, // 99th percentile
    pub total_throughput: f32,
}

impl PerformanceStats {
    /// Create statistics from metrics
    pub fn from_metrics(operation: &str, metrics: &[PerformanceMetrics]) -> Self {
        if metrics.is_empty() {
            return PerformanceStats {
                operation: operation.to_string(),
                call_count: 0,
                total_duration_us: 0,
                avg_duration_us: 0.0,
                min_duration_us: 0,
                max_duration_us: 0,
                p95_duration_us: 0,
                p99_duration_us: 0,
                total_throughput: 0.0,
            };
        }

        let call_count = metrics.len() as u64;
        let total_duration_us = metrics.iter().map(|m| m.duration_us).sum();
        let avg_duration_us = total_duration_us as f32 / call_count as f32;

        let min_duration_us = metrics.iter().map(|m| m.duration_us).min().unwrap_or(0);
        let max_duration_us = metrics.iter().map(|m| m.duration_us).max().unwrap_or(0);

        let mut durations: Vec<u64> = metrics.iter().map(|m| m.duration_us).collect();
        durations.sort_unstable();

        let p95_idx = (durations.len() * 95 / 100).max(1) - 1;
        let p99_idx = (durations.len() * 99 / 100).max(1) - 1;
        let p95_duration_us = durations[p95_idx];
        let p99_duration_us = durations[p99_idx];

        let total_throughput = metrics.iter().map(|m| m.throughput_ops_per_sec).sum();

        PerformanceStats {
            operation: operation.to_string(),
            call_count,
            total_duration_us,
            avg_duration_us,
            min_duration_us,
            max_duration_us,
            p95_duration_us,
            p99_duration_us,
            total_throughput,
        }
    }
}

/// Performance profiler
pub struct PerformanceProfiler {
    pub metrics: Vec<PerformanceMetrics>,
    pub stats_cache: HashMap<String, PerformanceStats>,
    pub enabled: bool,
}

impl PerformanceProfiler {
    /// Create new profiler
    pub fn new() -> Self {
        PerformanceProfiler {
            metrics: Vec::new(),
            stats_cache: HashMap::new(),
            enabled: true,
        }
    }

    /// Record metric
    pub fn record(&mut self, metric: PerformanceMetrics) {
        if !self.enabled {
            return;
        }
        self.metrics.push(metric);
        self.stats_cache.clear(); // Invalidate cache
    }

    /// Start timing an operation
    pub fn start_timer(name: &str) -> OperationTimer {
        OperationTimer::start(name)
    }

    /// Get statistics for operation
    pub fn get_stats(&mut self, operation: &str) -> Option<PerformanceStats> {
        // Check cache first
        if let Some(cached) = self.stats_cache.get(operation) {
            return Some(cached.clone());
        }

        // Filter metrics for this operation
        let operation_metrics: Vec<_> = self.metrics
            .iter()
            .filter(|m| m.operation == operation)
            .cloned()
            .collect();

        if operation_metrics.is_empty() {
            return None;
        }

        let stats = PerformanceStats::from_metrics(operation, &operation_metrics);
        self.stats_cache.insert(operation.to_string(), stats.clone());
        Some(stats)
    }

    /// Get all statistics
    pub fn get_all_stats(&mut self) -> Vec<PerformanceStats> {
        let operations: Vec<String> = self.metrics
            .iter()
            .map(|m| m.operation.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        operations
            .iter()
            .filter_map(|op| self.get_stats(op))
            .collect()
    }

    /// Get profiler summary
    pub fn summary(&mut self) -> ProfilerSummary {
        let all_stats = self.get_all_stats();
        let total_metrics = self.metrics.len() as u64;
        let total_duration_us = self.metrics.iter().map(|m| m.duration_us).sum();
        let total_throughput = self.metrics.iter().map(|m| m.throughput_ops_per_sec).sum();

        let slowest_operation = all_stats.iter()
            .max_by(|a, b| a.avg_duration_us.partial_cmp(&b.avg_duration_us).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        let fastest_operation = all_stats.iter()
            .min_by(|a, b| a.avg_duration_us.partial_cmp(&b.avg_duration_us).unwrap_or(std::cmp::Ordering::Equal))
            .cloned();

        ProfilerSummary {
            total_metrics,
            total_duration_us,
            total_throughput,
            slowest_operation,
            fastest_operation,
            operation_count: all_stats.len() as u32,
        }
    }

    /// Clear metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
        self.stats_cache.clear();
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Profiler summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfilerSummary {
    pub total_metrics: u64,
    pub total_duration_us: u64,
    pub total_throughput: f32,
    pub slowest_operation: Option<PerformanceStats>,
    pub fastest_operation: Option<PerformanceStats>,
    pub operation_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics() {
        let metric = PerformanceMetrics::new("test_op".to_string(), 1000, 5000, 100.0);
        assert_eq!(metric.operation, "test_op");
        assert_eq!(metric.duration_us, 1000);
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start("test");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let elapsed = timer.elapsed_us();

        assert!(elapsed > 1000); // At least 1ms
    }

    #[test]
    fn test_performance_stats() {
        let metrics = vec![
            PerformanceMetrics::new("op".to_string(), 100, 1000, 10.0),
            PerformanceMetrics::new("op".to_string(), 200, 1000, 10.0),
            PerformanceMetrics::new("op".to_string(), 300, 1000, 10.0),
        ];

        let stats = PerformanceStats::from_metrics("op", &metrics);
        assert_eq!(stats.call_count, 3);
        assert_eq!(stats.min_duration_us, 100);
        assert_eq!(stats.max_duration_us, 300);
    }

    #[test]
    fn test_profiler() {
        let mut profiler = PerformanceProfiler::new();
        let metric = PerformanceMetrics::new("test".to_string(), 100, 1000, 10.0);
        profiler.record(metric);

        let stats = profiler.get_stats("test");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().call_count, 1);
    }

    #[test]
    fn test_profiler_summary() {
        let mut profiler = PerformanceProfiler::new();
        let metric = PerformanceMetrics::new("test".to_string(), 100, 1000, 10.0);
        profiler.record(metric);

        let summary = profiler.summary();
        assert_eq!(summary.total_metrics, 1);
        assert_eq!(summary.operation_count, 1);
    }
}
