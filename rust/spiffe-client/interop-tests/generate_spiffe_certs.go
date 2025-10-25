package main

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"flag"
	"fmt"
	"log"
	"math/big"
	"net"
	"net/url"
	"os"
	"path/filepath"
	"time"
)

var (
	certDir        = flag.String("cert-dir", "certs", "Certificate directory path")
	trustDomain    = flag.String("trust-domain", "example.org", "SPIFFE trust domain")
	clientSpiffeID = flag.String("client-spiffe-id", "spiffe://example.org/go-client", "Client SPIFFE ID")
	serverSpiffeID = flag.String("server-spiffe-id", "spiffe://example.org/go-server", "Server SPIFFE ID")
	rustServerID   = flag.String("rust-server-spiffe-id", "spiffe://example.org/rust-server", "Rust Server SPIFFE ID")
	rustClientID   = flag.String("rust-client-spiffe-id", "spiffe://example.org/rust-client", "Rust Client SPIFFE ID")
)

func main() {
	flag.Parse()

	log.Printf("Generating SPIFFE-compliant certificates for trust domain: %s", *trustDomain)

	// Create certificate directory
	if err := os.MkdirAll(*certDir, 0755); err != nil {
		log.Fatalf("Failed to create cert directory: %v", err)
	}

	// Generate CA certificate
	caCert, caKey, err := generateCA()
	if err != nil {
		log.Fatalf("Failed to generate CA: %v", err)
	}

	// Generate Go client certificate
	if err := generateCert("go-client.crt", "go-client.key", *clientSpiffeID, x509.ExtKeyUsageClientAuth, caCert, caKey); err != nil {
		log.Fatalf("Failed to generate Go client cert: %v", err)
	}

	// Generate Go server certificate
	if err := generateCert("go-server.crt", "go-server.key", *serverSpiffeID, x509.ExtKeyUsageServerAuth, caCert, caKey); err != nil {
		log.Fatalf("Failed to generate Go server cert: %v", err)
	}

	// Generate Rust client certificate
	if err := generateCert("rust-client.crt", "rust-client.key", *rustClientID, x509.ExtKeyUsageClientAuth, caCert, caKey); err != nil {
		log.Fatalf("Failed to generate Rust client cert: %v", err)
	}

	// Generate Rust server certificate
	if err := generateCert("rust-server.crt", "rust-server.key", *rustServerID, x509.ExtKeyUsageServerAuth, caCert, caKey); err != nil {
		log.Fatalf("Failed to generate Rust server cert: %v", err)
	}

	// Create trust bundle (CA certificate)
	trustBundlePath := filepath.Join(*certDir, "trust-bundle.pem")
	caCertPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: caCert.Raw})
	if err := os.WriteFile(trustBundlePath, caCertPEM, 0644); err != nil {
		log.Fatalf("Failed to write trust bundle: %v", err)
	}

	log.Printf("✓ Generated SPIFFE-compliant certificates in %s/", *certDir)
	log.Printf("✓ Trust bundle created: %s", trustBundlePath)
}

func generateCA() (*x509.Certificate, *rsa.PrivateKey, error) {
	log.Printf("Generating CA certificate for trust domain: %s", *trustDomain)

	caKey, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate CA key: %v", err)
	}

	caTemplate := x509.Certificate{
		SerialNumber: big.NewInt(1),
		Subject: pkix.Name{
			CommonName:   fmt.Sprintf("SPIFFE CA - %s", *trustDomain),
			Organization: []string{*trustDomain},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(10 * 365 * 24 * time.Hour), // 10 years
		KeyUsage:              x509.KeyUsageKeyEncipherment | x509.KeyUsageDigitalSignature | x509.KeyUsageCertSign,
		ExtKeyUsage:           []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth, x509.ExtKeyUsageClientAuth},
		BasicConstraintsValid: true,
		IsCA:                  true,
	}

	caCertDER, err := x509.CreateCertificate(rand.Reader, &caTemplate, &caTemplate, &caKey.PublicKey, caKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create CA certificate: %v", err)
	}

	caCert, err := x509.ParseCertificate(caCertDER)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to parse CA certificate: %v", err)
	}

	// Save CA certificate and key
	caCertPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: caCertDER})
	caKeyDER, err := x509.MarshalPKCS8PrivateKey(caKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to marshal CA key: %v", err)
	}
	caKeyPEM := pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: caKeyDER})

	if err := os.WriteFile(filepath.Join(*certDir, "ca.crt"), caCertPEM, 0644); err != nil {
		return nil, nil, fmt.Errorf("failed to write CA cert: %v", err)
	}
	if err := os.WriteFile(filepath.Join(*certDir, "ca.key"), caKeyPEM, 0600); err != nil {
		return nil, nil, fmt.Errorf("failed to write CA key: %v", err)
	}

	log.Printf("✓ Generated CA certificate")
	return caCert, caKey, nil
}

func generateCert(certFile, keyFile, spiffeID string, extKeyUsage x509.ExtKeyUsage, caCert *x509.Certificate, caKey *rsa.PrivateKey) error {
	log.Printf("Generating certificate for SPIFFE ID: %s", spiffeID)

	// Generate private key
	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return fmt.Errorf("failed to generate private key: %v", err)
	}

	// Parse SPIFFE ID
	spiffeURI, err := url.Parse(spiffeID)
	if err != nil {
		return fmt.Errorf("failed to parse SPIFFE URI: %v", err)
	}

	// Create certificate template
	template := x509.Certificate{
		SerialNumber: big.NewInt(time.Now().UnixNano()),
		Subject: pkix.Name{
			CommonName:   spiffeID,
			Organization: []string{*trustDomain},
		},
		NotBefore:             time.Now(),
		NotAfter:              time.Now().Add(365 * 24 * time.Hour),
		KeyUsage:              x509.KeyUsageKeyEncipherment | x509.KeyUsageDigitalSignature,
		ExtKeyUsage:           []x509.ExtKeyUsage{extKeyUsage},
		BasicConstraintsValid: true,
		URIs:                  []*url.URL{spiffeURI},
	}

	// Add DNS names for server certificates
	if extKeyUsage == x509.ExtKeyUsageServerAuth {
		template.DNSNames = []string{"localhost", "server"}
		template.IPAddresses = []net.IP{net.IPv4(127, 0, 0, 1), net.IPv6loopback}
	}

	// Create certificate
	certDER, err := x509.CreateCertificate(rand.Reader, &template, caCert, &privateKey.PublicKey, caKey)
	if err != nil {
		return fmt.Errorf("failed to create certificate: %v", err)
	}

	// Save certificate
	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER})
	certPath := filepath.Join(*certDir, certFile)
	if err := os.WriteFile(certPath, certPEM, 0644); err != nil {
		return fmt.Errorf("failed to write certificate: %v", err)
	}

	// Save private key
	keyDER, err := x509.MarshalPKCS8PrivateKey(privateKey)
	if err != nil {
		return fmt.Errorf("failed to marshal private key: %v", err)
	}
	keyPEM := pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: keyDER})
	keyPath := filepath.Join(*certDir, keyFile)
	if err := os.WriteFile(keyPath, keyPEM, 0600); err != nil {
		return fmt.Errorf("failed to write private key: %v", err)
	}

	log.Printf("✓ Generated certificate: %s", certFile)
	return nil
}