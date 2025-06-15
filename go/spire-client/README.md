# SPIRE Client Library (Sandbox)

⚠️ **This is an experimental sandbox project and is NOT intended for production use.**

A Go client library for SPIRE Server's gRPC API with SPIFFE-compliant certificate validation.

## Features

- TLS/mTLS connections to SPIRE Server
- SPIFFE-compliant server certificate validation
- Support for all SPIRE Server gRPC APIs
- Simple client creation with `New()`, `NewMTLS()`, and `NewWithConfig()`

## Quick Start

```go
package main

import (
    "context"
    spireclient "github.com/hiyosi/sandbox/go/spire-client"
)

func main() {
    ctx := context.Background()

    // TLS connection
    client, err := spireclient.New(ctx, "localhost:8081")
    if err != nil {
        panic(err)
    }
    defer client.Close()

    // Use Bundle API
    bundleClient := client.BundleClient()
    bundle, err := bundleClient.GetBundle(ctx, &bundlev1.GetBundleRequest{})
    // ...
}
```

## Development

### Prerequisites

- Docker with buildx support
- Make

### Getting Started

```bash
# Enter development container
make dev-shell

# Start SPIRE Server for testing
./scripts/start-spire-server.sh

# Run tests
make test                    # Unit tests
make test-integration        # Integration tests

# Stop SPIRE Server
./scripts/stop-spire-server.sh
```

### Available Commands

```bash
make dev-shell        # Launch development container
make build            # Build the library
make test             # Run unit tests
make test-integration # Run integration tests
make fmt              # Format code
make lint             # Run linter
make clean            # Clean build artifacts
```

## Architecture

- **Client Types**: Basic TLS (`New`) and mTLS (`NewMTLS`) support
- **Certificate Validation**: SPIFFE-compliant server certificate validation
- **API Coverage**: All SPIRE Server APIs (Bundle, Entry, Agent, SVID, TrustDomain)
- **Authentication**: Designed for SPIRE Agent connections with peer certificate identity

## Limitations

- Experimental project - API may change
- CA certificate validation is out of scope
- No support for Unix domain socket connections
- Development/testing focus only

## Contributing

This is a sandbox project for experimentation. Feel free to explore and modify as needed.

## License

This project is for experimental use only.
