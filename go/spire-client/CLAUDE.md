# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This project provides a Go client library for SPIRE Server's gRPC API.

## Project Requirements

- **Purpose**: Provide a Go language client library for SPIRE Server's gRPC API
- **Proto Files**: Import and use the official generated Go code from spiffe/spire-api-sdk (proto files are NOT managed in this project)
- **Status**: This is an experimental project and is NOT intended for production use
- **Connection**: Support TLS connections ONLY to SPIRE Server
- **Certificate Validation**: Server certificate validation must be SPIFFE-compliant, but CA certificate validation is out of scope

## Common Commands

```bash
# Launch development container shell
make dev-shell

# Build the library
make build

# Run unit tests
make test

# Run integration tests (requires SPIRE Server to be running)
make test-integration

# Format code
make fmt

# Lint code (requires golangci-lint)
make lint

# Clean build artifacts
make clean

# Manual commands (without Make)
go mod download
go test ./...
go build ./...
go fmt ./...
golangci-lint run
```

## Development Container

A Debian-based development container is available with all necessary tools:
- Go 1.23
- curl, unzip, git
- build-essential
- ca-certificates
- libssl-dev
- golangci-lint
- openssl

Use `make dev-shell` to enter the container with the project mounted at `/workspace`.

### Running SPIRE Server in Development Container

The development environment includes scripts to run a local SPIRE Server (v1.12.2) for testing:

```bash
# Inside the development container
# Start SPIRE Server (runs in background)
./scripts/start-spire-server.sh

# Check server status
./scripts/status-spire-server.sh

# Create test registration entries
./scripts/create-registration-entry.sh

# Stop SPIRE Server
./scripts/stop-spire-server.sh

# Run integration tests (automatically starts/checks SPIRE Server)
./scripts/run-integration-tests.sh
```

The SPIRE Server will be available at:
- TCP gRPC API: `localhost:8081` (TLS/mTLS for SPIRE Agents, authenticated by peer certificate identity)
- Unix socket: `/tmp/spire-server/private/api.sock` (admin privileges for local management)

Note:
- TCP endpoint is used by SPIRE Agents with TLS/mTLS authentication
- Unix socket provides admin-level access for local management operations
- This client library targets the TCP endpoint with proper SPIFFE certificate validation

## Architecture Notes

As a SPIRE Server gRPC client library, this project should include:
- Import the generated Go code from github.com/spiffe/spire-api-sdk
- TLS connection handling with SPIFFE-compliant certificate validation
- Client implementations for various SPIRE Server services (Bundle, Entry, Agent, etc.)
- Error handling for gRPC operations

## Development Guidelines

- Import generated types and service clients from github.com/spiffe/spire-api-sdk
- Implement TLS-only connections (no plain text)
- Follow SPIFFE specifications for server certificate validation
- Keep the experimental nature in mind - prioritize clarity over optimization
- Document all public APIs clearly indicating experimental status

## Memories
