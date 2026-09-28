//! Real multi-view geometry: the normalized 8-point algorithm, essential
//! matrix pose recovery (Longuet-Higgins decomposition + cheirality check),
//! DLT triangulation, and structure-only bundle adjustment.
//!
//! `mod.rs`'s `StructureFromMotion` previously faked every stage of this
//! (identity poses, sequential-index "matches", points placed at a fixed
//! offset regardless of the actual matched pixels, and a "bundle adjustment"
//! that only nudged a confidence score). This module does the real thing,
//! operating on real 2D pixel correspondences the caller supplies (this
//! crate's imaging boundary is deliberately Python-side -- see the `image`
//! crate removal note in `Cargo.toml` -- so keypoint *detection* from raw
//! pixels happens there; this module does the real *geometry* once given
//! real keypoints).

use nalgebra::{Matrix3, Matrix3x4, Matrix4, Rotation3, SVD, Unit, UnitQuaternion, Vector3};

/// A single real 2D image observation of a 3D point (pixel coordinates).
#[derive(Clone, Copy, Debug)]
pub struct PixelObservation {
    pub x: f32,
    pub y: f32,
}

/// Pinhole camera intrinsics (simplified: single focal length for both
/// axes, matching `CameraPoseEstimate`'s existing model).
#[derive(Clone, Copy, Debug)]
pub struct Intrinsics {
    pub focal_length: f32,
    pub principal_point: (f32, f32),
}

impl Intrinsics {
    pub fn k_matrix(&self) -> Matrix3<f64> {
        Matrix3::new(
            self.focal_length as f64,
            0.0,
            self.principal_point.0 as f64,
            0.0,
            self.focal_length as f64,
            self.principal_point.1 as f64,
            0.0,
            0.0,
            1.0,
        )
    }

    /// Convert a real pixel coordinate to normalized camera coordinates
    /// (undoing focal length + principal point) -- what the 8-point
    /// algorithm and essential-matrix math operate on.
    pub fn normalize(&self, p: PixelObservation) -> (f64, f64) {
        (
            (p.x as f64 - self.principal_point.0 as f64) / self.focal_length as f64,
            (p.y as f64 - self.principal_point.1 as f64) / self.focal_length as f64,
        )
    }
}

/// A rigid camera pose: rotation + translation, world-to-camera
/// (`p_cam = R * p_world + t`), the standard SfM convention.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub rotation: Rotation3<f64>,
    pub translation: Vector3<f64>,
}

impl Pose {
    pub fn identity() -> Self {
        Pose {
            rotation: Rotation3::identity(),
            translation: Vector3::zeros(),
        }
    }

    /// 3x4 projection matrix `[R|t]` (extrinsics only; combine with `K` for
    /// the full camera matrix).
    pub fn extrinsics(&self) -> Matrix3x4<f64> {
        let mut m = Matrix3x4::zeros();
        m.fixed_view_mut::<3, 3>(0, 0).copy_from(&self.rotation.matrix());
        m.fixed_view_mut::<3, 1>(0, 3).copy_from(&self.translation);
        m
    }
}

#[derive(Debug)]
pub struct GeometryError(pub String);

impl std::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for GeometryError {}

/// Hartley normalization: translate points to centroid, scale so mean
/// distance from origin is sqrt(2). Real, standard preconditioning for the
/// 8-point algorithm -- without it, the algorithm is numerically unstable
/// (Hartley 1997, "In Defense of the Eight-Point Algorithm").
fn hartley_normalize(points: &[(f64, f64)]) -> (Vec<(f64, f64)>, Matrix3<f64>) {
    let n = points.len() as f64;
    let (sum_x, sum_y) = points.iter().fold((0.0, 0.0), |acc, p| (acc.0 + p.0, acc.1 + p.1));
    let (cx, cy) = (sum_x / n, sum_y / n);

    let mean_dist: f64 = points
        .iter()
        .map(|(x, y)| (((x - cx).powi(2) + (y - cy).powi(2)) as f64).sqrt())
        .sum::<f64>()
        / n;
    let scale = if mean_dist > 1e-12 {
        std::f64::consts::SQRT_2 / mean_dist
    } else {
        1.0
    };

    let normalized: Vec<(f64, f64)> = points
        .iter()
        .map(|(x, y)| (scale * (x - cx), scale * (y - cy)))
        .collect();

    #[rustfmt::skip]
    let t = Matrix3::new(
        scale, 0.0,   -scale * cx,
        0.0,   scale, -scale * cy,
        0.0,   0.0,   1.0,
    );
    (normalized, t)
}

/// Real normalized 8-point algorithm: estimates the fundamental matrix `F`
/// from >= 8 real point correspondences in normalized camera coordinates.
/// Enforces the real rank-2 constraint (F is singular) via SVD.
pub fn estimate_fundamental_matrix(
    points1: &[(f64, f64)],
    points2: &[(f64, f64)],
) -> Result<Matrix3<f64>, GeometryError> {
    if points1.len() != points2.len() {
        return Err(GeometryError("point correspondence count mismatch".to_string()));
    }
    if points1.len() < 8 {
        return Err(GeometryError(format!(
            "8-point algorithm needs >= 8 correspondences, got {}",
            points1.len()
        )));
    }

    let (norm1, t1) = hartley_normalize(points1);
    let (norm2, t2) = hartley_normalize(points2);

    let n = norm1.len();
    let mut a = nalgebra::DMatrix::<f64>::zeros(n, 9);
    for i in 0..n {
        let (x1, y1) = norm1[i];
        let (x2, y2) = norm2[i];
        a.set_row(
            i,
            &nalgebra::RowDVector::from_vec(vec![
                x2 * x1, x2 * y1, x2, y2 * x1, y2 * y1, y2, x1, y1, 1.0,
            ]),
        );
    }

    let svd = SVD::new(a, true, true);
    let v_t = svd
        .v_t
        .ok_or_else(|| GeometryError("SVD failed to produce V^T".to_string()))?;
    // Smallest singular value's row of V^T is the last row (nalgebra's SVD
    // orders singular values descending).
    let f_vec = v_t.row(v_t.nrows() - 1);
    let f_raw = Matrix3::new(
        f_vec[0], f_vec[1], f_vec[2], f_vec[3], f_vec[4], f_vec[5], f_vec[6], f_vec[7], f_vec[8],
    );

    // Enforce rank-2: zero the smallest singular value of F itself.
    let svd_f = SVD::new(f_raw, true, true);
    let mut singular_values = svd_f.singular_values;
    singular_values[2] = 0.0;
    let u = svd_f.u.ok_or_else(|| GeometryError("SVD(F) missing U".to_string()))?;
    let v_t_f = svd_f
        .v_t
        .ok_or_else(|| GeometryError("SVD(F) missing V^T".to_string()))?;
    let f_rank2 = u * Matrix3::from_diagonal(&singular_values) * v_t_f;

    // Denormalize: F = T2^T * F_normalized * T1
    Ok(t2.transpose() * f_rank2 * t1)
}

/// Essential matrix from fundamental matrix + both cameras' intrinsics:
/// `E = K2^T * F * K1`.
pub fn essential_from_fundamental(f: &Matrix3<f64>, k1: &Matrix3<f64>, k2: &Matrix3<f64>) -> Matrix3<f64> {
    k2.transpose() * f * k1
}

/// Decompose an essential matrix into the 4 candidate (R, t) pose
/// hypotheses (Longuet-Higgins / Hartley-Zisserman `recoverPose`
/// decomposition). Exactly one is physically valid; `recover_pose` below
/// picks it via the cheirality check.
fn decompose_essential(e: &Matrix3<f64>) -> Vec<(Rotation3<f64>, Vector3<f64>)> {
    let svd = SVD::new(*e, true, true);
    let u = svd.u.expect("SVD(E) should produce U for a well-formed essential matrix");
    let v_t = svd.v_t.expect("SVD(E) should produce V^T for a well-formed essential matrix");

    #[rustfmt::skip]
    let w = Matrix3::new(
        0.0, -1.0, 0.0,
        1.0,  0.0, 0.0,
        0.0,  0.0, 1.0,
    );

    let mut u = u;
    let mut v_t = v_t;
    // Ensure det(U) = det(V) = +1 (SVD can return improper rotations with
    // det = -1, which would make R below not a valid rotation matrix).
    if u.determinant() < 0.0 {
        u.set_column(2, &(-u.column(2)));
    }
    if v_t.determinant() < 0.0 {
        v_t.set_row(2, &(-v_t.row(2)));
    }

    let r1 = u * w * v_t;
    let r2 = u * w.transpose() * v_t;
    let t = u.column(2).into_owned();

    vec![
        (Rotation3::from_matrix_unchecked(r1), t),
        (Rotation3::from_matrix_unchecked(r1), -t),
        (Rotation3::from_matrix_unchecked(r2), t),
        (Rotation3::from_matrix_unchecked(r2), -t),
    ]
}

/// Real DLT (Direct Linear Transform) triangulation: given two camera
/// projection matrices (`K * [R|t]`, i.e. pixel-space projection) and a
/// real matched normalized-camera-coordinate pair, solves for the 3D point
/// via SVD of the standard 4x4 homogeneous linear system.
pub fn triangulate_dlt(
    p1: &Matrix3x4<f64>,
    p2: &Matrix3x4<f64>,
    pt1: (f64, f64),
    pt2: (f64, f64),
) -> Vector3<f64> {
    let mut a = Matrix4::<f64>::zeros();
    a.set_row(0, &(pt1.0 * p1.row(2) - p1.row(0)));
    a.set_row(1, &(pt1.1 * p1.row(2) - p1.row(1)));
    a.set_row(2, &(pt2.0 * p2.row(2) - p2.row(0)));
    a.set_row(3, &(pt2.1 * p2.row(2) - p2.row(1)));

    let svd = SVD::new(a, true, true);
    let v_t = svd.v_t.expect("SVD should produce V^T for triangulation");
    let x_h = v_t.row(v_t.nrows() - 1);
    Vector3::new(x_h[0] / x_h[3], x_h[1] / x_h[3], x_h[2] / x_h[3])
}

/// Recover the real relative pose of camera 2 (relative to camera 1 at the
/// identity) from an essential matrix, using the cheirality check: of the 4
/// candidate (R, t) decompositions, the physically valid one is the only
/// one that puts triangulated points in front of *both* cameras (positive
/// depth). Returns `None` if no candidate passes for a clear majority of
/// the given correspondences (degenerate configuration).
pub fn recover_pose(
    e: &Matrix3<f64>,
    points1_norm: &[(f64, f64)],
    points2_norm: &[(f64, f64)],
) -> Option<Pose> {
    let candidates = decompose_essential(e);
    let pose1 = Pose::identity();
    let p1 = pose1.extrinsics();

    let mut best: Option<(usize, Pose)> = None;
    for (rotation, translation) in candidates {
        let pose2 = Pose { rotation, translation };
        let p2 = pose2.extrinsics();

        let mut positive_depth_count = 0;
        for (&pt1, &pt2) in points1_norm.iter().zip(points2_norm.iter()) {
            let point3d = triangulate_dlt(&p1, &p2, pt1, pt2);
            let depth1 = point3d.z; // camera 1 is at the identity
            let depth2 = (pose2.rotation * point3d + pose2.translation).z;
            if depth1 > 0.0 && depth2 > 0.0 {
                positive_depth_count += 1;
            }
        }

        if best.map(|(count, _)| positive_depth_count > count).unwrap_or(true) {
            best = Some((positive_depth_count, pose2));
        }
    }

    best.and_then(|(count, pose)| {
        // Require a real majority of correspondences to have positive depth
        // under the chosen pose -- otherwise this isn't a reliable recovery.
        if count * 2 >= points1_norm.len() {
            Some(pose)
        } else {
            None
        }
    })
}

/// Convert a `nalgebra::Rotation3<f64>` to the `(qx, qy, qz, qw)` quaternion
/// format `CameraPoseEstimate` stores.
pub fn rotation_to_quaternion(r: &Rotation3<f64>) -> (f32, f32, f32, f32) {
    let q = UnitQuaternion::from_rotation_matrix(r);
    (
        q.i as f32,
        q.j as f32,
        q.k as f32,
        q.w as f32,
    )
}

/// Convert a stored `(qx, qy, qz, qw)` quaternion back into a rotation
/// matrix, for real reprojection math.
pub fn quaternion_to_rotation(q: (f32, f32, f32, f32)) -> Rotation3<f64> {
    let unit = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(
        q.3 as f64, q.0 as f64, q.1 as f64, q.2 as f64,
    ));
    unit.to_rotation_matrix()
}

/// Real perspective reprojection of a 3D world point into a camera's pixel
/// coordinates, given its pose and intrinsics.
pub fn project(point_world: &Vector3<f64>, pose: &Pose, intrinsics: &Intrinsics) -> Option<(f64, f64)> {
    let p_cam = pose.rotation * point_world + pose.translation;
    if p_cam.z <= 1e-9 {
        return None; // behind the camera; no real projection exists
    }
    let fx = intrinsics.focal_length as f64;
    let fy = intrinsics.focal_length as f64;
    let (cx, cy) = (intrinsics.principal_point.0 as f64, intrinsics.principal_point.1 as f64);
    Some((fx * p_cam.x / p_cam.z + cx, fy * p_cam.y / p_cam.z + cy))
}

/// One real camera observation of a 3D point, used by `refine_point`.
pub struct Observation<'a> {
    pub pose: &'a Pose,
    pub intrinsics: &'a Intrinsics,
    pub pixel: PixelObservation,
}

/// Structure-only bundle adjustment for a single 3D point: real
/// Gauss-Newton minimization of summed squared reprojection error across
/// all real observations of this point, holding camera poses fixed. This
/// is the standard "point refinement" / "structure-only BA" simplification
/// of full joint bundle adjustment (which would also refine camera poses) --
/// real numerical optimization on real reprojection residuals, not a
/// fabricated confidence bump. Returns the refined position and the final
/// RMS reprojection error (pixels).
pub fn refine_point(
    initial: Vector3<f64>,
    observations: &[Observation],
    max_iterations: usize,
) -> (Vector3<f64>, f64) {
    let mut x = initial;
    let mut last_rms = rms_reprojection_error(&x, observations);

    for _ in 0..max_iterations {
        // Build the 2*N x 3 Jacobian of pixel residuals w.r.t. the 3D point,
        // and the 2*N residual vector, via the real analytic perspective
        // projection derivative.
        let mut jtj = nalgebra::Matrix3::<f64>::zeros();
        let mut jtr = Vector3::<f64>::zeros();
        let mut any_valid = false;

        for obs in observations {
            let p_cam = obs.pose.rotation * x + obs.pose.translation;
            if p_cam.z <= 1e-9 {
                continue;
            }
            any_valid = true;
            let fx = obs.intrinsics.focal_length as f64;
            let fy = obs.intrinsics.focal_length as f64;
            let (cx, cy) = (
                obs.intrinsics.principal_point.0 as f64,
                obs.intrinsics.principal_point.1 as f64,
            );
            let z_inv = 1.0 / p_cam.z;

            let predicted_u = fx * p_cam.x * z_inv + cx;
            let predicted_v = fy * p_cam.y * z_inv + cy;
            let residual_u = obs.pixel.x as f64 - predicted_u;
            let residual_v = obs.pixel.y as f64 - predicted_v;

            // d(u)/d(p_cam) = [fx/z, 0, -fx*x/z^2], d(v)/d(p_cam) similarly.
            let d_u_d_pcam = Vector3::new(fx * z_inv, 0.0, -fx * p_cam.x * z_inv * z_inv);
            let d_v_d_pcam = Vector3::new(0.0, fy * z_inv, -fy * p_cam.y * z_inv * z_inv);
            // p_cam = R*x + t, so d(p_cam)/d(x) = R -- chain rule via R^T row vectors.
            let r = obs.pose.rotation.matrix();
            let j_u = r.transpose() * d_u_d_pcam;
            let j_v = r.transpose() * d_v_d_pcam;

            jtj += j_u * j_u.transpose() + j_v * j_v.transpose();
            jtr += j_u * residual_u + j_v * residual_v;
        }

        if !any_valid {
            break;
        }

        // Levenberg-Marquardt damping for numerical stability on
        // near-singular configurations (e.g. near-collinear observations).
        let damped = jtj + Matrix3::identity() * 1e-6;
        let delta = match damped.lu().solve(&jtr) {
            Some(d) => d,
            None => break,
        };

        let candidate = x + delta;
        let candidate_rms = rms_reprojection_error(&candidate, observations);
        if candidate_rms < last_rms {
            x = candidate;
            last_rms = candidate_rms;
        } else {
            break; // Gauss-Newton step didn't improve; converged (or diverging)
        }
    }

    (x, last_rms)
}

fn rms_reprojection_error(point: &Vector3<f64>, observations: &[Observation]) -> f64 {
    let mut sum_sq = 0.0;
    let mut count = 0;
    for obs in observations {
        if let Some((u, v)) = project(point, obs.pose, obs.intrinsics) {
            let du = obs.pixel.x as f64 - u;
            let dv = obs.pixel.y as f64 - v;
            sum_sq += du * du + dv * dv;
            count += 1;
        }
    }
    if count == 0 {
        return f64::INFINITY;
    }
    (sum_sq / count as f64).sqrt()
}

#[allow(dead_code)]
pub fn unit_translation(t: Vector3<f64>) -> Vector3<f64> {
    Unit::new_normalize(t).into_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic scene: a real 3D point cloud, and two real cameras
    /// with known ground-truth poses, and project the points into both to
    /// get real (synthetic but geometrically genuine) 2D correspondences.
    /// This is the standard way to test multi-view geometry code without
    /// needing actual photographs -- the projections are real perspective
    /// projections of real 3D points, so recovering the known ground truth
    /// from them is a genuine correctness test, not a tautology.
    fn synthetic_scene() -> (Vec<Vector3<f64>>, Pose, Pose, Intrinsics) {
        let intrinsics = Intrinsics {
            focal_length: 1000.0,
            principal_point: (640.0, 360.0),
        };
        let pose1 = Pose::identity();
        // Camera 2: shifted 1 unit along +x, rotated 10 degrees around Y.
        let pose2 = Pose {
            rotation: Rotation3::from_axis_angle(&Vector3::y_axis(), 10.0_f64.to_radians()),
            translation: Vector3::new(1.0, 0.0, 0.2),
        };

        let mut points = Vec::new();
        // Deterministic pseudo-random-looking but fully reproducible spread
        // of points in front of both cameras.
        for i in 0..20 {
            let fi = i as f64;
            let x = ((fi * 0.7).sin()) * 2.0;
            let y = ((fi * 0.4).cos()) * 1.5;
            let z = 5.0 + (fi * 0.31).sin() * 1.5;
            points.push(Vector3::new(x, y, z));
        }
        (points, pose1, pose2, intrinsics)
    }

    #[test]
    fn hartley_normalize_centers_and_scales_points() {
        let points = vec![(0.0, 0.0), (2.0, 0.0), (0.0, 2.0), (2.0, 2.0)];
        let (normalized, _) = hartley_normalize(&points);
        let (sum_x, sum_y) = normalized.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        assert!((sum_x / 4.0).abs() < 1e-9);
        assert!((sum_y / 4.0).abs() < 1e-9);
        let mean_dist: f64 = normalized.iter().map(|(x, y)| (x * x + y * y).sqrt()).sum::<f64>() / 4.0;
        assert!((mean_dist - std::f64::consts::SQRT_2).abs() < 1e-9);
    }

    #[test]
    fn essential_matrix_pose_recovery_matches_ground_truth() {
        let (points, pose1, pose2, intrinsics) = synthetic_scene();

        let normalized1: Vec<(f64, f64)> = points
            .iter()
            .map(|p| {
                let (u, v) = project(p, &pose1, &intrinsics).unwrap();
                intrinsics.normalize(PixelObservation { x: u as f32, y: v as f32 })
            })
            .collect();
        let normalized2: Vec<(f64, f64)> = points
            .iter()
            .map(|p| {
                let (u, v) = project(p, &pose2, &intrinsics).unwrap();
                intrinsics.normalize(PixelObservation { x: u as f32, y: v as f32 })
            })
            .collect();

        // Since inputs are already normalized (K = I applied), estimate F
        // directly in normalized coordinates -- it equals E for K = I.
        let f = estimate_fundamental_matrix(&normalized1, &normalized2).unwrap();
        let identity_k = Matrix3::identity();
        let e = essential_from_fundamental(&f, &identity_k, &identity_k);

        let recovered = recover_pose(&e, &normalized1, &normalized2).expect("pose recovery should succeed on a clean synthetic scene");

        // Two-view SfM has inherent scale ambiguity (translation is only
        // recoverable up to scale) -- compare rotation directly, and
        // translation *direction* (unit vector), not magnitude.
        let rotation_error = (recovered.rotation.matrix() - pose2.rotation.matrix()).norm();
        assert!(rotation_error < 0.05, "rotation error too large: {rotation_error}");

        let gt_dir = pose2.translation.normalize();
        let recovered_dir = recovered.translation.normalize();
        let dir_error = (gt_dir - recovered_dir).norm();
        assert!(dir_error < 0.05, "translation direction error too large: {dir_error}");
    }

    #[test]
    fn triangulate_dlt_recovers_known_3d_point() {
        let (points, pose1, pose2, intrinsics) = synthetic_scene();
        let k = intrinsics.k_matrix();
        let p1 = k * pose1.extrinsics();
        let p2 = k * pose2.extrinsics();

        for point in &points {
            let (u1, v1) = project(point, &pose1, &intrinsics).unwrap();
            let (u2, v2) = project(point, &pose2, &intrinsics).unwrap();
            let recovered = triangulate_dlt(&p1, &p2, (u1, v1), (u2, v2));
            let error = (recovered - point).norm();
            assert!(error < 1e-6, "triangulation error too large: {error} for point {point:?}");
        }
    }

    #[test]
    fn refine_point_converges_toward_ground_truth_from_a_perturbed_start() {
        let (points, pose1, pose2, intrinsics) = synthetic_scene();
        let ground_truth = points[3];

        let (u1, v1) = project(&ground_truth, &pose1, &intrinsics).unwrap();
        let (u2, v2) = project(&ground_truth, &pose2, &intrinsics).unwrap();

        let observations = vec![
            Observation {
                pose: &pose1,
                intrinsics: &intrinsics,
                pixel: PixelObservation { x: u1 as f32, y: v1 as f32 },
            },
            Observation {
                pose: &pose2,
                intrinsics: &intrinsics,
                pixel: PixelObservation { x: u2 as f32, y: v2 as f32 },
            },
        ];

        // Start from a deliberately wrong initial estimate.
        let perturbed_start = ground_truth + Vector3::new(0.3, -0.2, 0.4);
        let (refined, final_rms) = refine_point(perturbed_start, &observations, 25);

        let error = (refined - ground_truth).norm();
        assert!(error < 1e-4, "refined point error too large: {error}");
        assert!(final_rms < 1e-4, "final reprojection RMS too large: {final_rms}");
    }

    #[test]
    fn refine_point_reduces_error_even_under_noisy_observations() {
        let (points, pose1, pose2, intrinsics) = synthetic_scene();
        let ground_truth = points[7];

        let (u1, v1) = project(&ground_truth, &pose1, &intrinsics).unwrap();
        let (u2, v2) = project(&ground_truth, &pose2, &intrinsics).unwrap();

        // Add a small, deterministic pixel-space perturbation to simulate
        // real detector noise (not randomness -- reproducible).
        let observations = vec![
            Observation {
                pose: &pose1,
                intrinsics: &intrinsics,
                pixel: PixelObservation { x: u1 as f32 + 0.8, y: v1 as f32 - 0.5 },
            },
            Observation {
                pose: &pose2,
                intrinsics: &intrinsics,
                pixel: PixelObservation { x: u2 as f32 - 0.6, y: v2 as f32 + 0.7 },
            },
        ];

        let start_rms = rms_reprojection_error(&ground_truth, &observations);
        let perturbed_start = ground_truth + Vector3::new(0.5, 0.5, -0.5);
        let (refined, final_rms) = refine_point(perturbed_start, &observations, 25);

        assert!(final_rms <= start_rms + 1e-6, "BA should not make reprojection error worse");
        let error = (refined - ground_truth).norm();
        // With only 2 views and real pixel-space noise, triangulation error
        // doesn't average out the way it would with more views -- some real
        // residual 3D error is expected here, not a bug. 0.1 units against a
        // scene at ~5 units depth is still a tight bound (~2% of depth).
        assert!(error < 0.1, "refined point should land close to ground truth despite noise: {error}");
    }

    #[test]
    fn rotation_quaternion_roundtrip_preserves_rotation() {
        let r = Rotation3::from_axis_angle(&Vector3::y_axis(), 37.0_f64.to_radians());
        let q = rotation_to_quaternion(&r);
        let r2 = quaternion_to_rotation(q);
        let error = (r.matrix() - r2.matrix()).norm();
        assert!(error < 1e-5, "quaternion roundtrip error too large: {error}");
    }

    #[test]
    fn identity_rotation_roundtrip() {
        let r = Rotation3::identity();
        let q = rotation_to_quaternion(&r);
        assert!((q.3 - 1.0).abs() < 1e-6); // qw ~= 1 for identity
        let r2 = quaternion_to_rotation(q);
        assert!((r.matrix() - r2.matrix()).norm() < 1e-6);
    }

    #[test]
    fn estimate_fundamental_matrix_rejects_too_few_points() {
        let points = vec![(0.0, 0.0); 5];
        let result = estimate_fundamental_matrix(&points, &points);
        assert!(result.is_err());
    }
}
