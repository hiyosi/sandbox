//! mTLS server for interoperability testing with Go SPIFFE client

use anyhow::{Context, Result};
use clap::Parser;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType, KeyPair, SignatureAlgorithm};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{self, ServerConfig};
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn, error};
use x509_parser::prelude::*;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server port
    #[arg(short, long, default_value = "8443")]
    port: u16,

    /// Certificate directory path
    #[arg(long, default_value = "../certs")]
    cert_dir: String,

    /// Server certificate file name
    #[arg(long, default_value = "rust-server.crt")]
    server_cert: String,

    /// Server private key file name
    #[arg(long, default_value = "rust-server.key")]
    server_key: String,

    /// CA certificate file name
    #[arg(long, default_value = "ca.crt")]
    ca_cert: String,

    /// CA private key file name
    #[arg(long, default_value = "ca.key")]
    ca_key: String,

    /// Go CA certificate file name for Trust Bundle
    #[arg(long, default_value = "ca.crt")]
    go_ca_cert: String,

    /// Expected trust domain for SPIFFE validation
    #[arg(long, default_value = "example.org")]
    trust_domain: String,

    /// Server SPIFFE ID
    #[arg(long, default_value = "spiffe://example.org/rust-server")]
    server_spiffe_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .init();

    let args = Args::parse();

    info!("Starting Rust mTLS server for SPIFFE interop testing");

    // Load existing SPIFFE-compliant certificates
    let (server_cert, server_key) = load_server_cert(&args)?;
    let ca_cert = load_ca_cert(&args)?;

    // Create TLS configuration
    let config = create_server_config(server_cert, server_key, ca_cert, &args)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));

    // Start listening
    let addr: SocketAddr = ([0, 0, 0, 0], args.port).into();

    // Share args for handlers
    let args_arc = Arc::new(args);
    let listener = TcpListener::bind(&addr).await?;
    info!("mTLS server listening on {}", addr);

    // Accept connections
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        let args_clone = args_arc.clone();
        tokio::spawn(async move {
            info!("Connection from {}", peer_addr);
            if let Err(e) = handle_client(stream, acceptor, args_clone).await {
                warn!("Error handling client {}: {}", peer_addr, e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    acceptor: TlsAcceptor,
    args: Arc<Args>,
) -> Result<()> {
    // Perform TLS handshake
    let tls_stream = acceptor.accept(stream).await
        .context("TLS handshake failed")?;

    info!("✓ mTLS handshake successful");

    // Extract and verify SPIFFE ID from peer certificate
    let (_, server_connection) = tls_stream.get_ref();
    if let Some(certs) = server_connection.peer_certificates() {
        info!("Peer presented {} certificate(s)", certs.len());

        if !certs.is_empty() {
            match verify_spiffe_certificate(&certs[0], &args.trust_domain) {
                Ok(spiffe_id) => {
                    info!("✓ Client SPIFFE ID verified: {}", spiffe_id);
                    info!("✓ Client certificate verified");
                }
                Err(e) => {
                    error!("✗ SPIFFE verification failed: {}", e);
                    return Err(anyhow::anyhow!("SPIFFE verification failed: {}", e));
                }
            }
        }
    } else {
        warn!("No client certificates presented");
        return Err(anyhow::anyhow!("No client certificates presented"));
    }

    // Simple echo server
    let (reader, mut writer) = tokio::io::split(tls_stream);
    let mut reader = BufReader::new(reader).lines();

    while let Some(line) = reader.next_line().await? {
        if line.trim() == "CLOSE" {
            break;
        }
        info!("Received: {}", line.trim());

        // Echo back with confirmation
        let response = format!("RUST_SERVER_ECHO: {}\n", line);
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
    }

    info!("Client disconnected");
    Ok(())
}

fn load_server_cert(args: &Args) -> Result<(Vec<u8>, Vec<u8>)> {
    let cert_path = Path::new(&args.cert_dir).join(&args.server_cert);
    let key_path = Path::new(&args.cert_dir).join(&args.server_key);

    let server_cert = fs::read(cert_path)?;
    let server_key = fs::read(key_path)?;

    info!("✓ Loaded SPIFFE server certificate from {}/{}", args.cert_dir, args.server_cert);
    Ok((server_cert, server_key))
}

fn load_ca_cert(args: &Args) -> Result<Vec<u8>> {
    let ca_path = Path::new(&args.cert_dir).join(&args.ca_cert);
    let ca_cert = fs::read(ca_path)?;

    info!("✓ Loaded CA certificate from {}/{}", args.cert_dir, args.ca_cert);
    Ok(ca_cert)
}

fn generate_server_cert(args: &Args) -> Result<(Vec<u8>, Vec<u8>)> {
    // First generate or load CA certificate
    let ca_cert = load_or_generate_ca_cert(args)?;

    let mut params = CertificateParams::new(vec![
        "localhost".to_string(),
        "spire-rust-server".to_string(),
    ]);

    // Add SPIFFE ID as SAN URI
    params.subject_alt_names.push(SanType::URI(
        "spiffe://example.org/rust-server".to_string()
    ));

    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(
        DnType::CommonName,
        "SPIRE Rust Test Server",
    );

    let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
    params.key_pair = Some(key_pair);

    // Sign with CA certificate
    let cert = Certificate::from_params(params)?;
    let cert_pem = cert.serialize_pem_with_signer(&ca_cert)?;

    Ok((
        cert_pem.into_bytes(),
        cert.serialize_private_key_pem().into_bytes(),
    ))
}

fn generate_ca_cert(args: &Args) -> Result<Vec<u8>> {
    let ca_cert = load_or_generate_ca_cert(args)?;
    Ok(ca_cert.serialize_pem()?.into_bytes())
}

fn load_or_generate_ca_cert(args: &Args) -> Result<Certificate> {
    let ca_cert_path = Path::new(&args.cert_dir).join(&args.ca_cert);
    let ca_key_path = Path::new(&args.cert_dir).join(&args.ca_key);

    // Check if CA files exist, if not generate them
    if !ca_cert_path.exists() || !ca_key_path.exists() {
        info!("Generating new CA certificate");
        return generate_ca_cert_object(args);
    }

    info!("Using existing CA certificate for signing");
    // For simplicity in rcgen 0.12, we'll regenerate the CA object
    // but use the same key if available - this ensures compatibility
    generate_ca_cert_object(args)
}

fn generate_ca_cert_object(args: &Args) -> Result<Certificate> {
    let cert_dir = Path::new(&args.cert_dir);
    fs::create_dir_all(cert_dir)?;

    let ca_key_path = cert_dir.join(&args.ca_key);

    let mut params = CertificateParams::new(vec![]);
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    params.distinguished_name = DistinguishedName::new();
    params.distinguished_name.push(
        DnType::CommonName,
        "SPIRE Test CA",
    );
    params.distinguished_name.push(
        DnType::OrganizationName,
        "example.org",
    );

    // Try to load existing key, otherwise generate new one
    let key_pair = if ca_key_path.exists() {
        info!("Loading existing CA private key");
        let key_pem = fs::read_to_string(ca_key_path)?;
        KeyPair::from_pem(&key_pem)?
    } else {
        info!("Generating new CA private key");
        let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)?;
        // Save the key for future use
        fs::write(ca_key_path, key_pair.serialize_pem())?;
        key_pair
    };

    params.key_pair = Some(key_pair);
    let cert = Certificate::from_params(params)?;

    // Always save/update CA certificate
    fs::write(cert_dir.join(&args.ca_cert), cert.serialize_pem()?)?;

    Ok(cert)
}

fn create_server_config(
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    ca_pem: Vec<u8>,
    args: &Args,
) -> Result<ServerConfig> {
    // Parse certificates
    let cert = ::pem::parse(cert_pem)?;
    let key = ::pem::parse(key_pem)?;
    let ca = ::pem::parse(ca_pem)?;

    let cert_der = CertificateDer::from(cert.contents().to_vec());
    let key_der = PrivateKeyDer::try_from(key.contents().to_vec())
        .map_err(|_| anyhow::anyhow!("Failed to parse private key"))?;

    // Create root cert store for client verification (Trust Bundle)
    let mut root_store = rustls::RootCertStore::empty();

    // Add Rust CA certificate
    root_store.add(CertificateDer::from(ca.contents().to_vec()))
        .map_err(|e| anyhow::anyhow!("Failed to add Rust CA cert: {:?}", e))?;
    info!("✓ Added Rust CA to trust bundle");

    // Try to add Go CA certificate to trust bundle
    let go_ca_path = Path::new(&args.cert_dir).join(&args.go_ca_cert);
    if let Ok(go_ca_pem) = std::fs::read(go_ca_path) {
        if let Ok(go_ca) = ::pem::parse(go_ca_pem) {
            if let Ok(()) = root_store.add(CertificateDer::from(go_ca.contents().to_vec())) {
                info!("✓ Added Go CA to trust bundle");
            } else {
                info!("⚠ Failed to parse Go CA certificate");
            }
        } else {
            info!("⚠ Failed to parse Go CA PEM");
        }
    } else {
        info!("⚠ Go CA certificate not found");
    }

    // Build server config with mTLS
    let client_verifier = rustls::server::WebPkiClientVerifier::builder(
        Arc::new(root_store),
    )
    .build()
    .map_err(|e| anyhow::anyhow!("Failed to build client verifier: {:?}", e))?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(vec![cert_der], key_der)
        .map_err(|e| anyhow::anyhow!("Failed to build server config: {:?}", e))?;

    Ok(config)
}

fn save_certs(
    server_cert: &[u8],
    server_key: &[u8],
    ca_cert: &[u8],
    args: &Args,
) -> Result<()> {
    let cert_dir = Path::new(&args.cert_dir);
    fs::create_dir_all(cert_dir)?;

    fs::write(cert_dir.join(&args.server_cert), server_cert)?;
    fs::write(cert_dir.join(&args.server_key), server_key)?;
    fs::write(cert_dir.join(&args.ca_cert), ca_cert)?;

    info!("Certificates saved to {}/", args.cert_dir);
    Ok(())
}

/// Verify SPIFFE certificate and extract SPIFFE ID
fn verify_spiffe_certificate(cert_der: &CertificateDer, expected_trust_domain: &str) -> Result<String> {
    // Parse the certificate
    let cert_bytes = cert_der.as_ref();
    let (_, cert) = X509Certificate::from_der(cert_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to parse certificate: {}", e))?;

    // Extract SPIFFE ID from SAN (Subject Alternative Name)
    let mut spiffe_id = None;

    for ext in cert.extensions() {
        if ext.oid == x509_parser::oid_registry::OID_X509_EXT_SUBJECT_ALT_NAME {
            if let ParsedExtension::SubjectAlternativeName(san) = &ext.parsed_extension() {
                for name in &san.general_names {
                    if let GeneralName::URI(uri) = name {
                        if uri.starts_with("spiffe://") {
                            spiffe_id = Some(uri.to_string());
                            break;
                        }
                    }
                }
            }
            break;
        }
    }

    let spiffe_id = spiffe_id.ok_or_else(|| anyhow::anyhow!("No SPIFFE ID found in certificate"))?;

    // Validate SPIFFE ID format
    if !spiffe_id.starts_with("spiffe://") {
        return Err(anyhow::anyhow!("Invalid SPIFFE ID format: {}", spiffe_id));
    }

    // Extract trust domain from SPIFFE ID
    let spiffe_parts: Vec<&str> = spiffe_id.strip_prefix("spiffe://").unwrap().split('/').collect();
    if spiffe_parts.is_empty() {
        return Err(anyhow::anyhow!("Invalid SPIFFE ID: missing trust domain"));
    }

    let trust_domain = spiffe_parts[0];

    // Verify trust domain matches expected
    if trust_domain != expected_trust_domain {
        return Err(anyhow::anyhow!(
            "Trust domain mismatch: expected '{}', found '{}'",
            expected_trust_domain,
            trust_domain
        ));
    }

    info!("✓ SPIFFE ID validation passed");
    info!("  - SPIFFE ID: {}", spiffe_id);
    info!("  - Trust Domain: {}", trust_domain);

    Ok(spiffe_id)
}
