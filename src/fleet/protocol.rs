//! Multi-robot Observation Protocol
//!
//! Defines protocol for robots to share observations, synchronize state,
//! and maintain consistent distributed maps.

use crate::temporal::TemporalPoint;
use super::RobotId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Timestamped observation from a robot
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RobotObservation {
    pub robot_id: RobotId,
    pub point: TemporalPoint,
    pub observation_time_us: i64,
    pub local_sequence: u64, // Sequence number in robot's local clock
    pub confidence: f32, // Additional confidence (0.0-1.0)
}

impl RobotObservation {
    /// Create new observation
    pub fn new(
        robot_id: RobotId,
        point: TemporalPoint,
        confidence: f32,
    ) -> Self {
        RobotObservation {
            robot_id,
            point,
            observation_time_us: point.timestamp,
            local_sequence: 0,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Get effective quality (combines point quality and robot confidence)
    pub fn effective_quality(&self) -> f32 {
        (self.point.quality + self.confidence) / 2.0
    }
}

/// Message exchanged between robots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationMessage {
    pub from_robot: RobotId,
    pub to_robot: Option<RobotId>, // None = broadcast
    pub message_id: u64,
    pub observations: Vec<RobotObservation>,
    pub message_timestamp_us: i64,
    pub sequence_number: u64,
}

impl ObservationMessage {
    /// Create new message
    pub fn new(from_robot: RobotId, observations: Vec<RobotObservation>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        ObservationMessage {
            from_robot,
            to_robot: None,
            message_id: now as u64,
            observations,
            message_timestamp_us: now,
            sequence_number: 0,
        }
    }

    /// Create unicast message (to specific robot)
    pub fn unicast(from_robot: RobotId, to_robot: RobotId, observations: Vec<RobotObservation>) -> Self {
        let mut msg = Self::new(from_robot, observations);
        msg.to_robot = Some(to_robot);
        msg
    }

    /// Get number of observations
    pub fn observation_count(&self) -> u32 {
        self.observations.len() as u32
    }

    /// Get average quality of observations
    pub fn average_quality(&self) -> f32 {
        if self.observations.is_empty() {
            return 0.0;
        }
        self.observations
            .iter()
            .map(|o| o.effective_quality())
            .sum::<f32>()
            / self.observations.len() as f32
    }
}

/// Observation receipt confirmation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservationAck {
    pub message_id: u64,
    pub from_robot: RobotId,
    pub received_at_us: i64,
    pub observations_accepted: u32,
    pub observations_rejected: u32,
    pub rejection_reasons: Vec<String>,
}

impl ObservationAck {
    /// Create new acknowledgment
    pub fn new(message_id: u64, from_robot: RobotId) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64;

        ObservationAck {
            message_id,
            from_robot,
            received_at_us: now,
            observations_accepted: 0,
            observations_rejected: 0,
            rejection_reasons: Vec::new(),
        }
    }

    /// Get acceptance rate
    pub fn acceptance_rate(&self) -> f32 {
        let total = self.observations_accepted as f32 + self.observations_rejected as f32;
        if total == 0.0 {
            return 1.0;
        }
        self.observations_accepted as f32 / total
    }
}

/// Protocol state for tracking received messages
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolState {
    pub robot_id: RobotId,
    pub received_messages: HashMap<u64, ObservationMessage>, // message_id -> message
    pub sent_messages: HashMap<u64, ObservationMessage>,
    pub received_acks: HashMap<u64, ObservationAck>,
    pub last_sync_us: i64,
}

impl ProtocolState {
    /// Create new protocol state
    pub fn new(robot_id: RobotId) -> Self {
        ProtocolState {
            robot_id,
            received_messages: HashMap::new(),
            sent_messages: HashMap::new(),
            received_acks: HashMap::new(),
            last_sync_us: 0,
        }
    }

    /// Record received message
    pub fn receive_message(&mut self, message: ObservationMessage) -> bool {
        if self.received_messages.contains_key(&message.message_id) {
            return false; // Duplicate
        }
        self.received_messages.insert(message.message_id, message);
        true
    }

    /// Record sent message
    pub fn send_message(&mut self, message: ObservationMessage) {
        self.sent_messages.insert(message.message_id, message);
    }

    /// Record acknowledgment
    pub fn receive_ack(&mut self, ack: ObservationAck) {
        self.received_acks.insert(ack.message_id, ack);
    }

    /// Get unacknowledged messages
    pub fn unacknowledged_messages(&self) -> Vec<u64> {
        self.sent_messages
            .iter()
            .filter(|(msg_id, _)| !self.received_acks.contains_key(msg_id))
            .map(|(msg_id, _)| *msg_id)
            .collect()
    }

    /// Get messages from specific robot
    pub fn messages_from(&self, robot_id: RobotId) -> Vec<ObservationMessage> {
        self.received_messages
            .values()
            .filter(|msg| msg.from_robot == robot_id)
            .cloned()
            .collect()
    }

    /// Get protocol statistics
    pub fn statistics(&self) -> ProtocolStatistics {
        ProtocolStatistics {
            robot_id: self.robot_id,
            messages_received: self.received_messages.len() as u32,
            messages_sent: self.sent_messages.len() as u32,
            acks_received: self.received_acks.len() as u32,
            total_observations_received: self.received_messages
                .values()
                .map(|m| m.observation_count())
                .sum(),
            average_message_quality: if self.received_messages.is_empty() {
                0.0
            } else {
                self.received_messages
                    .values()
                    .map(|m| m.average_quality())
                    .sum::<f32>()
                    / self.received_messages.len() as f32
            },
        }
    }

    /// Clear old messages (older than threshold_us)
    pub fn cleanup_old_messages(&mut self, threshold_us: i64) {
        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as i64 - threshold_us;

        self.received_messages.retain(|_, msg| msg.message_timestamp_us > cutoff_time);
        self.sent_messages.retain(|_, msg| msg.message_timestamp_us > cutoff_time);
    }
}

/// Protocol statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtocolStatistics {
    pub robot_id: RobotId,
    pub messages_received: u32,
    pub messages_sent: u32,
    pub acks_received: u32,
    pub total_observations_received: u32,
    pub average_message_quality: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_observation() {
        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot_id, point, 0.9);

        assert_eq!(obs.robot_id, robot_id);
        assert_eq!(obs.effective_quality(), (0.8 + 0.9) / 2.0);
    }

    #[test]
    fn test_observation_message() {
        let robot_id = RobotId::new(1);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot_id, point, 0.9);

        let msg = ObservationMessage::new(robot_id, vec![obs]);
        assert_eq!(msg.observation_count(), 1);
        assert!((msg.average_quality() - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_unicast_message() {
        let from_id = RobotId::new(1);
        let to_id = RobotId::new(2);
        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(from_id, point, 0.9);

        let msg = ObservationMessage::unicast(from_id, to_id, vec![obs]);
        assert_eq!(msg.to_robot, Some(to_id));
    }

    #[test]
    fn test_observation_ack() {
        let from_id = RobotId::new(1);
        let mut ack = ObservationAck::new(123, from_id);
        ack.observations_accepted = 10;
        ack.observations_rejected = 2;

        assert!((ack.acceptance_rate() - (10.0 / 12.0)).abs() < 0.01);
    }

    #[test]
    fn test_protocol_state() {
        let robot_id = RobotId::new(1);
        let mut state = ProtocolState::new(robot_id);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot_id, point, 0.9);
        let msg = ObservationMessage::new(robot_id, vec![obs]);

        assert!(state.receive_message(msg.clone()));
        assert!(!state.receive_message(msg)); // Duplicate
    }

    #[test]
    fn test_protocol_state_unacknowledged() {
        let robot_id = RobotId::new(1);
        let mut state = ProtocolState::new(robot_id);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot_id, point, 0.9);
        let msg = ObservationMessage::new(robot_id, vec![obs]);

        let msg_id = msg.message_id;
        state.send_message(msg);

        let unacked = state.unacknowledged_messages();
        assert_eq!(unacked.len(), 1);
        assert_eq!(unacked[0], msg_id);
    }

    #[test]
    fn test_protocol_state_messages_from() {
        let robot1_id = RobotId::new(1);
        let robot2_id = RobotId::new(2);
        let mut state = ProtocolState::new(robot1_id);

        let point = TemporalPoint::new(1.0, 2.0, 3.0, 1000, 0.8);
        let obs = RobotObservation::new(robot2_id, point, 0.9);
        let msg = ObservationMessage::new(robot2_id, vec![obs]);

        state.receive_message(msg);

        let from_robot2 = state.messages_from(robot2_id);
        assert_eq!(from_robot2.len(), 1);
    }
}
