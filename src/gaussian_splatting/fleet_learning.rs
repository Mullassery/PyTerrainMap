use crate::gaussian_splatting::core::GaussianCovariance;
use crate::gaussian_splatting::objects::{DynamicObjectSplat, ObjectClass};
use crate::gaussian_splatting::change_events::{ChangeEvent, ChangeEventLog, ChangeEventType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An observation of an object from a bot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectObservation {
    pub object_class: ObjectClass,
    pub position: [f64; 3],
    pub covariance: GaussianCovariance,
    pub timestamp: i64,
    pub confidence: f32,
    pub dimensions: Option<[f32; 3]>,
}

/// Current state of an object from the shared map perspective
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectState {
    pub object: DynamicObjectSplat,
    pub position_confidence: f32,      // Time-decayed confidence
    pub is_out_of_sight: bool,          // True if no bot currently watching
    pub predicted_position: Option<[f64; 3]>,  // For Dynamic objects
}

/// Prediction of where an object might be (even unseen)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectPrediction {
    pub object_id: Uuid,
    pub object_class: ObjectClass,
    pub predicted_position: [f64; 3],
    pub confidence: f32,
    pub is_direct_observation: bool,  // False = inferred via dynamics
}

/// Dynamics profile of an area (how many changes per hour, what classes appear)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AreaDynamicsProfile {
    pub change_rate_per_hour: f32,
    pub typical_object_classes: Vec<(String, f32)>,  // Class name, frequency
    pub last_updated: i64,
}

/// The shared fleet learning engine: one bot learns, all bots know
///
/// All observations from all bots feed into this engine, which maintains:
/// - A global object store with decayed confidence
/// - A change event log for environmental dynamics
/// - Area profiles for predicting what objects to expect
pub struct FleetLearningEngine {
    /// All known objects, keyed by UUID
    pub object_store: HashMap<Uuid, DynamicObjectSplat>,
    /// Log of all observed environmental changes
    pub change_log: ChangeEventLog,
    /// Dynamics profiles for different regions
    pub area_profiles: HashMap<String, AreaDynamicsProfile>,
}

impl FleetLearningEngine {
    /// Create a new fleet learning engine
    pub fn new() -> Self {
        FleetLearningEngine {
            object_store: HashMap::new(),
            change_log: ChangeEventLog::new(10_000),
            area_profiles: HashMap::new(),
        }
    }

    /// **THE KEY METHOD**: Ingest observations from a bot
    ///
    /// This is called whenever any bot observes objects. It:
    /// 1. Finds or creates splats for each observation
    /// 2. Detects movement by comparing positions
    /// 3. Emits ChangeEvents
    /// 4. Updates area dynamics profiles
    ///
    /// Returns the list of change events that occurred
    pub fn ingest_observation(
        &mut self,
        bot_id: &str,
        observations: Vec<ObjectObservation>,
    ) -> Vec<ChangeEvent> {
        let mut events = Vec::new();
        let first_obs_pos = observations.first().map(|o| o.position);

        for obs in observations {
            let class = obs.object_class;
            let movement_threshold = match class {
                ObjectClass::Person => 0.0,      // People: even tiny movement counts
                ObjectClass::Pallet => 0.5,      // Pallets: 50cm movement
                ObjectClass::Cart => 0.5,
                _ => 1.0,                         // Others: 1m
            };

            // Find nearby splat of same class
            let mut found_splat = None;
            for (id, splat) in &self.object_store {
                if splat.object_class == class {
                    let dist = [
                        (splat.position[0] - obs.position[0]) as f32,
                        (splat.position[1] - obs.position[1]) as f32,
                        (splat.position[2] - obs.position[2]) as f32,
                    ];
                    let d_sq = dist[0] * dist[0] + dist[1] * dist[1] + dist[2] * dist[2];
                    // Within 3σ of covariance
                    if obs.covariance.mahalanobis_sq(dist) <= 9.0 {
                        found_splat = Some(*id);
                        break;
                    }
                }
            }

            match found_splat {
                Some(splat_id) => {
                    // Update existing splat
                    if let Some(splat) = self.object_store.get_mut(&splat_id) {
                        let old_pos = splat.position;

                        // Check for movement
                        let move_dist = [
                            (obs.position[0] - old_pos[0]) as f32,
                            (obs.position[1] - old_pos[1]) as f32,
                            (obs.position[2] - old_pos[2]) as f32,
                        ];
                        let move_len = (move_dist[0] * move_dist[0]
                            + move_dist[1] * move_dist[1]
                            + move_dist[2] * move_dist[2])
                        .sqrt();

                        if move_len > movement_threshold {
                            // Object moved: emit event
                            let mut evt = ChangeEvent::new(
                                ChangeEventType::ObjectMoved {
                                    object_id: splat_id,
                                    from: old_pos,
                                    to: obs.position,
                                    distance_m: move_len,
                                },
                                bot_id,
                                obs.confidence,
                            );
                            evt.increase_confidence(0.05);  // High confidence for movement
                            events.push(evt);
                        }

                        // Update splat position
                        splat.add_position_snapshot(obs.position, bot_id, obs.confidence);
                        if let Some(dims) = obs.dimensions {
                            splat.dimensions = dims;
                        }
                    }
                }
                None => {
                    // New object: create splat and emit event
                    let mut new_splat = DynamicObjectSplat::new(class, obs.position, bot_id);
                    new_splat.covariance = obs.covariance;
                    if let Some(dims) = obs.dimensions {
                        new_splat.dimensions = dims;
                    }
                    new_splat.confidence = obs.confidence;

                    let splat_id = new_splat.id;
                    self.object_store.insert(splat_id, new_splat);

                    let evt = ChangeEvent::new(
                        ChangeEventType::ObjectAppeared {
                            object_id: splat_id,
                            class,
                        },
                        bot_id,
                        obs.confidence,
                    );
                    events.push(evt);
                }
            }
        }

        // Record all events
        for evt in &events {
            self.change_log.record(evt.clone());
        }

        // Update area profiles based on observations
        if let Some(pos) = first_obs_pos {
            self.update_area_profile(pos, &events);
        }

        events
    }

    /// Query: what objects are near this position right now?
    ///
    /// Includes out-of-sight objects with decayed confidence. Useful for path planning.
    pub fn objects_near(
        &self,
        pos: [f64; 3],
        radius_m: f64,
        current_time_us: i64,
    ) -> Vec<ObjectState> {
        self.object_store
            .values()
            .filter_map(|obj| {
                let dist = Self::distance_between(pos, obj.position);
                if dist <= radius_m {
                    let position_confidence = obj.decayed_confidence(current_time_us);
                    let time_since_seen_us = current_time_us - obj.last_seen;
                    let is_out_of_sight = time_since_seen_us > 60_000_000;  // > 1 minute

                    Some(ObjectState {
                        object: obj.clone(),
                        position_confidence,
                        is_out_of_sight,
                        predicted_position: None,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Path planning integration: is a path currently blocked?
    pub fn path_blocked(
        &self,
        from: [f64; 3],
        to: [f64; 3],
        width_m: f32,
        current_time_us: i64,
    ) -> Option<Uuid> {
        let path_dist = Self::distance_between(from, to);
        let path_center = [
            (from[0] + to[0]) / 2.0,
            (from[1] + to[1]) / 2.0,
            (from[2] + to[2]) / 2.0,
        ];

        // Check objects near path center
        let objects = self.objects_near(path_center, (path_dist / 2.0) + (width_m as f64), current_time_us);
        for obj_state in objects {
            // Object is blocking if it's an obstacle and has decent confidence
            if obj_state.position_confidence > 0.5 && matches!(obj_state.object.object_class, ObjectClass::Cart | ObjectClass::Forklift | ObjectClass::Shelf) {
                return Some(obj_state.object.id);
            }
        }
        None
    }

    /// Collective intelligence: predict all object positions at a future time
    pub fn predict_object_positions(&self, time_us: i64) -> Vec<ObjectPrediction> {
        self.object_store
            .values()
            .map(|obj| {
                let (pos, conf) = obj.predict_position_at(time_us);
                ObjectPrediction {
                    object_id: obj.id,
                    object_class: obj.object_class,
                    predicted_position: pos,
                    confidence: conf,
                    is_direct_observation: (time_us - obj.last_seen).abs() < 1_000_000,  // < 1 sec
                }
            })
            .collect()
    }

    /// Update area dynamics profile after ingesting observations
    fn update_area_profile(&mut self, _pos: [f64; 3], _events: &[ChangeEvent]) {
        // In a full implementation, this would:
        // 1. Compute region key from position (e.g., H3 cell)
        // 2. Update change_rate_per_hour based on recent events
        // 3. Track typical object classes and frequencies
        // Simplified for now.
    }

    /// Helper: Haversine distance between two 3D points
    fn distance_between(pos1: [f64; 3], pos2: [f64; 3]) -> f64 {
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

impl Default for FleetLearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fleet_learning_new_observation() {
        let mut fleet = FleetLearningEngine::new();
        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let events = fleet.ingest_observation("bot_01", vec![obs]);
        assert_eq!(events.len(), 1);
        assert_eq!(fleet.object_store.len(), 1);
    }

    #[test]
    fn test_fleet_learning_object_update() {
        let mut fleet = FleetLearningEngine::new();
        let obs1 = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let _events1 = fleet.ingest_observation("bot_01", vec![obs1]);
        assert_eq!(fleet.object_store.len(), 1);

        // Same pallet, slightly moved
        let obs2 = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.6, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        let events2 = fleet.ingest_observation("bot_02", vec![obs2]);
        assert_eq!(fleet.object_store.len(), 1);  // Still 1 object
        assert!(events2.iter().any(|e| matches!(e.event_type, ChangeEventType::ObjectMoved { .. })));
    }

    #[test]
    fn test_fleet_learning_objects_near() {
        let mut fleet = FleetLearningEngine::new();
        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        fleet.ingest_observation("bot_01", vec![obs]);
        let now = chrono::Utc::now().timestamp_micros();
        let nearby = fleet.objects_near([10.001, 20.001, 0.0], 100.0, now);
        assert_eq!(nearby.len(), 1);
    }

    #[test]
    fn test_fleet_learning_change_event_log() {
        let mut fleet = FleetLearningEngine::new();
        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        fleet.ingest_observation("bot_01", vec![obs]);
        assert!(fleet.change_log.len() > 0);
    }

    #[test]
    fn test_fleet_learning_predict_positions() {
        let mut fleet = FleetLearningEngine::new();
        let obs = ObjectObservation {
            object_class: ObjectClass::Pallet,
            position: [10.0, 20.0, 0.0],
            covariance: GaussianCovariance::isotropic(0.5),
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: 0.9,
            dimensions: None,
        };

        fleet.ingest_observation("bot_01", vec![obs]);
        let now = chrono::Utc::now().timestamp_micros();
        let predictions = fleet.predict_object_positions(now);
        assert_eq!(predictions.len(), 1);
        assert!(predictions[0].is_direct_observation);
    }
}
