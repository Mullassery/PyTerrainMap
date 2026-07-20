//! Multi-bot synchronization for warehouse fleet coordination
//!
//! Enables distributed fleet learning where one bot's observations
//! immediately benefit all bots through shared Gaussian world model.

use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::gaussian_splatting::{GaussianSplatStore, TerrainGaussian};

/// Message from bot to fleet about observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotObservationMessage {
    pub bot_id: String,
    pub timestamp_us: i64,
    pub location: (f64, f64, f32),  // (lat, lon, elevation)
    pub observation_type: String,    // "terrain", "obstacle", "path"
    pub terrain_type: Option<String>,
    pub traversability: f32,
    pub confidence: f32,
}

/// Conflict resolution policy when different bots observe same location
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Use majority vote (most common observation wins)
    MajorityVote,
    /// Use highest confidence
    HighestConfidence,
    /// Use most recent observation
    MostRecent,
    /// Blend both observations (consensus)
    Consensus,
}

/// Fleet synchronization state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FleetSyncState {
    /// Bot IDs currently in fleet
    pub active_bots: Vec<String>,
    /// Last sync timestamp for each bot
    pub last_sync_time: HashMap<String, i64>,
    /// Observations pending sync
    pub pending_observations: usize,
    /// Conflicts detected this session
    pub conflicts_resolved: u32,
    /// Total observations fused
    pub total_fused: u64,
}

/// Bot status in fleet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotStatus {
    pub bot_id: String,
    pub is_active: bool,
    pub last_heard_from_us: i64,
    pub observations_contributed: u64,
    pub conflicts_triggered: u32,
    pub location: Option<(f64, f64, f32)>,
}

/// Multi-bot coordinator for warehouse fleet
pub struct FleetCoordinator {
    /// Shared world model
    store: Arc<RwLock<GaussianSplatStore>>,
    /// Fleet state
    state: Arc<RwLock<FleetSyncState>>,
    /// Bot statuses
    bot_statuses: Arc<RwLock<HashMap<String, BotStatus>>>,
    /// Conflict resolution policy
    conflict_policy: ConflictResolution,
    /// Message history (for debugging)
    message_log: Arc<RwLock<Vec<BotObservationMessage>>>,
    /// Max message history to keep
    max_log_size: usize,
}

impl FleetCoordinator {
    /// Create new fleet coordinator
    pub fn new(store: Arc<RwLock<GaussianSplatStore>>) -> Self {
        FleetCoordinator {
            store,
            state: Arc::new(RwLock::new(FleetSyncState {
                active_bots: Vec::new(),
                last_sync_time: HashMap::new(),
                pending_observations: 0,
                conflicts_resolved: 0,
                total_fused: 0,
            })),
            bot_statuses: Arc::new(RwLock::new(HashMap::new())),
            conflict_policy: ConflictResolution::Consensus,
            message_log: Arc::new(RwLock::new(Vec::new())),
            max_log_size: 10000,
        }
    }

    /// Register bot in fleet
    pub fn register_bot(&self, bot_id: &str) {
        let mut state = self.state.write();
        if !state.active_bots.contains(&bot_id.to_string()) {
            state.active_bots.push(bot_id.to_string());
        }

        let mut statuses = self.bot_statuses.write();
        statuses.insert(
            bot_id.to_string(),
            BotStatus {
                bot_id: bot_id.to_string(),
                is_active: true,
                last_heard_from_us: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as i64,
                observations_contributed: 0,
                conflicts_triggered: 0,
                location: None,
            },
        );
    }

    /// Ingest observation from bot
    pub fn ingest_bot_observation(&self, message: BotObservationMessage) -> Result<(), String> {
        // Register bot if not already registered
        if !self.state.read().active_bots.contains(&message.bot_id) {
            self.register_bot(&message.bot_id);
        }

        // Log message
        let mut log = self.message_log.write();
        log.push(message.clone());
        if log.len() > self.max_log_size {
            log.remove(0);
        }
        drop(log);

        // Create Gaussian splat from observation
        if let Some(_terrain_type) = &message.terrain_type {
            let splat = TerrainGaussian::from_point_observation(
                [message.location.0, message.location.1, message.location.2 as f64],
                &message.bot_id,
                message.traversability,
            );

            let mut store = self.store.write();
            store.insert(splat);
        }

        // Update bot status
        let mut statuses = self.bot_statuses.write();
        if let Some(status) = statuses.get_mut(&message.bot_id) {
            status.last_heard_from_us = message.timestamp_us;
            status.observations_contributed += 1;
            status.location = Some(message.location);
        }
        drop(statuses);

        // Update fleet state
        let mut state = self.state.write();
        state.last_sync_time
            .insert(message.bot_id.clone(), message.timestamp_us);
        state.total_fused += 1;

        Ok(())
    }

    /// Broadcast observation to all bots (simulated)
    pub fn broadcast_observation(&self, message: BotObservationMessage) -> Result<u32, String> {
        self.ingest_bot_observation(message)?;

        // In real deployment, would send to all other bots
        // For now, just track that it was broadcast
        let state = self.state.read();
        let broadcast_count = state.active_bots.len();
        Ok(broadcast_count as u32)
    }

    /// Get fleet state
    pub fn fleet_state(&self) -> FleetSyncState {
        self.state.read().clone()
    }

    /// Get bot status
    pub fn get_bot_status(&self, bot_id: &str) -> Option<BotStatus> {
        self.bot_statuses.read().get(bot_id).cloned()
    }

    /// Get all bot statuses
    pub fn all_bot_statuses(&self) -> Vec<BotStatus> {
        self.bot_statuses.read().values().cloned().collect()
    }

    /// Check fleet health (all bots active and synced)
    pub fn fleet_health(&self) -> f32 {
        let statuses = self.bot_statuses.read();
        if statuses.is_empty() {
            return 0.0;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as i64;

        let active_count = statuses
            .values()
            .filter(|b| {
                let age_us = now - b.last_heard_from_us;
                age_us < (30 * 1_000_000)  // Consider active if heard from within 30s
            })
            .count();

        (active_count as f32) / (statuses.len() as f32)
    }

    /// Resolve conflict between observations
    fn resolve_conflict(
        &self,
        existing: &TerrainGaussian,
        incoming: &TerrainGaussian,
    ) -> TerrainGaussian {
        match self.conflict_policy {
            ConflictResolution::HighestConfidence => {
                if incoming.confidence > existing.confidence {
                    incoming.clone()
                } else {
                    existing.clone()
                }
            }
            ConflictResolution::MostRecent => {
                if incoming.last_updated > existing.last_updated {
                    incoming.clone()
                } else {
                    existing.clone()
                }
            }
            _ => existing.clone(),  // Default to existing
        }
    }

    /// Get message history (for debugging/forensics)
    pub fn message_history(&self, limit: Option<usize>) -> Vec<BotObservationMessage> {
        let log = self.message_log.read();
        let start = if let Some(lim) = limit {
            log.len().saturating_sub(lim)
        } else {
            0
        };
        log[start..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use parking_lot::RwLock;

    #[test]
    fn test_coordinator_creation() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);
        assert_eq!(coordinator.fleet_state().active_bots.len(), 0);
    }

    #[test]
    fn test_register_bot() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        coordinator.register_bot("bot_01");

        let state = coordinator.fleet_state();
        assert_eq!(state.active_bots.len(), 1);
        assert!(state.active_bots.contains(&"bot_01".to_string()));
    }

    #[test]
    fn test_ingest_observation() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        let msg = BotObservationMessage {
            bot_id: "bot_01".to_string(),
            timestamp_us: 0,
            location: (40.0, -74.0, 10.0),
            observation_type: "terrain".to_string(),
            terrain_type: Some("Road".to_string()),
            traversability: 0.85,
            confidence: 0.9,
        };

        coordinator.ingest_bot_observation(msg).unwrap();

        let state = coordinator.fleet_state();
        assert_eq!(state.total_fused, 1);
    }

    #[test]
    fn test_broadcast_observation() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        // Register multiple bots
        coordinator.register_bot("bot_01");
        coordinator.register_bot("bot_02");
        coordinator.register_bot("bot_03");

        let msg = BotObservationMessage {
            bot_id: "bot_01".to_string(),
            timestamp_us: 0,
            location: (40.0, -74.0, 10.0),
            observation_type: "terrain".to_string(),
            terrain_type: Some("Road".to_string()),
            traversability: 0.85,
            confidence: 0.9,
        };

        let broadcast_count = coordinator.broadcast_observation(msg).unwrap();
        assert_eq!(broadcast_count, 3);
    }

    #[test]
    fn test_fleet_health() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        coordinator.register_bot("bot_01");
        coordinator.register_bot("bot_02");

        let health = coordinator.fleet_health();
        assert!(health > 0.0 && health <= 1.0);
    }

    #[test]
    fn test_bot_status() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        coordinator.register_bot("bot_01");

        let status = coordinator.get_bot_status("bot_01");
        assert!(status.is_some());
        assert_eq!(status.unwrap().bot_id, "bot_01");
    }

    #[test]
    fn test_message_history() {
        let store = Arc::new(RwLock::new(GaussianSplatStore::new()));
        let coordinator = FleetCoordinator::new(store);

        let msg = BotObservationMessage {
            bot_id: "bot_01".to_string(),
            timestamp_us: 0,
            location: (40.0, -74.0, 10.0),
            observation_type: "terrain".to_string(),
            terrain_type: Some("Road".to_string()),
            traversability: 0.85,
            confidence: 0.9,
        };

        coordinator.ingest_bot_observation(msg).unwrap();

        let history = coordinator.message_history(Some(10));
        assert_eq!(history.len(), 1);
    }
}
