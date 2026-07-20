use crate::gaussian_splatting::core::{GaussianCovariance, SplatKind, TerrainGaussian};
use crate::gaussian_splatting::fusion::{FusionAction, FusionResult, ObservationFuser};
use crate::gaussian_splatting::fleet_learning::{FleetLearningEngine, ObjectObservation, ObjectState};
use crate::gaussian_splatting::change_events::ChangeEvent;
use crate::temporal::DecayFunction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Spatial index key: H3 cell + elevation tier
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct H3SplatKey {
    pub h3_index: u64,
    pub elevation_tier: i32,  // floor(elevation_m / 2.0) for 2m vertical resolution
}

impl H3SplatKey {
    pub fn from_position(lat: f64, lon: f64, elevation_m: f64) -> Option<Self> {
        // Simplified H3-like indexing: hash lat/lon to create a deterministic cell index
        // In a full implementation, would use xs_h3 crate's proper H3 implementation
        let lat_idx = ((lat * 1000.0) as i64) & 0xFFFFF;
        let lon_idx = ((lon * 1000.0) as i64) & 0xFFFFF;
        let h3_index = ((lat_idx << 32) | lon_idx) as u64;
        let elevation_tier = (elevation_m / 2.0).floor() as i32;

        Some(H3SplatKey {
            h3_index,
            elevation_tier,
        })
    }
}

/// Statistics about the global splat store
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreStats {
    pub total_splats: usize,
    pub terrain_splats: usize,
    pub passage_splats: usize,
    pub prediction_splats: usize,
    pub object_splats: usize,
    pub total_fusions: u64,
    pub coverage_area_m2: f32,
    pub change_events_recorded: u64,
}

/// Global Gaussian Splat Store: the shared probabilistic world model
///
/// All bots read from and write to this store. Wrapped in Arc<RwLock<>> for thread-safe sharing.
pub struct GaussianSplatStore {
    /// All static terrain splats
    splats: HashMap<Uuid, TerrainGaussian>,
    /// Spatial index: H3 cell + elevation -> splat IDs
    spatial_index: HashMap<H3SplatKey, Vec<Uuid>>,
    /// Fleet learning engine (dynamic objects + change detection)
    pub fleet: FleetLearningEngine,
    /// Statistics
    stats: StoreStats,
}

impl GaussianSplatStore {
    /// Create a new store
    pub fn new() -> Self {
        GaussianSplatStore {
            splats: HashMap::new(),
            spatial_index: HashMap::new(),
            fleet: FleetLearningEngine::new(),
            stats: StoreStats {
                total_splats: 0,
                terrain_splats: 0,
                passage_splats: 0,
                prediction_splats: 0,
                object_splats: 0,
                total_fusions: 0,
                coverage_area_m2: 0.0,
                change_events_recorded: 0,
            },
        }
    }

    /// Insert a new terrain splat
    pub fn insert(&mut self, splat: TerrainGaussian) -> Uuid {
        let id = splat.id;
        self.update_stats_for_insert(&splat);

        if let Some(key) = H3SplatKey::from_position(splat.position[0], splat.position[1], splat.position[2]) {
            self.spatial_index.entry(key).or_insert_with(Vec::new).push(id);
        }

        self.splats.insert(id, splat);
        id
    }

    /// Insert or fuse a splat with nearby overlapping splats
    pub fn insert_or_fuse(&mut self, incoming: TerrainGaussian) -> FusionResult {
        let incoming_id = incoming.id;

        // Find nearby splats within 3σ (Mahalanobis distance)
        let nearby = self.find_nearby_splats(&incoming.position, &incoming.covariance);

        if nearby.is_empty() {
            // No overlap: insert as new
            self.insert(incoming);
            FusionResult {
                action: crate::gaussian_splatting::fusion::FusionAction::Created,
                splat_id: incoming_id,
                confidence_delta: 0.0,
                observations_merged: 1,
            }
        } else {
            // Fuse with first nearby splat
            let target_id = nearby[0];
            let mut target = self.splats.get_mut(&target_id).unwrap().clone();

            let result = ObservationFuser::fuse(&mut target, &incoming);
            self.stats.total_fusions += 1;

            self.splats.insert(target_id, target);
            result
        }
    }

    /// Query splats in a radius
    pub fn query_radius(&self, center: [f64; 3], radius_m: f64) -> Vec<&TerrainGaussian> {
        self.splats
            .values()
            .filter(|splat| {
                let dist = Self::haversine_m(center, splat.position);
                dist <= radius_m
            })
            .collect()
    }

    /// Compute uncertainty at a position (0 = fully known, 1 = completely unknown)
    pub fn uncertainty_at(&self, pos: [f64; 3]) -> f32 {
        let nearby = self.query_radius(pos, 50.0);  // Check within 50m
        if nearby.is_empty() {
            return 1.0;
        }

        // Average the inverse of confidence from nearby splats
        let avg_confidence = nearby.iter().map(|s| s.confidence).sum::<f32>() / (nearby.len() as f32);
        (1.0 - avg_confidence).clamp(0.0, 1.0)
    }

    /// Apply temporal decay to all splats
    pub fn apply_temporal_decay(&mut self, current_time_us: i64, decay: &DecayFunction) {
        for splat in self.splats.values_mut() {
            let age_ms = (current_time_us - splat.last_updated) / 1000;
            let decayed = decay.apply(splat.confidence, age_ms);
            splat.set_confidence(decayed);
        }
    }

    /// Remove stale splats below confidence threshold
    pub fn remove_stale(&mut self, min_confidence: f32) -> u32 {
        let initial_count = self.splats.len() as u32;
        let ids_to_remove: Vec<Uuid> = self.splats
            .iter()
            .filter(|(_, splat)| splat.confidence < min_confidence)
            .map(|(id, _)| *id)
            .collect();

        for id in ids_to_remove {
            self.splats.remove(&id);
        }

        initial_count - (self.splats.len() as u32)
    }

    /// Delegate to fleet learning engine
    pub fn ingest_bot_observation(
        &mut self,
        bot_id: &str,
        observations: Vec<ObjectObservation>,
    ) -> Vec<ChangeEvent> {
        let events = self.fleet.ingest_observation(bot_id, observations);
        self.stats.change_events_recorded += events.len() as u64;
        events
    }

    /// Query dynamic objects near a position
    pub fn objects_near(
        &self,
        pos: [f64; 3],
        radius_m: f64,
        now_us: i64,
    ) -> Vec<ObjectState> {
        self.fleet.objects_near(pos, radius_m, now_us)
    }

    /// Get current statistics
    pub fn stats(&self) -> StoreStats {
        self.stats.clone()
    }

    // === Private helpers ===

    fn find_nearby_splats(
        &self,
        position: &[f64; 3],
        covariance: &GaussianCovariance,
    ) -> Vec<Uuid> {
        self.splats
            .iter()
            .filter_map(|(id, splat)| {
                let overlap = splat.overlap_mahalanobis(&TerrainGaussian {
                    id: Uuid::nil(),
                    position: *position,
                    covariance: covariance.clone(),
                    traversability: 0.0,
                    terrain_type: crate::gaussian_splatting::core::TerrainType::Unknown(0),
                    confidence: 0.0,
                    observation_count: 0,
                    created_at: 0,
                    last_updated: 0,
                    source_bots: vec![],
                    splat_kind: SplatKind::Terrain,
                    metadata: std::collections::HashMap::new(),
                });

                if overlap <= 9.0 {  // Within 3σ
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    fn update_stats_for_insert(&mut self, splat: &TerrainGaussian) {
        self.stats.total_splats += 1;
        match splat.splat_kind {
            SplatKind::Terrain => self.stats.terrain_splats += 1,
            SplatKind::Passage => self.stats.passage_splats += 1,
            SplatKind::Prediction => self.stats.prediction_splats += 1,
            _ => {}
        }
    }

    /// Haversine distance between two 3D points
    fn haversine_m(pos1: [f64; 3], pos2: [f64; 3]) -> f64 {
        let lat1_rad = pos1[0].to_radians();
        let lat2_rad = pos2[0].to_radians();
        let delta_lat = (pos2[0] - pos1[0]).to_radians();
        let delta_lon = (pos2[1] - pos1[1]).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let earth_radius = 6_371_000.0;  // meters

        let horiz_dist = earth_radius * c;
        let vert_dist = (pos2[2] - pos1[2]).abs();

        (horiz_dist.powi(2) + vert_dist.powi(2)).sqrt()
    }
}

impl Default for GaussianSplatStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_creation() {
        let store = GaussianSplatStore::new();
        assert_eq!(store.stats.total_splats, 0);
    }

    #[test]
    fn test_store_insert() {
        let mut store = GaussianSplatStore::new();
        let splat = TerrainGaussian::from_point_observation([40.7128, -74.0060, 10.0], "bot_01", 0.85);
        store.insert(splat);
        assert_eq!(store.stats.total_splats, 1);
    }

    #[test]
    fn test_store_insert_or_fuse_creates_new() {
        let mut store = GaussianSplatStore::new();
        let splat = TerrainGaussian::from_point_observation([40.7128, -74.0060, 10.0], "bot_01", 0.85);
        let result = store.insert_or_fuse(splat);
        assert!(matches!(result.action, FusionAction::Created));
        assert_eq!(store.stats.total_splats, 1);
    }

    #[test]
    fn test_store_query_radius() {
        let mut store = GaussianSplatStore::new();
        let splat = TerrainGaussian::from_point_observation([40.7128, -74.0060, 10.0], "bot_01", 0.85);
        store.insert(splat);

        let results = store.query_radius([40.7128, -74.0060, 10.0], 1000.0);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_store_uncertainty_at_empty() {
        let store = GaussianSplatStore::new();
        let uncertainty = store.uncertainty_at([40.7128, -74.0060, 10.0]);
        assert_eq!(uncertainty, 1.0);
    }

    #[test]
    fn test_store_uncertainty_at_with_splat() {
        let mut store = GaussianSplatStore::new();
        let splat = TerrainGaussian::from_point_observation([40.7128, -74.0060, 10.0], "bot_01", 0.9);
        store.insert(splat);

        let uncertainty = store.uncertainty_at([40.7128, -74.0060, 10.0]);
        assert!(uncertainty < 0.5);  // Should be low uncertainty near observed point
    }
}
