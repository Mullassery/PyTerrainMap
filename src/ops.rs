//! Allowlisted JSON operation bridge for Rust-only advanced capabilities.
//!
//! This is deliberately a small, file-oriented surface: each operation takes
//! one JSON payload string and returns one JSON string. File-producing
//! operations require explicit input/output paths and can be confined to an
//! optional `output_root` directory, so Python callers do not need to link
//! against every internal Rust type just to reach these features.

use crate::elevation::srtm::SrtmTileProvider;
use crate::photogrammetry::{CameraPoseEstimate, FeatureKeypoint, StructureFromMotion};
use crate::reference_images::{ReferenceImage, VisualDescriptor};
use crate::slam::{CameraFrame, DepthMeasurement, IMUReading, SLAMTracker, VisualFeature};
use crate::tiles_3d::{write_pnts, BoundingVolume, Content, PointCloudPoint, TilesExporter};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpErrorKind {
    UnknownOp,
    InvalidPayload,
    NotFound,
    Forbidden,
    Internal,
}

#[derive(Debug)]
pub struct OpError {
    pub kind: OpErrorKind,
    pub message: String,
}

impl OpError {
    fn new(kind: OpErrorKind, message: impl Into<String>) -> Self {
        OpError {
            kind,
            message: message.into(),
        }
    }

    fn unknown(op: &str) -> Self {
        Self::new(
            OpErrorKind::UnknownOp,
            format!("unknown op '{op}' (call ops() for the allowlist)"),
        )
    }

    fn invalid(message: impl Into<String>) -> Self {
        Self::new(OpErrorKind::InvalidPayload, message)
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self::new(OpErrorKind::NotFound, message)
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self::new(OpErrorKind::Forbidden, message)
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::new(OpErrorKind::Internal, message)
    }
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OpError {}

pub const OP_NAMES: &[&str] = &[
    "photogrammetry.reconstruct",
    "slam.track",
    "cesium.export_tileset",
    "elevation.srtm_lookup",
];

/// Return the allowlisted operation names, descriptions, and JSON schemas.
pub fn ops_json() -> String {
    let operations = json!([
        {
            "name": "photogrammetry.reconstruct",
            "description": "Run real multi-view SfM geometry (feature matching, pose recovery, DLT triangulation, optional structure-only bundle adjustment) on caller-supplied image keypoints. Reads a scene JSON file and writes a point-cloud/camera-pose JSON file.",
            "input_schema": {
                "type": "object",
                "required": ["input_path", "output_path"],
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "Path to a scene JSON file containing an 'images' array. Each image has id, optional source/focal_length/principal_point, and a keypoints array of {x,y,color:[r,g,b],descriptor:[f32,...]}."
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path for the reconstructed point-cloud/camera-pose JSON."
                    },
                    "output_root": {
                        "type": "string",
                        "description": "Optional existing directory that output_path must resolve inside."
                    }
                }
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "op": {"type": "string"},
                    "status": {"type": "string", "const": "ok"},
                    "output_path": {"type": "string"},
                    "image_count": {"type": "integer"},
                    "matched_pairs": {"type": "integer"},
                    "triangulated_points": {"type": "integer"},
                    "point_count": {"type": "integer"},
                    "pair_errors": {"type": "array", "items": {"type": "string"}}
                }
            }
        },
        {
            "name": "slam.track",
            "description": "Track an incrementally built SLAM pose graph from caller-supplied visual features (plus optional IMU and depth measurements), including the real loop-closure detector. Reads a frame JSON file and writes a keyframe/trajectory JSON file.",
            "input_schema": {
                "type": "object",
                "required": ["input_path", "output_path"],
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "Path to a JSON file containing a 'frames' array of {frame_id,timestamp,features:[{position:[u,v],descriptor:[u8,...],point_3d:[x,y,z]?}]}, optional 'imu' and 'depth' arrays, and optional 'enable_loop_closure' (default true)."
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Path for the trajectory/keyframe JSON."
                    },
                    "output_root": {
                        "type": "string",
                        "description": "Optional existing directory that output_path must resolve inside."
                    }
                }
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "op": {"type": "string"},
                    "status": {"type": "string", "const": "ok"},
                    "output_path": {"type": "string"},
                    "keyframe_count": {"type": "integer"},
                    "loop_closures": {"type": "integer"},
                    "imu_readings_buffered": {"type": "integer"},
                    "depth_measurements_buffered": {"type": "integer"}
                }
            }
        },
        {
            "name": "cesium.export_tileset",
            "description": "Export a point cloud as a real 3D Tiles 1.0 tileset (tileset.json + quantized root.pnts) for Cesium. Reads a point-cloud JSON file (or a photogrammetry.reconstruct output) and writes a tileset directory.",
            "input_schema": {
                "type": "object",
                "required": ["input_path", "output_path"],
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "Path to JSON with 'points' (or a photogrammetry output with point_cloud.points). Each point needs position [x,y,z] and optional color [r,g,b]."
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Directory to write tileset.json and root.pnts into."
                    },
                    "output_root": {
                        "type": "string",
                        "description": "Optional existing directory that output_path must resolve inside."
                    },
                    "tileset_name": {
                        "type": "string",
                        "description": "Optional tileset display name."
                    }
                }
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "op": {"type": "string"},
                    "status": {"type": "string", "const": "ok"},
                    "output_path": {"type": "string"},
                    "tileset_path": {"type": "string"},
                    "pnts_path": {"type": "string"},
                    "point_count": {"type": "integer"}
                }
            }
        },
        {
            "name": "elevation.srtm_lookup",
            "description": "Look up elevation from local SRTM .hgt tiles for a WGS84 latitude/longitude, using bilinear interpolation over the real big-endian i16 raster.",
            "input_schema": {
                "type": "object",
                "required": ["tile_dir", "lat", "lon"],
                "properties": {
                    "tile_dir": {
                        "type": "string",
                        "description": "Directory containing SRTM .hgt tiles (for example N40W075.hgt)."
                    },
                    "lat": {"type": "number", "minimum": -90, "exclusiveMaximum": 90},
                    "lon": {"type": "number", "minimum": -180, "exclusiveMaximum": 180}
                }
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "op": {"type": "string"},
                    "status": {"type": "string", "const": "ok"},
                    "lat": {"type": "number"},
                    "lon": {"type": "number"},
                    "tile": {"type": "string"},
                    "elevation_m": {"type": "number"}
                }
            }
        }
    ]);

    serde_json::to_string_pretty(&operations)
        .expect("the static operation schema is always serializable")
}

/// Execute one allowlisted operation with a JSON payload, returning JSON.
pub fn run_op(op: &str, payload_json: &str) -> Result<String, OpError> {
    if !OP_NAMES.contains(&op) {
        return Err(OpError::unknown(op));
    }

    let payload: Value = serde_json::from_str(payload_json)
        .map_err(|e| OpError::invalid(format!("payload is not valid JSON: {e}")))?;
    let object = payload
        .as_object()
        .ok_or_else(|| OpError::invalid("payload must be a JSON object"))?;

    match op {
        "photogrammetry.reconstruct" => photogrammetry_reconstruct(object),
        "slam.track" => slam_track(object),
        "cesium.export_tileset" => cesium_export_tileset(object),
        "elevation.srtm_lookup" => elevation_srtm_lookup(object),
        _ => Err(OpError::unknown(op)),
    }
}

fn required_str<'a>(payload: &'a Map<String, Value>, key: &str) -> Result<&'a str, OpError> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| OpError::invalid(format!("missing or empty string field '{key}'")))
}

fn optional_str<'a>(payload: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(Value::as_str)
}

fn required_f64(payload: &Map<String, Value>, key: &str) -> Result<f64, OpError> {
    payload
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| OpError::invalid(format!("missing or non-numeric field '{key}'")))
}

fn absolute_path(path: &Path) -> Result<PathBuf, OpError> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| OpError::internal(format!("cannot resolve current directory: {e}")))?
            .join(path)
    };
    Ok(normalize(&path))
}

fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn canonicalize_allow_missing(path: &Path) -> Result<PathBuf, OpError> {
    let mut existing = path.to_path_buf();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| OpError::invalid(format!("cannot resolve path {}", path.display())))?
            .to_os_string();
        missing.push(name);
        existing = existing
            .parent()
            .ok_or_else(|| OpError::invalid(format!("cannot resolve path {}", path.display())))?
            .to_path_buf();
    }
    let mut resolved = fs::canonicalize(&existing)
        .map_err(|e| OpError::internal(format!("cannot resolve {}: {e}", existing.display())))?;
    for name in missing.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn resolve_input_file(path: &str) -> Result<PathBuf, OpError> {
    let absolute = absolute_path(Path::new(path))?;
    if !absolute.exists() {
        return Err(OpError::not_found(format!(
            "input file not found: {}",
            absolute.display()
        )));
    }
    let canonical = fs::canonicalize(&absolute).map_err(|e| {
        OpError::not_found(format!(
            "cannot open input file {}: {e}",
            absolute.display()
        ))
    })?;
    if !canonical.is_file() {
        return Err(OpError::invalid(format!(
            "input path is not a regular file: {}",
            canonical.display()
        )));
    }
    Ok(canonical)
}

fn resolve_output_path(payload: &Map<String, Value>, output_key: &str) -> Result<PathBuf, OpError> {
    let output = absolute_path(Path::new(required_str(payload, output_key)?))?;

    if let Ok(metadata) = fs::symlink_metadata(&output) {
        if metadata.file_type().is_symlink() {
            return Err(OpError::forbidden(format!(
                "output path must not be a symlink: {}",
                output.display()
            )));
        }
    }

    if let Some(root) = optional_str(payload, "output_root") {
        let root = absolute_path(Path::new(root))?;
        let root = fs::canonicalize(&root).map_err(|e| {
            OpError::not_found(format!(
                "output_root is not accessible: {}: {e}",
                root.display()
            ))
        })?;
        if !root.is_dir() {
            return Err(OpError::invalid(format!(
                "output_root is not a directory: {}",
                root.display()
            )));
        }
        let candidate = canonicalize_allow_missing(&output)?;
        if !candidate.starts_with(&root) {
            return Err(OpError::forbidden(format!(
                "output path {} escapes output_root {}",
                output.display(),
                root.display()
            )));
        }
    }

    Ok(output)
}

fn ensure_parent(path: &Path) -> Result<(), OpError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                OpError::internal(format!(
                    "cannot create output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn read_json(path: &Path) -> Result<Value, OpError> {
    let bytes = fs::read(path)
        .map_err(|e| OpError::not_found(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| OpError::invalid(format!("invalid JSON in {}: {e}", path.display())))
}

#[derive(Deserialize)]
struct PhotogrammetryScene {
    images: Vec<SceneImage>,
    #[serde(default)]
    pairs: Option<Vec<[String; 2]>>,
    #[serde(default)]
    bundle_adjustment_iterations: Option<usize>,
}

#[derive(Deserialize)]
struct SceneImage {
    id: String,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    focal_length: Option<f32>,
    #[serde(default)]
    principal_point: Option<[f32; 2]>,
    keypoints: Vec<FeatureKeypoint>,
}

fn photogrammetry_reconstruct(payload: &Map<String, Value>) -> Result<String, OpError> {
    let input_path = resolve_input_file(required_str(payload, "input_path")?)?;
    let output_path = resolve_output_path(payload, "output_path")?;
    let scene_value = read_json(&input_path)?;
    let scene: PhotogrammetryScene = serde_json::from_value(scene_value).map_err(|e| {
        OpError::invalid(format!(
            "invalid photogrammetry scene in {}: {e}",
            input_path.display()
        ))
    })?;

    if scene.images.len() < 2 {
        return Err(OpError::invalid(
            "photogrammetry.reconstruct needs at least 2 images",
        ));
    }

    let mut ids = HashSet::new();
    for image in &scene.images {
        if image.id.trim().is_empty() {
            return Err(OpError::invalid("every image needs a non-empty id"));
        }
        if !ids.insert(image.id.clone()) {
            return Err(OpError::invalid(format!(
                "duplicate image id '{}'",
                image.id
            )));
        }
        if image.keypoints.is_empty() {
            return Err(OpError::invalid(format!(
                "image '{}' has no keypoints",
                image.id
            )));
        }
    }

    let mut sfm = StructureFromMotion::new();
    for image in &scene.images {
        let source = image.source.clone().unwrap_or_else(|| image.id.clone());
        let descriptor = VisualDescriptor::new("caller-supplied", &image.id);
        let reference = ReferenceImage::ungeoreferenced(&source, descriptor);
        sfm.add_image(image.id.clone(), reference);

        if let Some(pose) = sfm.camera_poses.get_mut(&image.id) {
            if let Some(focal_length) = image.focal_length {
                pose.focal_length = focal_length;
            }
            if let Some(principal_point) = image.principal_point {
                pose.principal_point = (principal_point[0], principal_point[1]);
            }
        }

        sfm.set_image_keypoints(&image.id, image.keypoints.clone())
            .map_err(|e| OpError::invalid(format!("image '{}': {e}", image.id)))?;
    }

    let image_ids: Vec<String> = scene.images.iter().map(|image| image.id.clone()).collect();
    let pairs: Vec<(String, String)> = match &scene.pairs {
        Some(pairs) => {
            for pair in pairs {
                for image_id in pair {
                    if !ids.contains(image_id) {
                        return Err(OpError::invalid(format!(
                            "pair references unknown image id '{image_id}'"
                        )));
                    }
                }
            }
            pairs
                .iter()
                .map(|pair| (pair[0].clone(), pair[1].clone()))
                .collect()
        }
        None => image_ids
            .windows(2)
            .map(|pair| (pair[0].clone(), pair[1].clone()))
            .collect(),
    };

    let mut pair_errors = Vec::new();
    for (first, second) in pairs {
        match sfm.match_image_pair(&first, &second) {
            Ok(pair) => {
                if let Err(e) = sfm.triangulate(&pair) {
                    pair_errors.push(format!("{first} <-> {second}: {e}"));
                }
                sfm.image_pairs.push(pair);
            }
            Err(e) => pair_errors.push(format!("{first} <-> {second}: {e}")),
        }
    }

    let iterations = scene.bundle_adjustment_iterations.unwrap_or(10);
    if iterations > 0 && !sfm.triangulated_points.is_empty() {
        sfm.bundle_adjustment(iterations)
            .map_err(|e| OpError::internal(format!("bundle adjustment failed: {e}")))?;
    }

    let cloud = sfm.to_dense_point_cloud();
    if cloud.points.is_empty() {
        return Err(OpError::invalid(format!(
            "no points were reconstructed; pair errors: {}",
            if pair_errors.is_empty() {
                "none".to_string()
            } else {
                pair_errors.join("; ")
            }
        )));
    }

    let camera_poses: Vec<Value> = scene
        .images
        .iter()
        .filter_map(|image| {
            sfm.camera_poses
                .get(&image.id)
                .map(|pose| camera_pose_json(&image.id, pose))
        })
        .collect();

    let result = json!({
        "op": "photogrammetry.reconstruct",
        "status": "ok",
        "image_count": sfm.image_count(),
        "matched_pairs": sfm.image_pairs.len(),
        "triangulated_points": sfm.point_count(),
        "bundle_adjustment_iterations": iterations,
        "pair_errors": pair_errors,
        "camera_poses": camera_poses,
        "point_cloud": serde_json::to_value(&cloud)
            .expect("dense point cloud is always serializable"),
    });

    ensure_parent(&output_path)?;
    fs::write(
        &output_path,
        serde_json::to_vec_pretty(&result).expect("JSON result"),
    )
    .map_err(|e| OpError::internal(format!("cannot write {}: {e}", output_path.display())))?;

    Ok(serde_json::to_string_pretty(&json!({
        "op": "photogrammetry.reconstruct",
        "status": "ok",
        "output_path": output_path.to_string_lossy(),
        "image_count": sfm.image_count(),
        "matched_pairs": sfm.image_pairs.len(),
        "triangulated_points": sfm.point_count(),
        "point_count": cloud.statistics.point_count,
        "pair_errors": pair_errors,
    }))
    .expect("JSON response"))
}

fn camera_pose_json(image_id: &str, pose: &CameraPoseEstimate) -> Value {
    json!({
        "image_id": image_id,
        "position": [pose.position.0, pose.position.1, pose.position.2],
        "rotation": [pose.rotation.0, pose.rotation.1, pose.rotation.2, pose.rotation.3],
        "focal_length": pose.focal_length,
        "principal_point": [pose.principal_point.0, pose.principal_point.1],
        "confidence": pose.confidence,
    })
}

#[derive(Deserialize)]
struct SlamInput {
    frames: Vec<SlamFrameInput>,
    #[serde(default)]
    imu: Vec<IMUReading>,
    #[serde(default)]
    depth: Vec<DepthMeasurement>,
    #[serde(default = "default_true")]
    enable_loop_closure: bool,
}

#[derive(Deserialize)]
struct SlamFrameInput {
    frame_id: u32,
    timestamp: i64,
    features: Vec<VisualFeature>,
}

fn default_true() -> bool {
    true
}

fn slam_track(payload: &Map<String, Value>) -> Result<String, OpError> {
    let input_path = resolve_input_file(required_str(payload, "input_path")?)?;
    let output_path = resolve_output_path(payload, "output_path")?;
    let input: SlamInput = serde_json::from_value(read_json(&input_path)?).map_err(|e| {
        OpError::invalid(format!(
            "invalid SLAM frame file {}: {e}",
            input_path.display()
        ))
    })?;

    if input.frames.is_empty() {
        return Err(OpError::invalid("slam.track needs at least one frame"));
    }

    let mut tracker = SLAMTracker::new();
    tracker.enable_loop_closure = input.enable_loop_closure;
    for reading in input.imu {
        tracker.add_imu_reading(reading);
    }
    for measurement in input.depth {
        tracker.add_depth(measurement);
    }

    for frame in input.frames {
        let mut camera_frame = CameraFrame::new(frame.frame_id, frame.timestamp);
        camera_frame.features = frame.features;
        tracker
            .process_frame(camera_frame)
            .map_err(|e| OpError::invalid(format!("frame {}: {e}", frame.frame_id)))?;
    }

    let stats = tracker.statistics();
    let keyframes: Vec<Value> = tracker
        .pose_graph
        .keyframes
        .iter()
        .map(|frame| {
            json!({
                "frame_id": frame.frame_id,
                "timestamp": frame.timestamp,
                "feature_count": frame.features.len(),
                "pose": {
                    "position": [frame.pose.position.0, frame.pose.position.1, frame.pose.position.2],
                    "rotation": [frame.pose.rotation.0, frame.pose.rotation.1, frame.pose.rotation.2, frame.pose.rotation.3],
                    "covariance": frame.pose.covariance,
                }
            })
        })
        .collect();
    let loop_closures: Vec<Value> = tracker
        .pose_graph
        .loop_closures
        .iter()
        .map(|closure| {
            json!({
                "current_id": closure.current_id,
                "previous_id": closure.previous_id,
                "similarity": closure.similarity,
                "timestamp": closure.timestamp,
            })
        })
        .collect();

    let result = json!({
        "op": "slam.track",
        "status": "ok",
        "statistics": serde_json::to_value(&stats).expect("SLAM stats are serializable"),
        "keyframes": keyframes,
        "loop_closures": loop_closures,
    });

    ensure_parent(&output_path)?;
    fs::write(
        &output_path,
        serde_json::to_vec_pretty(&result).expect("JSON result"),
    )
    .map_err(|e| OpError::internal(format!("cannot write {}: {e}", output_path.display())))?;

    Ok(serde_json::to_string_pretty(&json!({
        "op": "slam.track",
        "status": "ok",
        "output_path": output_path.to_string_lossy(),
        "keyframe_count": stats.keyframe_count,
        "loop_closures": stats.loop_closures,
        "imu_readings_buffered": stats.imu_readings_buffered,
        "depth_measurements_buffered": stats.depth_measurements_buffered,
    }))
    .expect("JSON response"))
}

fn cesium_export_tileset(payload: &Map<String, Value>) -> Result<String, OpError> {
    let input_path = resolve_input_file(required_str(payload, "input_path")?)?;
    let output_dir = resolve_output_path(payload, "output_path")?;
    if output_dir.exists() && !output_dir.is_dir() {
        return Err(OpError::invalid(format!(
            "cesium.export_tileset output_path is not a directory: {}",
            output_dir.display()
        )));
    }
    let input = read_json(&input_path)?;
    let points = parse_point_cloud(&input)?;
    if points.is_empty() {
        return Err(OpError::invalid("point cloud contains no points"));
    }

    fs::create_dir_all(&output_dir).map_err(|e| {
        OpError::internal(format!(
            "cannot create tileset directory {}: {e}",
            output_dir.display()
        ))
    })?;

    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for point in &points {
        for axis in 0..3 {
            let value = point.position[axis] as f64;
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    let center = [
        (min[0] + max[0]) / 2.0,
        (min[1] + max[1]) / 2.0,
        (min[2] + max[2]) / 2.0,
    ];
    let half = [
        ((max[0] - min[0]) / 2.0).max(0.001),
        ((max[1] - min[1]) / 2.0).max(0.001),
        ((max[2] - min[2]) / 2.0).max(0.001),
    ];

    let pnts_path = output_dir.join("root.pnts");
    write_pnts(&pnts_path, &points)
        .map_err(|e| OpError::internal(format!("cannot write {}: {e}", pnts_path.display())))?;

    let name = optional_str(payload, "tileset_name").unwrap_or("PyTerrainMap Point Cloud");
    let mut exporter = TilesExporter::new(
        BoundingVolume::box_region(
            center[0] as f32,
            center[1] as f32,
            center[2] as f32,
            half[0] as f32,
            half[1] as f32,
            half[2] as f32,
        ),
        points.len() as u32,
    );
    exporter.geometric_error_base = 10.0;
    let mut tileset = exporter.create_tileset(name);
    tileset.root.content = Some(Content {
        uri: "root.pnts".to_string(),
    });
    tileset.asset.generator = format!("PyTerrainMap {}", env!("CARGO_PKG_VERSION"));

    let tileset_path = output_dir.join("tileset.json");
    fs::write(
        &tileset_path,
        serde_json::to_vec_pretty(&tileset).expect("tileset is serializable"),
    )
    .map_err(|e| OpError::internal(format!("cannot write {}: {e}", tileset_path.display())))?;

    Ok(serde_json::to_string_pretty(&json!({
        "op": "cesium.export_tileset",
        "status": "ok",
        "output_path": output_dir.to_string_lossy(),
        "tileset_path": tileset_path.to_string_lossy(),
        "pnts_path": pnts_path.to_string_lossy(),
        "point_count": points.len(),
    }))
    .expect("JSON response"))
}

fn parse_point_cloud(value: &Value) -> Result<Vec<PointCloudPoint>, OpError> {
    let points = value
        .get("points")
        .or_else(|| {
            value
                .get("point_cloud")
                .and_then(|cloud| cloud.get("points"))
        })
        .or_else(|| value.as_array().map(|_| value))
        .and_then(Value::as_array)
        .ok_or_else(|| {
            OpError::invalid("point cloud JSON needs a 'points' array (or point_cloud.points)")
        })?;

    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let position = extract_position(point).ok_or_else(|| {
                OpError::invalid(format!(
                    "point {index} needs position [x,y,z] (or x/y/z fields)"
                ))
            })?;
            let color = extract_color(point).unwrap_or([200, 200, 200]);
            Ok(PointCloudPoint::new(position, color))
        })
        .collect()
}

fn extract_position(point: &Value) -> Option<[f32; 3]> {
    if let Some(position) = point.get("position") {
        if let Some(array) = extract_vec3(position) {
            return Some(array);
        }
        if let Some(object) = position.as_object() {
            return Some([
                object.get("x")?.as_f64()? as f32,
                object.get("y")?.as_f64()? as f32,
                object.get("z")?.as_f64()? as f32,
            ]);
        }
    }
    if let Some(array) = extract_vec3(point) {
        return Some(array);
    }
    Some([
        point.get("x")?.as_f64()? as f32,
        point.get("y")?.as_f64()? as f32,
        point.get("z")?.as_f64()? as f32,
    ])
}

fn extract_color(point: &Value) -> Option<[u8; 3]> {
    if let Some(color) = point.get("color") {
        if let Some(array) = color.as_array() {
            if array.len() >= 3 {
                return Some([
                    array[0].as_f64()? as u8,
                    array[1].as_f64()? as u8,
                    array[2].as_f64()? as u8,
                ]);
            }
        }
        if let Some(object) = color.as_object() {
            return Some([
                object.get("r")?.as_f64()? as u8,
                object.get("g")?.as_f64()? as u8,
                object.get("b")?.as_f64()? as u8,
            ]);
        }
    }
    if let Some(rgb) = point.get("rgb").and_then(Value::as_array) {
        if rgb.len() >= 3 {
            return Some([
                rgb[0].as_f64()? as u8,
                rgb[1].as_f64()? as u8,
                rgb[2].as_f64()? as u8,
            ]);
        }
    }
    Some([
        point.get("r")?.as_f64()? as u8,
        point.get("g")?.as_f64()? as u8,
        point.get("b")?.as_f64()? as u8,
    ])
}

fn extract_vec3(value: &Value) -> Option<[f32; 3]> {
    let array = value.as_array()?;
    if array.len() < 3 {
        return None;
    }
    Some([
        array[0].as_f64()? as f32,
        array[1].as_f64()? as f32,
        array[2].as_f64()? as f32,
    ])
}

fn elevation_srtm_lookup(payload: &Map<String, Value>) -> Result<String, OpError> {
    let tile_dir = absolute_path(Path::new(required_str(payload, "tile_dir")?))?;
    let tile_dir = fs::canonicalize(&tile_dir).map_err(|e| {
        OpError::not_found(format!(
            "SRTM tile directory is not accessible: {}: {e}",
            tile_dir.display()
        ))
    })?;
    if !tile_dir.is_dir() {
        return Err(OpError::invalid(format!(
            "SRTM tile_dir is not a directory: {}",
            tile_dir.display()
        )));
    }

    let lat = required_f64(payload, "lat")?;
    let lon = required_f64(payload, "lon")?;
    let tile =
        SrtmTileProvider::tile_file_name(lat, lon).map_err(|e| OpError::invalid(e.to_string()))?;
    let tile_path = tile_dir.join(&tile);
    if !tile_path.is_file() {
        return Err(OpError::not_found(format!(
            "SRTM tile not found: {}",
            tile_path.display()
        )));
    }

    let provider = SrtmTileProvider::new(&tile_dir);
    let elevation = provider
        .elevation(lat, lon)
        .map_err(|e| OpError::internal(e.to_string()))?;

    Ok(serde_json::to_string_pretty(&json!({
        "op": "elevation.srtm_lookup",
        "status": "ok",
        "lat": lat,
        "lon": lon,
        "tile": tile,
        "tile_path": tile_path.to_string_lossy(),
        "elevation_m": elevation,
    }))
    .expect("JSON response"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ptm_ops_{name}_{}_{}",
                std::process::id(),
                unique
            ));
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write_json(&self, name: &str, value: &Value) -> PathBuf {
            let path = self.0.join(name);
            fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
            path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn synthetic_scene() -> Value {
        let points = [
            [-1.0, 0.5, 5.0],
            [0.0, 1.0, 5.5],
            [1.0, -0.5, 6.0],
            [0.5, 0.0, 4.5],
            [-0.5, 0.5, 5.2],
            [0.2, -0.8, 6.1],
            [0.8, 0.9, 4.8],
            [-0.9, -0.4, 5.7],
            [0.1, 0.3, 5.1],
            [0.6, 0.6, 5.9],
        ];
        let make_keypoints = |yaw_deg: f64| {
            points
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    let rotation = yaw_deg.to_radians();
                    let x = point[0] * rotation.cos() - point[2] * rotation.sin() + 0.2;
                    let y = point[1];
                    let z = point[0] * rotation.sin() + point[2] * rotation.cos();
                    let u = 640.0 + 1000.0 * x / z;
                    let v = 360.0 + 1000.0 * y / z;
                    let descriptor = index as f32;
                    json!({
                        "x": u,
                        "y": v,
                        "color": [100, 120, 140],
                        "descriptor": [descriptor * 7.0, descriptor * 3.0 + 1.0, descriptor * 1.5],
                    })
                })
                .collect::<Vec<_>>()
        };

        json!({
            "images": [
                {
                    "id": "cam1",
                    "source": "cam1.jpg",
                    "focal_length": 1000.0,
                    "principal_point": [640.0, 360.0],
                    "keypoints": make_keypoints(0.0),
                },
                {
                    "id": "cam2",
                    "source": "cam2.jpg",
                    "focal_length": 1000.0,
                    "principal_point": [640.0, 360.0],
                    "keypoints": make_keypoints(10.0),
                }
            ],
            "bundle_adjustment_iterations": 2
        })
    }

    #[test]
    fn ops_json_lists_all_allowlisted_operations() {
        let specs: Value = serde_json::from_str(&ops_json()).unwrap();
        let names: Vec<&str> = specs
            .as_array()
            .unwrap()
            .iter()
            .map(|spec| spec["name"].as_str().unwrap())
            .collect();
        for expected in OP_NAMES {
            assert!(names.contains(expected), "missing {expected}");
        }
        for spec in specs.as_array().unwrap() {
            assert!(spec["description"].as_str().unwrap().len() > 20);
            assert!(spec["input_schema"].is_object());
            assert!(spec["output_schema"].is_object());
        }
    }

    #[test]
    fn photogrammetry_op_writes_real_point_cloud() {
        let dir = TempDir::new("photogrammetry");
        let input = dir.write_json("scene.json", &synthetic_scene());
        let output = dir.path().join("cloud.json");

        let response = run_op(
            "photogrammetry.reconstruct",
            &json!({
                "input_path": input,
                "output_path": output,
                "output_root": dir.path(),
            })
            .to_string(),
        )
        .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["status"], "ok");
        assert_eq!(response["image_count"], 2);
        assert_eq!(response["point_count"], 10);

        let cloud: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert_eq!(cloud["point_cloud"]["points"].as_array().unwrap().len(), 10);
        assert_eq!(cloud["camera_poses"].as_array().unwrap().len(), 2);
        assert_eq!(cloud["triangulated_points"], 10);
    }

    #[test]
    fn photogrammetry_op_reports_missing_input_file() {
        let dir = TempDir::new("missing_input");
        let err = run_op(
            "photogrammetry.reconstruct",
            &json!({
                "input_path": dir.path().join("nope.json"),
                "output_path": dir.path().join("out.json"),
            })
            .to_string(),
        )
        .unwrap_err();
        assert_eq!(err.kind, OpErrorKind::NotFound);
    }

    #[test]
    fn output_root_rejects_escaping_output_path() {
        let dir = TempDir::new("output_root");
        let root = dir.path().join("root");
        fs::create_dir_all(&root).unwrap();
        let input = dir.write_json("scene.json", &synthetic_scene());

        let err = run_op(
            "photogrammetry.reconstruct",
            &json!({
                "input_path": input,
                "output_path": dir.path().join("escaped.json"),
                "output_root": root,
            })
            .to_string(),
        )
        .unwrap_err();
        assert_eq!(err.kind, OpErrorKind::Forbidden);
    }

    #[test]
    fn slam_op_writes_keyframes_and_statistics() {
        let dir = TempDir::new("slam");
        let frames = (0..3)
            .map(|index| {
                json!({
                    "frame_id": index,
                    "timestamp": 1000 + index as i64 * 100,
                    "features": [
                        {
                            "position": [100.0 + index as f32, 200.0],
                            "descriptor": [255, 128, 64],
                            "point_3d": [index as f32, 0.0, 5.0]
                        }
                    ]
                })
            })
            .collect::<Vec<_>>();
        let input = dir.write_json(
            "frames.json",
            &json!({
                "frames": frames,
                "imu": [{
                    "timestamp": 1000,
                    "acceleration": [0.0, 0.0, 9.8],
                    "angular_velocity": [0.01, 0.02, 0.03],
                    "magnetic_field": null
                }],
            }),
        );
        let output = dir.path().join("trajectory.json");

        let response = run_op(
            "slam.track",
            &json!({"input_path": input, "output_path": output}).to_string(),
        )
        .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["status"], "ok");
        assert_eq!(response["keyframe_count"], 3);
        assert_eq!(response["imu_readings_buffered"], 1);

        let trajectory: Value = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
        assert_eq!(trajectory["keyframes"].as_array().unwrap().len(), 3);
        assert_eq!(trajectory["statistics"]["keyframe_count"], 3);
    }

    #[test]
    fn cesium_op_writes_tileset_and_real_pnts() {
        let dir = TempDir::new("cesium");
        let input = dir.write_json(
            "points.json",
            &json!({
                "points": [
                    {"position": [0.0, 0.0, 0.0], "color": [255, 0, 0]},
                    {"position": [1.0, 2.0, 3.0], "color": [0, 255, 0]},
                    {"position": [-1.0, -2.0, -3.0], "color": [0, 0, 255]}
                ]
            }),
        );
        let output = dir.path().join("tiles");

        let response = run_op(
            "cesium.export_tileset",
            &json!({"input_path": input, "output_path": output}).to_string(),
        )
        .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["point_count"], 3);

        let tileset: Value =
            serde_json::from_slice(&fs::read(output.join("tileset.json")).unwrap()).unwrap();
        assert_eq!(tileset["root"]["content"]["uri"], "root.pnts");

        let pnts = fs::read(output.join("root.pnts")).unwrap();
        assert_eq!(&pnts[0..4], b"pnts");
        let byte_length = u32::from_le_bytes(pnts[8..12].try_into().unwrap()) as usize;
        assert_eq!(byte_length, pnts.len());
    }

    #[test]
    fn srtm_op_reads_local_tile() {
        let dir = TempDir::new("srtm");
        let values: [i16; 9] = [30, 60, 90, 10, 20, 40, 0, 5, 15];
        let mut bytes = Vec::new();
        for value in values {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        fs::write(dir.path().join("N10E020.hgt"), bytes).unwrap();

        let response = run_op(
            "elevation.srtm_lookup",
            &json!({"tile_dir": dir.path(), "lat": 10.5, "lon": 20.5}).to_string(),
        )
        .unwrap();
        let response: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response["status"], "ok");
        assert_eq!(response["tile"], "N10E020.hgt");
        assert_eq!(response["elevation_m"], 20.0);
    }

    #[test]
    fn srtm_op_reports_missing_tile() {
        let dir = TempDir::new("srtm_missing");
        let err = run_op(
            "elevation.srtm_lookup",
            &json!({"tile_dir": dir.path(), "lat": 10.5, "lon": 20.5}).to_string(),
        )
        .unwrap_err();
        assert_eq!(err.kind, OpErrorKind::NotFound);
    }

    #[test]
    fn malformed_json_and_unknown_ops_are_rejected() {
        let malformed = run_op("photogrammetry.reconstruct", "{not json").unwrap_err();
        assert_eq!(malformed.kind, OpErrorKind::InvalidPayload);

        let unknown = run_op("not.an.op", "{}").unwrap_err();
        assert_eq!(unknown.kind, OpErrorKind::UnknownOp);
    }
}
