//! Enterprise Compliance & Features
//!
//! Phase 8: Security, compliance, audit logging, data privacy,
//! multi-tenancy, and enterprise integrations.
//!
//! Phase 8 (Enterprise - 10-12 dev-days):
//! - Enterprise compliance & audit
//! - Data privacy & encryption
//! - Multi-tenancy & isolation
//! - Enterprise authentication
//! - SLA management

pub mod compliance;    // Phase 8.1: Compliance & audit
pub mod privacy;       // Phase 8.2: Data privacy & encryption
pub mod multitenancy;  // Phase 8.3: Multi-tenancy
pub mod auth;          // Phase 8.4: Enterprise auth
pub mod sla;           // Phase 8.5: SLA management

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComplianceStandard {
    GDPR,
    HIPAA,
    SOC2,
    ISO27001,
}
