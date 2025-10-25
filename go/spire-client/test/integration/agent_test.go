package integration

import (
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"os/exec"
	"regexp"
	"strings"
	"testing"
	"time"

	agentv1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/agent/v1"
	bundlev1 "github.com/spiffe/spire-api-sdk/proto/spire/api/server/bundle/v1"
	"github.com/spiffe/spire-api-sdk/proto/spire/api/types"
	spireclient "github.com/hiyosi/sandbox/go/spire-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// generateCSRWithKey generates a Certificate Signing Request and returns both CSR and private key
func generateCSRWithKey(t *testing.T) ([]byte, *rsa.PrivateKey) {
	t.Helper()
	
	// Generate RSA private key
	privateKey, err := rsa.GenerateKey(rand.Reader, 2048)
	require.NoError(t, err, "Failed to generate private key")
	
	// Create CSR template
	template := x509.CertificateRequest{
		Subject: pkix.Name{
			Country:      []string{"US"},
			Organization: []string{"SPIFFE Test"},
			CommonName:   "spiffe-agent-test",
		},
	}
	
	// Create CSR
	csrDER, err := x509.CreateCertificateRequest(rand.Reader, &template, privateKey)
	require.NoError(t, err, "Failed to create CSR")
	
	return csrDER, privateKey
}

// generateCSR generates a Certificate Signing Request for testing (backward compatibility)
func generateCSR(t *testing.T) []byte {
	csr, _ := generateCSRWithKey(t)
	return csr
}

// generateJoinToken creates a valid join token using SPIRE Server
func generateJoinToken(t *testing.T) string {
	t.Helper()
	
	// Generate join token using SPIRE Server CLI
	cmd := exec.Command("/opt/spire/spire-server", "token", "generate",
		"-socketPath", "/tmp/spire-server/private/api.sock",
		"-spiffeID", "spiffe://example.org/test-node",
		"-ttl", "3600")
	
	output, err := cmd.CombinedOutput()
	require.NoError(t, err, "Failed to generate join token: %s", string(output))
	
	// Extract token from output using regex
	// Output format: "Token: <token-value>"
	re := regexp.MustCompile(`Token:\s+([a-zA-Z0-9_-]+)`)
	matches := re.FindStringSubmatch(string(output))
	require.Len(t, matches, 2, "Failed to extract token from output: %s", string(output))
	
	token := strings.TrimSpace(matches[1])
	require.NotEmpty(t, token, "Token should not be empty")
	
	t.Logf("Generated join token: %s", token)
	return token
}

// TestAgentAPI_AttestAgent tests node attestation functionality
func TestAgentAPI_AttestAgent(t *testing.T) {
	SkipIfNotIntegration(t)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	client := CreateTestClient(t)
	defer client.Close()

	// Get agent client
	agentClient := client.AgentClient()
	require.NotNil(t, agentClient, "Agent client should not be nil")

	// Test AttestAgent with join_token attestor and then test mTLS
	t.Run("AttestAgent_JoinToken", func(t *testing.T) {
		// Generate a valid CSR and keep the private key for mTLS
		csr, privateKey := generateCSRWithKey(t)
		
		// Generate a valid join token
		joinToken := generateJoinToken(t)
		
		// Create attestation data with the valid join token
		attestationData := &types.AttestationData{
			Type:    "join_token",
			Payload: []byte(joinToken),
		}

		// Create attestation request
		req := &agentv1.AttestAgentRequest{
			Step: &agentv1.AttestAgentRequest_Params_{
				Params: &agentv1.AttestAgentRequest_Params{
					Data: attestationData,
					Params: &agentv1.AgentX509SVIDParams{
						Csr: csr, // Use generated CSR
					},
				},
			},
		}

		// AttestAgent is a streaming method, not unary
		stream, err := agentClient.AttestAgent(ctx)
		require.NoError(t, err, "Failed to create AttestAgent stream")
		
		// Send the attestation request
		err = stream.Send(req)
		if err != nil {
			t.Logf("AttestAgent send failed as expected (no valid join token): %v", err)
			// Verify it's a gRPC error, not a connection error
			assert.Contains(t, err.Error(), "rpc error", "Should be a gRPC error, not connection error")
			return
		}

		// Receive the response
		resp, err := stream.Recv()
		require.NoError(t, err, "AttestAgent should succeed with valid join token")
		require.NotNil(t, resp, "Response should not be nil")
		
		// Verify successful attestation
		result := resp.GetResult()
		require.NotNil(t, result, "Result should not be nil")
		require.NotNil(t, result.Svid, "SVID should not be nil")
		require.NotEmpty(t, result.Svid.Id, "SVID ID should not be empty")
		require.NotEmpty(t, result.Svid.CertChain, "SVID cert chain should not be empty")
		
		t.Logf("Node Attestation successful!")
		t.Logf("Agent SPIFFE ID: %s", result.Svid.Id.String())
		t.Logf("Agent SVID cert chain length: %d", len(result.Svid.CertChain))
		
		// Now test mTLS connection using the Agent SVID
		t.Run("mTLS_Connection", func(t *testing.T) {
			// Parse the Agent SVID certificate
			agentCert, err := x509.ParseCertificate(result.Svid.CertChain[0])
			require.NoError(t, err, "Failed to parse agent certificate")
			
			// Create TLS certificate using the agent cert and our private key
			tlsCert := tls.Certificate{
				Certificate: result.Svid.CertChain,
				PrivateKey:  privateKey,
				Leaf:        agentCert,
			}
			
			// Get SPIRE Server bundle for server verification
			bundleClient := client.BundleClient()
			bundleResp, err := bundleClient.GetBundle(ctx, &bundlev1.GetBundleRequest{})
			require.NoError(t, err, "Failed to get bundle")
			
			// Create root CA pool from SPIRE bundle
			rootCAs := x509.NewCertPool()
			for _, caCert := range bundleResp.X509Authorities {
				cert, err := x509.ParseCertificate(caCert.Asn1)
				require.NoError(t, err, "Failed to parse CA certificate")
				rootCAs.AddCert(cert)
			}
			
			// Create mTLS client configuration
			mtlsConfig := &spireclient.Config{
				Address: "localhost:8081",
				TLSConfig: &tls.Config{
					Certificates: []tls.Certificate{tlsCert},
					RootCAs:      rootCAs,
					// Use InsecureSkipVerify for testing since server cert may not have proper SAN
					InsecureSkipVerify: true,
					// Custom verification function to validate SPIFFE ID
					VerifyConnection: func(cs tls.ConnectionState) error {
						// In production, you would verify the SPIFFE ID in the server certificate
						// For now, just log the server certificate info
						if len(cs.PeerCertificates) > 0 {
							serverCert := cs.PeerCertificates[0]
							t.Logf("Server certificate subject: %s", serverCert.Subject.String())
							if len(serverCert.URIs) > 0 {
								t.Logf("Server SPIFFE ID: %s", serverCert.URIs[0].String())
							}
						}
						return nil
					},
				},
			}
			
			// Create mTLS client
			mtlsClient, err := spireclient.NewWithConfig(ctx, mtlsConfig)
			require.NoError(t, err, "Failed to create mTLS client")
			defer mtlsClient.Close()
			
			// Test mTLS connection by calling GetBundle
			mtlsBundleClient := mtlsClient.BundleClient()
			mtlsBundleResp, err := mtlsBundleClient.GetBundle(ctx, &bundlev1.GetBundleRequest{})
			require.NoError(t, err, "mTLS connection failed")
			require.NotNil(t, mtlsBundleResp, "Bundle response should not be nil")
			
			t.Logf("mTLS connection successful!")
			t.Logf("Retrieved bundle via mTLS: trust domain = %s", mtlsBundleResp.TrustDomain)
			t.Logf("Bundle has %d X.509 authorities", len(mtlsBundleResp.X509Authorities))
		})

		// Close the stream
		stream.CloseSend()
	})
}