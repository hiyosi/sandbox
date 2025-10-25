//! Basic mTLS example using SPIRE client

use spiffe_client::{MtlsConfig, SpiffeId, X509Svid, TrustBundle};
use chrono::Utc;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("SPIRE Client - Basic mTLS Example\n");

    // Create a SPIFFE ID
    let spiffe_id = SpiffeId::new("example.org", "/service/api")?;
    println!("Created SPIFFE ID: {}", spiffe_id);

    // Create a mock X.509 SVID (in production, fetch from SPIRE)
    let svid = create_mock_svid(spiffe_id.clone())?;
    println!("Created X.509 SVID with serial: {}", svid.serial_number());

    // Create a trust bundle
    let trust_bundle = create_mock_trust_bundle()?;
    println!("Created trust bundle for domain: {}", trust_bundle.trust_domain());

    // Create mTLS configuration
    println!("\nCreating mTLS configuration...");
    // Note: This will fail with mock certificates, but demonstrates the API
    match MtlsConfig::from_svid(&svid, &trust_bundle) {
        Ok(config) => {
            println!("✓ mTLS configuration created successfully");
            println!("  - SPIFFE ID: {}", config.spiffe_id());
            println!("  - TLS 1.2+ enforced");
            println!("  - Client certificates configured");
            println!("  - Server configuration available");

            // Example: Create TLS connector for client connections
            let _connector = config.connector();
            println!("✓ TLS connector ready for outbound connections");

            // Example: Create TLS acceptor for server connections
            match config.acceptor() {
                Ok(_acceptor) => {
                    println!("✓ TLS acceptor ready for inbound connections");
                }
                Err(e) => {
                    println!("✗ Failed to create acceptor: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create mTLS config (expected with mock data): {}", e);
        }
    }

    println!("\n--- mTLS Security Requirements (Per Constitution) ---");
    println!("• All communications MUST use mutual TLS");
    println!("• Both client and server authenticate via X.509 certificates");
    println!("• TLS 1.2 minimum, prefer TLS 1.3");
    println!("• Strong cipher suites only");
    println!("• Automatic certificate rotation before expiry");

    Ok(())
}

fn create_mock_svid(spiffe_id: SpiffeId) -> Result<X509Svid, Box<dyn Error>> {
    // In production, this would be fetched from SPIRE agent
    let cert_chain = vec![
        vec![0x30, 0x82], // Mock certificate data
    ];
    let private_key = vec![0x30, 0x82]; // Mock private key
    let expiry = Utc::now() + chrono::Duration::hours(24);
    let serial_number = "1234567890".to_string();

    Ok(X509Svid::new(
        spiffe_id,
        cert_chain,
        private_key,
        expiry,
        serial_number,
    )?)
}

fn create_mock_trust_bundle() -> Result<TrustBundle, Box<dyn Error>> {
    // In production, this would be fetched from SPIRE agent
    let root_cert = vec![0x30, 0x82]; // Mock root certificate
    Ok(TrustBundle::new(
        "example.org".to_string(),
        vec![root_cert],
    ))
}