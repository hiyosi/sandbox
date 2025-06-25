package main

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/openfga/go-sdk/client"
	"github.com/openfga/go-sdk/credentials"
	"github.com/spiffe/go-spiffe/v2/svid/jwtsvid"
	"github.com/spiffe/go-spiffe/v2/workloadapi"
)

type OpenFGAClient struct {
	client  *client.OpenFgaClient
	storeID string
}

func NewOpenFGAClient(apiURL, storeID string, jwtToken string) (*OpenFGAClient, error) {
	configuration := client.ClientConfiguration{
		ApiUrl: apiURL,
		Credentials: &credentials.Credentials{
			Method: credentials.CredentialsMethodApiToken,
			Config: &credentials.Config{
				ApiToken: jwtToken,
			},
		},
		// CA証明書を信頼するためのHTTPクライアント設定
		HTTPClient: &http.Client{
			Timeout: 30 * time.Second,
			Transport: &http.Transport{
				TLSClientConfig: &tls.Config{
					InsecureSkipVerify: true, // デモ用のため
				},
			},
		},
	}

	fgaClient, err := client.NewSdkClient(&configuration)
	if err != nil {
		return nil, fmt.Errorf("failed to create OpenFGA client: %v", err)
	}

	return &OpenFGAClient{
		client:  fgaClient,
		storeID: storeID,
	}, nil
}

// SPIRE認証を使用してOpenFGAクライアントを作成
func NewOpenFGAClientWithSPIRE(apiURL, storeID string) (*OpenFGAClient, error) {
	// SPIRE Workload APIからJWT SVIDを取得
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	log.Printf("Process ID: %d", os.Getpid())

	socketPath := "/tmp/spire-agent/public/api.sock"

	// ソケットファイルの存在確認
	if _, err := os.Stat(socketPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("SPIRE Agent socket not found at %s", socketPath)
	} else if err != nil {
		return nil, fmt.Errorf("failed to check SPIRE Agent socket: %v", err)
	}

	log.Printf("SPIRE Agent socket found at: %s", socketPath)
	log.Printf("Connecting to SPIRE Agent at: unix://%s", socketPath)

	source, err := workloadapi.NewJWTSource(ctx, workloadapi.WithClientOptions(workloadapi.WithAddr("unix://"+socketPath)))
	if err != nil {
		return nil, fmt.Errorf("failed to create JWT source: %v", err)
	}
	defer source.Close()

	log.Printf("JWT Source created successfully, fetching JWT SVID...")

	// aud=openfgaのJWT SVIDを取得
	svid, err := source.FetchJWTSVID(ctx, jwtsvid.Params{
		Audience: "openfga",
	})
	if err != nil {
		return nil, fmt.Errorf("failed to fetch JWT SVID: %v", err)
	}

	log.Printf("Obtained JWT SVID for SPIFFE ID: %s", svid.ID)
	log.Printf("JWT Token (first 50 chars): %s...", svid.Marshal()[:50])

	configuration := client.ClientConfiguration{
		ApiUrl: apiURL,
		Credentials: &credentials.Credentials{
			Method: credentials.CredentialsMethodApiToken,
			Config: &credentials.Config{
				ApiToken: svid.Marshal(),
			},
		},
		Debug: true,
	}

	fgaClient, err := client.NewSdkClient(&configuration)
	if err != nil {
		return nil, fmt.Errorf("failed to create OpenFGA client: %v", err)
	}
	
	// HTTPClientのTLS設定を変更（CA証明書を使用）
	httpClient := fgaClient.APIClient.GetConfig().HTTPClient
	if httpClient.Transport == nil {
		httpClient.Transport = &http.Transport{}
	}
	if transport, ok := httpClient.Transport.(*http.Transport); ok {
		// CA証明書を読み込み
		caCert, err := os.ReadFile("/opt/certs/ca.crt")
		if err != nil {
			log.Printf("Warning: Failed to read CA certificate, falling back to insecure: %v", err)
			if transport.TLSClientConfig == nil {
				transport.TLSClientConfig = &tls.Config{}
			}
			transport.TLSClientConfig.InsecureSkipVerify = true
		} else {
			// CA証明書プールを作成
			caCertPool := x509.NewCertPool()
			caCertPool.AppendCertsFromPEM(caCert)
			
			if transport.TLSClientConfig == nil {
				transport.TLSClientConfig = &tls.Config{}
			}
			transport.TLSClientConfig.RootCAs = caCertPool
			log.Printf("CA certificate loaded successfully")
		}
	}

	return &OpenFGAClient{
		client:  fgaClient,
		storeID: storeID,
	}, nil
}

// ユーザーの権限をチェック
func (c *OpenFGAClient) CheckPermission(ctx context.Context, user, relation, object string) (bool, error) {
	body := client.ClientCheckRequest{
		User:     user,
		Relation: relation,
		Object:   object,
	}

	resp, err := c.client.Check(ctx).Body(body).Options(client.ClientCheckOptions{
		StoreId: &c.storeID,
	}).Execute()
	if err != nil {
		return false, fmt.Errorf("failed to check permission: %v", err)
	}

	return resp.GetAllowed(), nil
}

// 複数の権限をバッチでチェック
func (c *OpenFGAClient) BatchCheck(ctx context.Context, checks []CheckRequest) ([]bool, error) {
	results := make([]bool, len(checks))

	for i, check := range checks {
		allowed, err := c.CheckPermission(ctx, check.User, check.Relation, check.Object)
		if err != nil {
			return nil, fmt.Errorf("failed to check permission for %s %s %s: %v",
				check.User, check.Relation, check.Object, err)
		}
		results[i] = allowed
	}

	return results, nil
}

type CheckRequest struct {
	User     string
	Relation string
	Object   string
}

func main() {
	apiURL := os.Getenv("OPENFGA_API_URL")
	if apiURL == "" {
		apiURL = "https://openfga:18443"
	}

	storeID := os.Getenv("OPENFGA_STORE_ID")
	if storeID == "" {
		log.Fatal("OPENFGA_STORE_ID environment variable is required")
	}

	ctx := context.Background()
	runWithSPIRE(ctx, apiURL, storeID)
}

func runWithSPIRE(ctx context.Context, apiURL, storeID string) {
	fmt.Println("=== SPIRE Authentication with OpenFGA ===")

	client, err := NewOpenFGAClientWithSPIRE(apiURL, storeID)
	if err != nil {
		log.Fatalf("Failed to create OpenFGA client with SPIRE: %v", err)
	}

	runPermissionTests(ctx, client)
}

func runPermissionTests(ctx context.Context, client *OpenFGAClient) {
	testCases := []CheckRequest{
		{"user:alice", "can_read", "resource:public-data"},
		{"user:alice", "can_write", "resource:public-data"},
		{"user:bob", "can_read", "resource:sensitive-data"},
		{"user:charlie", "can_read", "resource:public-data"},
		{"user:charlie", "can_read", "resource:sensitive-data"},
		{"user:admin", "can_delete", "resource:sensitive-data"},
		{"user:frank", "can_write", "resource:user-interface-config"},
	}

	fmt.Println("\n--- Permission Check Results ---")
	for _, test := range testCases {
		allowed, err := client.CheckPermission(ctx, test.User, test.Relation, test.Object)
		if err != nil {
			fmt.Printf("ERROR: %s %s %s -> %v\n", test.User, test.Relation, test.Object, err)
			continue
		}

		status := "❌ DENIED"
		if allowed {
			status = "✅ ALLOWED"
		}

		fmt.Printf("%s: %s %s %s\n", status, test.User, test.Relation, test.Object)
	}
}
