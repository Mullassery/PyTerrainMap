use crate::gaussian_splatting::objects::ObjectClass;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Types of environmental changes detected by the fleet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChangeEventType {
    /// New object appeared
    ObjectAppeared {
        object_id: Uuid,
        class: ObjectClass,
    },
    /// Object moved to a new location
    ObjectMoved {
        object_id: Uuid,
        from: [f64; 3],
        to: [f64; 3],
        distance_m: f32,
    },
    /// Object disappeared from view
    ObjectDisappeared {
        object_id: Uuid,
        last_position: [f64; 3],
    },
    /// Path became blocked by an object
    PathBlocked {
        from: [f64; 3],
        to: [f64; 3],
        by_object: Uuid,
    },
    /// Path cleared after being blocked
    PathCleared {
        from: [f64; 3],
        to: [f64; 3],
    },
    /// Large area was cleared
    AreaCleared {
        center: [f64; 3],
        radius_m: f32,
    },
}

/// An individual environment change event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: Uuid,
    pub event_type: ChangeEventType,
    pub detected_by: Vec<String>,  // Bot IDs that observed this
    pub timestamp: i64,             // Microseconds since epoch
    pub confidence: f32,
}

impl ChangeEvent {
    /// Create a new change event
    pub fn new(event_type: ChangeEventType, bot_id: &str, confidence: f32) -> Self {
        ChangeEvent {
            id: Uuid::new_v4(),
            event_type,
            detected_by: vec![bot_id.to_string()],
            timestamp: chrono::Utc::now().timestamp_micros(),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Add a bot as an observer of this event
    pub fn add_observer(&mut self, bot_id: &str) {
        if !self.detected_by.contains(&bot_id.to_string()) {
            self.detected_by.push(bot_id.to_string());
        }
    }

    /// Increase confidence when multiple bots confirm
    pub fn increase_confidence(&mut self, delta: f32) {
        self.confidence = (self.confidence + delta).min(1.0);
    }
}

/// Log of environmental changes with bounded history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeEventLog {
    events: VecDeque<ChangeEvent>,
    max_events: usize,
}

impl ChangeEventLog {
    /// Create a new change event log
    pub fn new(max_events: usize) -> Self {
        ChangeEventLog {
            events: VecDeque::with_capacity(max_events),
            max_events,
        }
    }

    /// Record a new event
    pub fn record(&mut self, event: ChangeEvent) {
        self.events.push_back(event);
        if self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    /// Get all events in a region and time window
    pub fn events_in_region(
        &self,
        center: [f64; 3],
        radius_m: f64,
        since_us: i64,
    ) -> Vec<&ChangeEvent> {
        self.events
            .iter()
            .filter(|e| {
                if e.timestamp < since_us {
                    return false;
                }
                match &e.event_type {
                    ChangeEventType::ObjectAppeared { object_id: _, class: _ } => false,  // No position
                    ChangeEventType::ObjectMoved { object_id: _, from, to, distance_m: _ } => {
                        Self::distance_between(center, *from) <= radius_m
                            || Self::distance_between(center, *to) <= radius_m
                    }
                    ChangeEventType::ObjectDisappeared { object_id: _, last_position } => {
                        Self::distance_between(center, *last_position) <= radius_m
                    }
                    ChangeEventType::PathBlocked { from, to, by_object: _ } => {
                        Self::distance_between(center, *from) <= radius_m
                            || Self::distance_between(center, *to) <= radius_m
                    }
                    ChangeEventType::PathCleared { from, to } => {
                        Self::distance_between(center, *from) <= radius_m
                            || Self::distance_between(center, *to) <= radius_m
                    }
                    ChangeEventType::AreaCleared { center: evt_center, radius_m: evt_radius } => {
                        Self::distance_between(center, *evt_center) <= (radius_m + (*evt_radius as f64))
                    }
                }
            })
            .collect()
    }

    /// Compute change rate in an area (events per hour)
    pub fn change_rate_per_hour(
        &self,
        center: [f64; 3],
        radius_m: f64,
        window_us: i64,
    ) -> f32 {
        let since_us = chrono::Utc::now().timestamp_micros() - window_us;
        let events = self.events_in_region(center, radius_m, since_us);
        let hours = (window_us as f32) / (3600.0 * 1_000_000.0);
        (events.len() as f32) / hours.max(0.01)
    }

    /// Get recent events (last N)
    pub fn recent(&self, limit: usize) -> Vec<&ChangeEvent> {
        self.events
            .iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Helper: Haversine distance between two 3D points (lat, lon, elev)
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

impl Default for ChangeEventLog {
    fn default() -> Self {
        Self::new(10_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_event_creation() {
        let evt = ChangeEvent::new(
            ChangeEventType::ObjectAppeared {
                object_id: Uuid::new_v4(),
                class: ObjectClass::Pallet,
            },
            "bot_01",
            0.9,
        );
        assert!(evt.detected_by.contains(&"bot_01".to_string()));
        assert_eq!(evt.confidence, 0.9);
    }

    #[test]
    fn test_change_event_add_observer() {
        let mut evt = ChangeEvent::new(
            ChangeEventType::ObjectAppeared {
                object_id: Uuid::new_v4(),
                class: ObjectClass::Pallet,
            },
            "bot_01",
            0.9,
        );
        evt.add_observer("bot_02");
        assert_eq!(evt.detected_by.len(), 2);
        evt.add_observer("bot_01");  // Duplicate
        assert_eq!(evt.detected_by.len(), 2);
    }

    #[test]
    fn test_change_event_log_record() {
        let mut log = ChangeEventLog::new(100);
        let evt = ChangeEvent::new(
            ChangeEventType::ObjectAppeared {
                object_id: Uuid::new_v4(),
                class: ObjectClass::Pallet,
            },
            "bot_01",
            0.9,
        );
        log.record(evt);
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_change_event_log_max_capacity() {
        let mut log = ChangeEventLog::new(5);
        for i in 0..10 {
            let evt = ChangeEvent::new(
                ChangeEventType::ObjectAppeared {
                    object_id: Uuid::new_v4(),
                    class: ObjectClass::Pallet,
                },
                &format!("bot_{}", i),
                0.9,
            );
            log.record(evt);
        }
        assert_eq!(log.len(), 5);
    }

    #[test]
    fn test_change_event_log_recent() {
        let mut log = ChangeEventLog::new(100);
        for i in 0..10 {
            let evt = ChangeEvent::new(
                ChangeEventType::ObjectAppeared {
                    object_id: Uuid::new_v4(),
                    class: ObjectClass::Pallet,
                },
                &format!("bot_{}", i),
                0.9,
            );
            log.record(evt);
        }
        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
    }
}
