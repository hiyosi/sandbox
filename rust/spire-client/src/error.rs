#[derive(Debug, thiserror::Error)]
pub enum SpiffeError {
  #[error("Invalid SPIFFE ID format: {0}")]
  InvalidSpiffeId(String),

  #[error(
    "Trust domain mismatch: expected {expected}, got
{actual}"
  )]
  TrustDomainMismatch { expected: String, actual: String },

  #[error("Certificate validation failed: {0}")]
  ValidationError(String),
}
