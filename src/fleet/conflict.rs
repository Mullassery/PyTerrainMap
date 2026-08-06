//! Conflict Resolution for Disagreements in Observations
//!
//! Phase 4.4: Handles situations where robots observe conflicting
//! terrain states and must reconcile differences.

use crate::temporal::TemporalPoint;
use super::RobotId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict between robot observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationConflict {
    pub location: (f32, f32), // (x, y)
    pub observations: Vec<(RobotId, TemporalPoint)>,
    pub conflict_type: ConflictType,
    pub magnitude: f32, // How severe the conflict is (0.0-1.0)
}

impl ObservationConflict {
    /// Create new conflict
    pub fn new(location: (f32, f32), observations: Vec<(RobotId, TemporalPoint)>) -> Self {
        let conflict_type = Self::detect_conflict_type(&observations);
        let magnitude = Self::calculate_magnitude(&observations);

        ObservationConflict {
            location,
            observations,
            conflict_type,
            magnitude,
        }
    }

    /// Detect type of conflict from observations
    fn detect_conflict_type(observations: &[(RobotId, TemporalPoint)]) -> ConflictType {
        if observations.len() < 2 {
            return ConflictType::ZValueDifference;
        }

        let z_values: Vec<f32> = observations.iter().map(|(_, p)| p.z).collect();
        let qualities: Vec<f32> = observations.iter().map(|(_, p)| p.quality).collect();
        let times: Vec<i64> = observations.iter().map(|(_, p)| p.timestamp).collect();

        // Calculate variances
        let z_mean = z_values.iter().sum::<f32>() / z_values.len() as f32;
        let z_var = z_values.iter().map(|z| (z - z_mean).powi(2)).sum::<f32>() / z_values.len() as f32;

        let q_mean = qualities.iter().sum::<f32>() / qualities.len() as f32;
        let q_var = qualities.iter().map(|q| (q - q_mean).powi(2)).sum::<f32>() / qualities.len() as f32;

        let time_min = times.iter().min().unwrap();
        let time_max = times.iter().max().unwrap();
        let time_span = (time_max - time_min).abs() as f32 / 1_000_000.0; // Convert to seconds

        if time_span > 10.0 {
            ConflictType::TimingDifference
        } else if q_var > z_var {
            ConflictType::QualityDifference
        } else {
            ConflictType::ZValueDifference
        }
    }

    /// Calculate conflict magnitude
    fn calculate_magnitude(observations: &[(RobotId, TemporalPoint)]) -> f32 {
        if observations.len() < 2 {
            return 0.0;
        }

        let z_values: Vec<f32> = observations.iter().map(|(_, p)| p.z).collect();
        let z_mean = z_values.iter().sum::<f32>() / z_values.len() as f32;
        let z_range = z_values.iter().map(|z| (z - z_mean).abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0);

        (z_range / 10.0).min(1.0) // Normalize: 10m difference = 1.0 magnitude
    }
}

/// Type of conflict
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    ZValueDifference, // Different z (elevation) values
    QualityDifference, // Different quality scores
    TimingDifference, // Significant time offset
}

/// Conflict resolution strategy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    TakeNewest,         // Use most recent observation
    TakeHighestQuality, // Use highest quality observation
    Average,            // Average all observations
    Weighted,           // Weighted average by quality
}

/// Resolved conflict result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub original_conflict: ObservationConflict,
    pub resolved_point: TemporalPoint,
    pub strategy_used: ResolutionStrategy,
    pub contributing_robots: Vec<RobotId>,
}

/// Conflict resolver
pub struct ConflictResolver {
    strategy: ResolutionStrategy,
    robot_reliability: HashMap<RobotId, f32>,
    resolution_history: Vec<ConflictResolution>,
}

impl ConflictResolver {
    /// Create new conflict resolver
    pub fn new(strategy: ResolutionStrategy) -> Self {
        ConflictResolver {
            strategy,
            robot_reliability: HashMap::new(),
            resolution_history: Vec::new(),
        }
    }

    /// Set robot reliability score
    pub fn set_reliability(&mut self, robot_id: RobotId, reliability: f32) {
        self.robot_reliability.insert(robot_id, reliability.clamp(0.0, 1.0));
    }

    /// Get robot reliability (default 0.5 if unknown)
    fn get_reliability(&self, robot_id: RobotId) -> f32 {
        self.robot_reliability.get(&robot_id).copied().unwrap_or(0.5)
    }

    /// Resolve a conflict
    pub fn resolve(&mut self, conflict: ObservationConflict) -> ConflictResolution {
        let resolved_point = match self.strategy {
            ResolutionStrategy::TakeNewest => self.resolve_newest(&conflict),
            ResolutionStrategy::TakeHighestQuality => self.resolve_highest_quality(&conflict),
            ResolutionStrategy::Average => self.resolve_average(&conflict),
            ResolutionStrategy::Weighted => self.resolve_weighted(&conflict),
        };

        let contributing_robots = conflict.observations.iter().map(|(id, _)| *id).collect();

        let resolution = ConflictResolution {
            original_conflict: conflict,
            resolved_point,
            strategy_used: self.strategy.clone(),
            contributing_robots,
        };

        self.resolution_history.push(resolution.clone());
        resolution
    }

    /// Resolve by taking newest observation
    fn resolve_newest(&self, conflict: &ObservationConflict) -> TemporalPoint {
        conflict
            .observations
            .iter()
            .max_by_key(|(_, p)| p.timestamp)
            .map(|(_, p)| *p)
            .unwrap_or_else(|| conflict.observations[0].1)
    }

    /// Resolve by taking highest quality observation
    fn resolve_highest_quality(&self, conflict: &ObservationConflict) -> TemporalPoint {
        conflict
            .observations
            .iter()
            .max_by(|(_, p1), (_, p2)| p1.quality.partial_cmp(&p2.quality).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, p)| *p)
            .unwrap_or_else(|| conflict.observations[0].1)
    }

    /// Resolve by averaging all observations
    fn resolve_average(&self, conflict: &ObservationConflict) -> TemporalPoint {
        let count = conflict.observations.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        let mut sum_quality = 0.0;
        let avg_time = conflict
            .observations
            .iter()
            .map(|(_, p)| p.timestamp)
            .sum::<i64>()
            / conflict.observations.len() as i64;

        for (_, point) in &conflict.observations {
            sum_x += point.x;
            sum_y += point.y;
            sum_z += point.z;
            sum_quality += point.quality;
        }

        TemporalPoint::new(
            sum_x / count,
            sum_y / count,
            sum_z / count,
            avg_time,
            (sum_quality / count).min(1.0),
        )
    }

    /// Resolve by weighted average (by robot reliability and quality)
    fn resolve_weighted(&self, conflict: &ObservationConflict) -> TemporalPoint {
        let mut weighted_x = 0.0;
        let mut weighted_y = 0.0;
        let mut weighted_z = 0.0;
        let mut weighted_quality = 0.0;
        let mut total_weight = 0.0;
        let mut weighted_time = 0.0;

        for (robot_id, point) in &conflict.observations {
            let robot_weight = self.get_reliability(*robot_id);
            let point_weight = point.quality;
            let weight = robot_weight * point_weight;

            weighted_x += point.x * weight;
            weighted_y += point.y * weight;
            weighted_z += point.z * weight;
            weighted_quality += point.quality * weight;
            weighted_time += point.timestamp as f32 * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            TemporalPoint::new(
                weighted_x / total_weight,
                weighted_y / total_weight,
                weighted_z / total_weight,
                (weighted_time / total_weight) as i64,
                (weighted_quality / total_weight).min(1.0),
            )
        } else {
            conflict.observations[0].1
        }
    }

    /// Get resolution statistics
    pub fn statistics(&self) -> ConflictStatistics {
        let total_conflicts = self.resolution_history.len() as u32;
        let average_magnitude = if total_conflicts > 0 {
            self.resolution_history
                .iter()
                .map(|r| r.original_conflict.magnitude)
                .sum::<f32>()
                / total_conflicts as f32
        } else {
            0.0
        };

        let conflict_types = self.resolution_history
            .iter()
            .fold(HashMap::new(), |mut map, r| {
                *map.entry(format!("{:?}", r.original_conflict.conflict_type)).or_insert(0u32) += 1;
                map
            });

        ConflictStatistics {
            total_conflicts,
            average_magnitude,
            conflict_types,
        }
    }

    /// Get resolution history
    pub fn history(&self) -> &[ConflictResolution] {
        &self.resolution_history
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ResolutionStrategy::Weighted)
    }
}

impl Clone for ConflictResolver {
    fn clone(&self) -> Self {
        ConflictResolver {
            strategy: self.strategy.clone(),
            robot_reliability: self.robot_reliability.clone(),
            resolution_history: self.resolution_history.clone(),
        }
    }
}

/// Conflict statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictStatistics {
    pub total_conflicts: u32,
    pub average_magnitude: f32,
    pub conflict_types: HashMap<String, u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_conflict() {
        let loc = (1.0, 2.0);
        let p1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let p2 = TemporalPoint::new(1.0, 2.0, 3.5, 1000, 0.8);

        let conflict = ObservationConflict::new(loc, vec![(RobotId::new(1), p1), (RobotId::new(2), p2)]);

        assert_eq!(conflict.location, loc);
        assert!(conflict.magnitude > 0.0);
    }

    #[test]
    fn test_conflict_resolver_newest() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::TakeNewest);
        let p1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let p2 = TemporalPoint::new(1.0, 2.0, 3.5, 2000, 0.8);

        let conflict = ObservationConflict::new((1.0, 2.0), vec![(RobotId::new(1), p1), (RobotId::new(2), p2)]);
        let resolution = resolver.resolve(conflict);

        assert!((resolution.resolved_point.z - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_conflict_resolver_quality() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::TakeHighestQuality);
        let p1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.5);
        let p2 = TemporalPoint::new(1.0, 2.0, 3.5, 1000, 0.9);

        let conflict = ObservationConflict::new((1.0, 2.0), vec![(RobotId::new(1), p1), (RobotId::new(2), p2)]);
        let resolution = resolver.resolve(conflict);

        assert!((resolution.resolved_point.z - 3.5).abs() < 0.01);
    }

    #[test]
    fn test_conflict_resolver_average() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::Average);
        let p1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let p2 = TemporalPoint::new(1.0, 2.0, 3.4, 1000, 0.8);

        let conflict = ObservationConflict::new((1.0, 2.0), vec![(RobotId::new(1), p1), (RobotId::new(2), p2)]);
        let resolution = resolver.resolve(conflict);

        assert!((resolution.resolved_point.z - 3.2).abs() < 0.01);
    }

    #[test]
    fn test_conflict_resolver_statistics() {
        let mut resolver = ConflictResolver::new(ResolutionStrategy::Average);
        let p1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let p2 = TemporalPoint::new(1.0, 2.0, 3.5, 1000, 0.8);

        let conflict = ObservationConflict::new((1.0, 2.0), vec![(RobotId::new(1), p1), (RobotId::new(2), p2)]);
        resolver.resolve(conflict);

        let stats = resolver.statistics();
        assert_eq!(stats.total_conflicts, 1);
    }
}
