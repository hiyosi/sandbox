//! SPIFFE ID types and validation

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use url::Url;

/// A SPIFFE ID uniquely identifies a workload
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpiffeId {
    trust_domain: String,
    path: String,
    raw_url: Url,
}

impl SpiffeId {
    /// Create a new SPIFFE ID from trust domain and path
    ///
    /// # Examples
    /// ```
    /// use spiffe_client::SpiffeId;
    ///
    /// let id = SpiffeId::new("example.org", "/service/web").unwrap();
    /// assert_eq!(id.to_string(), "spiffe://example.org/service/web");
    /// ```
    pub fn new(trust_domain: impl AsRef<str>, path: impl AsRef<str>) -> Result<Self> {
        let trust_domain = trust_domain.as_ref();
        let path = path.as_ref();

        // Validate trust domain
        if trust_domain.is_empty() {
            return Err(Error::invalid_spiffe_id("Trust domain cannot be empty"));
        }

        if trust_domain.contains('/') || trust_domain.contains(':') {
            return Err(Error::invalid_spiffe_id(
                "Trust domain cannot contain '/' or ':'",
            ));
        }

        // Validate path - must not be empty
        if path.is_empty() {
            return Err(Error::invalid_spiffe_id("Path cannot be empty"));
        }

        let path = if !path.starts_with('/') {
            format!("/{}", path)
        } else {
            path.to_string()
        };

        // Construct full SPIFFE ID URL
        let url_str = format!("spiffe://{}{}", trust_domain, path);
        let raw_url = Url::parse(&url_str)?;

        // Additional SPIFFE-specific validation
        if raw_url.scheme() != "spiffe" {
            return Err(Error::invalid_spiffe_id("Scheme must be 'spiffe'"));
        }

        if raw_url.cannot_be_a_base() {
            return Err(Error::invalid_spiffe_id("Invalid SPIFFE ID structure"));
        }

        Ok(SpiffeId {
            trust_domain: trust_domain.to_string(),
            path,
            raw_url,
        })
    }

    /// Parse a SPIFFE ID from a string
    ///
    /// # Examples
    /// ```
    /// use spiffe_client::SpiffeId;
    ///
    /// let id = SpiffeId::parse("spiffe://example.org/service/web").unwrap();
    /// assert_eq!(id.trust_domain(), "example.org");
    /// assert_eq!(id.path(), "/service/web");
    /// ```
    pub fn parse(s: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(s.as_ref())?;

        if url.scheme() != "spiffe" {
            return Err(Error::invalid_spiffe_id(format!(
                "Invalid scheme '{}', expected 'spiffe'",
                url.scheme()
            )));
        }

        let trust_domain = url
            .host_str()
            .ok_or_else(|| Error::invalid_spiffe_id("Missing trust domain"))?;

        if trust_domain.is_empty() {
            return Err(Error::invalid_spiffe_id("Empty trust domain"));
        }

        let path = url.path();
        if path.is_empty() || path == "/" {
            return Err(Error::invalid_spiffe_id(
                "SPIFFE ID must have a non-empty path",
            ));
        }

        Ok(SpiffeId {
            trust_domain: trust_domain.to_string(),
            path: path.to_string(),
            raw_url: url,
        })
    }

    /// Get the trust domain
    pub fn trust_domain(&self) -> &str {
        &self.trust_domain
    }

    /// Get the path component
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the full SPIFFE ID as a URL
    pub fn as_url(&self) -> &Url {
        &self.raw_url
    }

    /// Check if this ID belongs to the specified trust domain
    pub fn is_member_of(&self, trust_domain: &str) -> bool {
        self.trust_domain == trust_domain
    }

    /// Validate that this ID matches expected patterns
    pub fn validate(&self) -> Result<()> {
        // Trust domain validation
        if self.trust_domain.is_empty() {
            return Err(Error::invalid_spiffe_id("Empty trust domain"));
        }

        // Path validation
        if self.path.is_empty() || self.path == "/" {
            return Err(Error::invalid_spiffe_id("Invalid path"));
        }

        // Check for invalid characters
        if self.path.contains("..") {
            return Err(Error::invalid_spiffe_id(
                "Path cannot contain '..' segments",
            ));
        }

        Ok(())
    }
}

impl fmt::Display for SpiffeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_url)
    }
}

impl FromStr for SpiffeId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiffe_id_creation() {
        let id = SpiffeId::new("example.org", "/service/web").unwrap();
        assert_eq!(id.trust_domain(), "example.org");
        assert_eq!(id.path(), "/service/web");
        assert_eq!(id.to_string(), "spiffe://example.org/service/web");
    }

    #[test]
    fn test_spiffe_id_parsing() {
        let id = SpiffeId::parse("spiffe://example.org/service/web").unwrap();
        assert_eq!(id.trust_domain(), "example.org");
        assert_eq!(id.path(), "/service/web");
    }

    #[test]
    fn test_invalid_spiffe_id() {
        assert!(SpiffeId::parse("http://example.org/service").is_err());
        assert!(SpiffeId::parse("spiffe://").is_err());
        assert!(SpiffeId::parse("spiffe://example.org").is_err());
        assert!(SpiffeId::new("", "/service").is_err());
        assert!(SpiffeId::new("example.org", "").is_err());
    }

    #[test]
    fn test_spiffe_id_validation() {
        let id = SpiffeId::new("example.org", "/service/web").unwrap();
        assert!(id.validate().is_ok());

        // Test path with .. segments (should fail)
        let url = Url::parse("spiffe://example.org/../etc/passwd").unwrap();
        let id = SpiffeId {
            trust_domain: "example.org".to_string(),
            path: "/../etc/passwd".to_string(),
            raw_url: url,
        };
        assert!(id.validate().is_err());
    }

    #[test]
    fn test_trust_domain_membership() {
        let id = SpiffeId::new("example.org", "/service/web").unwrap();
        assert!(id.is_member_of("example.org"));
        assert!(!id.is_member_of("other.org"));
    }
}