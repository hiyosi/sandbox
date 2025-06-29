#!/bin/bash
set -e

# Wait for SPIRE Server to start
echo "Waiting for SPIRE Server to start..."
sleep 10

# Create a join token for the agent
echo "Creating join token for agent..."
docker exec spire-server \
    /opt/spire/bin/spire-server token generate \
    -spiffeID spiffe://example.org/agent/spire-agent \
    -ttl 3600 > join-token.txt

echo "Join token created: $(cat join-token.txt)"

# Register the workload (Client)
echo "Registering client workload..."
docker exec spire-server \
    /opt/spire/bin/spire-server entry create \
    -parentID spiffe://example.org/agent/spire-agent \
    -spiffeID spiffe://example.org/client \
    -selector unix:uid:1000 \
    -ttl 3600

echo "SPIRE Server initialization complete!"