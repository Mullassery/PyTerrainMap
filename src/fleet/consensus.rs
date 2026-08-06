//! Consensus Algorithms for Multi-robot Terrain State Agreement
//!
//! Phase 4.2: Enables robots to reach consensus on terrain observations
//! despite differences in local measurements and temporal ordering.

use crate::temporal::TemporalPoint;
use super::{RobotId, protocol::RobotObservation};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Consensus decision for terrain state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusDecision {
    pub point: TemporalPoint,
    pub agreeing_robots: Vec<RobotId>,
    pub confidence: f32,
    pub consensus_method: ConsensusMethod,
    pub decision_time_us: i64,
}

impl ConsensusDecision {
    /// Create new consensus decision
    pub fn new(
        point: TemporalPoint,
        agreeing_robots: Vec<RobotId>,
        method: ConsensusMethod,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        ConsensusDecision {
            point,
            agreeing_robots: agreeing_robots.clone(),
            confidence: calculate_agreement_confidence(agreeing_robots.len()),
            consensus_method: method,
            decision_time_us: now,
        }
    }

    /// Get consensus strength (0.0-1.0)
    pub fn strength(&self) -> f32 {
        self.confidence
    }
}

/// Consensus method used
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConsensusMethod {
    Majority,       // >50% of robots agree
    Supermajority,  // >66% of robots agree
    WeightedVote,   // Weighted by robot reliability scores
    BayesianFusion, // Bayesian combination of estimates
}

/// Calculate confidence from agreement count
fn calculate_agreement_confidence(agreement_count: usize) -> f32 {
    // More robots agreeing = higher confidence
    // Even 2 robots agreeing is 0.4, 3+ is 0.6+
    match agreement_count {
        0 => 0.0,
        1 => 0.2,
        2 => 0.4,
        3 => 0.6,
        4 => 0.75,
        5 => 0.85,
        n => ((n as f32).ln() / 5.0_f32.ln()).min(1.0),
    }
}

/// Observation vote from a robot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationVote {
    pub robot_id: RobotId,
    pub observation: RobotObservation,
    pub reliability: f32, // 0.0-1.0, robot's historical accuracy
    pub timestamp_us: i64,
}

impl ObservationVote {
    /// Create new vote
    pub fn new(robot_id: RobotId, observation: RobotObservation, reliability: f32) -> Self {
        let timestamp_us = observation.observation_time_us;
        ObservationVote {
            robot_id,
            observation,
            reliability: reliability.clamp(0.0, 1.0),
            timestamp_us,
        }
    }

    /// Get weighted quality
    pub fn weighted_quality(&self) -> f32 {
        (self.observation.effective_quality() + self.reliability) / 2.0
    }
}

/// Consensus engine for multi-robot agreement
pub struct ConsensusEngine {
    pub robot_id: RobotId,
    pub method: ConsensusMethod,
    votes: HashMap<String, VecDeque<ObservationVote>>, // Location key -> votes
    decisions: Vec<ConsensusDecision>,
    robot_reliability: HashMap<RobotId, f32>,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(robot_id: RobotId, method: ConsensusMethod) -> Self {
        ConsensusEngine {
            robot_id,
            method,
            votes: HashMap::new(),
            decisions: Vec::new(),
            robot_reliability: HashMap::new(),
        }
    }

    /// Set robot reliability score
    pub fn set_reliability(&mut self, robot_id: RobotId, reliability: f32) {
        self.robot_reliability.insert(robot_id, reliability.clamp(0.0, 1.0));
    }

    /// Get robot reliability (default 0.5 if unknown)
    pub fn get_reliability(&self, robot_id: RobotId) -> f32 {
        self.robot_reliability.get(&robot_id).copied().unwrap_or(0.5)
    }

    /// Add observation vote
    pub fn add_vote(&mut self, vote: ObservationVote) {
        let location_key = format!("{:.2}_{:.2}", vote.observation.point.x, vote.observation.point.y);
        self.votes
            .entry(location_key)
            .or_insert_with(VecDeque::new)
            .push_back(vote);
    }

    /// Reach consensus on observations at a location
    pub fn consensus_at_location(&mut self, x: f32, y: f32) -> Option<ConsensusDecision> {
        let location_key = format!("{:.2}_{:.2}", x, y);
        let votes = self.votes.get(&location_key)?;

        if votes.is_empty() {
            return None;
        }

        match self.method {
            ConsensusMethod::Majority => self.majority_consensus(votes),
            ConsensusMethod::Supermajority => self.supermajority_consensus(votes),
            ConsensusMethod::WeightedVote => self.weighted_consensus(votes),
            ConsensusMethod::BayesianFusion => self.bayesian_consensus(votes),
        }
    }

    /// Majority consensus (>50% agreement)
    fn majority_consensus(&self, votes: &VecDeque<ObservationVote>) -> Option<ConsensusDecision> {
        if votes.is_empty() {
            return None;
        }

        let threshold = votes.len() / 2;
        let mut z_votes = Vec::new();

        for vote in votes {
            z_votes.push((vote.observation.point.z, vote.robot_id));
        }

        // Group by z value (within tolerance)
        let mut groups: HashMap<i32, Vec<RobotId>> = HashMap::new();
        for (z, robot_id) in &z_votes {
            let z_key = (z * 100.0).round() as i32; // Group within 0.01m tolerance
            groups.entry(z_key).or_insert_with(Vec::new).push(*robot_id);
        }

        // Find largest group
        let (z_key, agreeing_robots) = groups.iter().max_by_key(|(_, robots)| robots.len())?;

        if agreeing_robots.len() > threshold {
            // Construct consensus point
            let avg_z = (*z_key as f32) / 100.0;
            let point = TemporalPoint::new(
                votes[0].observation.point.x,
                votes[0].observation.point.y,
                avg_z,
                votes[0].observation.point.timestamp,
                0.7, // Consensus quality
            );

            Some(ConsensusDecision::new(
                point,
                agreeing_robots.clone(),
                ConsensusMethod::Majority,
            ))
        } else {
            None
        }
    }

    /// Supermajority consensus (>66% agreement)
    fn supermajority_consensus(&self, votes: &VecDeque<ObservationVote>) -> Option<ConsensusDecision> {
        let threshold = (votes.len() * 2 / 3).max(1);
        let mut z_votes = Vec::new();

        for vote in votes {
            z_votes.push((vote.observation.point.z, vote.robot_id));
        }

        let mut groups: HashMap<i32, Vec<RobotId>> = HashMap::new();
        for (z, robot_id) in &z_votes {
            let z_key = (z * 100.0).round() as i32;
            groups.entry(z_key).or_insert_with(Vec::new).push(*robot_id);
        }

        let (z_key, agreeing_robots) = groups.iter().max_by_key(|(_, robots)| robots.len())?;

        if agreeing_robots.len() > threshold {
            let avg_z = (*z_key as f32) / 100.0;
            let point = TemporalPoint::new(
                votes[0].observation.point.x,
                votes[0].observation.point.y,
                avg_z,
                votes[0].observation.point.timestamp,
                0.8, // Higher quality for supermajority
            );

            Some(ConsensusDecision::new(
                point,
                agreeing_robots.clone(),
                ConsensusMethod::Supermajority,
            ))
        } else {
            None
        }
    }

    /// Weighted consensus (by robot reliability)
    fn weighted_consensus(&self, votes: &VecDeque<ObservationVote>) -> Option<ConsensusDecision> {
        if votes.is_empty() {
            return None;
        }

        let mut weighted_z = 0.0;
        let mut total_weight = 0.0;
        let mut agreeing_robots = Vec::new();

        for vote in votes {
            let weight = self.get_reliability(vote.robot_id);
            weighted_z += vote.observation.point.z * weight;
            total_weight += weight;
            agreeing_robots.push(vote.robot_id);
        }

        if total_weight > 0.0 {
            let avg_z = weighted_z / total_weight;
            let point = TemporalPoint::new(
                votes[0].observation.point.x,
                votes[0].observation.point.y,
                avg_z,
                votes[0].observation.point.timestamp,
                (0.7 + total_weight) / 2.0,
            );

            Some(ConsensusDecision::new(
                point,
                agreeing_robots,
                ConsensusMethod::WeightedVote,
            ))
        } else {
            None
        }
    }

    /// Bayesian fusion of observations
    fn bayesian_consensus(&self, votes: &VecDeque<ObservationVote>) -> Option<ConsensusDecision> {
        if votes.is_empty() {
            return None;
        }

        // Bayesian: multiply confidence scores, normalize
        let mut posterior = 1.0;
        let mut agreeing_robots = Vec::new();
        let mut weighted_z = 0.0;

        for vote in votes {
            let likelihood = vote.weighted_quality();
            posterior *= likelihood;
            agreeing_robots.push(vote.robot_id);
            weighted_z += vote.observation.point.z * likelihood;
        }

        if posterior > 0.0 {
            let avg_z = weighted_z / agreeing_robots.len() as f32;
            let point = TemporalPoint::new(
                votes[0].observation.point.x,
                votes[0].observation.point.y,
                avg_z,
                votes[0].observation.point.timestamp,
                posterior.min(1.0),
            );

            Some(ConsensusDecision::new(
                point,
                agreeing_robots,
                ConsensusMethod::BayesianFusion,
            ))
        } else {
            None
        }
    }

    /// Get all decisions
    pub fn decisions(&self) -> &[ConsensusDecision] {
        &self.decisions
    }

    /// Get consensus statistics
    pub fn statistics(&self) -> ConsensusStatistics {
        ConsensusStatistics {
            total_votes: self.votes.values().map(|v| v.len()).sum::<usize>() as u32,
            locations_voted: self.votes.len() as u32,
            decisions_made: self.decisions.len() as u32,
            average_agreement: if self.decisions.is_empty() {
                0.0
            } else {
                self.decisions.iter().map(|d| d.agreeing_robots.len() as f32).sum::<f32>()
                    / self.decisions.len() as f32
            },
        }
    }
}

impl Default for ConsensusEngine {
    fn default() -> Self {
        Self::new(RobotId::new(0), ConsensusMethod::Majority)
    }
}

/// Consensus statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusStatistics {
    pub total_votes: u32,
    pub locations_voted: u32,
    pub decisions_made: u32,
    pub average_agreement: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::TemporalPoint;

    #[test]
    fn test_consensus_decision() {
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let robots = vec![RobotId::new(1), RobotId::new(2)];
        let decision = ConsensusDecision::new(point, robots, ConsensusMethod::Majority);

        assert_eq!(decision.agreeing_robots.len(), 2);
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_confidence_calculation() {
        assert_eq!(calculate_agreement_confidence(0), 0.0);
        assert_eq!(calculate_agreement_confidence(1), 0.2);
        assert_eq!(calculate_agreement_confidence(2), 0.4);
        assert!(calculate_agreement_confidence(5) > 0.8);
    }

    #[test]
    fn test_observation_vote() {
        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot_id, point, 0.9);
        let vote = ObservationVote::new(robot_id, obs, 0.8);

        assert_eq!(vote.robot_id, robot_id);
        assert!(vote.weighted_quality() > 0.0);
    }

    #[test]
    fn test_consensus_engine_majority() {
        let mut engine = ConsensusEngine::new(RobotId::new(1), ConsensusMethod::Majority);

        let point1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs1 = RobotObservation::new(RobotId::new(1), point1, 0.9);
        let vote1 = ObservationVote::new(RobotId::new(1), obs1, 0.8);
        engine.add_vote(vote1);

        let point2 = TemporalPoint::new(1.0, 2.0, 3.05, 1000, 0.8);
        let obs2 = RobotObservation::new(RobotId::new(2), point2, 0.9);
        let vote2 = ObservationVote::new(RobotId::new(2), obs2, 0.8);
        engine.add_vote(vote2);

        let consensus = engine.consensus_at_location(1.0, 2.0);
        assert!(consensus.is_some());
    }

    #[test]
    fn test_consensus_engine_weighted() {
        let mut engine = ConsensusEngine::new(RobotId::new(1), ConsensusMethod::WeightedVote);
        engine.set_reliability(RobotId::new(1), 0.9);
        engine.set_reliability(RobotId::new(2), 0.5);

        let point1 = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs1 = RobotObservation::new(RobotId::new(1), point1, 0.9);
        let vote1 = ObservationVote::new(RobotId::new(1), obs1, 0.9);
        engine.add_vote(vote1);

        let point2 = TemporalPoint::new(1.0, 2.0, 3.5, 1000, 0.8);
        let obs2 = RobotObservation::new(RobotId::new(2), point2, 0.9);
        let vote2 = ObservationVote::new(RobotId::new(2), obs2, 0.5);
        engine.add_vote(vote2);

        let consensus = engine.consensus_at_location(1.0, 2.0);
        assert!(consensus.is_some());
        // Higher reliability robot should pull average closer to their vote
    }

    #[test]
    fn test_consensus_statistics() {
        let mut engine = ConsensusEngine::new(RobotId::new(1), ConsensusMethod::Majority);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(RobotId::new(1), point, 0.9);
        let vote = ObservationVote::new(RobotId::new(1), obs, 0.8);
        engine.add_vote(vote);

        let stats = engine.statistics();
        assert_eq!(stats.total_votes, 1);
        assert_eq!(stats.locations_voted, 1);
    }
}
