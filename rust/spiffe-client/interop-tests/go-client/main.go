package main

import (
	"bufio"
	"crypto/tls"
	"crypto/x509"
	"encoding/pem"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/spiffe/go-spiffe/v2/bundle/x509bundle"
	"github.com/spiffe/go-spiffe/v2/spiffeid"
	"github.com/spiffe/go-spiffe/v2/spiffetls/tlsconfig"
	"github.com/spiffe/go-spiffe/v2/svid/x509svid"
)

var (
	serverAddr     = flag.String("server", "localhost", "Server address")
	port           = flag.Int("port", 8443, "Server port")
	certDir        = flag.String("cert-dir", "certs", "Certificate directory path")
	clientCert     = flag.String("client-cert", "go-client.crt", "Client certificate file name")
	clientKey      = flag.String("client-key", "go-client.key", "Client private key file name")
	trustBundle    = flag.String("trust-bundle", "trust-bundle.pem", "Trust bundle file name")
	clientSpiffeID = flag.String("client-spiffe-id", "spiffe://example.org/go-client", "Client SPIFFE ID")
	serverSpiffeID = flag.String("server-spiffe-id", "", "Expected server SPIFFE ID (optional)")
)

func main() {
	flag.Parse()

	log.Printf("Starting SPIFFE Go mTLS client for interop testing")
	log.Printf("Connecting to %s:%d", *serverAddr, *port)

	// ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	// defer cancel()

	// Load SPIFFE SVID from files
	clientCertPath := filepath.Join(*certDir, *clientCert)
	clientKeyPath := filepath.Join(*certDir, *clientKey)

	svid, err := x509svid.Load(clientCertPath, clientKeyPath)
	if err != nil {
		log.Fatalf("Failed to load SPIFFE SVID: %v", err)
	}

	// Parse client SPIFFE ID
	spiffeID, err := spiffeid.FromString(*clientSpiffeID)
	if err != nil {
		log.Fatalf("Invalid client SPIFFE ID: %v", err)
	}

	log.Printf("✓ Loaded SPIFFE SVID for: %s", svid.ID)

	// Load trust bundle from file
	trustBundlePath := filepath.Join(*certDir, *trustBundle)
	bundle, err := x509bundle.Load(spiffeID.TrustDomain(), trustBundlePath)
	if err != nil {
		log.Printf("⚠ Failed to load trust bundle, will create from available CAs: %v", err)

		// Fallback: create bundle from available CA certificates
		bundle, err = createTrustBundleFromCAs(spiffeID.TrustDomain())
		if err != nil {
			log.Fatalf("Failed to create trust bundle: %v", err)
		}
	}

	log.Printf("✓ Loaded trust bundle for domain: %s", spiffeID.TrustDomain())

	// Configure TLS with SPIFFE validation
	var tlsConfig *tls.Config
	if *serverSpiffeID != "" {
		// Validate specific server SPIFFE ID
		serverID, err := spiffeid.FromString(*serverSpiffeID)
		if err != nil {
			log.Fatalf("Invalid server SPIFFE ID: %v", err)
		}
		tlsConfig = tlsconfig.MTLSClientConfig(svid, bundle, tlsconfig.AuthorizeID(serverID))
		log.Printf("✓ Configured to validate server SPIFFE ID: %s", serverID)
	} else {
		// Accept any SPIFFE ID from the same trust domain
		tlsConfig = tlsconfig.MTLSClientConfig(svid, bundle, tlsconfig.AuthorizeMemberOf(spiffeID.TrustDomain()))
		log.Printf("✓ Configured to accept any server from trust domain: %s", spiffeID.TrustDomain())
	}

	// Connect to server
	address := fmt.Sprintf("%s:%d", *serverAddr, *port)
	conn, err := tls.Dial("tcp", address, tlsConfig)
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	log.Printf("✓ SPIFFE mTLS handshake successful")

	// Verify server certificate contains SPIFFE ID
	state := conn.ConnectionState()
	if len(state.PeerCertificates) > 0 {
		cert := state.PeerCertificates[0]
		log.Printf("Server certificate subject: %s", cert.Subject)

		// Extract SPIFFE ID from SAN
		for _, uri := range cert.URIs {
			if uri.Scheme == "spiffe" {
				log.Printf("✓ Server SPIFFE ID verified: %s", uri.String())
			}
		}
	}

	// Send test messages
	writer := bufio.NewWriter(conn)
	reader := bufio.NewReader(conn)

	for i := 1; i <= 3; i++ {
		message := fmt.Sprintf("Test message %d from SPIFFE Go client\n", i)
		log.Printf("Sending: %s", message[:len(message)-1])

		_, err := writer.WriteString(message)
		if err != nil {
			log.Fatalf("Failed to send message: %v", err)
		}
		writer.Flush()

		// Read response
		response, err := reader.ReadString('\n')
		if err != nil {
			log.Printf("Failed to read response: %v", err)
			break
		}
		log.Printf("Received: %s", response[:len(response)-1])

		time.Sleep(1 * time.Second)
	}

	// Send close message
	writer.WriteString("CLOSE\n")
	writer.Flush()

	log.Printf("✓ SPIFFE interop test completed successfully")
}

// createTrustBundleFromCAs creates a trust bundle from available CA certificates
func createTrustBundleFromCAs(td spiffeid.TrustDomain) (*x509bundle.Bundle, error) {
	bundle := x509bundle.New(td)

	// Try to load available CA certificates
	caFiles := []string{"go-ca.crt", "ca.crt", "rust-ca.crt"}

	for _, caFile := range caFiles {
		caPath := filepath.Join(*certDir, caFile)
		if caCertPEM, err := os.ReadFile(caPath); err == nil {
			// Parse PEM blocks
			block, _ := pem.Decode(caCertPEM)
			if block != nil {
				if cert, err := x509.ParseCertificate(block.Bytes); err == nil {
					bundle.AddX509Authority(cert)
					log.Printf("✓ Added CA certificate from %s to trust bundle", caFile)
				}
			}
		}
	}

	return bundle, nil
}