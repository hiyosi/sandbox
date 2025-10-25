package main

import (
	"bufio"
	"crypto/tls"
	"crypto/x509"
	"encoding/pem"
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"strings"

	"github.com/spiffe/go-spiffe/v2/bundle/x509bundle"
	"github.com/spiffe/go-spiffe/v2/spiffeid"
	"github.com/spiffe/go-spiffe/v2/spiffetls/tlsconfig"
	"github.com/spiffe/go-spiffe/v2/svid/x509svid"
)

var (
	port           = flag.Int("port", 8444, "Server port")
	certDir        = flag.String("cert-dir", "certs", "Certificate directory path")
	serverCert     = flag.String("server-cert", "go-server.crt", "Server certificate file name")
	serverKey      = flag.String("server-key", "go-server.key", "Server private key file name")
	trustBundle    = flag.String("trust-bundle", "trust-bundle.pem", "Trust bundle file name")
	serverSpiffeID = flag.String("server-spiffe-id", "spiffe://example.org/go-server", "Server SPIFFE ID")
)

func main() {
	flag.Parse()

	log.Printf("Starting SPIFFE Go mTLS server for interop testing")
	log.Printf("Listening on port %d", *port)

	// Load SPIFFE SVID from files
	serverCertPath := filepath.Join(*certDir, *serverCert)
	serverKeyPath := filepath.Join(*certDir, *serverKey)

	svid, err := x509svid.Load(serverCertPath, serverKeyPath)
	if err != nil {
		log.Fatalf("Failed to load SPIFFE SVID: %v", err)
	}

	// Parse server SPIFFE ID
	spiffeID, err := spiffeid.FromString(*serverSpiffeID)
	if err != nil {
		log.Fatalf("Invalid server SPIFFE ID: %v", err)
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
	// Accept any client from the same trust domain
	tlsConfig := tlsconfig.MTLSServerConfig(svid, bundle, tlsconfig.AuthorizeMemberOf(spiffeID.TrustDomain()))

	// Start listening
	address := fmt.Sprintf(":%d", *port)
	listener, err := tls.Listen("tcp", address, tlsConfig)
	if err != nil {
		log.Fatalf("Failed to start TLS listener: %v", err)
	}
	defer listener.Close()

	log.Printf("SPIFFE mTLS server listening on %s", address)

	// Accept connections
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("Failed to accept connection: %v", err)
			continue
		}

		go handleClient(conn)
	}
}

func handleClient(conn net.Conn) {
	defer conn.Close()

	// Get client info
	clientAddr := conn.RemoteAddr()
	log.Printf("Connection from %s", clientAddr)

	// Extract client certificate info
	if tlsConn, ok := conn.(*tls.Conn); ok {
		state := tlsConn.ConnectionState()
		log.Printf("✓ SPIFFE mTLS handshake successful")

		if len(state.PeerCertificates) > 0 {
			cert := state.PeerCertificates[0]
			log.Printf("Client certificate subject: %s", cert.Subject)

			// Check for SPIFFE ID in SAN
			for _, uri := range cert.URIs {
				if uri.Scheme == "spiffe" {
					log.Printf("✓ Client SPIFFE ID verified: %s", uri.String())
				}
			}
			log.Printf("✓ Client certificate verified")
		} else {
			log.Printf("No client certificates presented")
		}
	}

	// Handle messages (simple echo server)
	reader := bufio.NewReader(conn)
	writer := bufio.NewWriter(conn)

	for {
		message, err := reader.ReadString('\n')
		if err != nil {
			log.Printf("Client %s disconnected", clientAddr)
			break
		}

		message = strings.TrimSpace(message)
		log.Printf("Received from %s: %s", clientAddr, message)

		if message == "CLOSE" {
			log.Printf("Client %s requested close", clientAddr)
			break
		}

		// Echo back with confirmation
		response := fmt.Sprintf("SPIFFE_GO_SERVER_ECHO: %s\n", message)
		_, err = writer.WriteString(response)
		if err != nil {
			log.Printf("Failed to send response: %v", err)
			break
		}
		writer.Flush()
	}

	log.Printf("Client %s disconnected", clientAddr)
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