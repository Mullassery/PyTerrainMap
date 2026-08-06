//! Enterprise Authentication
//!
//! Phase 8.4: SAML, OAuth2, LDAP integration.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth2,
    SAML,
    LDAP,
}

pub struct EnterpriseAuth {
    pub method: AuthMethod,
}

impl EnterpriseAuth {
    pub fn new(method: AuthMethod) -> Self {
        EnterpriseAuth { method }
    }
}

impl Default for EnterpriseAuth {
    fn default() -> Self {
        Self::new(AuthMethod::OAuth2)
    }
}
