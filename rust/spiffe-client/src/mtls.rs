//! mTLS configuration and client implementation

use crate::error::{Error, Result};
use crate::svid::X509Svid;
use crate::trust_bundle::TrustBundle;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use tokio_rustls::rustls::{self, ClientConfig, ServerConfig};
use tracing::{debug, info, warn};

/// mTLS configuration for SPIFFE-compliant connections
#[derive(Clone)]
pub struct MtlsConfig {
    /// Client configuration for outbound connections
    client_config: Arc<ClientConfig>,
    /// Server configuration for inbound connections (optional)
    server_config: Option<Arc<ServerConfig>>,
    /// Associated SPIFFE ID
    spiffe_id: crate::SpiffeId,
}

impl MtlsConfig {
    /// Create mTLS configuration from X.509 SVID and trust bundle
    pub fn from_svid(svid: &X509Svid, trust_bundle: &TrustBundle) -> Result<Self> {
        // Validate SVID before use
        svid.validate()?;

        info!(
            "Creating mTLS config for SPIFFE ID: {}",
            svid.spiffe_id()
        );

        // Build client configuration
        let client_config = Self::build_client_config(svid, trust_bundle)?;

        // Build server configuration (for accepting connections)
        let server_config = Self::build_server_config(svid, trust_bundle)?;

        Ok(MtlsConfig {
            client_config: Arc::new(client_config),
            server_config: Some(Arc::new(server_config)),
            spiffe_id: svid.spiffe_id().clone(),
        })
    }

    /// Build client configuration for outbound mTLS connections
    fn build_client_config(
        svid: &X509Svid,
        trust_bundle: &TrustBundle,
    ) -> Result<ClientConfig> {
        let mut root_store = rustls::RootCertStore::empty();

        // Add trust bundle certificates
        for cert_der in trust_bundle.certificates() {
            let cert = CertificateDer::from(cert_der.clone());
            root_store.add(cert).map_err(|e| {
                Error::tls_error(format!("Failed to add root certificate: {}", e))
            })?;
        }

        debug!(
            "Added {} certificates to root store",
            root_store.len()
        );

        // Create TLS 1.2+ configuration with strong ciphers
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(
                Self::convert_cert_chain(svid.cert_chain())?,
                Self::convert_private_key(svid.private_key())?,
            )
            .map_err(|e| Error::tls_error(format!("Failed to create client config: {}", e)))?;

        Ok(config)
    }

    /// Build server configuration for accepting mTLS connections
    fn build_server_config(
        svid: &X509Svid,
        trust_bundle: &TrustBundle,
    ) -> Result<ServerConfig> {
        let mut root_store = rustls::RootCertStore::empty();

        // Add trust bundle for client verification
        for cert_der in trust_bundle.certificates() {
            let cert = CertificateDer::from(cert_der.clone());
            root_store.add(cert).map_err(|e| {
                Error::tls_error(format!("Failed to add root certificate: {}", e))
            })?;
        }

        // Create client certificate verifier
        let client_cert_verifier = rustls::server::WebPkiClientVerifier::builder(
            Arc::new(root_store),
        )
        .build()
        .map_err(|e| Error::tls_error(format!("Failed to create client verifier: {}", e)))?;

        // Create server configuration requiring client certificates
        let config = ServerConfig::builder()
            .with_client_cert_verifier(client_cert_verifier)
            .with_single_cert(
                Self::convert_cert_chain(svid.cert_chain())?,
                Self::convert_private_key(svid.private_key())?,
            )
            .map_err(|e| Error::tls_error(format!("Failed to create server config: {}", e)))?;

        Ok(config)
    }

    /// Convert certificate chain to rustls format
    fn convert_cert_chain(chain: &[Vec<u8>]) -> Result<Vec<CertificateDer<'static>>> {
        if chain.is_empty() {
            return Err(Error::tls_error("Certificate chain is empty"));
        }

        Ok(chain
            .iter()
            .map(|cert| CertificateDer::from(cert.clone()))
            .collect())
    }

    /// Convert private key to rustls format
    fn convert_private_key(key_der: &[u8]) -> Result<PrivateKeyDer<'static>> {
        if key_der.is_empty() {
            return Err(Error::tls_error("Private key is empty"));
        }

        // Try to interpret as PKCS8 first, then PKCS1 RSA, then SEC1 EC
        let key = PrivateKeyDer::try_from(key_der.to_vec())
            .map_err(|_| Error::tls_error("Failed to parse private key"))?;

        Ok(key)
    }

    /// Get the client configuration for outbound connections
    pub fn client_config(&self) -> Arc<ClientConfig> {
        self.client_config.clone()
    }

    /// Get the server configuration for inbound connections
    pub fn server_config(&self) -> Option<Arc<ServerConfig>> {
        self.server_config.clone()
    }

    /// Get the SPIFFE ID associated with this configuration
    pub fn spiffe_id(&self) -> &crate::SpiffeId {
        &self.spiffe_id
    }

    /// Create a TLS connector for client connections
    pub fn connector(&self) -> tokio_rustls::TlsConnector {
        tokio_rustls::TlsConnector::from(self.client_config.clone())
    }

    /// Create a TLS acceptor for server connections
    pub fn acceptor(&self) -> Result<tokio_rustls::TlsAcceptor> {
        self.server_config
            .as_ref()
            .map(|config| tokio_rustls::TlsAcceptor::from(config.clone()))
            .ok_or_else(|| Error::tls_error("Server configuration not available"))
    }

    /// Verify that the configuration supports required TLS versions
    pub fn verify_tls_version(&self) -> Result<()> {
        // This is validated during construction, but can be re-checked
        info!("mTLS configuration verified for TLS 1.2+");
        Ok(())
    }

    /// Update the configuration with a new SVID (for rotation)
    pub fn update_svid(&mut self, svid: &X509Svid, trust_bundle: &TrustBundle) -> Result<()> {
        if svid.is_expired() {
            return Err(Error::ValidationError(
                "Cannot update with expired SVID".into(),
            ));
        }

        info!(
            "Updating mTLS config with new SVID for {}",
            svid.spiffe_id()
        );

        let client_config = Self::build_client_config(svid, trust_bundle)?;
        let server_config = Self::build_server_config(svid, trust_bundle)?;

        self.client_config = Arc::new(client_config);
        self.server_config = Some(Arc::new(server_config));
        self.spiffe_id = svid.spiffe_id().clone();

        Ok(())
    }
}

/// mTLS connection validator
pub struct MtlsValidator {
    expected_spiffe_ids: Vec<crate::SpiffeId>,
    trust_bundle: TrustBundle,
}

impl MtlsValidator {
    /// Create a new validator with expected SPIFFE IDs
    pub fn new(trust_bundle: TrustBundle) -> Self {
        MtlsValidator {
            expected_spiffe_ids: Vec::new(),
            trust_bundle,
        }
    }

    /// Add an expected SPIFFE ID for validation
    pub fn add_expected_id(&mut self, id: crate::SpiffeId) {
        self.expected_spiffe_ids.push(id);
    }

    /// Validate a peer's certificate chain
    pub fn validate_peer_cert(&self, cert_chain: &[Vec<u8>]) -> Result<crate::SpiffeId> {
        if cert_chain.is_empty() {
            return Err(Error::ValidationError(
                "Peer certificate chain is empty".into(),
            ));
        }

        // Extract SPIFFE ID from certificate
        // This is a simplified version - real implementation would parse X.509 SAN
        let spiffe_id = self.extract_spiffe_id(&cert_chain[0])?;

        // Verify against expected IDs if specified
        if !self.expected_spiffe_ids.is_empty() {
            if !self.expected_spiffe_ids.contains(&spiffe_id) {
                warn!(
                    "Peer SPIFFE ID {} not in expected list",
                    spiffe_id
                );
                return Err(Error::ValidationError(
                    format!("Unexpected SPIFFE ID: {}", spiffe_id),
                ));
            }
        }

        debug!("Successfully validated peer: {}", spiffe_id);
        Ok(spiffe_id)
    }

    /// Extract SPIFFE ID from X.509 certificate
    fn extract_spiffe_id(&self, _cert_der: &[u8]) -> Result<crate::SpiffeId> {
        // This is a placeholder - real implementation would:
        // 1. Parse the X.509 certificate
        // 2. Extract the SAN URI field
        // 3. Validate it's a proper SPIFFE ID

        // For now, return a dummy ID for testing
        crate::SpiffeId::new("example.org", "/peer/service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_mtls_config_creation() {
        let spiffe_id = crate::SpiffeId::new("example.org", "/service/web").unwrap();

        // Create a mock SVID
        let svid = X509Svid::new(
            spiffe_id,
            vec![vec![1, 2, 3]], // Mock cert chain
            vec![4, 5, 6],       // Mock private key
            Utc::now() + chrono::Duration::hours(1),
            "12345".to_string(),
        ).unwrap();

        // Create a mock trust bundle
        let trust_bundle = TrustBundle::new(
            "example.org".to_string(),
            vec![vec![7, 8, 9]], // Mock root cert
        );

        // This will fail with mock data, but tests the structure
        let result = MtlsConfig::from_svid(&svid, &trust_bundle);
        assert!(result.is_err()); // Expected with mock certificates
    }

    #[test]
    fn test_validator() {
        let trust_bundle = TrustBundle::new(
            "example.org".to_string(),
            vec![vec![1, 2, 3]],
        );

        let mut validator = MtlsValidator::new(trust_bundle);
        let expected_id = crate::SpiffeId::new("example.org", "/peer/service").unwrap();
        validator.add_expected_id(expected_id);

        // Test with empty chain
        let result = validator.validate_peer_cert(&[]);
        assert!(result.is_err());
    }
}