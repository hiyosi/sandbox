use crate::proto::spire::api::server::bundle::v1::{
    bundle_client::BundleClient as GrpcBundleClient,
    GetBundleRequest,
};
use crate::proto::spire::api::types::Bundle;
use tonic::transport::Channel;
use tonic::Status;

pub struct BundleClient {
    inner: GrpcBundleClient<Channel>,
    endpoint: String,
}

impl BundleClient {
    // コンストラクタ
    pub async fn new(endpoint: &str) -> Result<Self, Status> {
        // Channel作成、TLS設定など
        unimplemented!("BundleClient::new is not yet implemented")
    }

    // 認証なしで呼べるAPI
    pub async fn get_bundle(&mut self) -> Result<Bundle, Status> {
        // GetBundleの実装
        unimplemented!("BundleClient::get_bundle is not yet implemented")
    }

    // 管理者権限が必要なAPI
    pub async fn append_bundle(&mut self, bundle: Bundle) -> Result<Bundle, Status> {
        // 認証設定を含む実装
        unimplemented!("BundleClient::append_bundle is not yet implemented")
    }
}