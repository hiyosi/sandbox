package spireclient

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc"
)

// mockConnection is a mock gRPC connection for testing
type mockConnection struct {
	*grpc.ClientConn
}

func TestClient_ServiceClients(t *testing.T) {
	// Create a client with a mock connection
	client := &Client{
		conn: &grpc.ClientConn{}, // This is just for testing, won't actually connect
	}

	t.Run("AgentClient", func(t *testing.T) {
		agentClient := client.AgentClient()
		assert.NotNil(t, agentClient)
	})

	t.Run("BundleClient", func(t *testing.T) {
		bundleClient := client.BundleClient()
		assert.NotNil(t, bundleClient)
	})

	t.Run("EntryClient", func(t *testing.T) {
		entryClient := client.EntryClient()
		assert.NotNil(t, entryClient)
	})

	t.Run("SVIDClient", func(t *testing.T) {
		svidClient := client.SVIDClient()
		assert.NotNil(t, svidClient)
	})

	t.Run("TrustDomainClient", func(t *testing.T) {
		trustDomainClient := client.TrustDomainClient()
		assert.NotNil(t, trustDomainClient)
	})
}
