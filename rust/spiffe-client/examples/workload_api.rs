//! Workload API example for interacting with SPIRE agent

use spiffe_client::{WorkloadApiClient, WorkloadApiConfig, ManagedWorkloadClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("SPIRE Client - Workload API Example\n");

    // Example 1: Direct Workload API Client
    println!("--- Direct Workload API Client ---");
    match WorkloadApiClient::new(None).await {
        Ok(client) => {
            println!("✓ Connected to SPIRE agent");

            // Fetch X.509 SVID
            println!("\nFetching X.509 SVID...");
            match client.fetch_x509_svid().await {
                Ok(svid) => {
                    println!("✓ Received X.509 SVID:");
                    println!("  - SPIFFE ID: {}", svid.spiffe_id());
                    println!("  - Expires: {}", svid.expiry());
                    println!("  - Serial: {}", svid.serial_number());
                }
                Err(e) => {
                    println!("✗ Failed to fetch SVID: {}", e);
                }
            }

            // Fetch JWT SVID
            println!("\nFetching JWT SVID for audience 'api.example.org'...");
            let audience = vec!["api.example.org".to_string()];
            match client.fetch_jwt_svid(audience).await {
                Ok(jwt) => {
                    println!("✓ Received JWT SVID:");
                    println!("  - SPIFFE ID: {}", jwt.spiffe_id());
                    println!("  - Audience: {:?}", jwt.audience());
                    println!("  - Expires: {}", jwt.expiry());
                }
                Err(e) => {
                    println!("✗ Failed to fetch JWT: {}", e);
                }
            }

            // Fetch trust bundles
            println!("\nFetching trust bundles...");
            match client.fetch_bundles().await {
                Ok(bundles) => {
                    println!("✓ Received {} trust bundles", bundles.len());
                    for bundle in bundles {
                        println!("  - Domain: {}, Certs: {}",
                            bundle.trust_domain(),
                            bundle.certificates().len()
                        );
                    }
                }
                Err(e) => {
                    println!("✗ Failed to fetch bundles: {}", e);
                }
            }

            // Health check
            println!("\nPerforming health check...");
            match client.health_check().await {
                Ok(()) => {
                    println!("✓ SPIRE agent is healthy");
                }
                Err(e) => {
                    println!("✗ Health check failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to connect to SPIRE agent: {}", e);
            println!("  Make sure SPIRE agent is running at /tmp/spire-agent/public/api.sock");
        }
    }

    // Example 2: Managed Client with Auto-Rotation
    println!("\n--- Managed Workload Client ---");
    let config = WorkloadApiConfig {
        socket_path: "/tmp/spire-agent/public/api.sock".to_string(),
        auto_rotate: true,
        rotation_interval: 300, // 5 minutes
        cache_bundles: true,
    };

    match ManagedWorkloadClient::new(config).await {
        Ok(managed) => {
            println!("✓ Created managed client with:");
            println!("  - Auto-rotation: enabled");
            println!("  - Rotation interval: 5 minutes");
            println!("  - Bundle caching: enabled");

            // Use the client
            let client = managed.client();

            // Refresh all SVIDs and bundles
            println!("\nRefreshing all credentials...");
            match client.refresh_all().await {
                Ok(()) => {
                    println!("✓ All credentials refreshed");
                }
                Err(e) => {
                    println!("✗ Refresh failed: {}", e);
                }
            }

            // Check cached bundle
            if let Some(bundle) = client.get_svid_bundle().await {
                println!("\n✓ Cached SVID bundle available");
                if bundle.needs_rotation() {
                    println!("  ⚠ Rotation needed soon");
                } else {
                    println!("  ✓ No rotation needed yet");
                }
            }

            // Shutdown the managed client
            managed.shutdown().await;
            println!("\n✓ Managed client shutdown");
        }
        Err(e) => {
            println!("✗ Failed to create managed client: {}", e);
        }
    }

    println!("\n--- SPIFFE Authentication Requirements (Per Constitution) ---");
    println!("• All authentication MUST use SPIFFE IDs");
    println!("• X.509-SVIDs for service-to-service mTLS");
    println!("• JWT-SVIDs for specific audience validation");
    println!("• Automatic rotation before expiry");
    println!("• Trust bundles validated against SPIRE server");

    Ok(())
}