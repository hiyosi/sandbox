package integration

import (
	"context"
	"crypto/tls"
	"os"
	"testing"
	"time"

	spireclient "github.com/hiyosi/sandbox/go/spire-client"
)

// CreateTestClient creates a SPIRE client for integration testing
func CreateTestClient(t *testing.T) *spireclient.Client {
	t.Helper()

	// Create client with insecure TLS for testing
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// For integration testing, we use insecure TLS since we're using self-signed certs
	tlsConfig := &tls.Config{
		InsecureSkipVerify: true,
	}

	// Allow override of server address via environment variable
	address := os.Getenv("SPIRE_SERVER_ADDRESS")
	if address == "" {
		address = "localhost:8081"
	}

	config := &spireclient.Config{
		Address:   address,
		TLSConfig: tlsConfig,
	}

	client, err := spireclient.NewWithConfig(ctx, config)
	if err != nil {
		t.Fatalf("Failed to create SPIRE client: %v", err)
	}

	return client
}

// SkipIfNotIntegration skips the test if integration tests are not enabled
func SkipIfNotIntegration(t *testing.T) {
	t.Helper()
	
	if os.Getenv("INTEGRATION_TEST") != "true" {
		t.Skip("Skipping integration test. Set INTEGRATION_TEST=true to run.")
	}
}