package spireclient

import (
	"context"
	"crypto/tls"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNew(t *testing.T) {
	tests := []struct {
		name    string
		address string
		wantErr bool
		errMsg  string
	}{
		{
			name:    "empty address",
			address: "",
			wantErr: true,
			errMsg:  "address is required",
		},
		{
			name:    "valid address",
			address: "localhost:8081",
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Use a context with timeout to avoid hanging on connection attempts
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()

			client, err := New(ctx, tt.address)

			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
				assert.Nil(t, client)
			} else {
				// For valid address, connection will be attempted but not blocked
				// The client will be created successfully (lazy connection)
				assert.NoError(t, err)
				assert.NotNil(t, client)
				if client != nil {
					client.Close()
				}
			}
		})
	}
}

func TestNewMTLS(t *testing.T) {
	tests := []struct {
		name     string
		address  string
		certFile string
		keyFile  string
		wantErr  bool
		errMsg   string
	}{
		{
			name:     "empty address",
			address:  "",
			certFile: "cert.pem",
			keyFile:  "key.pem",
			wantErr:  true,
			errMsg:   "address is required",
		},
		{
			name:     "empty certFile",
			address:  "localhost:8081",
			certFile: "",
			keyFile:  "key.pem",
			wantErr:  true,
			errMsg:   "both certFile and keyFile are required",
		},
		{
			name:     "empty keyFile",
			address:  "localhost:8081",
			certFile: "cert.pem",
			keyFile:  "",
			wantErr:  true,
			errMsg:   "both certFile and keyFile are required",
		},
		{
			name:     "valid parameters",
			address:  "localhost:8081",
			certFile: "cert.pem",
			keyFile:  "key.pem",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Use a context with timeout to avoid hanging on connection attempts
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()

			client, err := NewMTLS(ctx, tt.address, tt.certFile, tt.keyFile)

			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
				assert.Nil(t, client)
			} else {
				// For valid parameters, connection will be attempted but not blocked
				// The client will be created successfully (lazy connection)
				assert.NoError(t, err)
				assert.NotNil(t, client)
				if client != nil {
					client.Close()
				}
			}
		})
	}
}

func TestNewWithConfig(t *testing.T) {
	tests := []struct {
		name    string
		config  *Config
		wantErr bool
		errMsg  string
	}{
		{
			name:    "nil config",
			config:  nil,
			wantErr: true,
			errMsg:  "config is required",
		},
		{
			name: "empty address",
			config: &Config{
				Address: "",
			},
			wantErr: true,
			errMsg:  "address is required",
		},
		{
			name: "valid config with TLSConfig",
			config: &Config{
				Address: "localhost:8081",
				TLSConfig: &tls.Config{
					MinVersion: tls.VersionTLS12,
				},
			},
			wantErr: false,
		},
		{
			name: "valid config with TLSOptions",
			config: &Config{
				Address:    "localhost:8081",
				TLSOptions: []TLSOption{},
			},
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Use a context with timeout to avoid hanging on connection attempts
			ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
			defer cancel()

			client, err := NewWithConfig(ctx, tt.config)

			if tt.wantErr {
				assert.Error(t, err)
				if tt.errMsg != "" {
					assert.Contains(t, err.Error(), tt.errMsg)
				}
				assert.Nil(t, client)
			} else {
				// For valid configs, connection will be attempted but not blocked
				// The client will be created successfully (lazy connection)
				assert.NoError(t, err)
				assert.NotNil(t, client)
				if client != nil {
					client.Close()
				}
			}
		})
	}
}

func TestClient_Close(t *testing.T) {
	t.Run("close with nil connection", func(t *testing.T) {
		client := &Client{}
		err := client.Close()
		assert.NoError(t, err)
	})
}

func TestClient_Connection(t *testing.T) {
	t.Run("get connection", func(t *testing.T) {
		client := &Client{conn: nil}
		conn := client.Connection()
		assert.Nil(t, conn)
	})
}
