//! Reference image management for ungeoreferenced imagery
//!
//! Stores reference images without coordinates, matches them against
//! robot observations to enable lazy georeferencing and multi-perspective alignment.

use crate::types::{GeoPoint, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Georeference status of a reference image
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeoreferenceStatus {
    /// Image location unknown, waiting for matching
    Ungeoreferenced,
    /// Location predicted from matching, low confidence
    PredictedLow,
    /// Location predicted from matching, medium confidence
    PredictedMedium,
    /// Location confirmed by robot visit
    Confirmed,
    /// Location refined by multiple matches
    Refined,
}

/// Image orientation information
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageOrientation {
    /// Heading/azimuth in degrees (0-360), if known
    pub heading_degrees: Option<f32>,
    /// Pitch/elevation in degrees (-90 to +90), if known
    pub pitch_degrees: Option<f32>,
    /// Roll/rotation in degrees (0-360), if known
    pub roll_degrees: Option<f32>,
    /// Camera field of view in degrees
    pub fov_degrees: Option<f32>,
}

impl ImageOrientation {
    /// Create orientation with unknown direction
    pub fn unknown() -> Self {
        ImageOrientation {
            heading_degrees: None,
            pitch_degrees: None,
            roll_degrees: None,
            fov_degrees: None,
        }
    }

    /// Create orientation with known heading
    pub fn with_heading(heading: f32) -> Self {
        ImageOrientation {
            heading_degrees: Some(heading % 360.0),
            pitch_degrees: None,
            roll_degrees: None,
            fov_degrees: None,
        }
    }

    pub fn is_fully_known(&self) -> bool {
        self.heading_degrees.is_some()
            && self.pitch_degrees.is_some()
            && self.roll_degrees.is_some()
    }
}

/// Visual descriptor for image matching
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualDescriptor {
    /// Feature detection method (SIFT, SURF, ORB, etc.)
    pub method: String,
    /// Number of keypoints detected
    pub keypoint_count: u32,
    /// Hash/fingerprint for quick similarity (perceptual hash)
    pub perceptual_hash: String,
    /// Descriptor vector (compact representation)
    pub descriptor_vector: Vec<f32>,
    /// Feature coordinates (simplified: count per quadrant)
    pub feature_density: Vec<u8>, // [top-left, top-right, bottom-left, bottom-right]
}

impl VisualDescriptor {
    /// Create descriptor with perceptual hash
    pub fn new(method: &str, perceptual_hash: &str) -> Self {
        VisualDescriptor {
            method: method.to_string(),
            keypoint_count: 0,
            perceptual_hash: perceptual_hash.to_string(),
            descriptor_vector: Vec::new(),
            feature_density: vec![0, 0, 0, 0],
        }
    }

    /// Compute hamming distance to another descriptor (quick similarity check)
    pub fn hamming_distance(&self, other: &VisualDescriptor) -> u32 {
        let mut distance = 0;
        let max_len = self.perceptual_hash.len().min(other.perceptual_hash.len());

        for i in 0..max_len {
            let a = self.perceptual_hash.chars().nth(i).unwrap_or('0');
            let b = other.perceptual_hash.chars().nth(i).unwrap_or('0');
            if a != b {
                distance += 1;
            }
        }

        distance
    }

    /// Quick similarity score (0.0-1.0) based on perceptual hash
    pub fn similarity_score(&self, other: &VisualDescriptor) -> f32 {
        if self.perceptual_hash.is_empty() || other.perceptual_hash.is_empty() {
            return 0.0;
        }

        let distance = self.hamming_distance(other);
        let max_distance = self.perceptual_hash.len() as u32;

        if max_distance == 0 {
            return 0.0;
        }

        1.0 - (distance as f32 / max_distance as f32)
    }
}

/// Reference image (ungeoreferenced or partially georeferenced)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceImage {
    /// Unique identifier
    pub id: String,
    /// Image filename or source
    pub source: String,
    /// Location if known
    pub location: Option<GeoPoint>,
    /// Georeferencing status
    pub georeference_status: GeoreferenceStatus,
    /// Confidence in current location (0.0-1.0)
    pub location_confidence: f32,
    /// Orientation information
    pub orientation: ImageOrientation,
    /// Timestamp when reference was added
    pub created_timestamp: i64,
    /// Timestamp when location was determined
    pub georeference_timestamp: Option<i64>,
    /// Visual descriptor for matching
    pub descriptor: VisualDescriptor,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Related reference images (different perspectives)
    pub related_image_ids: Vec<String>,
    /// Matches found against robot observations
    pub match_history: Vec<ImageMatch>,
}

impl ReferenceImage {
    /// Create ungeoreferenced reference image
    pub fn ungeoreferenced(
        source: &str,
        descriptor: VisualDescriptor,
    ) -> Self {
        ReferenceImage {
            id: Uuid::new_v4().to_string(),
            source: source.to_string(),
            location: None,
            georeference_status: GeoreferenceStatus::Ungeoreferenced,
            location_confidence: 0.0,
            orientation: ImageOrientation::unknown(),
            created_timestamp: chrono::Utc::now().timestamp_micros(),
            georeference_timestamp: None,
            descriptor,
            metadata: HashMap::new(),
            related_image_ids: Vec::new(),
            match_history: Vec::new(),
        }
    }

    /// Update location from robot observation match
    pub fn update_location(&mut self, location: GeoPoint, confidence: f32) {
        self.location = Some(location);
        self.location_confidence = confidence;
        self.georeference_status = match confidence {
            c if c >= 0.8 => GeoreferenceStatus::Confirmed,
            c if c >= 0.6 => GeoreferenceStatus::PredictedMedium,
            _ => GeoreferenceStatus::PredictedLow,
        };
        self.georeference_timestamp = Some(chrono::Utc::now().timestamp_micros());
    }

    /// Update orientation from observation
    pub fn update_orientation(&mut self, orientation: ImageOrientation) {
        self.orientation = orientation;
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Link related image (different perspective)
    pub fn link_related(&mut self, image_id: &str) {
        if !self.related_image_ids.contains(&image_id.to_string()) {
            self.related_image_ids.push(image_id.to_string());
        }
    }

    /// Record match with robot observation
    pub fn record_match(&mut self, match_result: ImageMatch) {
        self.match_history.push(match_result);
    }
}

/// Result of matching reference image against robot observation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageMatch {
    /// ID of robot observation
    pub observation_id: String,
    /// Robot ID
    pub robot_id: String,
    /// Location where match occurred
    pub robot_location: GeoPoint,
    /// Match confidence (0.0-1.0)
    pub confidence: f32,
    /// Matching features
    pub matched_features: u32,
    /// Timestamp of match
    pub timestamp: i64,
}

/// Reference image store
pub struct ReferenceImageStore {
    images: HashMap<String, ReferenceImage>,
    /// Index by perceptual hash for quick lookup
    hash_index: HashMap<String, Vec<String>>,
}

impl ReferenceImageStore {
    /// Create new store
    pub fn new() -> Self {
        ReferenceImageStore {
            images: HashMap::new(),
            hash_index: HashMap::new(),
        }
    }

    /// Add reference image
    pub fn add_image(&mut self, image: ReferenceImage) -> Result<String> {
        let id = image.id.clone();
        let hash = image.descriptor.perceptual_hash.clone();

        // Index by hash
        self.hash_index.entry(hash).or_insert_with(Vec::new).push(id.clone());
        self.images.insert(id.clone(), image);

        Ok(id)
    }

    /// Get image by ID
    pub fn get_image(&self, id: &str) -> Option<&ReferenceImage> {
        self.images.get(id)
    }

    /// Get mutable reference to image
    pub fn get_image_mut(&mut self, id: &str) -> Option<&mut ReferenceImage> {
        self.images.get_mut(id)
    }

    /// Find similar images by descriptor
    pub fn find_similar(
        &self,
        descriptor: &VisualDescriptor,
        threshold: f32,
    ) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        for (_, image) in &self.images {
            let similarity = image.descriptor.similarity_score(descriptor);
            if similarity >= threshold {
                results.push((image.id.clone(), similarity));
            }
        }

        // Sort by similarity descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Get ungeoreferenced images
    pub fn ungeoreferenced_images(&self) -> Vec<&ReferenceImage> {
        self.images
            .values()
            .filter(|img| img.georeference_status == GeoreferenceStatus::Ungeoreferenced)
            .collect()
    }

    /// Get georeferenced images
    pub fn georeferenced_images(&self) -> Vec<&ReferenceImage> {
        self.images
            .values()
            .filter(|img| img.georeference_status != GeoreferenceStatus::Ungeoreferenced)
            .collect()
    }

    /// Get images by location (radius)
    pub fn images_near_location(&self, location: GeoPoint, radius_degrees: f64) -> Vec<&ReferenceImage> {
        self.images
            .values()
            .filter(|img| {
                if let Some(loc) = img.location {
                    (loc.lat - location.lat).abs() <= radius_degrees
                        && (loc.lon - location.lon).abs() <= radius_degrees
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get related image sets (perspectives of same scene)
    pub fn get_related_images(&self, image_id: &str) -> Vec<&ReferenceImage> {
        if let Some(image) = self.images.get(image_id) {
            image
                .related_image_ids
                .iter()
                .filter_map(|id| self.images.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Total images
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    /// Get all images
    pub fn all_images(&self) -> Vec<&ReferenceImage> {
        self.images.values().collect()
    }
}

impl Default for ReferenceImageStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_descriptor() -> VisualDescriptor {
        VisualDescriptor::new("ORB", "1010101010101010")
    }

    #[test]
    fn test_reference_image_creation() {
        let desc = create_descriptor();
        let image = ReferenceImage::ungeoreferenced("test_image.jpg", desc);

        assert_eq!(image.source, "test_image.jpg");
        assert_eq!(image.georeference_status, GeoreferenceStatus::Ungeoreferenced);
        assert_eq!(image.location_confidence, 0.0);
        assert!(image.location.is_none());
    }

    #[test]
    fn test_image_georeference_update() {
        let desc = create_descriptor();
        let mut image = ReferenceImage::ungeoreferenced("test.jpg", desc);
        let location = GeoPoint::new(40.7128, -74.0060);

        image.update_location(location, 0.85);

        assert_eq!(image.location, Some(location));
        assert_eq!(image.location_confidence, 0.85);
        assert_eq!(image.georeference_status, GeoreferenceStatus::Confirmed);
    }

    #[test]
    fn test_orientation_unknown() {
        let orientation = ImageOrientation::unknown();
        assert!(!orientation.is_fully_known());
        assert!(orientation.heading_degrees.is_none());
    }

    #[test]
    fn test_orientation_with_heading() {
        let orientation = ImageOrientation::with_heading(45.0);
        assert!(orientation.heading_degrees.is_some());
        assert_eq!(orientation.heading_degrees, Some(45.0));
        assert!(!orientation.is_fully_known());
    }

    #[test]
    fn test_visual_descriptor_similarity() {
        let desc1 = VisualDescriptor::new("ORB", "1010101010101010");
        let desc2 = VisualDescriptor::new("ORB", "1010101010101010");
        let desc3 = VisualDescriptor::new("ORB", "1111111111111111");

        assert_eq!(desc1.hamming_distance(&desc2), 0);
        assert_eq!(desc1.similarity_score(&desc2), 1.0);

        assert_eq!(desc1.hamming_distance(&desc3), 8);
        assert_eq!(desc1.similarity_score(&desc3), 0.5); // 8/16 = 50% different
    }

    #[test]
    fn test_reference_image_store_add() {
        let mut store = ReferenceImageStore::new();
        let desc = create_descriptor();
        let image = ReferenceImage::ungeoreferenced("test.jpg", desc);

        let id = store.add_image(image).unwrap();
        assert_eq!(store.len(), 1);
        assert!(store.get_image(&id).is_some());
    }

    #[test]
    fn test_reference_image_store_find_similar() {
        let mut store = ReferenceImageStore::new();
        let desc1 = VisualDescriptor::new("ORB", "1010101010101010");
        let image1 = ReferenceImage::ungeoreferenced("test1.jpg", desc1);
        store.add_image(image1).unwrap();

        let desc2 = VisualDescriptor::new("ORB", "1010101010101010");
        let similar = store.find_similar(&desc2, 0.8);

        assert_eq!(similar.len(), 1);
        assert_eq!(similar[0].1, 1.0);
    }

    #[test]
    fn test_ungeoreferenced_filter() {
        let mut store = ReferenceImageStore::new();

        let desc1 = create_descriptor();
        let image1 = ReferenceImage::ungeoreferenced("test1.jpg", desc1);
        store.add_image(image1).unwrap();

        let desc2 = create_descriptor();
        let mut image2 = ReferenceImage::ungeoreferenced("test2.jpg", desc2);
        image2.update_location(GeoPoint::new(40.7128, -74.0060), 0.9);
        store.add_image(image2).unwrap();

        let ungeoreferenced = store.ungeoreferenced_images();
        assert_eq!(ungeoreferenced.len(), 1);

        let georeferenced = store.georeferenced_images();
        assert_eq!(georeferenced.len(), 1);
    }

    #[test]
    fn test_reference_image_metadata() {
        let desc = create_descriptor();
        let image = ReferenceImage::ungeoreferenced("test.jpg", desc)
            .with_metadata("source", "aerial_survey")
            .with_metadata("resolution", "4K");

        assert_eq!(image.metadata.get("source"), Some(&"aerial_survey".to_string()));
        assert_eq!(image.metadata.get("resolution"), Some(&"4K".to_string()));
    }

    #[test]
    fn test_related_images() {
        let mut store = ReferenceImageStore::new();
        let desc1 = create_descriptor();
        let mut image1 = ReferenceImage::ungeoreferenced("test1.jpg", desc1);

        let desc2 = create_descriptor();
        let mut image2 = ReferenceImage::ungeoreferenced("test2.jpg", desc2);
        let image2_id = image2.id.clone();

        image1.link_related(&image2_id);

        let id1 = store.add_image(image1).unwrap();
        store.add_image(image2).unwrap();

        let related = store.get_related_images(&id1);
        assert_eq!(related.len(), 1);
    }

    #[test]
    fn test_image_match_record() {
        let desc = create_descriptor();
        let mut image = ReferenceImage::ungeoreferenced("test.jpg", desc);

        let match_result = ImageMatch {
            observation_id: "obs_1".to_string(),
            robot_id: "bot_1".to_string(),
            robot_location: GeoPoint::new(40.7128, -74.0060),
            confidence: 0.92,
            matched_features: 45,
            timestamp: 1000000,
        };

        image.record_match(match_result);
        assert_eq!(image.match_history.len(), 1);
        assert_eq!(image.match_history[0].confidence, 0.92);
    }

    #[test]
    fn test_images_near_location() {
        let mut store = ReferenceImageStore::new();

        let desc = create_descriptor();
        let mut image = ReferenceImage::ungeoreferenced("test.jpg", desc);
        image.update_location(GeoPoint::new(40.7128, -74.0060), 0.95);
        store.add_image(image).unwrap();

        let location = GeoPoint::new(40.7128, -74.0060);
        let nearby = store.images_near_location(location, 0.01);
        assert_eq!(nearby.len(), 1);

        // Different location outside radius
        let far_location = GeoPoint::new(50.0, -74.0060);
        let far = store.images_near_location(far_location, 0.01);
        assert_eq!(far.len(), 0);
    }
}
