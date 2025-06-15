package integration

import (
	"context"
	"testing"
	"time"

	bundlev1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/bundle/v1"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestBundleAPI_GetBundle tests retrieving trust bundle from SPIRE Server
func TestBundleAPI_GetBundle(t *testing.T) {
	SkipIfNotIntegration(t)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	client := CreateTestClient(t)
	defer client.Close()

	// Get bundle client
	bundleClient := client.BundleClient()
	require.NotNil(t, bundleClient, "Bundle client should not be nil")

	// Test GetBundle
	t.Run("GetBundle", func(t *testing.T) {
		resp, err := bundleClient.GetBundle(ctx, &bundlev1.GetBundleRequest{})
		require.NoError(t, err, "Failed to get bundle")
		require.NotNil(t, resp, "Response should not be nil")

		// The response is the bundle itself (types.Bundle), not wrapped
		assert.Equal(t, "example.org", resp.TrustDomain, "Trust domain should match")
		assert.NotEmpty(t, resp.X509Authorities, "Should have X509 authorities")
		
		t.Logf("Successfully retrieved bundle for trust domain: %s", resp.TrustDomain)
		t.Logf("Bundle has %d X.509 authorities", len(resp.X509Authorities))
	})

}