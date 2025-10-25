//! Error types for the SPIRE client library

use thiserror::Error;

/// Main error type for SPIRE client operations
#[derive(Error, Debug)]
pub enum Error {
    /// SPIFFE ID validation failed
    #[error("Invalid SPIFFE ID: {0}")]
    InvalidSpiffeId(String),

    /// X.509 certificate error
    #[error("X.509 certificate error: {0}")]
    X509Error(String),

    /// JWT token error
    #[error("JWT token error: {0}")]
    JwtError(String),

    /// TLS configuration error
    #[error("TLS configuration error: {0}")]
    TlsError(String),

    /// SPIRE agent communication error
    #[error("SPIRE agent error: {0}")]
    AgentError(String),

    /// Trust bundle validation failed
    #[error("Trust bundle validation failed: {0}")]
    TrustBundleError(String),

    /// Certificate validation failed
    #[error("Certificate validation failed: {0}")]
    ValidationError(String),

    /// Network or I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// gRPC communication error
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    /// URL parsing error
    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    /// Generic error for unspecified conditions
    #[error("{0}")]
    Other(String),
}

/// Convenience type alias for Results with our Error type
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create an InvalidSpiffeId error with detailed message
    pub fn invalid_spiffe_id(msg: impl Into<String>) -> Self {
        Self::InvalidSpiffeId(msg.into())
    }

    /// Create a TlsError with detailed message
    pub fn tls_error(msg: impl Into<String>) -> Self {
        Self::TlsError(msg.into())
    }

    /// Create an AgentError with detailed message
    pub fn agent_error(msg: impl Into<String>) -> Self {
        Self::AgentError(msg.into())
    }
}