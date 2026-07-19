//! HTTPS/TLS support for PyTerrainMap API
//!
//! Handles certificate management, TLS configuration, and secure communication.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// TLS Configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to certificate file (PEM format)
    pub cert_path: PathBuf,
    /// Path to private key file (PEM format)
    pub key_path: PathBuf,
    /// Minimum TLS version to accept
    pub min_tls_version: TlsVersion,
    /// Enable certificate validation for clients (mTLS)
    pub require_client_cert: bool,
    /// Path to CA certificate for client validation
    pub ca_cert_path: Option<PathBuf>,
    /// Cipher suites to use (None = system default)
    pub cipher_suites: Option<Vec<String>>,
}

/// TLS version specification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2 (minimum recommended)
    V1_2,
    /// TLS 1.3 (modern, preferred)
    V1_3,
}

impl TlsVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            TlsVersion::V1_2 => "1.2",
            TlsVersion::V1_3 => "1.3",
        }
    }
}

impl std::fmt::Display for TlsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TLS {}", self.as_str())
    }
}

/// HTTPS mode configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HttpsMode {
    /// HTTP only (development only)
    Http,
    /// HTTPS with certificate
    Https(TlsConfig),
    /// Both HTTP and HTTPS (redirect HTTP to HTTPS)
    Redirect { https: TlsConfig, http_port: u16 },
}

impl HttpsMode {
    /// Create HTTPS from certificate and key files
    pub fn from_files<P: AsRef<Path>>(cert_path: P, key_path: P) -> Self {
        HttpsMode::Https(TlsConfig {
            cert_path: cert_path.as_ref().to_path_buf(),
            key_path: key_path.as_ref().to_path_buf(),
            min_tls_version: TlsVersion::V1_3,
            require_client_cert: false,
            ca_cert_path: None,
            cipher_suites: None,
        })
    }

    /// Create HTTPS with client certificate verification (mTLS)
    pub fn mtls<P: AsRef<Path>>(
        cert_path: P,
        key_path: P,
        ca_path: P,
    ) -> Self {
        HttpsMode::Https(TlsConfig {
            cert_path: cert_path.as_ref().to_path_buf(),
            key_path: key_path.as_ref().to_path_buf(),
            min_tls_version: TlsVersion::V1_3,
            require_client_cert: true,
            ca_cert_path: Some(ca_path.as_ref().to_path_buf()),
            cipher_suites: None,
        })
    }

    /// Development mode (HTTP only)
    pub fn development() -> Self {
        HttpsMode::Http
    }

    /// Production mode (HTTPS with redirect)
    pub fn production<P: AsRef<Path>>(cert_path: P, key_path: P) -> Self {
        HttpsMode::Redirect {
            https: TlsConfig {
                cert_path: cert_path.as_ref().to_path_buf(),
                key_path: key_path.as_ref().to_path_buf(),
                min_tls_version: TlsVersion::V1_3,
                require_client_cert: false,
                ca_cert_path: None,
                cipher_suites: None,
            },
            http_port: 8080,
        }
    }

    pub fn is_secure(&self) -> bool {
        !matches!(self, HttpsMode::Http)
    }
}

/// Certificate information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Certificate subject (Distinguished Name)
    pub subject: String,
    /// Certificate issuer
    pub issuer: String,
    /// Not valid before (Unix timestamp)
    pub not_before: i64,
    /// Not valid after (Unix timestamp)
    pub not_after: i64,
    /// Certificate is valid now
    pub is_valid: bool,
    /// Days until expiration
    pub days_to_expiry: i32,
}

impl CertificateInfo {
    /// Check if certificate will expire soon
    pub fn expiring_soon(&self, days_threshold: i32) -> bool {
        self.days_to_expiry <= days_threshold
    }

    /// Check if certificate is already expired
    pub fn is_expired(&self) -> bool {
        self.days_to_expiry < 0
    }
}

/// Certificate validation
#[derive(Clone, Debug)]
pub struct CertificateValidator {
    /// Enable hostname verification
    pub verify_hostname: bool,
    /// Enable certificate chain validation
    pub verify_chain: bool,
    /// Enable certificate revocation checking (CRL/OCSP)
    pub check_revocation: bool,
    /// Days before expiry to warn
    pub expiry_warning_days: i32,
}

impl CertificateValidator {
    /// Create validator with strict settings (production)
    pub fn strict() -> Self {
        CertificateValidator {
            verify_hostname: true,
            verify_chain: true,
            check_revocation: true,
            expiry_warning_days: 30,
        }
    }

    /// Create validator with relaxed settings (development)
    pub fn permissive() -> Self {
        CertificateValidator {
            verify_hostname: false,
            verify_chain: false,
            check_revocation: false,
            expiry_warning_days: 7,
        }
    }

    /// Default validator (balanced)
    pub fn default() -> Self {
        CertificateValidator {
            verify_hostname: true,
            verify_chain: true,
            check_revocation: false,
            expiry_warning_days: 14,
        }
    }
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::default()
    }
}

/// Cipher suite recommendations
pub mod cipher_suites {
    /// Modern cipher suites (TLS 1.3+)
    pub fn modern() -> Vec<String> {
        vec![
            "TLS_AES_256_GCM_SHA384".to_string(),
            "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            "TLS_AES_128_GCM_SHA256".to_string(),
        ]
    }

    /// Balanced cipher suites (TLS 1.2+)
    pub fn balanced() -> Vec<String> {
        vec![
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
        ]
    }

    /// Compatibility cipher suites (broader support)
    pub fn compatible() -> Vec<String> {
        vec![
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
            "TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA".to_string(),
        ]
    }
}

/// Security headers for HTTPS responses
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecurityHeaders {
    /// Strict-Transport-Security: enforce HTTPS
    pub hsts: Option<String>,
    /// X-Content-Type-Options: prevent MIME sniffing
    pub content_type_options: Option<String>,
    /// X-Frame-Options: prevent clickjacking
    pub frame_options: Option<String>,
    /// X-XSS-Protection: enable XSS protection
    pub xss_protection: Option<String>,
    /// Content-Security-Policy: prevent injection attacks
    pub csp: Option<String>,
}

impl SecurityHeaders {
    /// Default secure headers (production recommended)
    pub fn default_secure() -> Self {
        SecurityHeaders {
            hsts: Some("max-age=31536000; includeSubDomains; preload".to_string()),
            content_type_options: Some("nosniff".to_string()),
            frame_options: Some("DENY".to_string()),
            xss_protection: Some("1; mode=block".to_string()),
            csp: Some("default-src 'none'".to_string()),
        }
    }

    /// Minimal headers (development)
    pub fn minimal() -> Self {
        SecurityHeaders {
            hsts: None,
            content_type_options: Some("nosniff".to_string()),
            frame_options: None,
            xss_protection: None,
            csp: None,
        }
    }

    /// Build header map for HTTP response
    pub fn to_headers(&self) -> Vec<(&'static str, String)> {
        let mut headers = Vec::new();

        if let Some(ref hsts) = self.hsts {
            headers.push(("Strict-Transport-Security", hsts.clone()));
        }
        if let Some(ref cto) = self.content_type_options {
            headers.push(("X-Content-Type-Options", cto.clone()));
        }
        if let Some(ref fo) = self.frame_options {
            headers.push(("X-Frame-Options", fo.clone()));
        }
        if let Some(ref xss) = self.xss_protection {
            headers.push(("X-XSS-Protection", xss.clone()));
        }
        if let Some(ref csp) = self.csp {
            headers.push(("Content-Security-Policy", csp.clone()));
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_version_display() {
        assert_eq!(TlsVersion::V1_2.to_string(), "TLS 1.2");
        assert_eq!(TlsVersion::V1_3.to_string(), "TLS 1.3");
    }

    #[test]
    fn test_https_mode_is_secure() {
        assert!(!HttpsMode::Http.is_secure());
        assert!(HttpsMode::from_files("cert.pem", "key.pem").is_secure());
    }

    #[test]
    fn test_https_mode_production() {
        let mode = HttpsMode::production("cert.pem", "key.pem");
        assert!(mode.is_secure());
        assert!(matches!(mode, HttpsMode::Redirect { .. }));
    }

    #[test]
    fn test_certificate_info() {
        let now = chrono::Utc::now().timestamp();
        let cert = CertificateInfo {
            subject: "CN=example.com".to_string(),
            issuer: "CN=CA".to_string(),
            not_before: now - 86400,
            not_after: now + (30 * 86400),
            is_valid: true,
            days_to_expiry: 30,
        };

        assert!(!cert.is_expired());
        assert!(cert.expiring_soon(31));
        assert!(!cert.expiring_soon(29));
    }

    #[test]
    fn test_certificate_expired() {
        let cert = CertificateInfo {
            subject: "CN=example.com".to_string(),
            issuer: "CN=CA".to_string(),
            not_before: 0,
            not_after: 1,
            is_valid: false,
            days_to_expiry: -1,
        };

        assert!(cert.is_expired());
        assert!(!cert.is_valid);
    }

    #[test]
    fn test_certificate_validator_strict() {
        let validator = CertificateValidator::strict();
        assert!(validator.verify_hostname);
        assert!(validator.verify_chain);
        assert_eq!(validator.expiry_warning_days, 30);
    }

    #[test]
    fn test_certificate_validator_permissive() {
        let validator = CertificateValidator::permissive();
        assert!(!validator.verify_hostname);
        assert!(!validator.verify_chain);
        assert!(!validator.check_revocation);
    }

    #[test]
    fn test_cipher_suites_modern() {
        let ciphers = cipher_suites::modern();
        assert!(ciphers.len() >= 2);
        assert!(ciphers.iter().any(|c| c.contains("AES_256_GCM")));
    }

    #[test]
    fn test_cipher_suites_balanced() {
        let ciphers = cipher_suites::balanced();
        assert!(ciphers.len() >= 3);
        assert!(ciphers.iter().any(|c| c.contains("ECDHE")));
    }

    #[test]
    fn test_security_headers_default() {
        let headers = SecurityHeaders::default_secure();
        assert!(headers.hsts.is_some());
        assert!(headers.csp.is_some());
    }

    #[test]
    fn test_security_headers_minimal() {
        let headers = SecurityHeaders::minimal();
        assert!(headers.hsts.is_none());
        assert!(headers.content_type_options.is_some());
    }

    #[test]
    fn test_security_headers_to_headers() {
        let headers = SecurityHeaders::default_secure();
        let header_vec = headers.to_headers();
        assert!(!header_vec.is_empty());

        let hsts_header = header_vec.iter().find(|(k, _)| k == &"Strict-Transport-Security");
        assert!(hsts_header.is_some());
    }

    #[test]
    fn test_tls_config_from_files() {
        let mode = HttpsMode::from_files("cert.pem", "key.pem");
        if let HttpsMode::Https(config) = mode {
            assert_eq!(config.cert_path.to_string_lossy(), "cert.pem");
            assert_eq!(config.key_path.to_string_lossy(), "key.pem");
            assert_eq!(config.min_tls_version, TlsVersion::V1_3);
        } else {
            panic!("Expected HTTPS mode");
        }
    }

    #[test]
    fn test_mtls_configuration() {
        let mode = HttpsMode::mtls("cert.pem", "key.pem", "ca.pem");
        if let HttpsMode::Https(config) = mode {
            assert!(config.require_client_cert);
            assert!(config.ca_cert_path.is_some());
        } else {
            panic!("Expected HTTPS mode");
        }
    }
}
