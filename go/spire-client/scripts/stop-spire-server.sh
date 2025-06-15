#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

PID_FILE="/tmp/spire-server.pid"

echo -e "${YELLOW}Stopping SPIRE Server...${NC}"

# Check if PID file exists
if [ ! -f "${PID_FILE}" ]; then
    echo -e "${RED}SPIRE Server PID file not found. Server may not be running.${NC}"
    exit 1
fi

# Read PID from file
SPIRE_PID=$(cat ${PID_FILE})

# Check if process is running
if ! kill -0 ${SPIRE_PID} 2>/dev/null; then
    echo -e "${RED}SPIRE Server with PID ${SPIRE_PID} is not running.${NC}"
    rm -f ${PID_FILE}
    exit 1
fi

# Send SIGTERM to gracefully stop the server
echo -e "${YELLOW}Sending SIGTERM to SPIRE Server (PID: ${SPIRE_PID})...${NC}"
kill ${SPIRE_PID}

# Wait for process to terminate
for i in {1..10}; do
    if ! kill -0 ${SPIRE_PID} 2>/dev/null; then
        echo -e "${GREEN}SPIRE Server stopped successfully${NC}"
        rm -f ${PID_FILE}
        exit 0
    fi
    sleep 1
done

# If still running, force kill
echo -e "${YELLOW}Force killing SPIRE Server...${NC}"
kill -9 ${SPIRE_PID} 2>/dev/null || true

# Wait a bit more
sleep 2

if ! kill -0 ${SPIRE_PID} 2>/dev/null; then
    echo -e "${GREEN}SPIRE Server force stopped${NC}"
    rm -f ${PID_FILE}
else
    echo -e "${RED}Failed to stop SPIRE Server${NC}"
    exit 1
fi