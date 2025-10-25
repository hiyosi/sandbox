//! Trust bundle management for SPIFFE

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Trust bundle containing root certificates for a trust domain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustBundle {
    /// Trust domain this bundle belongs to
    trust_domain: String,
    /// Root CA certificates (DER encoded)
    certificates: Vec<Vec<u8>>,
    /// Bundle sequence number for versioning
    sequence_number: u64,
    /// Last update time
    updated_at: DateTime<Utc>,
}

impl TrustBundle {
    /// Create a new trust bundle
    pub fn new(trust_domain: String, certificates: Vec<Vec<u8>>) -> Self {
        TrustBundle {
            trust_domain,
            certificates,
            sequence_number: 0,
            updated_at: Utc::now(),
        }
    }

    /// Create a trust bundle with sequence number
    pub fn with_sequence(
        trust_domain: String,
        certificates: Vec<Vec<u8>>,
        sequence_number: u64,
    ) -> Self {
        TrustBundle {
            trust_domain,
            certificates,
            sequence_number,
            updated_at: Utc::now(),
        }
    }

    /// Get the trust domain
    pub fn trust_domain(&self) -> &str {
        &self.trust_domain
    }

    /// Get the root certificates
    pub fn certificates(&self) -> &[Vec<u8>] {
        &self.certificates
    }

    /// Get the sequence number
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Get the last update time
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Validate the trust bundle
    pub fn validate(&self) -> Result<()> {
        if self.trust_domain.is_empty() {
            return Err(Error::TrustBundleError(
                "Trust domain cannot be empty".into(),
            ));
        }

        if self.certificates.is_empty() {
            return Err(Error::TrustBundleError(
                "Trust bundle must contain at least one certificate".into(),
            ));
        }

        for (i, cert) in self.certificates.iter().enumerate() {
            if cert.is_empty() {
                return Err(Error::TrustBundleError(
                    format!("Certificate {} is empty", i),
                ));
            }
        }

        Ok(())
    }

    /// Check if this bundle is newer than another
    pub fn is_newer_than(&self, other: &TrustBundle) -> bool {
        self.sequence_number > other.sequence_number
    }

    /// Merge with another trust bundle (for federation)
    pub fn merge(&mut self, other: &TrustBundle) -> Result<()> {
        if self.trust_domain != other.trust_domain {
            return Err(Error::TrustBundleError(
                "Cannot merge bundles from different trust domains".into(),
            ));
        }

        if other.is_newer_than(self) {
            self.certificates = other.certificates.clone();
            self.sequence_number = other.sequence_number;
            self.updated_at = other.updated_at;
            info!(
                "Updated trust bundle for {} to sequence {}",
                self.trust_domain, self.sequence_number
            );
        }

        Ok(())
    }

    /// Add a certificate to the bundle
    pub fn add_certificate(&mut self, cert: Vec<u8>) -> Result<()> {
        if cert.is_empty() {
            return Err(Error::TrustBundleError(
                "Cannot add empty certificate".into(),
            ));
        }

        self.certificates.push(cert);
        self.sequence_number += 1;
        self.updated_at = Utc::now();

        debug!(
            "Added certificate to trust bundle for {}, now {} certificates",
            self.trust_domain,
            self.certificates.len()
        );

        Ok(())
    }

    /// Remove expired certificates (placeholder - needs X.509 parsing)
    pub fn prune_expired(&mut self) -> usize {
        // In a real implementation, this would:
        // 1. Parse each certificate
        // 2. Check expiration dates
        // 3. Remove expired ones
        // For now, just return 0
        0
    }
}

/// Trust bundle store for managing multiple trust domains
#[derive(Clone, Debug)]
pub struct TrustBundleStore {
    /// Trust bundles by domain
    bundles: Arc<parking_lot::RwLock<HashMap<String, TrustBundle>>>,
}

impl TrustBundleStore {
    /// Create a new trust bundle store
    pub fn new() -> Self {
        TrustBundleStore {
            bundles: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a trust bundle
    pub fn set_bundle(&self, bundle: TrustBundle) -> Result<()> {
        bundle.validate()?;

        let trust_domain = bundle.trust_domain().to_string();
        let seq = bundle.sequence_number();

        let mut bundles = self.bundles.write();
        bundles.insert(trust_domain.clone(), bundle);

        info!(
            "Set trust bundle for domain {} with sequence {}",
            trust_domain, seq
        );

        Ok(())
    }

    /// Get a trust bundle for a domain
    pub fn get_bundle(&self, trust_domain: &str) -> Option<TrustBundle> {
        let bundles = self.bundles.read();
        bundles.get(trust_domain).cloned()
    }

    /// Remove a trust bundle
    pub fn remove_bundle(&self, trust_domain: &str) -> Option<TrustBundle> {
        let mut bundles = self.bundles.write();
        let removed = bundles.remove(trust_domain);

        if removed.is_some() {
            info!("Removed trust bundle for domain {}", trust_domain);
        }

        removed
    }

    /// Get all trust domains
    pub fn domains(&self) -> Vec<String> {
        let bundles = self.bundles.read();
        bundles.keys().cloned().collect()
    }

    /// Get all trust bundles
    pub fn all_bundles(&self) -> Vec<TrustBundle> {
        let bundles = self.bundles.read();
        bundles.values().cloned().collect()
    }

    /// Update bundle if newer
    pub fn update_if_newer(&self, bundle: TrustBundle) -> Result<bool> {
        bundle.validate()?;

        let trust_domain = bundle.trust_domain().to_string();
        let mut bundles = self.bundles.write();

        let updated = if let Some(existing) = bundles.get(&trust_domain) {
            if bundle.is_newer_than(existing) {
                bundles.insert(trust_domain.clone(), bundle);
                true
            } else {
                false
            }
        } else {
            bundles.insert(trust_domain.clone(), bundle);
            true
        };

        if updated {
            info!("Updated trust bundle for domain {}", trust_domain);
        }

        Ok(updated)
    }

    /// Prune expired certificates from all bundles
    pub fn prune_all_expired(&self) -> usize {
        let mut total_pruned = 0;
        let mut bundles = self.bundles.write();

        for bundle in bundles.values_mut() {
            total_pruned += bundle.prune_expired();
        }

        if total_pruned > 0 {
            info!("Pruned {} expired certificates", total_pruned);
        }

        total_pruned
    }
}

impl Default for TrustBundleStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Federated trust bundles for cross-domain trust
#[derive(Clone, Debug)]
pub struct FederatedBundle {
    /// Primary trust domain
    primary_domain: String,
    /// Store of all trust bundles
    store: TrustBundleStore,
}

impl FederatedBundle {
    /// Create a new federated bundle
    pub fn new(primary_domain: String) -> Self {
        FederatedBundle {
            primary_domain,
            store: TrustBundleStore::new(),
        }
    }

    /// Set the primary trust bundle
    pub fn set_primary(&self, bundle: TrustBundle) -> Result<()> {
        if bundle.trust_domain() != self.primary_domain {
            return Err(Error::TrustBundleError(
                format!(
                    "Bundle domain {} doesn't match primary {}",
                    bundle.trust_domain(),
                    self.primary_domain
                ),
            ));
        }

        self.store.set_bundle(bundle)
    }

    /// Add a federated trust bundle
    pub fn add_federated(&self, bundle: TrustBundle) -> Result<()> {
        if bundle.trust_domain() == self.primary_domain {
            return Err(Error::TrustBundleError(
                "Use set_primary for primary domain".into(),
            ));
        }

        self.store.set_bundle(bundle)
    }

    /// Get the primary bundle
    pub fn primary(&self) -> Option<TrustBundle> {
        self.store.get_bundle(&self.primary_domain)
    }

    /// Get all federated domains (excluding primary)
    pub fn federated_domains(&self) -> Vec<String> {
        self.store
            .domains()
            .into_iter()
            .filter(|d| d != &self.primary_domain)
            .collect()
    }

    /// Validate trust for a SPIFFE ID
    pub fn validate_spiffe_id(&self, spiffe_id: &crate::SpiffeId) -> Result<()> {
        let trust_domain = spiffe_id.trust_domain();

        if self.store.get_bundle(trust_domain).is_none() {
            return Err(Error::ValidationError(
                format!("No trust bundle for domain: {}", trust_domain),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_bundle_creation() {
        let bundle = TrustBundle::new(
            "example.org".to_string(),
            vec![vec![1, 2, 3]],
        );

        assert_eq!(bundle.trust_domain(), "example.org");
        assert_eq!(bundle.certificates().len(), 1);
        assert_eq!(bundle.sequence_number(), 0);
    }

    #[test]
    fn test_trust_bundle_validation() {
        let bundle = TrustBundle::new(
            "example.org".to_string(),
            vec![vec![1, 2, 3]],
        );
        assert!(bundle.validate().is_ok());

        let empty_domain = TrustBundle::new(
            "".to_string(),
            vec![vec![1, 2, 3]],
        );
        assert!(empty_domain.validate().is_err());

        let empty_certs = TrustBundle::new(
            "example.org".to_string(),
            vec![],
        );
        assert!(empty_certs.validate().is_err());
    }

    #[test]
    fn test_trust_bundle_store() {
        let store = TrustBundleStore::new();

        let bundle1 = TrustBundle::new(
            "example.org".to_string(),
            vec![vec![1, 2, 3]],
        );
        store.set_bundle(bundle1.clone()).unwrap();

        let retrieved = store.get_bundle("example.org");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().trust_domain(), "example.org");

        let bundle2 = TrustBundle::with_sequence(
            "example.org".to_string(),
            vec![vec![4, 5, 6]],
            2,
        );
        assert!(store.update_if_newer(bundle2).unwrap());

        let updated = store.get_bundle("example.org").unwrap();
        assert_eq!(updated.sequence_number(), 2);
    }

    #[test]
    fn test_federated_bundle() {
        let federated = FederatedBundle::new("primary.org".to_string());

        let primary = TrustBundle::new(
            "primary.org".to_string(),
            vec![vec![1, 2, 3]],
        );
        federated.set_primary(primary).unwrap();

        let secondary = TrustBundle::new(
            "partner.org".to_string(),
            vec![vec![4, 5, 6]],
        );
        federated.add_federated(secondary).unwrap();

        assert!(federated.primary().is_some());
        assert_eq!(federated.federated_domains(), vec!["partner.org"]);

        let spiffe_id = crate::SpiffeId::new("partner.org", "/service").unwrap();
        assert!(federated.validate_spiffe_id(&spiffe_id).is_ok());

        let unknown_id = crate::SpiffeId::new("unknown.org", "/service").unwrap();
        assert!(federated.validate_spiffe_id(&unknown_id).is_err());
    }
}