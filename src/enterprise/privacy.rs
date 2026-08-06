//! Data Privacy & Encryption
//!
//! Phase 8.2: Encrypt data at rest and in transit, PII handling.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    AES256,
    ChaCha20,
}

pub struct PrivacyManager {
    pub encryption: EncryptionAlgorithm,
}

impl PrivacyManager {
    pub fn new() -> Self {
        PrivacyManager {
            encryption: EncryptionAlgorithm::AES256,
        }
    }
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self::new()
    }
}
