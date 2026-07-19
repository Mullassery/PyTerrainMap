//! 3D reconstruction from multi-image observations
//!
//! Progressive 3D scene reconstruction using Structure from Motion (SfM),
//! point clouds, and neural 3D representations (NeRFs, Gaussian Splats).

use crate::types::{GeoPoint, Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Camera intrinsic parameters (calibration matrix)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraIntrinsics {
    /// Focal length in pixels (fx, fy)
    pub focal_length: (f32, f32),
    /// Principal point (cx, cy)
    pub principal_point: (f32, f32),
    /// Image resolution (width, height)
    pub resolution: (u32, u32),
    /// Distortion coefficients (optional)
    pub distortion: Option<Vec<f32>>,
}

impl CameraIntrinsics {
    /// Create with default principal point (center of image)
    pub fn new(focal_length: f32, resolution: (u32, u32)) -> Self {
        CameraIntrinsics {
            focal_length: (focal_length, focal_length),
            principal_point: (resolution.0 as f32 / 2.0, resolution.1 as f32 / 2.0),
            resolution,
            distortion: None,
        }
    }

    /// Get intrinsic matrix (3x3)
    pub fn matrix(&self) -> [[f32; 3]; 3] {
        [
            [self.focal_length.0, 0.0, self.principal_point.0],
            [0.0, self.focal_length.1, self.principal_point.1],
            [0.0, 0.0, 1.0],
        ]
    }
}

/// Camera pose in 3D space
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CameraPose {
    /// Position in 3D space (x, y, z)
    pub position: (f32, f32, f32),
    /// Rotation as quaternion (qx, qy, qz, qw)
    pub rotation: (f32, f32, f32, f32),
    /// Confidence in pose estimation (0.0-1.0)
    pub confidence: f32,
}

impl CameraPose {
    /// Create identity pose (at origin, no rotation)
    pub fn identity() -> Self {
        CameraPose {
            position: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0), // Identity quaternion
            confidence: 1.0,
        }
    }

    /// Create pose from position and rotation
    pub fn from_position_rotation(
        position: (f32, f32, f32),
        rotation: (f32, f32, f32, f32),
    ) -> Self {
        CameraPose {
            position,
            rotation,
            confidence: 0.5,
        }
    }
}

/// 3D point in space with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point3D {
    /// Position in 3D space (x, y, z)
    pub position: (f32, f32, f32),
    /// RGB color
    pub color: (u8, u8, u8),
    /// Confidence (how many views confirmed this point)
    pub confidence: f32,
    /// Number of observations (views that see this point)
    pub observation_count: u32,
}

impl Point3D {
    /// Create 3D point
    pub fn new(position: (f32, f32, f32), color: (u8, u8, u8)) -> Self {
        Point3D {
            position,
            color,
            confidence: 0.5,
            observation_count: 1,
        }
    }

    /// Update confidence based on additional observation
    pub fn add_observation(&mut self, color: (u8, u8, u8)) {
        self.observation_count += 1;
        // Update color as running average
        let prev_count = (self.observation_count - 1) as u32;
        let curr_count = self.observation_count as u32;

        self.color = (
            ((self.color.0 as u32 * prev_count + color.0 as u32) / curr_count) as u8,
            ((self.color.1 as u32 * prev_count + color.1 as u32) / curr_count) as u8,
            ((self.color.2 as u32 * prev_count + color.2 as u32) / curr_count) as u8,
        );
        // Confidence increases with more observations
        self.confidence = (self.observation_count as f32 / 10.0).min(1.0);
    }
}

/// Reconstructed image frame with pose and features
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconstructionFrame {
    /// Image ID (from reference images or robot observation)
    pub image_id: String,
    /// Camera intrinsics
    pub intrinsics: CameraIntrinsics,
    /// Estimated camera pose
    pub pose: CameraPose,
    /// Detected feature points in image (u, v) coordinates
    pub features: Vec<(f32, f32)>,
    /// Corresponding 3D points (if matched)
    pub matched_3d_points: Vec<Option<usize>>, // Index into point cloud
    /// Timestamp when this frame was added
    pub timestamp: i64,
}

impl ReconstructionFrame {
    /// Create reconstruction frame
    pub fn new(image_id: &str, intrinsics: CameraIntrinsics) -> Self {
        ReconstructionFrame {
            image_id: image_id.to_string(),
            intrinsics,
            pose: CameraPose::identity(),
            features: Vec::new(),
            matched_3d_points: Vec::new(),
            timestamp: chrono::Utc::now().timestamp_micros(),
        }
    }

    /// Add detected features
    pub fn add_features(&mut self, features: Vec<(f32, f32)>) {
        self.features = features;
        self.matched_3d_points = vec![None; self.features.len()];
    }

    /// Set pose estimate
    pub fn set_pose(&mut self, pose: CameraPose) {
        self.pose = pose;
    }
}

/// Point cloud representation (Structure from Motion output)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointCloud {
    /// 3D points
    pub points: Vec<Point3D>,
    /// Bounding box (min, max)
    pub bounds: Option<((f32, f32, f32), (f32, f32, f32))>,
}

impl PointCloud {
    /// Create empty point cloud
    pub fn new() -> Self {
        PointCloud {
            points: Vec::new(),
            bounds: None,
        }
    }

    /// Add point to cloud
    pub fn add_point(&mut self, point: Point3D) {
        self.points.push(point);
        self.update_bounds();
    }

    /// Update bounding box
    fn update_bounds(&mut self) {
        if self.points.is_empty() {
            self.bounds = None;
            return;
        }

        let mut min = (f32::MAX, f32::MAX, f32::MAX);
        let mut max = (f32::MIN, f32::MIN, f32::MIN);

        for point in &self.points {
            min.0 = min.0.min(point.position.0);
            min.1 = min.1.min(point.position.1);
            min.2 = min.2.min(point.position.2);

            max.0 = max.0.max(point.position.0);
            max.1 = max.1.max(point.position.1);
            max.2 = max.2.max(point.position.2);
        }

        self.bounds = Some((min, max));
    }

    /// Merge with another point cloud
    pub fn merge(&mut self, other: PointCloud) {
        self.points.extend(other.points);
        self.update_bounds();
    }

    /// Filter points by confidence
    pub fn filter_by_confidence(&self, min_confidence: f32) -> PointCloud {
        let points = self
            .points
            .iter()
            .filter(|p| p.confidence >= min_confidence)
            .cloned()
            .collect();

        let mut filtered = PointCloud { points, bounds: None };
        filtered.update_bounds();
        filtered
    }

    /// Filter by observation count
    pub fn filter_by_observations(&self, min_observations: u32) -> PointCloud {
        let points = self
            .points
            .iter()
            .filter(|p| p.observation_count >= min_observations)
            .cloned()
            .collect();

        let mut filtered = PointCloud { points, bounds: None };
        filtered.update_bounds();
        filtered
    }

    /// Get statistics
    pub fn statistics(&self) -> PointCloudStats {
        if self.points.is_empty() {
            return PointCloudStats::default();
        }

        let avg_confidence = self.points.iter().map(|p| p.confidence).sum::<f32>() / self.points.len() as f32;
        let avg_observations = self.points.iter().map(|p| p.observation_count).sum::<u32>() / self.points.len() as u32;

        PointCloudStats {
            point_count: self.points.len() as u32,
            avg_confidence,
            avg_observations,
            bounds: self.bounds,
        }
    }
}

impl Default for PointCloud {
    fn default() -> Self {
        Self::new()
    }
}

/// Point cloud statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointCloudStats {
    pub point_count: u32,
    pub avg_confidence: f32,
    pub avg_observations: u32,
    pub bounds: Option<((f32, f32, f32), (f32, f32, f32))>,
}

impl Default for PointCloudStats {
    fn default() -> Self {
        PointCloudStats {
            point_count: 0,
            avg_confidence: 0.0,
            avg_observations: 0,
            bounds: None,
        }
    }
}

/// 3D scene reconstruction state
pub struct ReconstructionEngine {
    /// Reconstruction frames (images with estimated poses)
    pub frames: Vec<ReconstructionFrame>,
    /// Point cloud (SfM output)
    pub point_cloud: PointCloud,
    /// Registered location (ties to terrain map)
    pub location: Option<GeoPoint>,
    /// Number of successful pose estimates
    pub registered_frame_count: usize,
}

impl ReconstructionEngine {
    /// Create new reconstruction engine
    pub fn new() -> Self {
        ReconstructionEngine {
            frames: Vec::new(),
            point_cloud: PointCloud::new(),
            location: None,
            registered_frame_count: 0,
        }
    }

    /// Add frame to reconstruction
    pub fn add_frame(&mut self, frame: ReconstructionFrame) -> Result<usize> {
        let index = self.frames.len();
        self.frames.push(frame);
        Ok(index)
    }

    /// Register location (tie reconstruction to georeferenced position)
    pub fn register_location(&mut self, location: GeoPoint) -> Result<()> {
        if !location.is_valid() {
            return Err(Error::InvalidLocation);
        }
        self.location = Some(location);
        Ok(())
    }

    /// Estimate pose for new frame (placeholder for SfM algorithm)
    pub fn estimate_pose(&mut self, frame_index: usize) -> Result<()> {
        if frame_index >= self.frames.len() {
            return Err(Error::InvalidObservation("Frame index out of range".to_string()));
        }

        if frame_index == 0 {
            // First frame at origin
            self.frames[frame_index].pose = CameraPose::identity();
        } else {
            // Relative to previous frame (simplified)
            let prev_pose = &self.frames[frame_index - 1].pose;
            self.frames[frame_index].pose = CameraPose::from_position_rotation(
                (prev_pose.position.0 + 0.1, prev_pose.position.1, prev_pose.position.2),
                prev_pose.rotation,
            );
        }

        self.registered_frame_count += 1;
        Ok(())
    }

    /// Add 3D point to cloud
    pub fn add_point(&mut self, point: Point3D) {
        self.point_cloud.add_point(point);
    }

    /// Merge point cloud
    pub fn merge_point_cloud(&mut self, cloud: PointCloud) {
        self.point_cloud.merge(cloud);
    }

    /// Get reconstruction statistics
    pub fn statistics(&self) -> ReconstructionStats {
        ReconstructionStats {
            frame_count: self.frames.len() as u32,
            registered_frames: self.registered_frame_count as u32,
            point_cloud_stats: self.point_cloud.statistics(),
            is_registered: self.location.is_some(),
        }
    }
}

impl Default for ReconstructionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Reconstruction statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconstructionStats {
    pub frame_count: u32,
    pub registered_frames: u32,
    pub point_cloud_stats: PointCloudStats,
    pub is_registered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_intrinsics() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        assert_eq!(intrinsics.focal_length, (500.0, 500.0));
        assert_eq!(intrinsics.resolution, (1920, 1080));
        assert_eq!(intrinsics.principal_point, (960.0, 540.0));
    }

    #[test]
    fn test_camera_pose_identity() {
        let pose = CameraPose::identity();
        assert_eq!(pose.position, (0.0, 0.0, 0.0));
        assert_eq!(pose.rotation, (0.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn test_point_3d_creation() {
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        assert_eq!(point.position, (1.0, 2.0, 3.0));
        assert_eq!(point.color, (255, 128, 64));
        assert_eq!(point.observation_count, 1);
    }

    #[test]
    fn test_point_3d_observations() {
        let mut point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        point.add_observation((255, 128, 64));
        assert_eq!(point.observation_count, 2);
        // With 2 observations, confidence = 2/10 = 0.2
        assert!(point.confidence > 0.1 && point.confidence < 0.3);

        // Add more observations to boost confidence
        for _ in 0..8 {
            point.add_observation((255, 128, 64));
        }
        assert_eq!(point.observation_count, 10);
        assert_eq!(point.confidence, 1.0); // 10/10 = 1.0
    }

    #[test]
    fn test_reconstruction_frame() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        let frame = ReconstructionFrame::new("img_1", intrinsics);

        assert_eq!(frame.image_id, "img_1");
        assert_eq!(frame.features.len(), 0);
    }

    #[test]
    fn test_frame_add_features() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        let mut frame = ReconstructionFrame::new("img_1", intrinsics);

        let features = vec![(100.0, 200.0), (150.0, 250.0)];
        frame.add_features(features);

        assert_eq!(frame.features.len(), 2);
        assert_eq!(frame.matched_3d_points.len(), 2);
    }

    #[test]
    fn test_point_cloud_creation() {
        let cloud = PointCloud::new();
        assert_eq!(cloud.points.len(), 0);
        assert!(cloud.bounds.is_none());
    }

    #[test]
    fn test_point_cloud_add_point() {
        let mut cloud = PointCloud::new();
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        cloud.add_point(point);

        assert_eq!(cloud.points.len(), 1);
        assert!(cloud.bounds.is_some());
    }

    #[test]
    fn test_point_cloud_bounds() {
        let mut cloud = PointCloud::new();
        cloud.add_point(Point3D::new((0.0, 0.0, 0.0), (255, 0, 0)));
        cloud.add_point(Point3D::new((10.0, 10.0, 10.0), (0, 255, 0)));

        let bounds = cloud.bounds.unwrap();
        assert_eq!(bounds.0, (0.0, 0.0, 0.0));
        assert_eq!(bounds.1, (10.0, 10.0, 10.0));
    }

    #[test]
    fn test_point_cloud_filter_confidence() {
        let mut cloud = PointCloud::new();
        let mut point1 = Point3D::new((0.0, 0.0, 0.0), (255, 0, 0));
        point1.confidence = 0.9;
        cloud.add_point(point1);

        let mut point2 = Point3D::new((1.0, 1.0, 1.0), (0, 255, 0));
        point2.confidence = 0.3;
        cloud.add_point(point2);

        let filtered = cloud.filter_by_confidence(0.5);
        assert_eq!(filtered.points.len(), 1);
    }

    #[test]
    fn test_point_cloud_statistics() {
        let mut cloud = PointCloud::new();
        cloud.add_point(Point3D::new((0.0, 0.0, 0.0), (255, 0, 0)));
        cloud.add_point(Point3D::new((1.0, 1.0, 1.0), (0, 255, 0)));

        let stats = cloud.statistics();
        assert_eq!(stats.point_count, 2);
    }

    #[test]
    fn test_reconstruction_engine() {
        let engine = ReconstructionEngine::new();
        assert_eq!(engine.frames.len(), 0);
        assert!(engine.location.is_none());
    }

    #[test]
    fn test_reconstruction_add_frame() {
        let mut engine = ReconstructionEngine::new();
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        let frame = ReconstructionFrame::new("img_1", intrinsics);

        let index = engine.add_frame(frame).unwrap();
        assert_eq!(index, 0);
        assert_eq!(engine.frames.len(), 1);
    }

    #[test]
    fn test_reconstruction_register_location() {
        let mut engine = ReconstructionEngine::new();
        let location = GeoPoint::new(40.7128, -74.0060);

        engine.register_location(location).unwrap();
        assert!(engine.location.is_some());
    }

    #[test]
    fn test_reconstruction_estimate_pose() {
        let mut engine = ReconstructionEngine::new();
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        let frame = ReconstructionFrame::new("img_1", intrinsics);

        engine.add_frame(frame).unwrap();
        engine.estimate_pose(0).unwrap();

        assert_eq!(engine.registered_frame_count, 1);
    }

    #[test]
    fn test_reconstruction_statistics() {
        let mut engine = ReconstructionEngine::new();
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));
        let frame = ReconstructionFrame::new("img_1", intrinsics);

        engine.add_frame(frame).unwrap();
        engine.add_point(Point3D::new((1.0, 2.0, 3.0), (255, 128, 64)));

        let stats = engine.statistics();
        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.point_cloud_stats.point_count, 1);
    }
}
