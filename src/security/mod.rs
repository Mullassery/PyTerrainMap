//! Security-First and Auditability-First Architecture
//!
//! Comprehensive security, trust, verification, and audit infrastructure.
//! Security and compliance emerge naturally from system design.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Trust score for observations (0.0-1.0)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub struct TrustScore(pub f32);

impl TrustScore {
    pub fn new(value: f32) -> Self {
        TrustScore(value.max(0.0).min(1.0))
    }

    pub fn is_trusted(&self) -> bool {
        self.0 >= 0.7
    }

    pub fn is_highly_trusted(&self) -> bool {
        self.0 >= 0.9
    }
}

/// Source of an observation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SourceIdentity {
    Sensor { sensor_id: String },
    Agent { agent_id: String },
    Model { model_name: String },
    User { user_id: String },
    API { api_name: String },
}

impl SourceIdentity {
    pub fn identifier(&self) -> String {
        match self {
            SourceIdentity::Sensor { sensor_id } => format!("sensor:{}", sensor_id),
            SourceIdentity::Agent { agent_id } => format!("agent:{}", agent_id),
            SourceIdentity::Model { model_name } => format!("model:{}", model_name),
            SourceIdentity::User { user_id } => format!("user:{}", user_id),
            SourceIdentity::API { api_name } => format!("api:{}", api_name),
        }
    }
}

/// Trust metadata for observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustMetadata {
    /// Who/what created this observation?
    pub source_identity: SourceIdentity,

    /// How much do we trust this source? (0.0-1.0)
    pub source_trust_score: TrustScore,

    /// How much do we trust this specific observation? (0.0-1.0)
    pub observation_confidence: TrustScore,

    /// How trustworthy is it given age?
    pub freshness_trust: TrustScore,

    /// How many times verified?
    pub verification_count: u32,

    /// Last verification time (microseconds since epoch)
    pub last_verified_us: i64,
}

impl Default for TrustMetadata {
    fn default() -> Self {
        TrustMetadata {
            source_identity: SourceIdentity::Sensor {
                sensor_id: "unknown".to_string(),
            },
            source_trust_score: TrustScore::new(0.5),
            observation_confidence: TrustScore::new(0.5),
            freshness_trust: TrustScore::new(0.5),
            verification_count: 0,
            last_verified_us: 0,
        }
    }
}

/// Verification status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Unverified,
    PendingVerification,
    PartiallyVerified,
    Verified,
    Invalid,
}

/// Verification metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerificationMetadata {
    /// Current verification status
    pub status: VerificationStatus,

    /// Number of independent confirmations
    pub confirmation_count: u32,

    /// Sensors that confirmed this
    pub confirming_sensors: Vec<String>,

    /// Anomaly detection results (0.0-1.0)
    pub anomaly_scores: HashMap<String, f32>,

    /// Consistency score (0.0-1.0)
    pub consistency_score: f32,

    /// Does this contradict known facts?
    pub has_contradictions: bool,
}

impl Default for VerificationMetadata {
    fn default() -> Self {
        VerificationMetadata {
            status: VerificationStatus::Unverified,
            confirmation_count: 0,
            confirming_sensors: Vec::new(),
            anomaly_scores: HashMap::new(),
            consistency_score: 0.5,
            has_contradictions: false,
        }
    }
}

/// Action in provenance chain
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProvenanceAction {
    Captured,
    Fused,
    Filtered,
    Processed,
    Cached,
    Predicted,
    Verified,
    Transformed,
    Exported,
}

/// Step in provenance chain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceStep {
    /// What action occurred?
    pub action: ProvenanceAction,

    /// Who performed it?
    pub actor: String,

    /// When? (microseconds since epoch)
    pub timestamp_us: i64,

    /// Did this preserve semantics?
    pub semantic_preserving: bool,
}

/// Provenance chain (immutable history)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProvenanceChain {
    /// Original capture step
    pub origin: ProvenanceStep,

    /// All transformations
    pub transformations: Vec<ProvenanceStep>,
}

impl ProvenanceChain {
    pub fn new(origin: ProvenanceStep) -> Self {
        ProvenanceChain {
            origin,
            transformations: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: ProvenanceStep) {
        self.transformations.push(step);
    }

    pub fn total_steps(&self) -> usize {
        1 + self.transformations.len()
    }
}

/// Data classification levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Retention policy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RetentionPolicy {
    Permanent,
    Years(u32),
    Months(u32),
    Days(u32),
    DeleteOnMissionComplete,
}

impl RetentionPolicy {
    pub fn expires_at_us(&self, created_at_us: i64) -> Option<i64> {
        match self {
            RetentionPolicy::Permanent => None,
            RetentionPolicy::Years(y) => Some(created_at_us + (*y as i64) * 365 * 24 * 3600 * 1_000_000),
            RetentionPolicy::Months(m) => Some(created_at_us + (*m as i64) * 30 * 24 * 3600 * 1_000_000),
            RetentionPolicy::Days(d) => Some(created_at_us + (*d as i64) * 24 * 3600 * 1_000_000),
            RetentionPolicy::DeleteOnMissionComplete => None,  // Dynamic
        }
    }

    pub fn is_expired(&self, created_at_us: i64) -> bool {
        if let Some(expiry) = self.expires_at_us(created_at_us) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64;
            now > expiry
        } else {
            false
        }
    }
}

/// Compliance metadata for observations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceMetadata {
    /// Who created this?
    pub created_by: String,

    /// When? (microseconds since epoch)
    pub created_at_us: i64,

    /// Last modified by
    pub modified_by: Option<String>,

    /// Last modified when
    pub modified_at_us: Option<i64>,

    /// How long to keep?
    pub retention_policy: RetentionPolicy,

    /// Sensitivity level
    pub classification: DataClassification,

    /// Verification level required
    pub trust_level_required: TrustScore,

    /// Audit reference ID
    pub audit_reference: Uuid,
}

impl Default for ComplianceMetadata {
    fn default() -> Self {
        ComplianceMetadata {
            created_by: "system".to_string(),
            created_at_us: 0,
            modified_by: None,
            modified_at_us: None,
            retention_policy: RetentionPolicy::Years(1),
            classification: DataClassification::Internal,
            trust_level_required: TrustScore::new(0.7),
            audit_reference: Uuid::new_v4(),
        }
    }
}

/// Audit event types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    ObservationIngested,
    SensorFusionDecision,
    CacheUpdated,
    PredictivePrefetch,
    AIInference,
    WorldStateModified,
    AccessRequest,
    PermissionChanged,
    SecurityEvent,
    VerificationPerformed,
}

/// Audit outcome
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditResult {
    Success,
    Partial,
    Failure(String),
    SecurityEvent(String),
}

/// Single audit event (immutable record)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub event_id: Uuid,

    /// Type of event
    pub event_type: AuditEventType,

    /// When? (microseconds since epoch)
    pub timestamp_us: i64,

    /// Who initiated?
    pub actor: String,

    /// Which system?
    pub system_component: String,

    /// Input observation IDs
    pub input_observations: Vec<Uuid>,

    /// Models involved
    pub models_involved: Vec<String>,

    /// Confidence level
    pub confidence: f32,

    /// Outcome
    pub result: AuditResult,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType) -> Self {
        AuditEvent {
            event_id: Uuid::new_v4(),
            event_type,
            timestamp_us: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as i64,
            actor: "unknown".to_string(),
            system_component: "unknown".to_string(),
            input_observations: Vec::new(),
            models_involved: Vec::new(),
            confidence: 0.5,
            result: AuditResult::Success,
        }
    }

    pub fn with_actor(mut self, actor: &str) -> Self {
        self.actor = actor.to_string();
        self
    }

    pub fn with_component(mut self, component: &str) -> Self {
        self.system_component = component.to_string();
        self
    }

    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }
}

/// Immutable audit log
pub struct AuditLog {
    events: parking_lot::RwLock<Vec<AuditEvent>>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            events: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Append new audit event (immutable)
    pub fn append(&self, event: AuditEvent) {
        let mut events = self.events.write();
        events.push(event);
    }

    /// Get all events
    pub fn all_events(&self) -> Vec<AuditEvent> {
        self.events.read().clone()
    }

    /// Query events by type
    pub fn events_by_type(&self, event_type: AuditEventType) -> Vec<AuditEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Query events by actor
    pub fn events_by_actor(&self, actor: &str) -> Vec<AuditEvent> {
        self.events
            .read()
            .iter()
            .filter(|e| e.actor == actor)
            .cloned()
            .collect()
    }

    /// Total event count
    pub fn event_count(&self) -> usize {
        self.events.read().len()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        AuditLog::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_score_bounds() {
        let score = TrustScore::new(1.5);
        assert_eq!(score.0, 1.0);

        let score = TrustScore::new(-0.5);
        assert_eq!(score.0, 0.0);
    }

    #[test]
    fn test_trust_score_thresholds() {
        let trusted = TrustScore::new(0.8);
        assert!(trusted.is_trusted());

        let highly_trusted = TrustScore::new(0.95);
        assert!(highly_trusted.is_highly_trusted());
    }

    #[test]
    fn test_source_identity_identifier() {
        let sensor = SourceIdentity::Sensor {
            sensor_id: "cam-01".to_string(),
        };
        assert_eq!(sensor.identifier(), "sensor:cam-01");
    }

    #[test]
    fn test_provenance_chain() {
        let origin = ProvenanceStep {
            action: ProvenanceAction::Captured,
            actor: "sensor-01".to_string(),
            timestamp_us: 1000,
            semantic_preserving: true,
        };

        let mut chain = ProvenanceChain::new(origin);
        assert_eq!(chain.total_steps(), 1);

        chain.add_step(ProvenanceStep {
            action: ProvenanceAction::Fused,
            actor: "fusion-engine".to_string(),
            timestamp_us: 2000,
            semantic_preserving: true,
        });

        assert_eq!(chain.total_steps(), 2);
    }

    #[test]
    fn test_retention_policy_expiry() {
        let policy = RetentionPolicy::Days(7);
        let created_at = 0i64;

        assert!(policy.expires_at_us(created_at).is_some());
        let expiry = policy.expires_at_us(created_at).unwrap();
        assert!(expiry > created_at);
    }

    #[test]
    fn test_audit_log_append() {
        let log = AuditLog::new();
        let event = AuditEvent::new(AuditEventType::ObservationIngested);

        log.append(event.clone());
        assert_eq!(log.event_count(), 1);

        let events = log.all_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::ObservationIngested);
    }

    #[test]
    fn test_audit_log_query_by_type() {
        let log = AuditLog::new();

        log.append(AuditEvent::new(AuditEventType::ObservationIngested));
        log.append(AuditEvent::new(AuditEventType::CacheUpdated));
        log.append(AuditEvent::new(AuditEventType::ObservationIngested));

        let ingested = log.events_by_type(AuditEventType::ObservationIngested);
        assert_eq!(ingested.len(), 2);
    }

    #[test]
    fn test_audit_log_query_by_actor() {
        let log = AuditLog::new();

        log.append(AuditEvent::new(AuditEventType::ObservationIngested).with_actor("agent-1"));
        log.append(AuditEvent::new(AuditEventType::ObservationIngested).with_actor("agent-2"));
        log.append(AuditEvent::new(AuditEventType::ObservationIngested).with_actor("agent-1"));

        let agent1_events = log.events_by_actor("agent-1");
        assert_eq!(agent1_events.len(), 2);
    }

    #[test]
    fn test_verification_metadata_default() {
        let verification = VerificationMetadata::default();
        assert_eq!(verification.status, VerificationStatus::Unverified);
        assert_eq!(verification.confirmation_count, 0);
    }

    #[test]
    fn test_data_classification() {
        let public = DataClassification::Public;
        let restricted = DataClassification::Restricted;

        assert_ne!(public, restricted);
    }
}

use parking_lot;
