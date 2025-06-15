package spireclient

import (
	"context"
	"crypto/tls"
	"fmt"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
)

// Client represents a SPIRE Server client
type Client struct {
	conn   *grpc.ClientConn
	config *Config
}

// Config holds the configuration for the SPIRE client
type Config struct {
	// Address is the SPIRE Server address (host:port)
	Address string
	// TLSConfig is the TLS configuration for the connection
	TLSConfig *tls.Config
	// TLSOptions are options for creating TLS configuration if TLSConfig is not provided
	TLSOptions []TLSOption
}

// New creates a new SPIRE client with TLS connection
func New(ctx context.Context, address string) (*Client, error) {
	if address == "" {
		return nil, fmt.Errorf("address is required")
	}

	config := &Config{
		Address: address,
	}

	return newClient(ctx, config)
}

// NewMTLS creates a new SPIRE client with mTLS connection
func NewMTLS(ctx context.Context, address string, certFile, keyFile string) (*Client, error) {
	if address == "" {
		return nil, fmt.Errorf("address is required")
	}

	if certFile == "" || keyFile == "" {
		return nil, fmt.Errorf("both certFile and keyFile are required for mTLS")
	}

	config := &Config{
		Address: address,
		TLSOptions: []TLSOption{
			WithClientCertificates(certFile, keyFile),
		},
	}

	return newClient(ctx, config)
}

// NewWithConfig creates a new SPIRE client with custom configuration
func NewWithConfig(ctx context.Context, config *Config) (*Client, error) {
	return newClient(ctx, config)
}

// newClient is the internal client creation function
func newClient(ctx context.Context, config *Config) (*Client, error) {
	if config == nil {
		return nil, fmt.Errorf("config is required")
	}

	if config.Address == "" {
		return nil, fmt.Errorf("address is required")
	}

	// Use provided TLSConfig or create one with options
	tlsConfig := config.TLSConfig
	if tlsConfig == nil {
		var err error
		tlsConfig, err = NewTLSConfig(config.TLSOptions...)
		if err != nil {
			return nil, fmt.Errorf("failed to create TLS configuration: %w", err)
		}
	}

	// Create TLS credentials
	creds := credentials.NewTLS(tlsConfig)

	// Dial with TLS
	conn, err := grpc.DialContext(ctx, config.Address,
		grpc.WithTransportCredentials(creds),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to SPIRE Server: %w", err)
	}

	return &Client{
		conn:   conn,
		config: config,
	}, nil
}

// Close closes the client connection
func (c *Client) Close() error {
	if c.conn != nil {
		return c.conn.Close()
	}
	return nil
}

// Connection returns the underlying gRPC connection
func (c *Client) Connection() *grpc.ClientConn {
	return c.conn
}
