//! Fleet-wide statistics aggregation
//!
//! Collect and aggregate statistics from all fleet observations for better predictions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::patterns::EnvironmentType;
use crate::traversability::TraversabilityObservation;

/// Robot capability profile
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotProfile {
    pub robot_type: String,
    pub success_rate: f32,
    pub failure_history: Vec<String>,
    pub preferred_environments: Vec<(String, f32)>,
    pub average_energy_consumption: f32,
    pub observation_count: u32,
}

impl RobotProfile {
    /// Create a new robot profile
    pub fn new(robot_type: String) -> Self {
        RobotProfile {
            robot_type,
            success_rate: 0.5,
            failure_history: Vec::new(),
            preferred_environments: Vec::new(),
            average_energy_consumption: 0.1,
            observation_count: 0,
        }
    }

    /// Update from an observation
    pub fn update_from_observation(&mut self, obs: &TraversabilityObservation) {
        self.observation_count += 1;

        if obs.is_failure() {
            if let crate::traversability::TraversalOutcome::Failure { reason } = &obs.outcome {
                self.failure_history.push(reason.clone());
            }
        }

        // Update success rate
        let success_count = self.observation_count as f32
            - self.failure_history.len() as f32;
        self.success_rate = success_count / self.observation_count as f32;
    }
}

/// Fleet-wide statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetStatistics {
    pub total_observations: u32,
    pub success_rate_by_environment: HashMap<String, f32>,
    pub average_traversal_time_by_connector: HashMap<String, u32>,
    pub energy_cost_by_terrain: HashMap<String, f32>,
    pub failure_reasons: HashMap<String, u32>,
    pub robot_capability_profiles: HashMap<String, RobotProfile>,
}

impl FleetStatistics {
    /// Create new fleet statistics
    pub fn new() -> Self {
        FleetStatistics {
            total_observations: 0,
            success_rate_by_environment: HashMap::new(),
            average_traversal_time_by_connector: HashMap::new(),
            energy_cost_by_terrain: HashMap::new(),
            failure_reasons: HashMap::new(),
            robot_capability_profiles: HashMap::new(),
        }
    }

    /// Update from an observation
    pub fn update_from_observation(&mut self, obs: &TraversabilityObservation) {
        self.total_observations += 1;

        // Update robot profile
        let profile = self.robot_capability_profiles
            .entry(obs.robot_type.clone())
            .or_insert_with(|| RobotProfile::new(obs.robot_type.clone()));
        profile.update_from_observation(obs);

        // Track failures
        if let crate::traversability::TraversalOutcome::Failure { reason } = &obs.outcome {
            *self.failure_reasons.entry(reason.clone()).or_insert(0) += 1;
        }

        // Track traversal times for successful traversals
        if let crate::traversability::TraversalOutcome::Success { time_ms, .. } = &obs.outcome {
            let edge_type = "generic";  // Placeholder - would use actual edge type
            let current_avg = self.average_traversal_time_by_connector
                .get(edge_type)
                .copied()
                .unwrap_or(0);

            let new_avg = if current_avg == 0 {
                *time_ms
            } else {
                ((current_avg as u64 + *time_ms as u64) / 2) as u32
            };

            self.average_traversal_time_by_connector
                .insert(edge_type.to_string(), new_avg);
        }
    }

    /// Get success probability for environment and robot type
    pub fn success_probability_for(
        &self,
        environment: &str,
        robot_type: &str,
    ) -> f32 {
        // Check robot-specific success rate
        if let Some(profile) = self.robot_capability_profiles.get(robot_type) {
            let env_success = self.success_rate_by_environment
                .get(environment)
                .copied()
                .unwrap_or(0.5);

            // Blend robot success rate with environment success rate
            (profile.success_rate * 0.6 + env_success * 0.4).max(0.0).min(1.0)
        } else {
            self.success_rate_by_environment
                .get(environment)
                .copied()
                .unwrap_or(0.5)
        }
    }

    /// Get expected cost for a connector and robot type
    pub fn expected_cost_for(
        &self,
        connector_type: &str,
        robot_type: &str,
    ) -> (u32, f32) {
        let time = self.average_traversal_time_by_connector
            .get(connector_type)
            .copied()
            .unwrap_or(1000);

        let energy = if let Some(profile) = self.robot_capability_profiles.get(robot_type) {
            profile.average_energy_consumption
        } else {
            0.1
        };

        (time, energy)
    }

    /// Get most common failure reason
    pub fn most_common_failure(&self) -> Option<(String, u32)> {
        self.failure_reasons
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(reason, count)| (reason.clone(), *count))
    }

    /// Get robot type with highest success rate
    pub fn best_robot_type(&self) -> Option<(String, f32)> {
        self.robot_capability_profiles
            .iter()
            .max_by(|(_, a), (_, b)| {
                a.success_rate.partial_cmp(&b.success_rate).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, profile)| (name.clone(), profile.success_rate))
    }
}

impl Default for FleetStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_profile_creation() {
        let profile = RobotProfile::new("wheeled".to_string());
        assert_eq!(profile.robot_type, "wheeled");
        assert_eq!(profile.success_rate, 0.5);
        assert_eq!(profile.observation_count, 0);
    }

    #[test]
    fn test_fleet_statistics_creation() {
        let stats = FleetStatistics::new();
        assert_eq!(stats.total_observations, 0);
        assert_eq!(stats.robot_capability_profiles.len(), 0);
    }

    #[test]
    fn test_success_probability() {
        let stats = FleetStatistics::new();
        let prob = stats.success_probability_for("office", "wheeled");
        assert_eq!(prob, 0.5);  // Default when no data
    }

    #[test]
    fn test_expected_cost() {
        let stats = FleetStatistics::new();
        let (time, energy) = stats.expected_cost_for("door", "wheeled");
        assert_eq!(time, 1000);
        assert_eq!(energy, 0.1);
    }

    #[test]
    fn test_most_common_failure_empty() {
        let stats = FleetStatistics::new();
        assert!(stats.most_common_failure().is_none());
    }

    #[test]
    fn test_best_robot_type_empty() {
        let stats = FleetStatistics::new();
        assert!(stats.best_robot_type().is_none());
    }
}
