#!/bin/bash
# DEPRECATED: Use complete-setup.sh for full automated setup
echo "WARNING: This script is deprecated."
echo "Use './scripts/complete-setup.sh' for complete automated setup including OpenFGA configuration."
echo ""
read -p "Continue with basic setup only? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted. Run './scripts/complete-setup.sh' instead."
    exit 1
fi

set -e

echo "=== Starting SPIRE + OpenFGA Demo (Basic Setup) ==="

# Generate certificates
echo "1. Generating certificates..."
chmod +x scripts/generate-certs.sh
./scripts/generate-certs.sh

# Start SPIRE services
echo "2. Starting SPIRE services..."
docker-compose up -d spire-server oidc-discovery-provider

# Wait for SPIRE Server to be ready
echo "3. Waiting for SPIRE Server to be ready..."
until docker exec spire-server /opt/spire/bin/spire-server healthcheck >/dev/null 2>&1; do
    echo "   Waiting for SPIRE Server..."
    sleep 2
done
echo "   SPIRE Server is ready!"

# Generate join token and register workload
echo "4. Generating join token and registering workload..."
JOIN_TOKEN=$(docker exec spire-server \
    /opt/spire/bin/spire-server token generate \
    -spiffeID spiffe://example.org/agent/spire-agent | tail -1)

echo "Join token: $JOIN_TOKEN"

# Register the workload (Client)
echo "5. Registering client workload..."
docker exec spire-server \
    /opt/spire/bin/spire-server entry create \
    -parentID spiffe://example.org/agent/spire-agent \
    -spiffeID spiffe://example.org/client \
    -selector unix:uid:1000

# Start SPIRE Agent with join token
echo "6. Starting SPIRE Agent..."
JOIN_TOKEN=$JOIN_TOKEN docker-compose up -d spire-agent

# Start OpenFGA
echo "7. Starting OpenFGA..."
docker-compose up -d openfga

# Wait for OpenFGA to be ready
echo "8. Waiting for OpenFGA to be ready..."
until curl -k -s https://localhost:18443/healthz | grep -q "SERVING" >/dev/null 2>&1; do
    echo "   Waiting for OpenFGA..."
    sleep 2
done
echo "   OpenFGA is ready!"

echo "=== Basic SPIRE + OpenFGA setup complete! ==="
echo ""
echo "NOTE: OpenFGA store, models, and tuples need to be configured manually."
echo "Run './scripts/setup-openfga.sh' to configure OpenFGA."
echo ""
echo "For complete automated setup, use './scripts/complete-setup.sh'"