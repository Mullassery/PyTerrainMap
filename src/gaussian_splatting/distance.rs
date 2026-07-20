use crate::gaussian_splatting::core::TerrainType;
use crate::gaussian_splatting::store::GaussianSplatStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 5-component path cost breakdown
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathCost {
    pub distance_cost: f32,      // Physical distance
    pub terrain_cost: f32,       // Terrain type difficulty
    pub elevation_cost: f32,     // Elevation changes
    pub passage_cost: f32,       // Doors, gates, passages
    pub uncertainty_cost: f32,   // Unknown regions
    pub total: f32,
}

/// Terrain-specific traversal cost (0 = easy, 1 = impossible)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainCostMap {
    costs: HashMap<String, f32>,
}

impl TerrainCostMap {
    pub fn new() -> Self {
        let mut costs = HashMap::new();
        costs.insert("Road".to_string(), 0.1);
        costs.insert("Grass".to_string(), 0.3);
        costs.insert("Mud".to_string(), 0.7);
        costs.insert("Water".to_string(), 0.9);
        costs.insert("Sand".to_string(), 0.5);
        costs.insert("Corridor".to_string(), 0.15);
        costs.insert("Stairs".to_string(), 0.6);
        costs.insert("Obstacle".to_string(), 1.0);
        costs.insert("ChargingStation".to_string(), 0.05);
        costs.insert("RestrictedArea".to_string(), 1.0);
        costs.insert("Unknown".to_string(), 0.4);

        TerrainCostMap { costs }
    }

    pub fn cost_for(&self, terrain_type: TerrainType) -> f32 {
        *self
            .costs
            .get(terrain_type.as_str())
            .unwrap_or(&0.4)
    }
}

impl Default for TerrainCostMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Traversability-aware distance engine with 5-component cost model
pub struct TraversabilityDistanceEngine {
    pub uncertainty_penalty: f32,  // Extra cost per unit uncertainty (default 0.5)
    pub elevation_weight: f32,     // Cost per meter of rise (default 0.1)
    pub terrain_cost_map: TerrainCostMap,
}

impl TraversabilityDistanceEngine {
    pub fn new() -> Self {
        TraversabilityDistanceEngine {
            uncertainty_penalty: 0.5,
            elevation_weight: 0.1,
            terrain_cost_map: TerrainCostMap::new(),
        }
    }

    /// Compute path cost from start to end through terrain
    pub fn path_cost(
        &self,
        from: [f64; 3],
        to: [f64; 3],
        store: &GaussianSplatStore,
    ) -> PathCost {
        // 1. Physical distance (Haversine)
        let distance_m = Self::haversine_m(from, to);
        let distance_cost = distance_m as f32 * 0.001;  // Normalize

        // 2. Elevation cost
        let elevation_delta = (to[2] - from[2]).abs() as f32;
        let elevation_cost = elevation_delta * self.elevation_weight;

        // 3-5. Sample path at 1m intervals for terrain/uncertainty
        let samples = ((distance_m / 1.0) as usize).max(2).min(100);
        let mut terrain_cost = 0.0;
        let mut uncertainty_cost = 0.0;

        for i in 0..=samples {
            let t = (i as f64) / (samples as f64);
            let sample_pos = [
                from[0] + (to[0] - from[0]) * t,
                from[1] + (to[1] - from[1]) * t,
                from[2] + (to[2] - from[2]) * t,
            ];

            // Query nearby splats
            let nearby = store.query_radius(sample_pos, 10.0);
            if !nearby.is_empty() {
                // Average terrain cost from nearby splats
                let avg_traversability =
                    nearby.iter().map(|s| s.traversability).sum::<f32>() / (nearby.len() as f32);
                terrain_cost += (1.0 - avg_traversability).clamp(0.0, 1.0);
            }

            // Uncertainty penalty
            let uncertainty = store.uncertainty_at(sample_pos);
            uncertainty_cost += uncertainty * self.uncertainty_penalty;
        }

        terrain_cost /= (samples + 1) as f32;
        uncertainty_cost /= (samples + 1) as f32;

        // Passage cost: simplified to 0 for now (would involve PassageSplat lookups)
        let passage_cost = 0.0;

        let total = distance_cost + terrain_cost + elevation_cost + passage_cost + uncertainty_cost;

        PathCost {
            distance_cost,
            terrain_cost,
            elevation_cost,
            passage_cost,
            uncertainty_cost,
            total,
        }
    }

    /// Haversine distance in meters between two 3D points
    pub fn haversine_m(pos1: [f64; 3], pos2: [f64; 3]) -> f64 {
        let lat1_rad = pos1[0].to_radians();
        let lat2_rad = pos2[0].to_radians();
        let delta_lat = (pos2[0] - pos1[0]).to_radians();
        let delta_lon = (pos2[1] - pos1[1]).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let earth_radius = 6_371_000.0;

        let horiz_dist = earth_radius * c;
        let vert_dist = (pos2[2] - pos1[2]).abs();

        (horiz_dist.powi(2) + vert_dist.powi(2)).sqrt()
    }
}

impl Default for TraversabilityDistanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_cost_map_default() {
        let map = TerrainCostMap::new();
        assert!(map.cost_for(TerrainType::Road) < map.cost_for(TerrainType::Mud));
        assert!(map.cost_for(TerrainType::ChargingStation) < 0.1);
    }

    #[test]
    fn test_distance_engine_creation() {
        let engine = TraversabilityDistanceEngine::new();
        assert_eq!(engine.uncertainty_penalty, 0.5);
        assert_eq!(engine.elevation_weight, 0.1);
    }

    #[test]
    fn test_haversine_distance() {
        let d = TraversabilityDistanceEngine::haversine_m(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        assert!(d < 1.0);  // Same location
    }
}
