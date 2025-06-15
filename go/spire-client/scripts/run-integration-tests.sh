#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Running integration tests...${NC}"

# Check if SPIRE Server is running
PID_FILE="/tmp/spire-server.pid"
if [ ! -f "${PID_FILE}" ]; then
    echo -e "${YELLOW}SPIRE Server is not running. Starting it...${NC}"
    ./scripts/start-spire-server.sh
    
    # Wait for server to be ready
    echo -e "${YELLOW}Waiting for SPIRE Server to be ready...${NC}"
    sleep 5
fi

# Check if server is actually running
SPIRE_PID=$(cat ${PID_FILE} 2>/dev/null || echo "")
if [ -z "${SPIRE_PID}" ] || ! kill -0 ${SPIRE_PID} 2>/dev/null; then
    echo -e "${RED}SPIRE Server is not running. Please start it first.${NC}"
    exit 1
fi

echo -e "${GREEN}SPIRE Server is running (PID: ${SPIRE_PID})${NC}"

# Run integration tests
echo -e "${YELLOW}Running integration tests...${NC}"
INTEGRATION_TEST=true go test -v ./test/integration/...

echo -e "${GREEN}Integration tests completed!${NC}"