use crate::error::SpiffeError;
use crate::proto::spire::api::types::Bundle;
use der_parser::{oid, oid::Oid};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::{Error as TlsError, RootCertStore};
use rustls::{DigitallySignedStruct, SignatureScheme};
use rustls_pki_types::{CertificateDer, ServerName};
use std::sync::Arc;
use tonic::transport::{Channel, ClientTlsConfig, Identity};
use x509_parser::prelude::*;

pub struct SpiffeChannelBuilder {
  trust_domain: String,
  bundle: Bundle,
  // オプション設定
  require_client_cert: bool,
  client_svid: Option<(Vec<u8>, Vec<u8>)>, // (cert, key)
}

#[derive(Debug)]
struct SpiffeCertVerifier {
  trust_domain: String,
  root_certs: RootCertStore,
}

impl ServerCertVerifier for SpiffeCertVerifier {
  fn verify_server_cert(
    &self,
    end_entity: &CertificateDer,
    intermediates: &[CertificateDer],
    server_name: &ServerName,
    ocsp_response: &[u8],
    now: rustls_pki_types::UnixTime,
  ) -> Result<ServerCertVerified, TlsError> {
    // 1. SPIFFEカスタム検証
    let spiffe_id = extract_spiffe_id(end_entity).map_err(|e| TlsError::General(e.to_string()))?;
    validate_spiffe_id(&spiffe_id, &self.trust_domain).map_err(|e| TlsError::General(e.to_string()))?;

    // 2. 証明書チェーン検証のみrustlsに委譲
    let temp_verifier = rustls::client::WebPkiServerVerifier::builder(
      Arc::new(self.root_certs.clone())
    ).build().map_err(|e| TlsError::General(format!("Failed to build verifier: {}", e)))?;

    // チェーン検証だけを委譲（server_nameは無視）
    temp_verifier.verify_server_cert(
      end_entity,
      intermediates,
      &ServerName::try_from("ignored").unwrap(), // SPIFFEではserver_nameは使わない
      ocsp_response,
      now
    )?;

    Ok(ServerCertVerified::assertion())
  }

  // 署名検証は rustls のデフォルト実装を使用
  fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, TlsError> {
    let default_verifier = rustls::client::WebPkiServerVerifier::builder(Arc::new(self.root_certs.clone())).build().unwrap();

    default_verifier.verify_tls12_signature(message, cert, dss)
  }

  fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, TlsError> {
    let default_verifier = rustls::client::WebPkiServerVerifier::builder(Arc::new(self.root_certs.clone())).build().unwrap();

    default_verifier.verify_tls13_signature(message, cert, dss)
  }

  fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
    vec![
      SignatureScheme::RSA_PSS_SHA256,
      SignatureScheme::RSA_PSS_SHA384,
      SignatureScheme::RSA_PSS_SHA512,
      SignatureScheme::ECDSA_NISTP256_SHA256,
      SignatureScheme::ECDSA_NISTP384_SHA384,
    ]
  }
}

impl SpiffeChannelBuilder {
  pub fn new(trust_domain: String, bundle: Bundle) -> Self {
    Self {
      trust_domain,
      bundle,
      require_client_cert: false,
      client_svid: None,
    }
  }

  pub fn with_client_svid(mut self, cert: Vec<u8>, key: Vec<u8>) -> Self {
    self.client_svid = Some((cert, key));
    self.require_client_cert = true;
    self
  }

  pub async fn connect(&self, endpoint: String) -> Result<Channel, anyhow::Error> {
    let tls_config = self.build_tls_config()?;

    Channel::from_shared(endpoint)?.tls_config(tls_config)?.connect().await.map_err(Into::into)
  }

  fn build_tls_config(&self) -> Result<ClientTlsConfig, anyhow::Error> {
    // 1. Bundleから証明書を抽出
    let _ca_certs = self.extract_ca_certificates()?;

    // 2. SPIFFE証明書検証器を作成
    let _verifier = SpiffeCertVerifier {
      trust_domain: self.trust_domain.clone(),
      root_certs: RootCertStore::empty(),
    };

    // 3. tonicのTLS設定に変換
    let mut tls = ClientTlsConfig::new().domain_name(&self.trust_domain);

    if let Some((cert, key)) = &self.client_svid {
      let identity = Identity::from_pem(cert.clone(), key.clone());
      tls = tls.identity(identity);
    }

    Ok(tls)
  }

  fn extract_ca_certificates(&self) -> Result<Vec<CertificateDer<'static>>, anyhow::Error> {
    // BundleからX.509証明書を抽出
    self.bundle.x509_authorities.iter().map(|auth| Ok(CertificateDer::from(auth.asn1.clone()))).collect()
  }
}

const SAN_OID: Oid<'static> = oid!(2.5.29.17);
fn extract_spiffe_id(cert: &CertificateDer) -> Result<String, SpiffeError> {
  let (_, cert) = X509Certificate::from_der(cert.as_ref()).map_err(|_| {
    SpiffeError::ValidationError(
      "Failed to parse certificate".to_string())
  })?;

  for ext in cert.extensions() {
    if ext.oid == SAN_OID {
      let san = SubjectAlternativeName::from_der(&ext.value).map_err(|_| SpiffeError::ValidationError("Failed to parse SAN".to_string()))?;

      for name in &san.1.general_names {
        if let GeneralName::URI(uri) = name {
          if uri.starts_with("spiffe://") {
            return Ok(uri.to_string());
          }
        }
      }
    }
  }

  Err(SpiffeError::ValidationError("No SPIFFE ID found in certificate".to_string()))
}

fn validate_spiffe_id(spiffe_id: &str, expected_trust_domain: &str) -> Result<(), anyhow::Error> {
  // spiffe://trust-domain/path の形式チェック
  let parts: Vec<&str> = spiffe_id.strip_prefix("spiffe://").ok_or_else(|| anyhow::anyhow!("Invalid SPIFFE ID format"))?.splitn(2, '/').collect();

  if parts.is_empty() {
    return Err(anyhow::anyhow!("Missing trust domain in SPIFFE ID"));
  }

  let trust_domain = parts[0];
  if trust_domain != expected_trust_domain {
    return Err(anyhow::anyhow!(
      "Trust domain mismatch: expected {}, got {}",
      expected_trust_domain,
      trust_domain
    ));
  }

  Ok(())
}

fn verify_signature(_message: &[u8], _cert: &CertificateDer<'_>, _dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, TlsError> {
  // 実際の署名検証はrustls-webpkiに委譲するか、
  // 独自実装する必要があります
  Ok(HandshakeSignatureValid::assertion())
}
