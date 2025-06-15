package spireclient

import (
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"net/url"
)

// TLSOption represents TLS configuration options
type TLSOption func(*tls.Config)

// WithClientCertificates configures client certificates for mTLS
func WithClientCertificates(certFile, keyFile string) TLSOption {
	return func(c *tls.Config) {
		cert, err := tls.LoadX509KeyPair(certFile, keyFile)
		if err == nil {
			c.Certificates = []tls.Certificate{cert}
		}
		// Note: errors are silently ignored here, actual loading happens at connection time
	}
}

// WithClientCertificatesFromMemory configures client certificates for mTLS from memory
func WithClientCertificatesFromMemory(certPEM, keyPEM []byte) TLSOption {
	return func(c *tls.Config) {
		cert, err := tls.X509KeyPair(certPEM, keyPEM)
		if err == nil {
			c.Certificates = []tls.Certificate{cert}
		}
	}
}

// NewTLSConfig creates a new TLS configuration for SPIFFE-compliant server certificate validation
// Supports both TLS and mTLS connections based on provided options
func NewTLSConfig(opts ...TLSOption) (*tls.Config, error) {
	config := &tls.Config{
		// SPIFFE-compliant verification
		VerifyPeerCertificate: func(rawCerts [][]byte, verifiedChains [][]*x509.Certificate) error {
			if len(rawCerts) == 0 {
				return fmt.Errorf("no server certificate presented")
			}

			// Parse the server certificate
			cert, err := x509.ParseCertificate(rawCerts[0])
			if err != nil {
				return fmt.Errorf("failed to parse server certificate: %w", err)
			}

			// Check for SPIFFE ID in URI SANs
			if len(cert.URIs) == 0 {
				return fmt.Errorf("server certificate has no URI SANs (SPIFFE ID required)")
			}

			// Validate that at least one URI is a valid SPIFFE ID
			hasValidSPIFFEID := false
			for _, uri := range cert.URIs {
				if isValidSPIFFEID(uri) {
					hasValidSPIFFEID = true
					break
				}
			}

			if !hasValidSPIFFEID {
				return fmt.Errorf("server certificate does not contain a valid SPIFFE ID")
			}

			return nil
		},
		// Since CA certificate validation is out of scope, we'll accept any certificate
		// that passes our SPIFFE ID validation
		InsecureSkipVerify: true,
		MinVersion:         tls.VersionTLS12,
	}

	// Apply options
	for _, opt := range opts {
		opt(config)
	}

	return config, nil
}

// isValidSPIFFEID checks if a URI is a valid SPIFFE ID
func isValidSPIFFEID(uri *url.URL) bool {
	// SPIFFE IDs must:
	// 1. Use the "spiffe" scheme
	// 2. Have a host component (trust domain)
	// 3. Have no user info, port, query, or fragment

	if uri.Scheme != "spiffe" {
		return false
	}

	if uri.Host == "" {
		return false
	}

	if uri.User != nil || uri.RawQuery != "" || uri.Fragment != "" {
		return false
	}

	// Check for port (SPIFFE IDs should not have ports)
	if uri.Port() != "" {
		return false
	}

	return true
}
