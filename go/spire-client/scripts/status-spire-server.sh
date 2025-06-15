#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

PID_FILE="/tmp/spire-server.pid"

echo -e "${YELLOW}Checking SPIRE Server status...${NC}"

# Check if PID file exists
if [ ! -f "${PID_FILE}" ]; then
    echo -e "${RED}SPIRE Server is not running (no PID file)${NC}"
    exit 1
fi

# Read PID from file
SPIRE_PID=$(cat ${PID_FILE})

# Check if process is running
if kill -0 ${SPIRE_PID} 2>/dev/null; then
    echo -e "${GREEN}SPIRE Server is running (PID: ${SPIRE_PID})${NC}"
    echo -e "${YELLOW}Endpoints:${NC}"
    echo -e "${YELLOW}  - TCP gRPC API: localhost:8081${NC}"
    echo -e "${YELLOW}  - Unix socket: /tmp/spire-server/private/api.sock${NC}"
else
    echo -e "${RED}SPIRE Server is not running (PID ${SPIRE_PID} not found)${NC}"
    rm -f ${PID_FILE}
    exit 1
fi