// SPIFFE準拠の証明書検証器
pub struct SpiffeVerifier {
  trust_domain: String,
  bundle: Bundle, // SPIREから取得したbundle
}

// 検証トレイト
pub trait CertificateVerifier {
  fn verify_server_cert(&self, cert: &Certificate) -> Result<SpiffeId, Error>;
}
