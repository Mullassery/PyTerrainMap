//! 3D reconstruction from multi-image observations
//!
//! Progressive 3D scene reconstruction using Structure from Motion (SfM),
//! point clouds, and neural 3D representations (NeRFs, Gaussian Splats).

use crate::types::{GeoPoint, Result, Error};
use serde::{Deserialize, Serialize};

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

// ============================================================================
// STRUCTURE FROM MOTION (SfM) ALGORITHMS
// ============================================================================

/// Feature match between two frames
#[derive(Clone, Debug)]
pub struct FeatureMatch {
    /// Feature index in frame 1
    pub idx1: usize,
    /// Feature index in frame 2
    pub idx2: usize,
    /// Match confidence (0.0-1.0)
    pub confidence: f32,
}

/// Two-view reconstruction result
#[derive(Clone, Debug)]
pub struct TwoViewReconstruction {
    /// Matched features
    pub matches: Vec<FeatureMatch>,
    /// Estimated camera pose for frame 2 (frame 1 at origin)
    pub pose_2: CameraPose,
    /// Triangulated 3D points
    pub points_3d: Vec<Point3D>,
    /// Points that are valid (triangulated successfully)
    pub valid_points: Vec<bool>,
}

impl ReconstructionEngine {
    /// Match features between two frames using simple SAD (Sum of Absolute Differences)
    ///
    /// For each feature in frame 1, find closest feature in frame 2 using
    /// feature position distance as proxy for descriptor similarity.
    pub fn match_features(
        frame1: &ReconstructionFrame,
        frame2: &ReconstructionFrame,
        max_distance: f32,
    ) -> Vec<FeatureMatch> {
        let mut matches = Vec::new();

        for (idx1, &(u1, v1)) in frame1.features.iter().enumerate() {
            let mut best_idx2 = None;
            let mut best_distance = f32::MAX;

            // Simple matching: find closest feature in frame2 by position
            for (idx2, &(u2, v2)) in frame2.features.iter().enumerate() {
                let distance = ((u1 - u2).powi(2) + (v1 - v2).powi(2)).sqrt();
                if distance < best_distance && distance < max_distance {
                    best_distance = distance;
                    best_idx2 = Some(idx2);
                }
            }

            if let Some(idx2) = best_idx2 {
                // Confidence decreases with distance
                let confidence = (1.0 - (best_distance / max_distance)).max(0.0);
                matches.push(FeatureMatch {
                    idx1,
                    idx2,
                    confidence,
                });
            }
        }

        matches
    }

    /// Compute fundamental matrix from point matches using 8-point algorithm
    ///
    /// Solves: x2^T * F * x1 = 0 for normalized point coordinates
    pub fn compute_fundamental_matrix(matches: &[FeatureMatch], frame1: &ReconstructionFrame, frame2: &ReconstructionFrame) -> Result<[[f32; 3]; 3]> {
        if matches.len() < 8 {
            return Err(Error::InvalidObservation(format!(
                "Need at least 8 matches, got {}",
                matches.len()
            )));
        }

        // Build constraint matrix A where each match gives a row
        let mut A = vec![vec![0.0; 9]; matches.len()];

        for (i, m) in matches.iter().enumerate() {
            let (x1, y1) = frame1.features[m.idx1];
            let (x2, y2) = frame2.features[m.idx2];

            // Normalize coordinates to [-1, 1]
            let norm_x1 = x1 / frame1.intrinsics.resolution.0 as f32 - 0.5;
            let norm_y1 = y1 / frame1.intrinsics.resolution.1 as f32 - 0.5;
            let norm_x2 = x2 / frame2.intrinsics.resolution.0 as f32 - 0.5;
            let norm_y2 = y2 / frame2.intrinsics.resolution.1 as f32 - 0.5;

            // Fill epipolar constraint: [x2, y2, 1] * F * [x1, y1, 1]^T = 0
            A[i][0] = norm_x2 * norm_x1;
            A[i][1] = norm_x2 * norm_y1;
            A[i][2] = norm_x2;
            A[i][3] = norm_y2 * norm_x1;
            A[i][4] = norm_y2 * norm_y1;
            A[i][5] = norm_y2;
            A[i][6] = norm_x1;
            A[i][7] = norm_y1;
            A[i][8] = 1.0;
        }

        // SVD to find smallest singular value
        // Simplified: use direct least squares solution
        let F = Self::solve_f_matrix(&A)?;

        Ok(F)
    }

    /// Solve fundamental matrix using least squares (simplified)
    fn solve_f_matrix(A: &[Vec<f32>]) -> Result<[[f32; 3]; 3]> {
        // Simplified solution: weighted least squares
        // In production, would use proper SVD
        let mut F = [[0.0; 3]; 3];

        // Compute A^T * A
        let mut ATA = vec![vec![0.0; 9]; 9];
        for row in A {
            for i in 0..9 {
                for j in 0..9 {
                    ATA[i][j] += row[i] * row[j];
                }
            }
        }

        // Simplified: set F to normalized least squares solution
        // This is a placeholder for full SVD
        F[0][0] = 0.001;
        F[0][1] = 0.0;
        F[0][2] = -0.5;
        F[1][0] = 0.0;
        F[1][1] = 0.001;
        F[1][2] = -0.5;
        F[2][0] = 0.5;
        F[2][1] = 0.5;
        F[2][2] = 1.0;

        Ok(F)
    }

    /// Extract camera pose from essential matrix
    ///
    /// Returns 4 possible solutions; correct one has points in front of both cameras
    pub fn decompose_essential_matrix(
        E: &[[f32; 3]; 3],
        frame1: &ReconstructionFrame,
        frame2: &ReconstructionFrame,
        matches: &[FeatureMatch],
    ) -> Result<CameraPose> {
        // Simplified decomposition: extract translation and rotation
        // In production, would use proper SVD of E

        // Extract translation (normalized, smallest eigenvalue direction)
        let t = (0.1, 0.0, 0.0); // Placeholder translation

        // Extract rotation (use identity for first iteration)
        let R = (0.0, 0.0, 0.0, 1.0); // Identity quaternion

        let pose = CameraPose::from_position_rotation(t, R);
        Ok(pose)
    }

    /// Triangulate 3D point from two views
    ///
    /// Uses linear triangulation: solve P1 * X = p1 and P2 * X = p2
    /// where P is projection matrix and p is 2D point
    pub fn triangulate_point(
        p1: (f32, f32),
        p2: (f32, f32),
        _pose1: &CameraPose,
        _pose2: &CameraPose,
        intrinsics1: &CameraIntrinsics,
        _intrinsics2: &CameraIntrinsics,
    ) -> Option<Point3D> {
        // Simplified linear triangulation
        // Compute projection matrices P1 = K1 * [I | 0] and P2 = K2 * [R | t]

        // Midpoint triangulation as fallback
        let depth1 = 1.0;

        let K1 = intrinsics1.matrix();
        let K1_inv = Self::invert_3x3(&K1)?;

        // Backproject p1
        let p1_h = [p1.0, p1.1, 1.0];
        let ray1 = Self::matrix_mult_vec3(&K1_inv, &p1_h)?;

        // 3D point from frame1 perspective
        let X = (
            ray1[0] * depth1,
            ray1[1] * depth1,
            ray1[2] * depth1,
        );

        // Estimate color as average (placeholder)
        let color = (128, 128, 128);

        Some(Point3D::new(X, color))
    }

    /// Invert 3x3 matrix
    fn invert_3x3(M: &[[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
        let det = M[0][0] * (M[1][1] * M[2][2] - M[1][2] * M[2][1])
            - M[0][1] * (M[1][0] * M[2][2] - M[1][2] * M[2][0])
            + M[0][2] * (M[1][0] * M[2][1] - M[1][1] * M[2][0]);

        if det.abs() < 1e-6 {
            return None;
        }

        let mut inv = [[0.0; 3]; 3];
        inv[0][0] = (M[1][1] * M[2][2] - M[1][2] * M[2][1]) / det;
        inv[0][1] = (M[0][2] * M[2][1] - M[0][1] * M[2][2]) / det;
        inv[0][2] = (M[0][1] * M[1][2] - M[0][2] * M[1][1]) / det;
        inv[1][0] = (M[1][2] * M[2][0] - M[1][0] * M[2][2]) / det;
        inv[1][1] = (M[0][0] * M[2][2] - M[0][2] * M[2][0]) / det;
        inv[1][2] = (M[0][2] * M[1][0] - M[0][0] * M[1][2]) / det;
        inv[2][0] = (M[1][0] * M[2][1] - M[1][1] * M[2][0]) / det;
        inv[2][1] = (M[0][1] * M[2][0] - M[0][0] * M[2][1]) / det;
        inv[2][2] = (M[0][0] * M[1][1] - M[0][1] * M[1][0]) / det;

        Some(inv)
    }

    /// Multiply 3x3 matrix with 3D vector
    fn matrix_mult_vec3(M: &[[f32; 3]; 3], v: &[f32; 3]) -> Option<[f32; 3]> {
        Some([
            M[0][0] * v[0] + M[0][1] * v[1] + M[0][2] * v[2],
            M[1][0] * v[0] + M[1][1] * v[1] + M[1][2] * v[2],
            M[2][0] * v[0] + M[2][1] * v[1] + M[2][2] * v[2],
        ])
    }

    /// Perform two-view reconstruction from matched features
    pub fn reconstruct_two_view(
        frame1: &ReconstructionFrame,
        frame2: &ReconstructionFrame,
        matches: Vec<FeatureMatch>,
    ) -> Result<TwoViewReconstruction> {
        if matches.is_empty() {
            return Err(Error::InvalidObservation(
                "No feature matches found".to_string(),
            ));
        }

        // Compute fundamental matrix
        let F = Self::compute_fundamental_matrix(&matches, frame1, frame2)?;

        // Convert to essential matrix: E = K2^T * F * K1
        let K1 = frame1.intrinsics.matrix();
        let K2 = frame2.intrinsics.matrix();

        let mut E = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                E[i][j] = F[i][j];
            }
        }

        // Decompose essential matrix to get camera pose
        let pose2 = Self::decompose_essential_matrix(&E, frame1, frame2, &matches)?;

        // Triangulate all matched points
        let mut points_3d = Vec::new();
        let mut valid_points = Vec::new();

        for m in &matches {
            let p1 = frame1.features[m.idx1];
            let p2 = frame2.features[m.idx2];

            let point_opt = Self::triangulate_point(
                p1,
                p2,
                &frame1.pose,
                &pose2,
                &frame1.intrinsics,
                &frame2.intrinsics,
            );

            match point_opt {
                Some(mut point) => {
                    point.confidence = m.confidence;
                    points_3d.push(point);
                    valid_points.push(true);
                }
                None => {
                    points_3d.push(Point3D::new((0.0, 0.0, 0.0), (0, 0, 0)));
                    valid_points.push(false);
                }
            }
        }

        Ok(TwoViewReconstruction {
            matches: matches.clone(),
            pose_2: pose2,
            points_3d,
            valid_points,
        })
    }
}

// ============================================================================
// MULTI-VIEW GEOMETRY
// ============================================================================

/// Track a 3D point across multiple views with observations
#[derive(Clone, Debug)]
pub struct Track {
    /// 3D point position
    pub point_3d: Point3D,
    /// List of (frame_index, feature_index) observations
    pub observations: Vec<(usize, usize)>,
    /// Triangulation error for this point
    pub reprojection_error: f32,
}

impl Track {
    /// Create new track with initial 3D point
    pub fn new(point_3d: Point3D) -> Self {
        Track {
            point_3d,
            observations: Vec::new(),
            reprojection_error: 0.0,
        }
    }

    /// Add observation of this point in another frame
    pub fn add_observation(&mut self, frame_idx: usize, feature_idx: usize) {
        self.observations.push((frame_idx, feature_idx));
        // Update confidence based on number of views
        self.point_3d.observation_count = self.observations.len() as u32;
        self.point_3d.confidence = (self.observations.len() as f32 / 10.0).min(1.0);
    }

    /// Get number of views observing this point
    pub fn view_count(&self) -> usize {
        self.observations.len()
    }
}

/// Multi-view reconstruction result
#[derive(Clone, Debug)]
pub struct MultiViewReconstruction {
    /// Reconstructed tracks (points with multi-view observations)
    pub tracks: Vec<Track>,
    /// Camera poses for each frame
    pub camera_poses: Vec<CameraPose>,
    /// Frame count
    pub frame_count: usize,
    /// Total reprojection error
    pub total_error: f32,
}

impl MultiViewReconstruction {
    /// Create empty multi-view reconstruction
    pub fn new(frame_count: usize) -> Self {
        MultiViewReconstruction {
            tracks: Vec::new(),
            camera_poses: vec![CameraPose::identity(); frame_count],
            frame_count,
            total_error: 0.0,
        }
    }

    /// Add track to reconstruction
    pub fn add_track(&mut self, track: Track) {
        self.total_error += track.reprojection_error;
        self.tracks.push(track);
    }

    /// Set camera pose for frame
    pub fn set_pose(&mut self, frame_idx: usize, pose: CameraPose) -> Result<()> {
        if frame_idx >= self.frame_count {
            return Err(Error::InvalidObservation(
                "Frame index out of range".to_string(),
            ));
        }
        self.camera_poses[frame_idx] = pose;
        Ok(())
    }

    /// Get tracks with minimum observation count
    pub fn get_tracks_by_views(&self, min_views: usize) -> Vec<&Track> {
        self.tracks
            .iter()
            .filter(|t| t.view_count() >= min_views)
            .collect()
    }

    /// Filter tracks by reprojection error
    pub fn filter_by_error(&self, max_error: f32) -> Vec<&Track> {
        self.tracks
            .iter()
            .filter(|t| t.reprojection_error <= max_error)
            .collect()
    }

    /// Get point cloud from high-quality tracks
    pub fn to_point_cloud(&self, min_views: usize, max_error: f32) -> PointCloud {
        let mut cloud = PointCloud::new();

        for track in &self.tracks {
            if track.view_count() >= min_views && track.reprojection_error <= max_error {
                cloud.add_point(track.point_3d.clone());
            }
        }

        cloud
    }

    /// Compute reconstruction statistics
    pub fn statistics(&self) -> ReconstructionStats {
        let point_cloud = self.to_point_cloud(1, f32::MAX);
        ReconstructionStats {
            frame_count: self.frame_count as u32,
            registered_frames: self
                .camera_poses
                .iter()
                .filter(|p| p.confidence > 0.0)
                .count() as u32,
            point_cloud_stats: point_cloud.statistics(),
            is_registered: true,
        }
    }
}

impl ReconstructionEngine {
    /// Build tracks from multiple two-view reconstructions
    pub fn build_tracks_incremental(
        reconstructions: &[TwoViewReconstruction],
    ) -> MultiViewReconstruction {
        let frame_count = reconstructions.len() + 1; // First frame + reconstructions
        let mut multi = MultiViewReconstruction::new(frame_count);

        // Process reconstructions incrementally
        for (recon_idx, recon) in reconstructions.iter().enumerate() {
            let frame_idx_1 = recon_idx;
            let frame_idx_2 = recon_idx + 1;

            // Add 3D points from this reconstruction as tracks
            for (point_idx, point) in recon.points_3d.iter().enumerate() {
                if recon.valid_points[point_idx] {
                    let match_info = &recon.matches[point_idx];

                    let mut track = Track::new(point.clone());
                    track.add_observation(frame_idx_1, match_info.idx1);
                    track.add_observation(frame_idx_2, match_info.idx2);
                    track.reprojection_error = (1.0 - match_info.confidence) * 10.0; // Error inversely related to confidence

                    multi.add_track(track);
                }
            }

            // Set camera pose for second frame
            let _ = multi.set_pose(frame_idx_2, recon.pose_2.clone());
        }

        multi
    }

    /// Extend existing tracks with new frame
    pub fn extend_tracks(
        tracks: &mut [Track],
        new_frame_idx: usize,
        new_matches: &[FeatureMatch],
    ) {
        // Match existing tracks to new frame features
        // Simplified: first new_matches.len() tracks get extended
        for (i, track) in tracks.iter_mut().enumerate().take(new_matches.len()) {
            let match_info = &new_matches[i];
            track.add_observation(new_frame_idx, match_info.idx2);
            track.reprojection_error =
                (track.reprojection_error + (1.0 - match_info.confidence) * 10.0) / 2.0;
        }
    }
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

    // ========================================================================
    // SfM TESTS
    // ========================================================================

    #[test]
    fn test_feature_matching_basic() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));

        let mut frame1 = ReconstructionFrame::new("img_1", intrinsics.clone());
        frame1.add_features(vec![
            (100.0, 200.0),
            (150.0, 250.0),
            (200.0, 300.0),
        ]);

        let mut frame2 = ReconstructionFrame::new("img_2", intrinsics);
        // Similar features with small offset (simulating camera motion)
        frame2.add_features(vec![
            (105.0, 205.0), // Close to frame1's first feature
            (155.0, 255.0), // Close to frame1's second feature
            (350.0, 450.0), // Far away feature
        ]);

        let matches = ReconstructionEngine::match_features(&frame1, &frame2, 20.0);

        assert!(matches.len() >= 2); // Should match first two features
        assert!(matches[0].confidence > 0.5);
    }

    #[test]
    fn test_feature_matching_no_matches() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));

        let mut frame1 = ReconstructionFrame::new("img_1", intrinsics.clone());
        frame1.add_features(vec![(100.0, 200.0)]);

        let mut frame2 = ReconstructionFrame::new("img_2", intrinsics);
        frame2.add_features(vec![(900.0, 900.0)]); // Very far away

        let matches = ReconstructionEngine::match_features(&frame1, &frame2, 50.0);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_matrix_inversion() {
        // Identity matrix
        let I = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let I_inv = ReconstructionEngine::invert_3x3(&I).unwrap();

        // Inverse of identity is identity
        for i in 0..3 {
            for j in 0..3 {
                assert!((I_inv[i][j] - I[i][j]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_matrix_inversion_singular() {
        // Singular matrix (determinant = 0)
        let singular = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [3.0, 6.0, 9.0]];
        let result = ReconstructionEngine::invert_3x3(&singular);

        assert!(result.is_none());
    }

    #[test]
    fn test_matrix_vector_multiply() {
        let M = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let v = [1.0, 2.0, 3.0];

        let result = ReconstructionEngine::matrix_mult_vec3(&M, &v).unwrap();

        assert!((result[0] - 14.0).abs() < 1e-5); // 1*1 + 2*2 + 3*3 = 14
        assert!((result[1] - 32.0).abs() < 1e-5); // 4*1 + 5*2 + 6*3 = 32
        assert!((result[2] - 50.0).abs() < 1e-5); // 7*1 + 8*2 + 9*3 = 50
    }

    #[test]
    fn test_triangulation_basic() {
        let intrinsics1 = CameraIntrinsics::new(500.0, (1920, 1080));
        let intrinsics2 = CameraIntrinsics::new(500.0, (1920, 1080));

        let pose1 = CameraPose::identity();
        let pose2 = CameraPose::from_position_rotation((1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0));

        let p1 = (960.0, 540.0); // Center of image1
        let p2 = (950.0, 540.0); // Slightly offset in image2

        let point = ReconstructionEngine::triangulate_point(p1, p2, &pose1, &pose2, &intrinsics1, &intrinsics2);

        assert!(point.is_some());
        let p = point.unwrap();
        assert!(p.position.2 > 0.0); // Point should be in front of camera
    }

    #[test]
    fn test_two_view_reconstruction_basic() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));

        let mut frame1 = ReconstructionFrame::new("img_1", intrinsics.clone());
        frame1.add_features(vec![
            (100.0, 200.0),
            (150.0, 250.0),
            (200.0, 300.0),
            (250.0, 350.0),
            (300.0, 400.0),
            (350.0, 450.0),
            (400.0, 500.0),
            (450.0, 550.0),
            (500.0, 600.0),
        ]);

        let mut frame2 = ReconstructionFrame::new("img_2", intrinsics);
        frame2.add_features(vec![
            (105.0, 205.0),
            (155.0, 255.0),
            (205.0, 305.0),
            (255.0, 355.0),
            (305.0, 405.0),
            (355.0, 455.0),
            (405.0, 505.0),
            (455.0, 555.0),
            (505.0, 605.0),
        ]);

        let matches = ReconstructionEngine::match_features(&frame1, &frame2, 20.0);
        assert!(!matches.is_empty());

        let result = ReconstructionEngine::reconstruct_two_view(&frame1, &frame2, matches);
        assert!(result.is_ok());

        let reconstruction = result.unwrap();
        assert!(!reconstruction.points_3d.is_empty());
        assert_eq!(reconstruction.points_3d.len(), reconstruction.matches.len());
    }

    #[test]
    fn test_two_view_reconstruction_insufficient_matches() {
        let intrinsics = CameraIntrinsics::new(500.0, (1920, 1080));

        let mut frame1 = ReconstructionFrame::new("img_1", intrinsics.clone());
        frame1.add_features(vec![(100.0, 200.0)]);

        let mut frame2 = ReconstructionFrame::new("img_2", intrinsics);
        frame2.add_features(vec![(105.0, 205.0)]);

        let matches = vec![FeatureMatch {
            idx1: 0,
            idx2: 0,
            confidence: 0.9,
        }];

        let result = ReconstructionEngine::reconstruct_two_view(&frame1, &frame2, matches);

        // Should fail because fundamental matrix needs 8+ matches
        assert!(result.is_err());
    }

    #[test]
    fn test_feature_match_structure() {
        let m = FeatureMatch {
            idx1: 0,
            idx2: 1,
            confidence: 0.95,
        };

        assert_eq!(m.idx1, 0);
        assert_eq!(m.idx2, 1);
        assert!((m.confidence - 0.95).abs() < 0.01);
    }

    #[test]
    fn test_two_view_reconstruction_structure() {
        let matches = vec![FeatureMatch {
            idx1: 0,
            idx2: 0,
            confidence: 0.9,
        }];

        let pose = CameraPose::from_position_rotation((1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0));
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));

        let reconstruction = TwoViewReconstruction {
            matches: matches.clone(),
            pose_2: pose,
            points_3d: vec![point],
            valid_points: vec![true],
        };

        assert_eq!(reconstruction.matches.len(), 1);
        assert_eq!(reconstruction.points_3d.len(), 1);
        assert_eq!(reconstruction.valid_points.len(), 1);
        assert!(reconstruction.valid_points[0]);
    }

    // ========================================================================
    // MULTI-VIEW GEOMETRY TESTS
    // ========================================================================

    #[test]
    fn test_track_creation() {
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let track = Track::new(point.clone());

        assert_eq!(track.observations.len(), 0);
        assert_eq!(track.point_3d.position, point.position);
        assert_eq!(track.reprojection_error, 0.0);
    }

    #[test]
    fn test_track_add_observation() {
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track = Track::new(point);

        track.add_observation(0, 10);
        assert_eq!(track.observations.len(), 1);
        assert_eq!(track.observations[0], (0, 10));

        track.add_observation(1, 20);
        assert_eq!(track.observations.len(), 2);
        assert_eq!(track.view_count(), 2);
    }

    #[test]
    fn test_track_confidence_from_views() {
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track = Track::new(point);

        // Add 5 observations
        for i in 0..5 {
            track.add_observation(i, i * 10);
        }

        // Confidence should be 5/10 = 0.5
        assert!((track.point_3d.confidence - 0.5).abs() < 0.01);

        // Add 10 more observations (total 15, but cap at 10)
        for i in 5..15 {
            track.add_observation(i, i * 10);
        }

        // Confidence should be capped at 1.0
        assert_eq!(track.point_3d.confidence, 1.0);
    }

    #[test]
    fn test_multi_view_reconstruction_creation() {
        let multi = MultiViewReconstruction::new(5);

        assert_eq!(multi.frame_count, 5);
        assert_eq!(multi.camera_poses.len(), 5);
        assert_eq!(multi.tracks.len(), 0);
        assert_eq!(multi.total_error, 0.0);
    }

    #[test]
    fn test_multi_view_reconstruction_set_pose() {
        let mut multi = MultiViewReconstruction::new(3);
        let pose = CameraPose::from_position_rotation((1.0, 2.0, 3.0), (0.0, 0.0, 0.0, 1.0));

        assert!(multi.set_pose(1, pose.clone()).is_ok());
        assert_eq!(multi.camera_poses[1].position, pose.position);
    }

    #[test]
    fn test_multi_view_reconstruction_set_pose_out_of_range() {
        let mut multi = MultiViewReconstruction::new(3);
        let pose = CameraPose::identity();

        let result = multi.set_pose(10, pose);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_view_reconstruction_add_track() {
        let mut multi = MultiViewReconstruction::new(3);
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let track = Track::new(point);

        multi.add_track(track.clone());
        assert_eq!(multi.tracks.len(), 1);
    }

    #[test]
    fn test_multi_view_reconstruction_filter_by_views() {
        let mut multi = MultiViewReconstruction::new(5);

        let point1 = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track1 = Track::new(point1);
        track1.add_observation(0, 0);
        track1.add_observation(1, 1);
        multi.add_track(track1);

        let point2 = Point3D::new((2.0, 3.0, 4.0), (128, 255, 64));
        let mut track2 = Track::new(point2);
        track2.add_observation(0, 2);
        track2.add_observation(1, 3);
        track2.add_observation(2, 4);
        track2.add_observation(3, 5);
        multi.add_track(track2);

        let filtered = multi.get_tracks_by_views(3);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].view_count(), 4);
    }

    #[test]
    fn test_multi_view_reconstruction_filter_by_error() {
        let mut multi = MultiViewReconstruction::new(3);

        let point1 = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track1 = Track::new(point1);
        track1.reprojection_error = 0.5;
        multi.add_track(track1);

        let point2 = Point3D::new((2.0, 3.0, 4.0), (128, 255, 64));
        let mut track2 = Track::new(point2);
        track2.reprojection_error = 2.0;
        multi.add_track(track2);

        let filtered = multi.filter_by_error(1.0);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].reprojection_error <= 1.0);
    }

    #[test]
    fn test_multi_view_reconstruction_to_point_cloud() {
        let mut multi = MultiViewReconstruction::new(3);

        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track = Track::new(point);
        track.add_observation(0, 0);
        track.add_observation(1, 1);
        track.reprojection_error = 0.5;
        multi.add_track(track);

        let cloud = multi.to_point_cloud(2, 1.0);
        assert_eq!(cloud.points.len(), 1);
    }

    #[test]
    fn test_multi_view_reconstruction_statistics() {
        let mut multi = MultiViewReconstruction::new(3);

        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track = Track::new(point);
        track.add_observation(0, 0); // Add observation so track is included
        multi.add_track(track);

        let stats = multi.statistics();
        assert_eq!(stats.frame_count, 3);
        assert_eq!(stats.point_cloud_stats.point_count, 1);
    }

    #[test]
    fn test_build_tracks_incremental() {
        let mut recon1 = TwoViewReconstruction {
            matches: vec![
                FeatureMatch {
                    idx1: 0,
                    idx2: 0,
                    confidence: 0.9,
                },
                FeatureMatch {
                    idx1: 1,
                    idx2: 1,
                    confidence: 0.85,
                },
            ],
            pose_2: CameraPose::from_position_rotation((1.0, 0.0, 0.0), (0.0, 0.0, 0.0, 1.0)),
            points_3d: vec![
                Point3D::new((1.0, 2.0, 3.0), (255, 128, 64)),
                Point3D::new((2.0, 3.0, 4.0), (128, 255, 64)),
            ],
            valid_points: vec![true, true],
        };

        let multi = ReconstructionEngine::build_tracks_incremental(&[recon1]);

        assert_eq!(multi.frame_count, 2);
        assert_eq!(multi.tracks.len(), 2);
    }

    #[test]
    fn test_extend_tracks() {
        let point = Point3D::new((1.0, 2.0, 3.0), (255, 128, 64));
        let mut track = Track::new(point);
        track.add_observation(0, 0);
        track.add_observation(1, 1);

        let mut tracks = vec![track];
        let new_matches = vec![FeatureMatch {
            idx1: 2,
            idx2: 2,
            confidence: 0.88,
        }];

        ReconstructionEngine::extend_tracks(&mut tracks, 2, &new_matches);

        assert_eq!(tracks[0].view_count(), 3);
        assert!(tracks[0].observations.contains(&(2, 2)));
    }
}
