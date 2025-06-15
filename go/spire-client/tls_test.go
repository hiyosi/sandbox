package spireclient

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"math/big"
	"net/url"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewTLSConfig(t *testing.T) {
	t.Run("basic TLS config", func(t *testing.T) {
		config, err := NewTLSConfig()
		require.NoError(t, err)
		assert.NotNil(t, config)
		assert.Equal(t, uint16(tls.VersionTLS12), config.MinVersion)
		assert.True(t, config.InsecureSkipVerify)
		assert.NotNil(t, config.VerifyPeerCertificate)
	})

	t.Run("with client certificates option", func(t *testing.T) {
		// This test just verifies the option is accepted
		// Actual file loading would fail in unit test
		config, err := NewTLSConfig(WithClientCertificates("cert.pem", "key.pem"))
		require.NoError(t, err)
		assert.NotNil(t, config)
	})

	t.Run("with client certificates from memory option", func(t *testing.T) {
		// This test just verifies the option is accepted
		// Invalid PEM data won't set certificates
		config, err := NewTLSConfig(WithClientCertificatesFromMemory([]byte("cert"), []byte("key")))
		require.NoError(t, err)
		assert.NotNil(t, config)
	})
}

func TestIsValidSPIFFEID(t *testing.T) {
	tests := []struct {
		name  string
		uri   string
		valid bool
	}{
		{
			name:  "valid SPIFFE ID",
			uri:   "spiffe://example.org/workload",
			valid: true,
		},
		{
			name:  "valid SPIFFE ID with path",
			uri:   "spiffe://example.org/ns/prod/sa/web",
			valid: true,
		},
		{
			name:  "invalid scheme",
			uri:   "https://example.org/workload",
			valid: false,
		},
		{
			name:  "missing host",
			uri:   "spiffe:///workload",
			valid: false,
		},
		{
			name:  "with user info",
			uri:   "spiffe://user@example.org/workload",
			valid: false,
		},
		{
			name:  "with port",
			uri:   "spiffe://example.org:8080/workload",
			valid: false,
		},
		{
			name:  "with query",
			uri:   "spiffe://example.org/workload?query=value",
			valid: false,
		},
		{
			name:  "with fragment",
			uri:   "spiffe://example.org/workload#fragment",
			valid: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			uri, err := url.Parse(tt.uri)
			require.NoError(t, err)
			assert.Equal(t, tt.valid, isValidSPIFFEID(uri))
		})
	}
}

func TestVerifyPeerCertificate(t *testing.T) {
	config, err := NewTLSConfig()
	require.NoError(t, err)

	t.Run("no certificates", func(t *testing.T) {
		err := config.VerifyPeerCertificate([][]byte{}, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "no server certificate presented")
	})

	t.Run("invalid certificate", func(t *testing.T) {
		err := config.VerifyPeerCertificate([][]byte{{0x00, 0x01}}, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "failed to parse server certificate")
	})

	t.Run("certificate without URI SANs", func(t *testing.T) {
		// Create a minimal valid certificate for testing
		certTemplate := &x509.Certificate{
			SerialNumber: big.NewInt(1),
			Subject: pkix.Name{
				CommonName: "test",
			},
			NotBefore: time.Now(),
			NotAfter:  time.Now().Add(365 * 24 * time.Hour),
		}

		// Generate a key pair
		key, err := rsa.GenerateKey(rand.Reader, 2048)
		require.NoError(t, err)

		// Create certificate
		certBytes, err := x509.CreateCertificate(rand.Reader, certTemplate, certTemplate, &key.PublicKey, key)
		require.NoError(t, err)

		err = config.VerifyPeerCertificate([][]byte{certBytes}, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "server certificate has no URI SANs")
	})

	t.Run("certificate with non-SPIFFE URI", func(t *testing.T) {
		uri, _ := url.Parse("https://example.org")
		certTemplate := &x509.Certificate{
			SerialNumber: big.NewInt(1),
			Subject: pkix.Name{
				CommonName: "test",
			},
			NotBefore: time.Now(),
			NotAfter:  time.Now().Add(365 * 24 * time.Hour),
			URIs:      []*url.URL{uri},
		}

		// Generate a key pair
		key, err := rsa.GenerateKey(rand.Reader, 2048)
		require.NoError(t, err)

		// Create certificate
		certBytes, err := x509.CreateCertificate(rand.Reader, certTemplate, certTemplate, &key.PublicKey, key)
		require.NoError(t, err)

		err = config.VerifyPeerCertificate([][]byte{certBytes}, nil)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "server certificate does not contain a valid SPIFFE ID")
	})

	t.Run("certificate with valid SPIFFE ID", func(t *testing.T) {
		uri, _ := url.Parse("spiffe://example.org/workload")
		certTemplate := &x509.Certificate{
			SerialNumber: big.NewInt(1),
			Subject: pkix.Name{
				CommonName: "test",
			},
			NotBefore: time.Now(),
			NotAfter:  time.Now().Add(365 * 24 * time.Hour),
			URIs:      []*url.URL{uri},
		}

		// Generate a key pair
		key, err := rsa.GenerateKey(rand.Reader, 2048)
		require.NoError(t, err)

		// Create certificate
		certBytes, err := x509.CreateCertificate(rand.Reader, certTemplate, certTemplate, &key.PublicKey, key)
		require.NoError(t, err)

		err = config.VerifyPeerCertificate([][]byte{certBytes}, nil)
		// Should pass SPIFFE ID validation
		assert.NoError(t, err)
	})
}
