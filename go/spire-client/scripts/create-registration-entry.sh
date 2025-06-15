#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

SPIRE_DIR="/opt/spire"
CONFIG_DIR="/etc/spire"

# Check if spire-server binary exists
if [ ! -f "${SPIRE_DIR}/spire-server" ]; then
    echo -e "${YELLOW}SPIRE Server not found. Please run start-spire-server.sh first.${NC}"
    exit 1
fi

# Download spire-server binary if not present (for entry commands)
echo -e "${GREEN}Creating test registration entry...${NC}"

# Create a test workload registration entry
${SPIRE_DIR}/spire-server entry create \
    -socketPath /tmp/spire-server/private/api.sock \
    -spiffeID spiffe://example.org/test-workload \
    -parentID spiffe://example.org/node \
    -selector unix:user:root

echo -e "${GREEN}Registration entry created successfully!${NC}"