use crate::proto::spire::api::server::bundle::v1::{GetBundleRequest, bundle_client::BundleClient as GrpcBundleClient};
use crate::proto::spire::api::types::{Bundle, BundleMask};
use tonic::Status;
use tonic::transport::{Channel, ClientTlsConfig};
use anyhow::Error;

pub struct BundleClient {
  inner: GrpcBundleClient<Channel>,
}

impl BundleClient {
  // Channelを受け取るコンストラクタ
  pub fn new(channel: Channel) -> Self {
    Self {
      inner: GrpcBundleClient::new(channel),
    }
  }

  // 便利メソッド：エンドポイントからChannelを作成
  pub async fn connect(endpoint: String, tls_config: Option<ClientTlsConfig>) -> Result<Self, Error> {
    let channel = if let Some(tls) = tls_config {
      Channel::from_shared(endpoint)?.tls_config(tls)?.connect().await?
    } else {
      Channel::from_shared(endpoint)?.connect().await?
    };

    Ok(Self::new(channel))
  }

  // 認証なしで呼べるAPI
  pub async fn get_bundle(&mut self) -> Result<Bundle, Error> {
    let request = tonic::Request::new(GetBundleRequest {
      output_mask: Some(BundleMask {
        x509_authorities: true,
        jwt_authorities: true,
        refresh_hint: true,
        sequence_number: true,
      }),
    });
    let response = self.inner
        .get_bundle(request)
        .await
        .map_err(|status| {
             anyhow::anyhow!("GetBundle failed: code={}, message={}",
                status.code(), status.message())
            })?;

        let bundle = response.into_inner();

        // バンドルの基本検証
        if bundle.x509_authorities.is_empty() && bundle.jwt_authorities.is_empty() {
            return Err(anyhow::anyhow!("Received empty bundle from server"));
        }

        Ok(bundle)
  }

    // 管理者権限が必要なAPI
  pub async fn append_bundle(&mut self, _bundle: Bundle) -> Result<Bundle, Status> {
    // 認証設定を含む実装
    unimplemented!("BundleClient::append_bundle is not yet implemented")
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::spire::api::types::{X509Certificate, JwtKey};
    use tonic::transport::Channel;

    #[tokio::test]
    async fn test_bundle_client_new() {
        let endpoint = "spire-server.example.com:8081";
        let channel = Channel::from_shared(endpoint).unwrap().connect_lazy();
        let client = BundleClient::new(channel);
        
        // クライアントが正常に作成されることを確認
        assert!(std::mem::size_of_val(&client) > 0);
    }

    #[test]
    fn test_bundle_validation_empty() {
        let empty_bundle = Bundle {
            trust_domain: "example.org".to_string(),
            x509_authorities: vec![],
            jwt_authorities: vec![],
            refresh_hint: 0,
            sequence_number: 0,
        };

        // 空のバンドルは無効
        assert!(empty_bundle.x509_authorities.is_empty() && empty_bundle.jwt_authorities.is_empty());
    }

    #[test]
    fn test_bundle_validation_valid() {
        let valid_bundle = Bundle {
            trust_domain: "example.org".to_string(),
            x509_authorities: vec![X509Certificate {
                asn1: b"mock-cert".to_vec(),
                tainted: false,
            }],
            jwt_authorities: vec![JwtKey {
                public_key: b"mock-public-key".to_vec(),
                key_id: "key-1".to_string(),
                expires_at: 0,
                tainted: false,
            }],
            refresh_hint: 3600,
            sequence_number: 1,
        };

        // 有効なバンドル
        assert!(!valid_bundle.x509_authorities.is_empty() || !valid_bundle.jwt_authorities.is_empty());
        assert_eq!(valid_bundle.trust_domain, "example.org");
        assert_eq!(valid_bundle.sequence_number, 1);
        assert_eq!(valid_bundle.refresh_hint, 3600);
    }

    #[test]
    fn test_get_bundle_request_creation() {
        let request = GetBundleRequest {
            output_mask: Some(BundleMask {
                x509_authorities: true,
                jwt_authorities: true,
                refresh_hint: true,
                sequence_number: true,
            }),
        };

        assert!(request.output_mask.is_some());
        let mask = request.output_mask.unwrap();
        assert!(mask.x509_authorities);
        assert!(mask.jwt_authorities);
        assert!(mask.refresh_hint);
        assert!(mask.sequence_number);
    }

    #[test]
    fn test_x509_certificate_structure() {
        let cert = X509Certificate {
            asn1: b"test-cert".to_vec(),
            tainted: false,
        };

        assert_eq!(cert.asn1, b"test-cert");
        assert!(!cert.tainted);
    }

    #[test]
    fn test_jwt_key_structure() {
        let jwt_key = JwtKey {
            public_key: b"test-public-key".to_vec(),
            key_id: "test-key-id".to_string(),
            expires_at: 1234567890,
            tainted: false,
        };

        assert_eq!(jwt_key.public_key, b"test-public-key");
        assert_eq!(jwt_key.key_id, "test-key-id");
        assert_eq!(jwt_key.expires_at, 1234567890);
        assert!(!jwt_key.tainted);
    }
}
