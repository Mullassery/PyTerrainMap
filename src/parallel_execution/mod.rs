//! Parallel execution runtime for space-time intelligence
//!
//! Provides GPU-aware scheduling, distributed observation processing,
//! and automatic workload distribution across available compute resources.
//!
//! Core principle: Reality is naturally parallel across space, time, sensors,
//! and agents. Parallelism is the default execution model.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// GPU device identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GPUDeviceId(pub u32);

/// Spatial region for partitioning observations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RegionId {
    /// H3 cell ID representing this region
    pub h3_cell: u64,
    /// Resolution level (0-15)
    pub resolution: u8,
}

/// Temporal window for partitioning observations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Start time (microseconds since epoch)
    pub start_us: i64,
    /// End time (microseconds since epoch)
    pub end_us: i64,
}

impl TimeWindow {
    pub fn new(start_us: i64, end_us: i64) -> Self {
        TimeWindow { start_us, end_us }
    }

    pub fn duration_us(&self) -> i64 {
        self.end_us - self.start_us
    }

    /// Check if observation falls within this window
    pub fn contains(&self, time_us: i64) -> bool {
        time_us >= self.start_us && time_us < self.end_us
    }
}

/// Agent identifier for multi-agent parallelism
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AgentId(pub String);

/// Work packet scheduled for GPU execution
#[derive(Clone, Debug)]
pub struct WorkPacket {
    pub id: Uuid,
    /// Target GPU device
    pub target_gpu: Option<GPUDeviceId>,
    /// Spatial region (if applicable)
    pub region: Option<RegionId>,
    /// Temporal window (if applicable)
    pub time_window: Option<TimeWindow>,
    /// Agent executing this work (if applicable)
    pub agent: Option<AgentId>,
    /// Work type (query, inference, fusion, etc.)
    pub work_type: WorkType,
    /// Priority (higher = execute sooner)
    pub priority: WorkPriority,
    /// When this work was created
    pub created_at_us: i64,
}

/// Type of work to execute
#[derive(Clone, Debug)]
pub enum WorkType {
    SpatialQuery,
    TemporalQuery,
    SensorFusion,
    AnomalyDetection,
    Inference { model_name: String },
    Reconstruction3D,
    GraphTraversal,
}

/// Work priority level
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkPriority {
    Background = 0,
    Batch = 1,
    Normal = 2,
    HighPriority = 3,
    LateArrivalCorrection = 4,  // Highest: temporal consistency
    RealTime = 5,
}

/// Watermark tracking for streams
#[derive(Clone, Debug)]
pub struct Watermark {
    /// Maximum event time seen in this stream
    pub max_event_time_us: i64,
    /// When watermark was last updated
    pub updated_at_us: i64,
}

/// GPU resource availability
#[derive(Clone, Debug)]
pub struct GPUResources {
    pub device_id: GPUDeviceId,
    /// Compute capability level (higher = more powerful)
    pub compute_capacity: u32,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Current utilization (0.0-1.0)
    pub utilization: f32,
    /// Whether device is healthy
    pub is_healthy: bool,
}

/// Space-time scheduler: Manages parallel execution across GPUs
pub struct SpaceTimeScheduler {
    /// Observation work queues by (region, time_window, agent)
    work_queues: Arc<RwLock<BTreeMap<(Option<RegionId>, Option<TimeWindow>, Option<AgentId>), VecDeque<WorkPacket>>>>,

    /// GPU resource pool
    gpu_resources: Arc<RwLock<Vec<GPUResources>>>,

    /// Watermarks per stream
    watermarks: Arc<RwLock<HashMap<String, Watermark>>>,

    /// Late-arrival correction tasks (high priority)
    late_arrival_queue: Arc<RwLock<Vec<LateArrivalTask>>>,

    /// Historical corrections to apply
    corrections_queue: Arc<RwLock<Vec<TemporalCorrection>>>,

    /// Performance metrics
    metrics: Arc<RwLock<SchedulerMetrics>>,
}

/// Task representing late-arriving observation
#[derive(Clone, Debug)]
pub struct LateArrivalTask {
    pub obs_id: Uuid,
    pub event_time_us: i64,
    pub ingestion_time_us: i64,
    pub affected_regions: Vec<RegionId>,
    pub affected_time_windows: Vec<TimeWindow>,
    pub priority: WorkPriority,
}

/// Temporal correction to apply
#[derive(Clone, Debug)]
pub struct TemporalCorrection {
    pub original_obs_id: Uuid,
    pub corrected_obs_id: Uuid,
    pub time_window: TimeWindow,
    pub region: RegionId,
}

/// Scheduler metrics and statistics
#[derive(Clone, Debug, Default)]
pub struct SchedulerMetrics {
    pub total_work_packets: u64,
    pub completed_work_packets: u64,
    pub failed_work_packets: u64,
    pub avg_queuing_time_us: u64,
    pub avg_execution_time_us: u64,
    pub late_arrivals_handled: u64,
    pub temporal_corrections_applied: u64,
}

impl Default for SpaceTimeScheduler {
    fn default() -> Self {
        SpaceTimeScheduler::new()
    }
}

impl SpaceTimeScheduler {
    /// Create new scheduler with auto-detected GPU resources
    pub fn new() -> Self {
        SpaceTimeScheduler {
            work_queues: Arc::new(RwLock::new(BTreeMap::new())),
            gpu_resources: Arc::new(RwLock::new(Vec::new())),
            watermarks: Arc::new(RwLock::new(HashMap::new())),
            late_arrival_queue: Arc::new(RwLock::new(Vec::new())),
            corrections_queue: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::default())),
        }
    }

    /// Register available GPU
    pub fn register_gpu(&self, gpu: GPUResources) {
        let mut resources = self.gpu_resources.write();
        resources.push(gpu);
    }

    /// Get available GPUs with sufficient memory
    pub fn available_gpus(&self, required_memory: u64) -> Vec<GPUResources> {
        self.gpu_resources
            .read()
            .iter()
            .filter(|gpu| gpu.is_healthy && gpu.available_memory >= required_memory)
            .cloned()
            .collect()
    }

    /// Submit work packet for scheduling
    pub fn submit_work(&self, packet: WorkPacket) {
        let key = (packet.region, packet.time_window, packet.agent.clone());
        let mut queues = self.work_queues.write();
        queues.entry(key).or_insert_with(VecDeque::new).push_back(packet);
    }

    /// Get next batch of work for available GPU
    pub fn schedule_next_batch(&self, gpu: GPUDeviceId) -> Vec<WorkPacket> {
        let mut queues = self.work_queues.write();

        // Priority: late arrivals > real-time > batch > background
        let late_arrivals = self.late_arrival_queue.write();
        if !late_arrivals.is_empty() {
            return late_arrivals
                .iter()
                .take(10)  // Limit batch size
                .map(|_task| WorkPacket {
                    id: Uuid::new_v4(),
                    target_gpu: Some(gpu),
                    region: None,
                    time_window: None,
                    agent: None,
                    work_type: WorkType::TemporalQuery,
                    priority: WorkPriority::LateArrivalCorrection,
                    created_at_us: 0,
                })
                .collect();
        }

        // Fall back to regular work queues
        let mut work = Vec::new();
        for (_, queue) in queues.iter_mut() {
            if let Some(packet) = queue.pop_front() {
                work.push(packet);
                if work.len() >= 32 {  // Max batch size
                    break;
                }
            }
        }

        work
    }

    /// Advance watermark for stream
    pub fn advance_watermark(&self, stream_id: String, max_event_time_us: i64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        let mut watermarks = self.watermarks.write();
        watermarks.insert(
            stream_id,
            Watermark {
                max_event_time_us,
                updated_at_us: now,
            },
        );

        // Observations <= max_event_time_us are complete for this stream
        // Can finalize windows
    }

    /// Handle late-arriving observation
    pub fn handle_late_arrival(&self, task: LateArrivalTask) {
        self.late_arrival_queue.write().push(task);

        let mut metrics = self.metrics.write();
        metrics.late_arrivals_handled += 1;
    }

    /// Get scheduler metrics
    pub fn metrics(&self) -> SchedulerMetrics {
        self.metrics.read().clone()
    }

    /// Mark work packet as completed
    pub fn mark_completed(&self, _packet_id: Uuid) {
        let mut metrics = self.metrics.write();
        metrics.completed_work_packets += 1;
    }

    /// Mark work packet as failed
    pub fn mark_failed(&self, _packet_id: Uuid) {
        let mut metrics = self.metrics.write();
        metrics.failed_work_packets += 1;
    }
}

/// Execution metadata for observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// GPU device where observation was processed
    pub processing_gpu: Option<GPUDeviceId>,

    /// Spatial partition ID
    pub spatial_partition: Option<RegionId>,

    /// Temporal partition ID
    pub temporal_partition: Option<TimeWindow>,

    /// Agent that created/processed this
    pub agent_id: Option<AgentId>,

    /// Processing latency in microseconds
    pub processing_latency_us: u32,

    /// How many times has this been reprocessed (for late arrivals)
    pub reprocess_count: u32,

    /// When was this observation last processed
    pub last_processed_us: i64,
}

impl Default for ExecutionMetadata {
    fn default() -> Self {
        ExecutionMetadata {
            processing_gpu: None,
            spatial_partition: None,
            temporal_partition: None,
            agent_id: None,
            processing_latency_us: 0,
            reprocess_count: 0,
            last_processed_us: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_window_contains() {
        let window = TimeWindow::new(1000, 2000);
        assert!(window.contains(1500));
        assert!(!window.contains(500));
        assert!(!window.contains(2000));  // Exclusive end
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = SpaceTimeScheduler::new();
        let gpu = GPUResources {
            device_id: GPUDeviceId(0),
            compute_capacity: 100,
            total_memory: 24 * 1024 * 1024 * 1024,  // 24GB
            available_memory: 20 * 1024 * 1024 * 1024,  // 20GB
            utilization: 0.2,
            is_healthy: true,
        };
        scheduler.register_gpu(gpu.clone());

        let available = scheduler.available_gpus(1024);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].device_id, GPUDeviceId(0));
    }

    #[test]
    fn test_watermark_tracking() {
        let scheduler = SpaceTimeScheduler::new();
        scheduler.advance_watermark("stream-0".to_string(), 5000);

        let watermarks = scheduler.watermarks.read();
        assert_eq!(watermarks.get("stream-0").unwrap().max_event_time_us, 5000);
    }

    #[test]
    fn test_late_arrival_queue() {
        let scheduler = SpaceTimeScheduler::new();
        let task = LateArrivalTask {
            obs_id: Uuid::new_v4(),
            event_time_us: 1000,
            ingestion_time_us: 2000,
            affected_regions: vec![],
            affected_time_windows: vec![],
            priority: WorkPriority::LateArrivalCorrection,
        };

        scheduler.handle_late_arrival(task);
        assert_eq!(scheduler.metrics().late_arrivals_handled, 1);
    }

    #[test]
    fn test_work_submission() {
        let scheduler = SpaceTimeScheduler::new();
        let packet = WorkPacket {
            id: Uuid::new_v4(),
            target_gpu: None,
            region: None,
            time_window: None,
            agent: None,
            work_type: WorkType::SpatialQuery,
            priority: WorkPriority::Normal,
            created_at_us: 0,
        };

        scheduler.submit_work(packet);
        let metrics = scheduler.metrics();
        assert_eq!(metrics.completed_work_packets, 0);  // Not completed yet
    }

    #[test]
    fn test_gpu_memory_filter() {
        let scheduler = SpaceTimeScheduler::new();
        let gpu1 = GPUResources {
            device_id: GPUDeviceId(0),
            compute_capacity: 100,
            total_memory: 24 * 1024 * 1024 * 1024,
            available_memory: 20 * 1024 * 1024 * 1024,  // 20GB
            utilization: 0.2,
            is_healthy: true,
        };
        let gpu2 = GPUResources {
            device_id: GPUDeviceId(1),
            compute_capacity: 50,
            total_memory: 8 * 1024 * 1024 * 1024,
            available_memory: 1024 * 1024,  // 1MB
            utilization: 0.9,
            is_healthy: true,
        };

        scheduler.register_gpu(gpu1);
        scheduler.register_gpu(gpu2);

        // Request 10GB
        let available = scheduler.available_gpus(10 * 1024 * 1024 * 1024);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].device_id, GPUDeviceId(0));
    }
}
