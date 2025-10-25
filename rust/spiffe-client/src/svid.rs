//! SPIFFE Verifiable Identity Documents (SVIDs)

use crate::error::{Error, Result};
use crate::spiffe_id::SpiffeId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// X.509 SVID for mTLS authentication
#[derive(Clone, Debug)]
pub struct X509Svid {
    /// The SPIFFE ID for this SVID
    spiffe_id: SpiffeId,
    /// X.509 certificate chain (DER encoded)
    cert_chain: Vec<Vec<u8>>,
    /// Private key (DER encoded)
    private_key: Vec<u8>,
    /// Certificate expiration time
    expiry: DateTime<Utc>,
    /// Certificate serial number
    serial_number: String,
}

impl X509Svid {
    /// Create a new X.509 SVID
    pub fn new(
        spiffe_id: SpiffeId,
        cert_chain: Vec<Vec<u8>>,
        private_key: Vec<u8>,
        expiry: DateTime<Utc>,
        serial_number: String,
    ) -> Result<Self> {
        if cert_chain.is_empty() {
            return Err(Error::X509Error("Certificate chain cannot be empty".into()));
        }

        if private_key.is_empty() {
            return Err(Error::X509Error("Private key cannot be empty".into()));
        }

        // Validate SPIFFE ID
        spiffe_id.validate()?;

        Ok(X509Svid {
            spiffe_id,
            cert_chain,
            private_key,
            expiry,
            serial_number,
        })
    }

    /// Get the SPIFFE ID
    pub fn spiffe_id(&self) -> &SpiffeId {
        &self.spiffe_id
    }

    /// Get the certificate chain
    pub fn cert_chain(&self) -> &[Vec<u8>] {
        &self.cert_chain
    }

    /// Get the leaf certificate
    pub fn leaf_cert(&self) -> &[u8] {
        &self.cert_chain[0]
    }

    /// Get the private key
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }

    /// Check if the SVID has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }

    /// Get the expiration time
    pub fn expiry(&self) -> &DateTime<Utc> {
        &self.expiry
    }

    /// Get time until expiration
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expiry - Utc::now()
    }

    /// Get the serial number
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    /// Validate the SVID
    pub fn validate(&self) -> Result<()> {
        if self.is_expired() {
            return Err(Error::ValidationError("SVID has expired".into()));
        }

        self.spiffe_id.validate()?;

        Ok(())
    }

    /// Check if rotation is needed (within 30% of lifetime)
    pub fn needs_rotation(&self) -> bool {
        let time_until = self.time_until_expiry();
        let total_lifetime = self.expiry - Utc::now() + chrono::Duration::hours(24); // Approximate

        time_until < total_lifetime / 3
    }
}

/// JWT SVID for authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtSvid {
    /// The SPIFFE ID for this SVID
    spiffe_id: SpiffeId,
    /// JWT token string
    token: String,
    /// Token expiration time
    expiry: DateTime<Utc>,
    /// Audience
    audience: Vec<String>,
}

impl JwtSvid {
    /// Create a new JWT SVID
    pub fn new(
        spiffe_id: SpiffeId,
        token: String,
        expiry: DateTime<Utc>,
        audience: Vec<String>,
    ) -> Result<Self> {
        if token.is_empty() {
            return Err(Error::JwtError("Token cannot be empty".into()));
        }

        // Basic JWT format validation
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::JwtError("Invalid JWT format".into()));
        }

        spiffe_id.validate()?;

        Ok(JwtSvid {
            spiffe_id,
            token,
            expiry,
            audience,
        })
    }

    /// Get the SPIFFE ID
    pub fn spiffe_id(&self) -> &SpiffeId {
        &self.spiffe_id
    }

    /// Get the JWT token
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }

    /// Get the expiration time
    pub fn expiry(&self) -> &DateTime<Utc> {
        &self.expiry
    }

    /// Get the audience
    pub fn audience(&self) -> &[String] {
        &self.audience
    }

    /// Check if a specific audience is included
    pub fn has_audience(&self, aud: &str) -> bool {
        self.audience.iter().any(|a| a == aud)
    }

    /// Validate the JWT SVID
    pub fn validate(&self) -> Result<()> {
        if self.is_expired() {
            return Err(Error::ValidationError("JWT has expired".into()));
        }

        self.spiffe_id.validate()?;

        Ok(())
    }
}

/// Bundle of SVIDs for a workload
#[derive(Clone, Debug)]
pub struct SvidBundle {
    /// X.509 SVID for mTLS
    pub x509_svid: Option<Arc<X509Svid>>,
    /// JWT SVIDs by audience
    pub jwt_svids: std::collections::HashMap<String, Arc<JwtSvid>>,
}

impl SvidBundle {
    /// Create a new SVID bundle
    pub fn new(x509_svid: Option<X509Svid>) -> Self {
        SvidBundle {
            x509_svid: x509_svid.map(Arc::new),
            jwt_svids: std::collections::HashMap::new(),
        }
    }

    /// Add a JWT SVID to the bundle
    pub fn add_jwt_svid(&mut self, audience: String, jwt_svid: JwtSvid) {
        self.jwt_svids.insert(audience, Arc::new(jwt_svid));
    }

    /// Get JWT SVID for specific audience
    pub fn get_jwt_svid(&self, audience: &str) -> Option<Arc<JwtSvid>> {
        self.jwt_svids.get(audience).cloned()
    }

    /// Check if any SVIDs need rotation
    pub fn needs_rotation(&self) -> bool {
        if let Some(x509) = &self.x509_svid {
            if x509.needs_rotation() {
                return true;
            }
        }

        self.jwt_svids.values().any(|jwt| jwt.is_expired())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x509_svid_creation() {
        let spiffe_id = SpiffeId::new("example.org", "/service/web").unwrap();
        let cert_chain = vec![vec![1, 2, 3]];
        let private_key = vec![4, 5, 6];
        let expiry = Utc::now() + chrono::Duration::hours(1);
        let serial = "12345".to_string();

        let svid = X509Svid::new(
            spiffe_id.clone(),
            cert_chain,
            private_key,
            expiry,
            serial,
        ).unwrap();

        assert_eq!(svid.spiffe_id(), &spiffe_id);
        assert!(!svid.is_expired());
        assert_eq!(svid.serial_number(), "12345");
    }

    #[test]
    fn test_jwt_svid_creation() {
        let spiffe_id = SpiffeId::new("example.org", "/service/web").unwrap();
        let token = "header.payload.signature".to_string();
        let expiry = Utc::now() + chrono::Duration::hours(1);
        let audience = vec!["api.example.org".to_string()];

        let svid = JwtSvid::new(
            spiffe_id.clone(),
            token,
            expiry,
            audience,
        ).unwrap();

        assert_eq!(svid.spiffe_id(), &spiffe_id);
        assert!(!svid.is_expired());
        assert!(svid.has_audience("api.example.org"));
        assert!(!svid.has_audience("other.example.org"));
    }

    #[test]
    fn test_invalid_jwt_format() {
        let spiffe_id = SpiffeId::new("example.org", "/service/web").unwrap();
        let token = "invalid_token".to_string();
        let expiry = Utc::now() + chrono::Duration::hours(1);
        let audience = vec![];

        let result = JwtSvid::new(spiffe_id, token, expiry, audience);
        assert!(result.is_err());
    }

    #[test]
    fn test_svid_expiration() {
        let spiffe_id = SpiffeId::new("example.org", "/service/web").unwrap();
        let cert_chain = vec![vec![1, 2, 3]];
        let private_key = vec![4, 5, 6];
        let expiry = Utc::now() - chrono::Duration::hours(1); // Already expired
        let serial = "12345".to_string();

        let svid = X509Svid::new(
            spiffe_id,
            cert_chain,
            private_key,
            expiry,
            serial,
        ).unwrap();

        assert!(svid.is_expired());
        assert!(svid.validate().is_err());
    }
}