//! Multi-Tenancy & Isolation
//!
//! Phase 8.3: Isolate data and resources per tenant.

use super::TenantId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub data_isolated: bool,
    pub resources_limited: bool,
}

pub struct MultiTenancyManager {
    pub tenants: HashMap<String, TenantContext>,
}

impl MultiTenancyManager {
    pub fn new() -> Self {
        MultiTenancyManager {
            tenants: HashMap::new(),
        }
    }

    pub fn register_tenant(&mut self, tenant_id: String) {
        self.tenants.insert(tenant_id.clone(), TenantContext {
            tenant_id: TenantId(tenant_id),
            data_isolated: true,
            resources_limited: true,
        });
    }
}

impl Default for MultiTenancyManager {
    fn default() -> Self {
        Self::new()
    }
}
