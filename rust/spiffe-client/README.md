# SPIRE Client Rust

> **Note**: This project was written by Claude, an AI assistant by Anthropic.

A secure, idiomatic Rust client library for [SPIFFE](https://spiffe.io/)/[SPIRE](https://github.com/spiffe/spire) with mandatory mTLS support.

## Features

- **Mandatory mTLS**: All network communications use mutual TLS authentication (non-negotiable per constitution)
- **SPIFFE-compliant**: Full compliance with SPIFFE specifications for identity and authentication
- **Security-first design**: Zero-trust assumptions, secure-by-default configurations
- **Library-first architecture**: Reusable components with minimal dependencies
- **Rust idiomatic**: Follows Rust API guidelines and best practices
- **Comprehensive testing**: Unit tests, integration tests, and security-focused fuzzing
- **Observable**: Structured logging via tracing crate ecosystem

## Core Components

### SPIFFE ID
Unique workload identifiers following the SPIFFE specification:

```rust
use spiffe_client::SpiffeId;

let id = SpiffeId::new("example.org", "/service/web")?;
assert_eq!(id.to_string(), "spiffe://example.org/service/web");
```

### X.509 SVID
SPIFFE Verifiable Identity Documents for mTLS authentication:

```rust
use spiffe_client::{X509Svid, SpiffeId};

let svid = X509Svid::new(
    spiffe_id,
    cert_chain,
    private_key,
    expiry,
    serial_number,
)?;

// Check if rotation is needed
if svid.needs_rotation() {
    // Rotate before expiry
}
```

### mTLS Configuration
Enforced mutual TLS for all communications:

```rust
use spiffe_client::{MtlsConfig, X509Svid, TrustBundle};

let config = MtlsConfig::from_svid(&svid, &trust_bundle)?;

// Create TLS connector for client connections
let connector = config.connector();

// Create TLS acceptor for server connections
let acceptor = config.acceptor()?;
```

### Workload API Client
Interface with SPIRE agent for credential management:

```rust
use spiffe_client::WorkloadApiClient;

let client = WorkloadApiClient::new(None).await?;

// Fetch X.509 SVID
let svid = client.fetch_x509_svid().await?;

// Fetch JWT SVID
let jwt = client.fetch_jwt_svid(vec!["audience".to_string()]).await?;

// Fetch trust bundles
let bundles = client.fetch_bundles().await?;
```

## Security Requirements

Per the project constitution, the following are **non-negotiable**:

1. **mTLS Communication**: ALL network communications between services MUST use mutual TLS
   - Both client and server authenticate via X.509 certificates
   - TLS 1.2 minimum (prefer 1.3)
   - Strong cipher suites only
   - Plain HTTP/unencrypted connections are STRICTLY PROHIBITED

2. **SPIFFE-Compliant Authentication**: ALL service authentication MUST be SPIFFE-compliant
   - Identity expressed as SPIFFE IDs (spiffe://trust-domain/path)
   - X.509-SVIDs for service-to-service authentication
   - JWT-SVIDs for specific use cases
   - Automatic identity rotation before expiration
   - Non-SPIFFE authentication methods are STRICTLY PROHIBITED

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spiffe-client = "0.1.0"
```

## Usage Examples

### Basic mTLS Setup

```rust
use spiffe_client::{SpiffeId, MtlsConfig, WorkloadApiClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to SPIRE agent
    let client = WorkloadApiClient::new(None).await?;

    // Fetch credentials
    let svid = client.fetch_x509_svid().await?;
    let trust_bundle = client.get_trust_bundle("example.org").await?;

    // Create mTLS configuration
    let mtls_config = MtlsConfig::from_svid(&svid, &trust_bundle)?;

    // Use for secure connections
    let connector = mtls_config.connector();

    Ok(())
}
```

### Managed Client with Auto-Rotation

```rust
use spiffe_client::{ManagedWorkloadClient, WorkloadApiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = WorkloadApiConfig {
        socket_path: "/tmp/spire-agent/public/api.sock".to_string(),
        auto_rotate: true,
        rotation_interval: 300,
        cache_bundles: true,
    };

    let managed = ManagedWorkloadClient::new(config).await?;

    // Auto-rotation happens in background
    // Use client as needed...

    managed.shutdown().await;
    Ok(())
}
```

## Performance Requirements

The library meets the following performance standards:

- SVID validation: < 1ms for cached trust bundles
- SVID rotation: < 100ms for standard workload attestation
- Memory usage: < 10MB baseline
- Connection pooling for gRPC connections
- Full async/await support

## Compatibility

- SPIFFE specification: v1.x
- SPIRE server: Current and previous major versions
- Rust: 1.70.0+ (stable)
- Platforms: Linux (primary), macOS, Windows (best-effort)
- Architectures: x86_64, aarch64

## Development

### Running Tests

```bash
cargo test
```

### Running Benchmarks

```bash
cargo bench
```

### Security Audit

```bash
cargo audit
cargo deny check
```

## Examples

See the `examples/` directory for complete examples:

- `basic_mtls.rs` - Basic mTLS configuration
- `workload_api.rs` - Workload API usage

Run examples:

```bash
cargo run --example basic_mtls
cargo run --example workload_api
```

## Contributing

Please ensure all contributions comply with the project constitution, particularly:
- Security-first design principles
- Mandatory mTLS and SPIFFE compliance
- Comprehensive testing requirements
- Rust idiomatic patterns

## License

Apache-2.0

## Links

- [SPIFFE](https://spiffe.io/)
- [SPIRE](https://github.com/spiffe/spire)
- [Project Constitution](.specify/memory/constitution.md)