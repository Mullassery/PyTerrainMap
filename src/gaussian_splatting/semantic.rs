use crate::gaussian_splatting::core::{TerrainGaussian, TerrainType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mission-specific terrain preferences for a bot type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotMissionProfile {
    pub bot_type: String,
    pub preferred_terrain: Vec<(String, f32)>,  // (terrain_type, weight 0-1)
    pub avoid_terrain: Vec<TerrainType>,
    pub mission_name: String,
}

impl BotMissionProfile {
    pub fn new(bot_type: &str, mission_name: &str) -> Self {
        BotMissionProfile {
            bot_type: bot_type.to_string(),
            preferred_terrain: vec![],
            avoid_terrain: vec![],
            mission_name: mission_name.to_string(),
        }
    }
}

/// Semantic mapping: terrain classification + bot mission profiles
pub struct SemanticGaussianMapper {
    pub profiles: HashMap<String, BotMissionProfile>,
}

impl SemanticGaussianMapper {
    /// Create mapper with default bot profiles
    pub fn new_with_defaults() -> Self {
        let mut profiles = HashMap::new();

        // Delivery bot: prefers roads, avoids obstacles
        let mut delivery = BotMissionProfile::new("DeliveryBot", "package_delivery");
        delivery.preferred_terrain = vec![
            ("Road".to_string(), 1.0),
            ("Corridor".to_string(), 0.9),
            ("ChargingStation".to_string(), 0.8),
        ];
        delivery.avoid_terrain = vec![
            TerrainType::Water,
            TerrainType::Obstacle,
            TerrainType::RestrictedArea,
        ];
        profiles.insert("DeliveryBot".to_string(), delivery);

        // Security bot: prefers corridors, aisles
        let mut security = BotMissionProfile::new("SecurityBot", "security_patrol");
        security.preferred_terrain = vec![
            ("Corridor".to_string(), 1.0),
            ("Road".to_string(), 0.8),
            ("Grass".to_string(), 0.5),
        ];
        security.avoid_terrain = vec![TerrainType::Water, TerrainType::Obstacle];
        profiles.insert("SecurityBot".to_string(), security);

        // Exploration bot: prefers unknown regions
        let mut exploration = BotMissionProfile::new("ExplorationBot", "exploration");
        exploration.preferred_terrain = vec![("Unknown".to_string(), 1.0)];
        exploration.avoid_terrain = vec![TerrainType::Obstacle, TerrainType::RestrictedArea];
        profiles.insert("ExplorationBot".to_string(), exploration);

        SemanticGaussianMapper { profiles }
    }

    /// Compute mission-specific terrain cost
    pub fn mission_terrain_cost(&self, bot_type: &str, splat: &TerrainGaussian) -> f32 {
        match self.profiles.get(bot_type) {
            None => 0.5,  // Default mid-cost for unknown bot types
            Some(profile) => {
                // Check if terrain is avoided
                if profile.avoid_terrain.contains(&splat.terrain_type) {
                    return 0.9;  // Very high cost
                }

                // Check preferred terrain
                for (terrain_str, weight) in &profile.preferred_terrain {
                    if splat.terrain_type.as_str() == terrain_str.as_str() {
                        return (1.0 - weight).clamp(0.0, 1.0);  // Lower cost = more preferred
                    }
                }

                // Default cost if not in preferences
                0.5
            }
        }
    }

    /// Classify terrain from metadata (placeholder)
    pub fn classify_from_metadata(&self, _metadata: &HashMap<String, String>) -> TerrainType {
        TerrainType::Unknown(0)
    }

    /// Filter splats to those preferred by a bot type
    pub fn preferred_splats<'a>(
        &self,
        bot_type: &str,
        splats: &[&'a TerrainGaussian],
    ) -> Vec<&'a TerrainGaussian> {
        splats
            .iter()
            .filter(|splat| {
                if let Some(profile) = self.profiles.get(bot_type) {
                    !profile.avoid_terrain.contains(&splat.terrain_type)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }
}

impl Default for SemanticGaussianMapper {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_mapper_defaults() {
        let mapper = SemanticGaussianMapper::new_with_defaults();
        assert!(mapper.profiles.contains_key("DeliveryBot"));
        assert!(mapper.profiles.contains_key("SecurityBot"));
        assert!(mapper.profiles.contains_key("ExplorationBot"));
    }

    #[test]
    fn test_mission_terrain_cost_delivery() {
        let mapper = SemanticGaussianMapper::new_with_defaults();
        let road = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let cost = mapper.mission_terrain_cost("DeliveryBot", &road);
        assert!(cost < 0.5);  // Road should be preferred
    }

    #[test]
    fn test_mission_terrain_cost_avoid() {
        let mapper = SemanticGaussianMapper::new_with_defaults();
        let mut water = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.2);
        water.terrain_type = TerrainType::Water;
        let cost = mapper.mission_terrain_cost("DeliveryBot", &water);
        assert!(cost > 0.8);  // Water should be avoided
    }

    #[test]
    fn test_preferred_splats() {
        let mapper = SemanticGaussianMapper::new_with_defaults();
        let road = TerrainGaussian::from_point_observation([0.0, 0.0, 0.0], "bot_01", 0.8);
        let water = {
            let mut w = TerrainGaussian::from_point_observation([1.0, 1.0, 0.0], "bot_01", 0.2);
            w.terrain_type = TerrainType::Water;
            w
        };

        let splats = vec![&road, &water];
        let preferred = mapper.preferred_splats("DeliveryBot", &splats);
        assert_eq!(preferred.len(), 1);
        assert_eq!(preferred[0].position, road.position);
    }
}
