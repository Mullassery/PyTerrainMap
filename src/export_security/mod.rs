//! Security layer for spatial data exports
//!
//! Implements data privacy, access control, and injection prevention
//! for sensitive observation data.

use crate::types::Observation;
use std::collections::HashSet;

/// Data classification level for export restrictions
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataClassification {
    /// Public: no restrictions, can export to any format
    Public = 0,
    /// Internal: restricted to authorized users only
    Internal = 1,
    /// Confidential: operational security sensitive (robot locations, timings)
    Confidential = 2,
    /// Restricted: mission-critical, compartmentalized access
    Restricted = 3,
}

/// User roles for access control
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UserRole {
    /// Anonymous viewer (public data only)
    Anonymous,
    /// Authenticated user (internal + public)
    User,
    /// System administrator (all data)
    Administrator,
    /// Mission operator (role-specific data)
    MissionOperator,
    /// Security auditor (logs only, no raw data)
    SecurityAuditor,
}

impl UserRole {
    /// Get maximum classification level this role can access
    pub fn max_classification(&self) -> DataClassification {
        match self {
            UserRole::Anonymous => DataClassification::Public,
            UserRole::User => DataClassification::Internal,
            UserRole::MissionOperator => DataClassification::Confidential,
            UserRole::Administrator => DataClassification::Restricted,
            UserRole::SecurityAuditor => DataClassification::Internal,
        }
    }

    /// Check if role can access this classification
    pub fn can_access(&self, classification: DataClassification) -> bool {
        self.max_classification() >= classification
    }
}

/// Export privacy settings
#[derive(Clone, Debug)]
pub struct ExportPrivacy {
    /// Remove exact robot IDs, replace with "bot_1", "bot_2", etc.
    pub anonymize_robots: bool,
    /// Degrade coordinate precision (e.g., 4 decimals -> 2)
    pub degrade_coordinates: bool,
    /// Number of decimal places for coordinates (null = no degradation)
    pub coordinate_precision: Option<u32>,
    /// Remove metadata fields (battery, signal strength, etc.)
    pub strip_metadata: bool,
    /// Allowed metadata keys (whitelist, None = no filtering)
    pub allowed_metadata_keys: Option<HashSet<String>>,
    /// Remove timestamps or degrade to day-level precision
    pub degrade_timestamps: bool,
    /// Redact specific sensor types
    pub redacted_sensors: HashSet<String>,
}

impl ExportPrivacy {
    /// No privacy restrictions
    pub fn none() -> Self {
        ExportPrivacy {
            anonymize_robots: false,
            degrade_coordinates: false,
            coordinate_precision: None,
            strip_metadata: false,
            allowed_metadata_keys: None,
            degrade_timestamps: false,
            redacted_sensors: HashSet::new(),
        }
    }

    /// Maximum privacy: anonymize everything sensitive
    pub fn maximum() -> Self {
        ExportPrivacy {
            anonymize_robots: true,
            degrade_coordinates: true,
            coordinate_precision: Some(2), // ~1.1km precision
            strip_metadata: true,
            allowed_metadata_keys: None,
            degrade_timestamps: true,
            redacted_sensors: HashSet::new(),
        }
    }

    /// Balanced privacy: redact operationally sensitive data
    pub fn balanced() -> Self {
        let mut redacted = HashSet::new();
        redacted.insert("movement".to_string()); // Hide movement/velocity

        ExportPrivacy {
            anonymize_robots: false,
            degrade_coordinates: false,
            coordinate_precision: Some(4), // ~11m precision
            strip_metadata: false,
            allowed_metadata_keys: Some(
                vec!["sensor_health", "confidence", "time_to_serve", "battery", "signal_strength"]
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            degrade_timestamps: false,
            redacted_sensors: redacted,
        }
    }

    /// Apply privacy filtering to an observation
    pub fn filter_observation(&self, mut obs: Observation) -> Option<Observation> {
        // Check if sensor type is redacted
        if self.redacted_sensors.contains(&obs.sensor_type.to_string()) {
            return None;
        }

        // Anonymize robot ID
        if self.anonymize_robots {
            // Replace with hash-based anonymous ID
            let hash = Self::hash_string(&obs.robot_id);
            obs.robot_id = format!("bot_{}", hash % 1000); // bot_0 to bot_999
        }

        // Degrade coordinates
        if self.degrade_coordinates {
            if let Some(decimals) = self.coordinate_precision {
                obs.location.lat = (obs.location.lat * 10_f64.powi(decimals as i32)).round()
                    / 10_f64.powi(decimals as i32);
                obs.location.lon = (obs.location.lon * 10_f64.powi(decimals as i32)).round()
                    / 10_f64.powi(decimals as i32);
            }
        }

        // Degrade timestamps to day-level if requested
        if self.degrade_timestamps {
            let day_micros = 24 * 60 * 60 * 1_000_000i64;
            obs.timestamp = (obs.timestamp / day_micros) * day_micros;
        }

        // Filter metadata
        if self.strip_metadata {
            obs.metadata.clear();
        } else if let Some(ref allowed_keys) = self.allowed_metadata_keys {
            obs.metadata.retain(|k, _| allowed_keys.contains(k));
        }

        Some(obs)
    }

    fn hash_string(s: &str) -> u32 {
        let mut hash = 0u32;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }
}

/// Export access control policy
#[derive(Clone, Debug)]
pub struct ExportPolicy {
    /// User role
    pub user_role: UserRole,
    /// Minimum data classification (reject if below this)
    pub min_classification: DataClassification,
    /// Privacy settings to apply
    pub privacy: ExportPrivacy,
    /// Allowed export formats for this role
    pub allowed_formats: HashSet<String>,
    /// Maximum observations per export
    pub max_observations: Option<usize>,
    /// Require audit logging
    pub audit_required: bool,
}

impl ExportPolicy {
    /// Create default policy for a role
    pub fn for_role(role: UserRole) -> Self {
        let privacy = match role {
            UserRole::Anonymous => ExportPrivacy::maximum(),
            UserRole::SecurityAuditor => ExportPrivacy::none(), // Auditors see everything
            _ => ExportPrivacy::balanced(),
        };

        let mut allowed_formats = HashSet::new();
        allowed_formats.insert("geojson".to_string());
        allowed_formats.insert("kml".to_string());

        match role {
            UserRole::Anonymous => {
                allowed_formats.remove("kml"); // No KML for anonymous
            }
            UserRole::Administrator => {
                allowed_formats.insert("shapefile".to_string());
                allowed_formats.insert("geotiff".to_string());
                allowed_formats.insert("3dtiles".to_string());
            }
            _ => {}
        }

        ExportPolicy {
            user_role: role,
            min_classification: role.max_classification(),
            privacy,
            allowed_formats,
            max_observations: match role {
                UserRole::Anonymous => Some(100),
                UserRole::User => Some(1000),
                UserRole::Administrator => None,
                UserRole::MissionOperator => Some(5000),
                UserRole::SecurityAuditor => Some(100),
            },
            audit_required: role != UserRole::Administrator,
        }
    }

    /// Check if export is allowed
    pub fn can_export(&self, format: &str, count: usize) -> Result<(), String> {
        // Check format
        if !self.allowed_formats.contains(format) {
            return Err(format!(
                "Format '{}' not allowed for role {:?}",
                format, self.user_role
            ));
        }

        // Check observation count
        if let Some(max) = self.max_observations {
            if count > max {
                return Err(format!(
                    "Export exceeds maximum of {} observations for role {:?}",
                    max, self.user_role
                ));
            }
        }

        Ok(())
    }
}

/// Export audit log entry
#[derive(Clone, Debug)]
pub struct AuditLogEntry {
    /// Timestamp of export
    pub timestamp: i64,
    /// User who performed export
    pub user_id: String,
    /// User role
    pub user_role: UserRole,
    /// Export format
    pub format: String,
    /// Number of observations exported
    pub observation_count: usize,
    /// Privacy settings used
    pub privacy_applied: bool,
    /// Status: "success" or error message
    pub status: String,
}

/// Audit logger for exports
pub struct AuditLogger {
    entries: Vec<AuditLogEntry>,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new() -> Self {
        AuditLogger {
            entries: Vec::new(),
        }
    }

    /// Log an export
    pub fn log_export(
        &mut self,
        user_id: &str,
        role: UserRole,
        format: &str,
        count: usize,
        privacy_applied: bool,
        status: &str,
    ) {
        self.entries.push(AuditLogEntry {
            timestamp: chrono::Utc::now().timestamp_micros(),
            user_id: user_id.to_string(),
            user_role: role,
            format: format.to_string(),
            observation_count: count,
            privacy_applied,
            status: status.to_string(),
        });
    }

    /// Get all audit entries
    pub fn entries(&self) -> &[AuditLogEntry] {
        &self.entries
    }

    /// Get audit trail for specific user
    pub fn entries_for_user(&self, user_id: &str) -> Vec<&AuditLogEntry> {
        self.entries
            .iter()
            .filter(|e| e.user_id == user_id)
            .collect()
    }

    /// Clear audit log (restricted operation, should require special access)
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Injection prevention utilities
pub mod sanitize {
    /// Sanitize string for XML/KML embedding
    pub fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Sanitize string for JSON embedding (already handled by serde, but available)
    pub fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    /// Validate coordinate to prevent injection
    pub fn validate_coordinate(coord: f64) -> Result<f64, String> {
        if coord.is_finite() && coord >= -180.0 && coord <= 180.0 {
            Ok(coord)
        } else {
            Err(format!("Invalid coordinate: {}", coord))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GeoPoint, SensorType, SensorValue};

    fn create_test_obs(robot: &str, lat: f64, lon: f64) -> Observation {
        Observation::new(
            robot.to_string(),
            1000000,
            GeoPoint::new(lat, lon),
            Some(100.0),
            SensorType::Thermal,
            SensorValue::Temperature { celsius: 22.5 },
            0.95,
        )
        .with_metadata("battery".to_string(), "87%".to_string())
        .with_metadata("signal_strength".to_string(), "-40dBm".to_string())
    }

    #[test]
    fn test_user_role_access() {
        assert!(UserRole::Administrator.can_access(DataClassification::Restricted));
        assert!(UserRole::User.can_access(DataClassification::Internal));
        assert!(!UserRole::User.can_access(DataClassification::Confidential));
        assert!(!UserRole::Anonymous.can_access(DataClassification::Internal));
    }

    #[test]
    fn test_privacy_none() {
        let privacy = ExportPrivacy::none();
        let obs = create_test_obs("bot_1", 40.7128, -74.0060);
        let filtered = privacy.filter_observation(obs.clone()).unwrap();

        assert_eq!(filtered.robot_id, "bot_1");
        assert_eq!(filtered.location.lat, 40.7128);
        assert_eq!(filtered.metadata.len(), 2);
    }

    #[test]
    fn test_privacy_maximum() {
        let privacy = ExportPrivacy::maximum();
        let obs = create_test_obs("bot_1", 40.7128, -74.0060);
        let filtered = privacy.filter_observation(obs).unwrap();

        assert_ne!(filtered.robot_id, "bot_1"); // Anonymized
        assert_ne!(filtered.location.lat, 40.7128); // Degraded
        assert!(filtered.metadata.is_empty()); // Stripped
    }

    #[test]
    fn test_privacy_balanced() {
        let privacy = ExportPrivacy::balanced();
        let obs = create_test_obs("bot_security", 40.7128, -74.0060);
        let filtered = privacy.filter_observation(obs).unwrap();

        assert_eq!(filtered.robot_id, "bot_security"); // Not anonymized
        assert!(filtered.metadata.contains_key("battery")); // Some metadata kept
    }

    #[test]
    fn test_coordinate_degradation() {
        let mut privacy = ExportPrivacy::none();
        privacy.degrade_coordinates = true;
        privacy.coordinate_precision = Some(2);

        let obs = create_test_obs("bot_1", 40.7128, -74.0060);
        let filtered = privacy.filter_observation(obs).unwrap();

        // 40.7128 -> 40.71 (2 decimals)
        assert!((filtered.location.lat - 40.71).abs() < 0.001);
    }

    #[test]
    fn test_sensor_redaction() {
        let mut privacy = ExportPrivacy::none();
        privacy.redacted_sensors.insert("movement".to_string());

        let obs = Observation::new(
            "bot_1".to_string(),
            1000,
            GeoPoint::new(40.7128, -74.0060),
            None,
            SensorType::Movement,
            SensorValue::Movement {
                velocity: 5.0,
                heading: 45.0,
            },
            0.95,
        );

        let filtered = privacy.filter_observation(obs);
        assert!(filtered.is_none()); // Redacted sensor returns None
    }

    #[test]
    fn test_export_policy_anonymous() {
        let policy = ExportPolicy::for_role(UserRole::Anonymous);

        assert!(policy.can_export("geojson", 50).is_ok());
        assert!(policy.can_export("geojson", 150).is_err()); // Over 100 limit
        assert!(policy.can_export("kml", 50).is_err()); // KML not allowed
    }

    #[test]
    fn test_export_policy_admin() {
        let policy = ExportPolicy::for_role(UserRole::Administrator);

        assert!(policy.can_export("geojson", 10000).is_ok());
        assert!(policy.can_export("shapefile", 10000).is_ok());
        assert!(policy.can_export("geotiff", 10000).is_ok());
    }

    #[test]
    fn test_audit_logging() {
        let mut logger = AuditLogger::new();

        logger.log_export("user_1", UserRole::User, "geojson", 50, true, "success");
        logger.log_export("user_2", UserRole::User, "kml", 100, true, "denied");

        assert_eq!(logger.entries().len(), 2);
        assert_eq!(logger.entries_for_user("user_1").len(), 1);
        assert_eq!(logger.entries_for_user("user_3").len(), 0);
    }

    #[test]
    fn test_xml_sanitization() {
        let dangerous = "<script>alert('xss')</script>";
        let safe = sanitize::xml_escape(dangerous);

        assert!(!safe.contains("<script>"));
        assert!(safe.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_coordinate_validation() {
        assert!(sanitize::validate_coordinate(40.7128).is_ok());
        assert!(sanitize::validate_coordinate(-74.0060).is_ok());
        assert!(sanitize::validate_coordinate(200.0).is_err()); // Out of range
        assert!(sanitize::validate_coordinate(f64::NAN).is_err()); // Invalid
    }
}
