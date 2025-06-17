// SPIFFE ID表現
pub struct SpiffeId {
  trust_domain: String,
  path: String,
}

// X.509 SVID
pub struct X509Svid {
  certificate: Certificate,
  spiffe_id: SpiffeId,
}
