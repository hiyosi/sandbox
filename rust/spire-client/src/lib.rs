pub mod proto {
  pub mod spire {
    pub mod api {
      pub mod server {
        pub mod bundle {
          pub mod v1 {
            tonic::include_proto!("spire.api.server.bundle.v1");
          }
        }
      }
      pub mod types {
        tonic::include_proto!("spire.api.types");
      }
    }
  }
}

pub mod client;
pub use client::bundle::BundleClient;
pub mod error;
pub mod transport;
