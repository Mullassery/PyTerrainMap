use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of passages in the environment
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PassageType {
    Door,
    Hallway,
    Gate,
    Elevator,
    Stairwell,
    NarrowCorridor,
    Bridge,
}

/// Record of a passage traversal attempt
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PassageTraversal {
    pub timestamp: i64,
    pub robot_id: String,
    pub success: bool,
    pub was_open: bool,
}

/// Passage splat: doors, gates, hallways, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PassageSplat {
    pub id: Uuid,
    pub position: [f64; 3],
    pub passage_type: PassageType,
    pub width_m: f32,
    pub height_m: f32,
    pub connects: (String, String),  // (from_zone, to_zone)
    pub traversal_history: Vec<PassageTraversal>,
    pub confidence: f32,
    pub last_updated: i64,
}

impl PassageSplat {
    /// Create a new passage splat
    pub fn new(
        passage_type: PassageType,
        position: [f64; 3],
        width_m: f32,
        height_m: f32,
        from_zone: &str,
        to_zone: &str,
    ) -> Self {
        PassageSplat {
            id: Uuid::new_v4(),
            position,
            passage_type,
            width_m,
            height_m,
            connects: (from_zone.to_string(), to_zone.to_string()),
            traversal_history: Vec::new(),
            confidence: 0.7,
            last_updated: chrono::Utc::now().timestamp_micros(),
        }
    }

    /// Compute open probability: recent observations (< 24h) get 2× weight
    pub fn open_probability(&self) -> f32 {
        if self.traversal_history.is_empty() {
            return 0.5;  // Unknown
        }

        let now = chrono::Utc::now().timestamp_micros();
        let day_us = 24 * 60 * 60 * 1_000_000;

        let mut weighted_open = 0.0;
        let mut weighted_closed = 0.0;

        for traversal in &self.traversal_history {
            let age_us = now - traversal.timestamp;
            let weight = if age_us < day_us { 2.0 } else { 1.0 };

            if traversal.was_open {
                weighted_open += weight;
            } else {
                weighted_closed += weight;
            }
        }

        weighted_open / (weighted_open + weighted_closed + 1e-6)
    }

    /// Compute success rate for traversals
    pub fn traversal_success_rate(&self) -> f32 {
        if self.traversal_history.is_empty() {
            return 0.8;  // Assume likely successful
        }

        let successes = self.traversal_history.iter().filter(|t| t.success).count() as f32;
        successes / (self.traversal_history.len() as f32)
    }

    /// Compute total passage cost for path planning
    pub fn passage_cost(&self) -> f32 {
        let open_prob = self.open_probability();
        let success_rate = self.traversal_success_rate();

        // Cost increases if passage is likely closed or has low success
        (1.0 - open_prob) * 2.0 + (1.0 - success_rate)
    }

    /// Record a traversal attempt
    pub fn record_traversal(&mut self, robot_id: &str, success: bool, was_open: bool) {
        self.traversal_history.push(PassageTraversal {
            timestamp: chrono::Utc::now().timestamp_micros(),
            robot_id: robot_id.to_string(),
            success,
            was_open,
        });
        self.last_updated = chrono::Utc::now().timestamp_micros();

        // Update confidence based on success history
        let success_rate = self.traversal_success_rate();
        self.confidence = (0.5 + success_rate * 0.5).clamp(0.0, 1.0);

        // Keep history bounded (max 1000 entries)
        if self.traversal_history.len() > 1000 {
            self.traversal_history.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passage_splat_creation() {
        let passage = PassageSplat::new(PassageType::Door, [10.0, 20.0, 0.0], 1.0, 2.0, "room_a", "room_b");
        assert_eq!(passage.width_m, 1.0);
        assert_eq!(passage.connects.0, "room_a");
    }

    #[test]
    fn test_passage_open_probability_empty() {
        let passage = PassageSplat::new(PassageType::Door, [10.0, 20.0, 0.0], 1.0, 2.0, "room_a", "room_b");
        assert_eq!(passage.open_probability(), 0.5);
    }

    #[test]
    fn test_passage_traversal_success_rate() {
        let mut passage = PassageSplat::new(PassageType::Door, [10.0, 20.0, 0.0], 1.0, 2.0, "room_a", "room_b");
        passage.record_traversal("bot_01", true, true);
        passage.record_traversal("bot_02", true, true);
        passage.record_traversal("bot_03", false, false);

        let rate = passage.traversal_success_rate();
        assert!(rate > 0.5 && rate < 0.8);
    }

    #[test]
    fn test_passage_cost() {
        let mut passage = PassageSplat::new(PassageType::Door, [10.0, 20.0, 0.0], 1.0, 2.0, "room_a", "room_b");
        passage.record_traversal("bot_01", true, true);
        let cost = passage.passage_cost();
        assert!(cost < 1.0);  // Should be relatively low cost
    }
}
