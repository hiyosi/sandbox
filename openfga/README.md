# SPIRE + OpenFGA Integration Demo

This project demonstrates the integration of SPIRE (workload identity and authentication) with OpenFGA (fine-grained authorization) using JWT SVID authentication.

## Quick Start

### Complete Automated Setup

Run the complete setup script to automatically configure everything:

```bash
./scripts/complete-setup.sh
```

This script will:
1. Generate all required certificates
2. Start all services (SPIRE Server, SPIRE Agent, OIDC Discovery Provider, OpenFGA)
3. Register SPIRE workload entries
4. Create OpenFGA store with JWT authentication
5. Register authorization model and initial tuples
6. Start the Go client for testing

### Testing the Integration

After setup, test the integration:

```bash
# View Go client logs (shows permission check results)
docker-compose logs -f go-client

# Or run the client manually
docker-compose run --rm go-client
```

### Cleanup

To clean up all resources:

```bash
./scripts/cleanup.sh
```

## Architecture

```
┌─────────────────┐    JWT SVID     ┌─────────────────┐  OIDC Auth +    ┌─────────────────┐
│   SPIRE Agent   │←────────────────│   Go Client     │ ─ Permission ──→│    OpenFGA      │
│                 │                 │                 │    Checks       │ ┌─────────────┐ │
└─────────────────┘                 └─────────────────┘                 │ │ OIDC Auth   │─┼────┐
         │                                                              │ └─────────────┘ │    │
         │                                                              │ ┌─────────────┐ │    │
         │                                                              │ │Authorization│ │    │
         │                                                              │ │   Model     │ │    │
         ▼                                                              │ └─────────────┘ │   JWK
┌─────────────────┐                                                     └─────────────────┘    │
│  SPIRE Server   │                                                                            │
│                 │←─────────────────── Public Key ─────────────────────────────────────────┐  │
└─────────────────┘                                                                         │  │
                                                                                            │  │
                                                                                            │  ▼
                                                                                   ┌─────────────────┐
                                                                                   │ OIDC Discovery  │
                                                                                   │   Provider      │
                                                                                   └─────────────────┘
```

## Components

- **SPIRE Server**: Issues workload identities and JWT SVIDs
- **SPIRE Agent**: Manages workload attestation and SVID delivery
- **OIDC Discovery Provider**: Provides JWT verification endpoint for OpenFGA
- **OpenFGA**: Fine-grained authorization service with OIDC authentication
- **Go Client**: Demo application using SPIRE JWT SVID to authenticate with OpenFGA

## Available Scripts

- `./scripts/complete-setup.sh` - Complete automated setup
- `./scripts/cleanup.sh` - Clean up all resources
- `./scripts/generate-certs.sh` - Generate certificates only
- `./scripts/start.sh` - Basic setup only (deprecated)

## Services and Ports

After setup, the following services will be available:

- **SPIRE Server**: http://localhost:8081
- **SPIRE Server Health**: http://localhost:8080/live
- **OIDC Discovery Provider**: https://localhost:8443
- **SPIRE Agent Health**: http://localhost:8082/live
- **OpenFGA HTTPS API**: https://localhost:18443
- **OpenFGA gRPC**: https://localhost:28443
- **OpenFGA Health**: https://localhost:18443/healthz

## Authorization Model

The demo includes a sample authorization model with:

- **Users**: alice, bob, charlie, admin, frank
- **Teams**: dev-team, admin-team, ui-team
- **Resources**: public-data, sensitive-data, user-interface-config
- **Relations**: can_read, can_write, can_delete

## Manual Steps (if not using complete-setup.sh)

1. Generate certificates: `./scripts/generate-certs.sh`
2. Start services: `docker-compose up -d`
3. Register SPIRE entries: See `complete-setup.sh` for commands
4. Setup OpenFGA: `./scripts/setup-openfga.sh` (for no-auth setup)
5. Configure JWT authentication manually

## Security Features

- ✅ mTLS between all SPIRE components
- ✅ JWT SVID authentication to OpenFGA
- ✅ OIDC token verification
- ✅ CA certificate validation
- ✅ Fine-grained authorization policies

## Requirements

- Docker and Docker Compose
- `jq` for JSON processing
- `curl` for HTTP requests
- `openssl` for certificate generation
