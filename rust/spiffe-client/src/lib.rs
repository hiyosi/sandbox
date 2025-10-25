//! SPIRE Client Library for Rust
//!
//! A secure, idiomatic Rust client for SPIFFE/SPIRE with mandatory mTLS support.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod spiffe_id;
pub mod svid;
pub mod trust_bundle;
pub mod workload;
pub mod mtls;

pub use error::{Error, Result};
pub use spiffe_id::SpiffeId;
pub use svid::{X509Svid, JwtSvid};
pub use trust_bundle::TrustBundle;
pub use workload::{WorkloadApiClient, WorkloadApiConfig, ManagedWorkloadClient};
pub use mtls::MtlsConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify all public types are accessible
        let _ = std::mem::size_of::<SpiffeId>();
        let _ = std::mem::size_of::<X509Svid>();
        let _ = std::mem::size_of::<TrustBundle>();
    }
}