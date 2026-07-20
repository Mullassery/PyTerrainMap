use crate::gaussian_splatting::core::GaussianCovariance;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Classification of dynamic objects in the warehouse
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ObjectClass {
    Pallet,
    Box,
    Cart,
    Shelf,
    Machine,
    Forklift,
    Person,
    Unknown,
}

impl ObjectClass {
    pub fn as_str(&self) -> &str {
        match self {
            ObjectClass::Pallet => "Pallet",
            ObjectClass::Box => "Box",
            ObjectClass::Cart => "Cart",
            ObjectClass::Shelf => "Shelf",
            ObjectClass::Machine => "Machine",
            ObjectClass::Forklift => "Forklift",
            ObjectClass::Person => "Person",
            ObjectClass::Unknown => "Unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Pallet" => ObjectClass::Pallet,
            "Box" => ObjectClass::Box,
            "Cart" => ObjectClass::Cart,
            "Shelf" => ObjectClass::Shelf,
            "Machine" => ObjectClass::Machine,
            "Forklift" => ObjectClass::Forklift,
            "Person" => ObjectClass::Person,
            _ => ObjectClass::Unknown,
        }
    }
}

/// How quickly an object's confidence decays
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ObjectMobility {
    /// Walls, permanent fixtures (90-day decay)
    Fixed,
    /// Pallets, boxes (8-hour decay)
    Movable,
    /// Carts, forklifts (2-hour decay)
    Mobile,
    /// People (30-minute decay)
    Dynamic,
}

impl ObjectMobility {
    /// Half-life in milliseconds
    pub fn decay_half_life_ms(&self) -> u64 {
        match self {
            ObjectMobility::Fixed => 90 * 24 * 60 * 60 * 1000,      // 90 days
            ObjectMobility::Movable => 8 * 60 * 60 * 1000,           // 8 hours
            ObjectMobility::Mobile => 2 * 60 * 60 * 1000,            // 2 hours
            ObjectMobility::Dynamic => 30 * 60 * 1000,               // 30 minutes
        }
    }

    pub fn for_class(class: ObjectClass) -> Self {
        match class {
            ObjectClass::Pallet => ObjectMobility::Movable,
            ObjectClass::Box => ObjectMobility::Movable,
            ObjectClass::Cart => ObjectMobility::Mobile,
            ObjectClass::Shelf => ObjectMobility::Fixed,
            ObjectClass::Machine => ObjectMobility::Fixed,
            ObjectClass::Forklift => ObjectMobility::Mobile,
            ObjectClass::Person => ObjectMobility::Dynamic,
            ObjectClass::Unknown => ObjectMobility::Movable,
        }
    }
}

/// Snapshot of object position at a point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub position: [f64; 3],
    pub timestamp: i64,
    pub bot_id: String,
    pub confidence: f32,
}

/// Dynamic object splat: position + covariance + movement history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicObjectSplat {
    pub id: Uuid,
    pub object_class: ObjectClass,
    pub mobility: ObjectMobility,
    pub position: [f64; 3],
    pub covariance: GaussianCovariance,
    pub dimensions: [f32; 3],  // width, depth, height in meters
    pub confidence: f32,
    pub first_seen: i64,
    pub last_seen: i64,
    pub source_bots: Vec<String>,
    pub position_history: Vec<PositionSnapshot>,  // Ring buffer, max 50
}

impl DynamicObjectSplat {
    /// Create a new dynamic object splat
    pub fn new(
        object_class: ObjectClass,
        position: [f64; 3],
        bot_id: &str,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_micros();
        DynamicObjectSplat {
            id: Uuid::new_v4(),
            object_class,
            mobility: ObjectMobility::for_class(object_class),
            position,
            covariance: GaussianCovariance::isotropic(0.5),  // 0.5m std dev for objects
            dimensions: [1.0, 1.0, 1.5],  // Default: ~1m³ pallet
            confidence: 0.8,
            first_seen: now,
            last_seen: now,
            source_bots: vec![bot_id.to_string()],
            position_history: vec![PositionSnapshot {
                position,
                timestamp: now,
                bot_id: bot_id.to_string(),
                confidence: 0.8,
            }],
        }
    }

    /// Check if object has moved significantly since first observation
    pub fn has_moved_since_first_seen(&self, threshold_m: f32) -> bool {
        if self.position_history.len() < 2 {
            return false;
        }
        let first = self.position_history[0].position;
        let delta = [
            (self.position[0] - first[0]) as f32,
            (self.position[1] - first[1]) as f32,
            (self.position[2] - first[2]) as f32,
        ];
        let dist = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        dist > threshold_m
    }

    /// Estimate velocity from position history in m/s [x, y, z]
    pub fn estimated_velocity_m_per_s(&self) -> Option<[f32; 3]> {
        if self.position_history.len() < 2 {
            return None;
        }
        let recent = &self.position_history[self.position_history.len() - 1];
        let prev = &self.position_history[self.position_history.len() - 2];

        let dt_us = recent.timestamp - prev.timestamp;
        if dt_us <= 0 {
            return None;
        }
        let dt_s = (dt_us as f32) / 1_000_000.0;

        let dx = (recent.position[0] - prev.position[0]) as f32;
        let dy = (recent.position[1] - prev.position[1]) as f32;
        let dz = (recent.position[2] - prev.position[2]) as f32;

        Some([dx / dt_s, dy / dt_s, dz / dt_s])
    }

    /// Predict object position at a future time using linear extrapolation
    pub fn predict_position_at(&self, time_us: i64) -> ([f64; 3], f32) {
        let dt_us = time_us - self.last_seen;
        if dt_us < 0 {
            return (self.position, self.confidence);
        }

        match self.estimated_velocity_m_per_s() {
            None => (self.position, self.confidence),
            Some(vel) => {
                let dt_s = (dt_us as f32) / 1_000_000.0;
                let predicted = [
                    self.position[0] + (vel[0] as f64) * (dt_s as f64),
                    self.position[1] + (vel[1] as f64) * (dt_s as f64),
                    self.position[2] + (vel[2] as f64) * (dt_s as f64),
                ];
                (predicted, self.confidence)
            }
        }
    }

    /// Add a position observation to history (maintains ring buffer max 50)
    pub fn add_position_snapshot(&mut self, position: [f64; 3], bot_id: &str, confidence: f32) {
        self.position = position;
        self.last_seen = chrono::Utc::now().timestamp_micros();
        self.confidence = confidence.clamp(0.0, 1.0);

        self.position_history.push(PositionSnapshot {
            position,
            timestamp: self.last_seen,
            bot_id: bot_id.to_string(),
            confidence,
        });

        if self.position_history.len() > 50 {
            self.position_history.remove(0);
        }

        if !self.source_bots.contains(&bot_id.to_string()) {
            self.source_bots.push(bot_id.to_string());
        }
    }

    /// Decay confidence based on time elapsed and mobility class
    pub fn decayed_confidence(&self, current_time_us: i64) -> f32 {
        let age_us = current_time_us - self.last_seen;
        if age_us < 0 {
            return self.confidence;
        }

        let age_ms = (age_us / 1000) as f32;
        let half_life_ms = self.mobility.decay_half_life_ms() as f32;

        // Exponential decay: confidence * 0.5^(age / half_life)
        let decay = 0.5_f32.powf(age_ms / half_life_ms);
        (self.confidence * decay).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_class_str() {
        assert_eq!(ObjectClass::Pallet.as_str(), "Pallet");
        assert_eq!(ObjectClass::Person.as_str(), "Person");
    }

    #[test]
    fn test_object_class_from_str() {
        assert_eq!(ObjectClass::from_str("Pallet"), ObjectClass::Pallet);
        assert_eq!(ObjectClass::from_str("Person"), ObjectClass::Person);
    }

    #[test]
    fn test_object_mobility_decay_half_life() {
        assert!(ObjectMobility::Dynamic.decay_half_life_ms() < ObjectMobility::Mobile.decay_half_life_ms());
        assert!(ObjectMobility::Mobile.decay_half_life_ms() < ObjectMobility::Movable.decay_half_life_ms());
        assert!(ObjectMobility::Movable.decay_half_life_ms() < ObjectMobility::Fixed.decay_half_life_ms());
    }

    #[test]
    fn test_dynamic_object_splat_creation() {
        let splat = DynamicObjectSplat::new(ObjectClass::Pallet, [10.0, 20.0, 0.0], "bot_01");
        assert_eq!(splat.object_class, ObjectClass::Pallet);
        assert_eq!(splat.mobility, ObjectMobility::Movable);
        assert_eq!(splat.position_history.len(), 1);
    }

    #[test]
    fn test_dynamic_object_splat_add_snapshot() {
        let mut splat = DynamicObjectSplat::new(ObjectClass::Pallet, [10.0, 20.0, 0.0], "bot_01");
        splat.add_position_snapshot([10.5, 20.0, 0.0], "bot_02", 0.9);
        assert_eq!(splat.position_history.len(), 2);
        assert!(splat.source_bots.contains(&"bot_02".to_string()));
    }

    #[test]
    fn test_dynamic_object_splat_has_moved() {
        let mut splat = DynamicObjectSplat::new(ObjectClass::Pallet, [10.0, 20.0, 0.0], "bot_01");
        splat.add_position_snapshot([10.6, 20.0, 0.0], "bot_02", 0.9);
        assert!(splat.has_moved_since_first_seen(0.5));
    }

    #[test]
    fn test_dynamic_object_splat_velocity() {
        let mut splat = DynamicObjectSplat::new(ObjectClass::Cart, [10.0, 20.0, 0.0], "bot_01");
        // Manually advance timestamp for second snapshot
        splat.position_history[0].timestamp = chrono::Utc::now().timestamp_micros() - 1_000_000;  // 1 second ago
        splat.add_position_snapshot([11.0, 20.0, 0.0], "bot_02", 0.9);

        if let Some(vel) = splat.estimated_velocity_m_per_s() {
            // Moved 1m in X in ~1 second = ~1 m/s
            assert!(vel[0] > 0.9 && vel[0] < 1.1);
        }
    }

    #[test]
    fn test_dynamic_object_splat_position_history_limit() {
        let mut splat = DynamicObjectSplat::new(ObjectClass::Pallet, [0.0, 0.0, 0.0], "bot_01");
        for i in 0..60 {
            splat.add_position_snapshot([(i as f64) * 0.1, 0.0, 0.0], "bot_01", 0.9);
        }
        assert_eq!(splat.position_history.len(), 50);
    }

    #[test]
    fn test_dynamic_object_splat_decayed_confidence() {
        let splat = DynamicObjectSplat::new(ObjectClass::Pallet, [10.0, 20.0, 0.0], "bot_01");
        let now = chrono::Utc::now().timestamp_micros();
        let decayed = splat.decayed_confidence(now);
        assert!(decayed >= splat.confidence * 0.99);  // Minimal decay at current time
    }
}
