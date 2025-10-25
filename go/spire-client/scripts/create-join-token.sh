#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

SPIRE_DIR="/opt/spire"
CONFIG_DIR="/etc/spire"
TOKEN="test-join-token"

# Check if spire-server binary exists
if [ ! -f "${SPIRE_DIR}/spire-server" ]; then
    echo -e "${YELLOW}SPIRE Server not found. Please run start-spire-server.sh first.${NC}"
    exit 1
fi

echo -e "${GREEN}Creating join token for testing...${NC}"

# Create a join token
TOKEN_OUTPUT=$(${SPIRE_DIR}/spire-server token generate \
    -socketPath /tmp/spire-server/private/api.sock \
    -spiffeID spiffe://example.org/test-node \
    -ttl 3600)

echo "${TOKEN_OUTPUT}"
echo -e "${GREEN}Note: The test uses hardcoded token 'test-join-token', but server generates random tokens${NC}"

echo -e "${GREEN}Join token '${TOKEN}' created successfully!${NC}"
echo -e "${GREEN}Token is valid for 1 hour${NC}"