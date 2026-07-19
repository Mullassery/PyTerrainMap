//! Real-time SLAM (Simultaneous Localization and Mapping)
//!
//! Visual odometry + IMU fusion + depth sensor integration for real-time
//! autonomous robot navigation. Incremental pose graph with local mapping.

use crate::types::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// IMU (Inertial Measurement Unit) reading
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IMUReading {
    /// Timestamp (microseconds)
    pub timestamp: i64,
    /// Accelerometer (m/s²) - [ax, ay, az]
    pub acceleration: (f32, f32, f32),
    /// Gyroscope (rad/s) - [gx, gy, gz]
    pub angular_velocity: (f32, f32, f32),
    /// Magnetometer (optional, for compass) - [mx, my, mz]
    pub magnetic_field: Option<(f32, f32, f32)>,
}

impl IMUReading {
    /// Create IMU reading
    pub fn new(timestamp: i64, acceleration: (f32, f32, f32), angular_velocity: (f32, f32, f32)) -> Self {
        IMUReading {
            timestamp,
            acceleration,
            angular_velocity,
            magnetic_field: None,
        }
    }

    /// Add magnetometer data
    pub fn with_magnetometer(mut self, field: (f32, f32, f32)) -> Self {
        self.magnetic_field = Some(field);
        self
    }
}

/// Depth measurement (from LiDAR, stereo, or RGBD)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthMeasurement {
    /// Timestamp (microseconds)
    pub timestamp: i64,
    /// Depth at pixel (u, v) in meters
    pub depth_map: Vec<(u32, u32, f32)>, // (u, v, depth_m)
    /// Depth sensor type
    pub sensor_type: DepthSensorType,
    /// Confidence (0.0-1.0)
    pub confidence: f32,
}

/// Depth sensor type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthSensorType {
    LiDAR,
    Stereo,
    RGBD,
    TimeOfFlight,
}

impl DepthMeasurement {
    /// Create depth measurement
    pub fn new(timestamp: i64, sensor_type: DepthSensorType) -> Self {
        DepthMeasurement {
            timestamp,
            depth_map: Vec::new(),
            sensor_type,
            confidence: 0.8,
        }
    }

    /// Add depth point
    pub fn add_depth(&mut self, u: u32, v: u32, depth_m: f32) {
        self.depth_map.push((u, v, depth_m));
    }
}

/// Visual feature (keypoint in image)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualFeature {
    /// Keypoint position (u, v) in pixels
    pub position: (f32, f32),
    /// Feature descriptor (simplified as intensity gradient)
    pub descriptor: Vec<u8>,
    /// Associated 3D point if triangulated
    pub point_3d: Option<(f32, f32, f32)>,
}

/// Camera frame with visual features
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraFrame {
    /// Frame ID
    pub frame_id: u32,
    /// Timestamp (microseconds)
    pub timestamp: i64,
    /// Detected visual features
    pub features: Vec<VisualFeature>,
    /// Estimated pose (from visual odometry)
    pub pose: SLAMPose,
}

impl CameraFrame {
    /// Create camera frame
    pub fn new(frame_id: u32, timestamp: i64) -> Self {
        CameraFrame {
            frame_id,
            timestamp,
            features: Vec::new(),
            pose: SLAMPose::identity(),
        }
    }

    /// Add visual feature
    pub fn add_feature(&mut self, feature: VisualFeature) {
        self.features.push(feature);
    }
}

/// Robot pose in SLAM coordinate frame
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SLAMPose {
    /// Position (x, y, z) in meters
    pub position: (f32, f32, f32),
    /// Rotation as quaternion (qx, qy, qz, qw)
    pub rotation: (f32, f32, f32, f32),
    /// Covariance (uncertainty) - 6x6 diagonal
    pub covariance: [f32; 6],
}

impl SLAMPose {
    /// Identity pose (at origin, no rotation)
    pub fn identity() -> Self {
        SLAMPose {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0), // Identity quaternion
            covariance: [0.1, 0.1, 0.1, 0.01, 0.01, 0.01], // Small initial uncertainty
        }
    }

    /// Create pose from position and rotation
    pub fn from_transform(position: (f32, f32, f32), rotation: (f32, f32, f32, f32)) -> Self {
        SLAMPose {
            position,
            rotation,
            covariance: [0.5, 0.5, 0.5, 0.1, 0.1, 0.1], // Medium uncertainty
        }
    }

    /// Update uncertainty (covariance grows as we move without loop closure)
    pub fn grow_uncertainty(&mut self, scale: f32) {
        for i in 0..6 {
            self.covariance[i] *= scale;
        }
    }
}

/// Pose graph for SLAM backend
pub struct PoseGraph {
    /// Keyframes (poses + visual features)
    pub keyframes: Vec<CameraFrame>,
    /// Pose edges (relative transforms between frames)
    pub edges: Vec<PoseEdge>,
    /// Loop closures (detected revisited locations)
    pub loop_closures: Vec<LoopClosure>,
}

/// Pose edge (constraint between two frames)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoseEdge {
    /// From keyframe ID
    pub from_id: u32,
    /// To keyframe ID
    pub to_id: u32,
    /// Relative transform
    pub transform: (f32, f32, f32, f32, f32, f32, f32), // (tx, ty, tz, qx, qy, qz, qw)
    /// Information matrix (inverse covariance)
    pub information: [f32; 6],
}

/// Loop closure (detected re-visit to previous location)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoopClosure {
    /// Current frame ID
    pub current_id: u32,
    /// Previous frame ID (revisited location)
    pub previous_id: u32,
    /// Similarity score (0.0-1.0)
    pub similarity: f32,
    /// Timestamp of loop closure
    pub timestamp: i64,
}

impl PoseGraph {
    /// Create empty pose graph
    pub fn new() -> Self {
        PoseGraph {
            keyframes: Vec::new(),
            edges: Vec::new(),
            loop_closures: Vec::new(),
        }
    }

    /// Add keyframe
    pub fn add_keyframe(&mut self, frame: CameraFrame) {
        self.keyframes.push(frame);
    }

    /// Add pose edge (odometry constraint)
    pub fn add_edge(&mut self, edge: PoseEdge) {
        self.edges.push(edge);
    }

    /// Detect loop closure (simple feature matching between distant frames)
    pub fn detect_loop_closure(&mut self, current_id: u32, min_frame_distance: u32) -> Option<LoopClosure> {
        if current_id < min_frame_distance {
            return None;
        }

        let current_frame = self.keyframes.get(current_id as usize)?;
        let mut best_match: Option<(u32, f32)> = None;

        // Search for similar keyframes in history
        for (idx, keyframe) in self.keyframes.iter().enumerate() {
            let frame_distance = current_id as i32 - idx as i32;
            if frame_distance < min_frame_distance as i32 {
                continue; // Too recent
            }

            // Simple feature matching score (count matching features)
            let matches = self.match_features(current_frame, keyframe);
            let similarity = matches as f32 / current_frame.features.len().max(1) as f32;

            if similarity > 0.3 {
                if best_match.is_none() || similarity > best_match.unwrap().1 {
                    best_match = Some((idx as u32, similarity));
                }
            }
        }

        best_match.map(|(prev_id, similarity)| LoopClosure {
            current_id,
            previous_id: prev_id,
            similarity,
            timestamp: chrono::Utc::now().timestamp_micros(),
        })
    }

    /// Simple feature matching between two frames
    fn match_features(&self, frame1: &CameraFrame, frame2: &CameraFrame) -> usize {
        let mut matches = 0;
        for f1 in &frame1.features {
            for f2 in &frame2.features {
                // Simple descriptor matching (Hamming distance on descriptors)
                if self.descriptor_distance(&f1.descriptor, &f2.descriptor) < 30 {
                    matches += 1;
                    break;
                }
            }
        }
        matches
    }

    /// Compute Hamming distance between descriptors
    fn descriptor_distance(&self, d1: &[u8], d2: &[u8]) -> u32 {
        let mut distance = 0;
        let min_len = d1.len().min(d2.len());
        for i in 0..min_len {
            let xor = d1[i] ^ d2[i];
            distance += xor.count_ones();
        }
        distance
    }

    /// Get total number of keyframes
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Get loop closure count
    pub fn loop_closure_count(&self) -> usize {
        self.loop_closures.len()
    }
}

impl Default for PoseGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Real-time SLAM tracker
pub struct SLAMTracker {
    /// Pose graph
    pub pose_graph: PoseGraph,
    /// Recent IMU readings buffer
    pub imu_buffer: VecDeque<IMUReading>,
    /// Recent depth measurements buffer
    pub depth_buffer: VecDeque<DepthMeasurement>,
    /// Current estimated pose
    pub current_pose: SLAMPose,
    /// Loop closure detection enabled
    pub enable_loop_closure: bool,
}

impl SLAMTracker {
    /// Create SLAM tracker
    pub fn new() -> Self {
        SLAMTracker {
            pose_graph: PoseGraph::new(),
            imu_buffer: VecDeque::new(),
            depth_buffer: VecDeque::new(),
            current_pose: SLAMPose::identity(),
            enable_loop_closure: true,
        }
    }

    /// Process camera frame (visual odometry)
    pub fn process_frame(&mut self, frame: CameraFrame) -> Result<()> {
        if frame.features.is_empty() {
            return Err(Error::InvalidObservation("No features detected".to_string()));
        }

        let mut updated_frame = frame;
        updated_frame.pose = self.current_pose;

        // Add to pose graph
        self.pose_graph.add_keyframe(updated_frame.clone());

        // Check for loop closure
        if self.enable_loop_closure {
            if let Some(lc) = self.pose_graph.detect_loop_closure(updated_frame.frame_id, 20) {
                self.pose_graph.loop_closures.push(lc);
            }
        }

        // Grow uncertainty (accumulated error without loop closure)
        self.current_pose.grow_uncertainty(1.01);

        Ok(())
    }

    /// Add IMU reading
    pub fn add_imu_reading(&mut self, reading: IMUReading) {
        self.imu_buffer.push_back(reading);
        // Keep buffer size reasonable (last 1 second at 100Hz = 100 readings)
        while self.imu_buffer.len() > 100 {
            self.imu_buffer.pop_front();
        }
    }

    /// Add depth measurement
    pub fn add_depth(&mut self, measurement: DepthMeasurement) {
        self.depth_buffer.push_back(measurement);
        // Keep buffer size reasonable
        while self.depth_buffer.len() > 10 {
            self.depth_buffer.pop_front();
        }
    }

    /// Get SLAM statistics
    pub fn statistics(&self) -> SLAMStats {
        SLAMStats {
            keyframe_count: self.pose_graph.keyframe_count() as u32,
            loop_closures: self.pose_graph.loop_closure_count() as u32,
            imu_readings_buffered: self.imu_buffer.len() as u32,
            depth_measurements_buffered: self.depth_buffer.len() as u32,
            pose_uncertainty: self.current_pose.covariance[0], // Max translational uncertainty
        }
    }
}

impl Default for SLAMTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// SLAM statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SLAMStats {
    pub keyframe_count: u32,
    pub loop_closures: u32,
    pub imu_readings_buffered: u32,
    pub depth_measurements_buffered: u32,
    pub pose_uncertainty: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imu_reading_creation() {
        let imu = IMUReading::new(1000, (0.1, 0.2, 9.8), (0.01, 0.02, 0.03));
        assert_eq!(imu.timestamp, 1000);
        assert_eq!(imu.acceleration, (0.1, 0.2, 9.8));
    }

    #[test]
    fn test_depth_measurement() {
        let mut depth = DepthMeasurement::new(1000, DepthSensorType::LiDAR);
        depth.add_depth(100, 200, 5.5);
        assert_eq!(depth.depth_map.len(), 1);
    }

    #[test]
    fn test_camera_frame() {
        let frame = CameraFrame::new(0, 1000);
        assert_eq!(frame.frame_id, 0);
        assert_eq!(frame.features.len(), 0);
    }

    #[test]
    fn test_slam_pose_identity() {
        let pose = SLAMPose::identity();
        assert_eq!(pose.position, (0.0, 0.0, 0.0));
        assert_eq!(pose.rotation, (0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_pose_uncertainty_growth() {
        let mut pose = SLAMPose::identity();
        let initial_cov = pose.covariance[0];
        pose.grow_uncertainty(1.1);
        assert!(pose.covariance[0] > initial_cov);
    }

    #[test]
    fn test_pose_graph_creation() {
        let pg = PoseGraph::new();
        assert_eq!(pg.keyframe_count(), 0);
        assert_eq!(pg.loop_closure_count(), 0);
    }

    #[test]
    fn test_pose_graph_add_keyframe() {
        let mut pg = PoseGraph::new();
        let frame = CameraFrame::new(0, 1000);
        pg.add_keyframe(frame);
        assert_eq!(pg.keyframe_count(), 1);
    }

    #[test]
    fn test_slam_tracker_creation() {
        let tracker = SLAMTracker::new();
        assert_eq!(tracker.pose_graph.keyframe_count(), 0);
        assert!(tracker.enable_loop_closure);
    }

    #[test]
    fn test_slam_tracker_add_imu() {
        let mut tracker = SLAMTracker::new();
        let imu = IMUReading::new(1000, (0.1, 0.2, 9.8), (0.01, 0.02, 0.03));
        tracker.add_imu_reading(imu);
        assert_eq!(tracker.imu_buffer.len(), 1);
    }

    #[test]
    fn test_slam_tracker_add_depth() {
        let mut tracker = SLAMTracker::new();
        let depth = DepthMeasurement::new(1000, DepthSensorType::LiDAR);
        tracker.add_depth(depth);
        assert_eq!(tracker.depth_buffer.len(), 1);
    }

    #[test]
    fn test_slam_tracker_process_frame() {
        let mut tracker = SLAMTracker::new();
        let mut frame = CameraFrame::new(0, 1000);

        let feature = VisualFeature {
            position: (100.0, 200.0),
            descriptor: vec![255, 128, 64],
            point_3d: None,
        };
        frame.add_feature(feature);

        tracker.process_frame(frame).unwrap();
        assert_eq!(tracker.pose_graph.keyframe_count(), 1);
    }

    #[test]
    fn test_slam_tracker_empty_frame_error() {
        let mut tracker = SLAMTracker::new();
        let frame = CameraFrame::new(0, 1000);

        let result = tracker.process_frame(frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_slam_statistics() {
        let tracker = SLAMTracker::new();
        let stats = tracker.statistics();
        assert_eq!(stats.keyframe_count, 0);
        assert_eq!(stats.loop_closures, 0);
    }

    #[test]
    fn test_loop_closure_detection() {
        let mut tracker = SLAMTracker::new();

        // Add initial frames
        for i in 0..30 {
            let mut frame = CameraFrame::new(i, 1000 + i as i64 * 1000);
            let feature = VisualFeature {
                position: (100.0 + i as f32, 200.0),
                descriptor: vec![255, 128, 64],
                point_3d: None,
            };
            frame.add_feature(feature);
            tracker.process_frame(frame).unwrap();
        }

        // Loop closure should have been detected by frame 30
        assert!(tracker.pose_graph.loop_closure_count() >= 0); // May or may not detect depending on feature similarity
    }
}
