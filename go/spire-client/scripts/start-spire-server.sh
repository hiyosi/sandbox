#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting SPIRE Server setup...${NC}"

# Configuration
SPIRE_VERSION="1.12.2"
SPIRE_DIR="/opt/spire"
DATA_DIR="/var/lib/spire/data"
CONFIG_DIR="/etc/spire"
SOCKET_PATH="/tmp/spire-server/private/api.sock"

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p ${SPIRE_DIR}
mkdir -p ${DATA_DIR}
mkdir -p ${CONFIG_DIR}
mkdir -p $(dirname ${SOCKET_PATH})

# Download and install SPIRE if not present
if [ ! -f "${SPIRE_DIR}/spire-server" ]; then
    echo -e "${YELLOW}Downloading SPIRE ${SPIRE_VERSION}...${NC}"
    cd /tmp
    curl -s -L "https://github.com/spiffe/spire/releases/download/v${SPIRE_VERSION}/spire-${SPIRE_VERSION}-linux-amd64-musl.tar.gz" | tar xz
    mv spire-${SPIRE_VERSION}/bin/spire-server ${SPIRE_DIR}/
    chmod +x ${SPIRE_DIR}/spire-server
    rm -rf spire-${SPIRE_VERSION}
    cd -
fi

# Create SPIRE Server configuration
echo -e "${YELLOW}Creating SPIRE Server configuration...${NC}"
cat > ${CONFIG_DIR}/server.conf <<EOF
server {
    bind_address = "0.0.0.0"
    bind_port = "8081"
    socket_path = "${SOCKET_PATH}"
    trust_domain = "example.org"
    data_dir = "${DATA_DIR}"
    log_level = "DEBUG"
    ca_key_type = "rsa-2048"
    ca_subject = {
        country = ["US"]
        organization = ["SPIRE"]
        common_name = "SPIRE Server CA"
    }
}

plugins {
    DataStore "sql" {
        plugin_data {
            database_type = "sqlite3"
            connection_string = "${DATA_DIR}/datastore.sqlite3"
        }
    }

    NodeAttestor "join_token" {
        plugin_data {}
    }

    KeyManager "disk" {
        plugin_data {
            keys_path = "${DATA_DIR}/keys.json"
        }
    }

    UpstreamAuthority "disk" {
        plugin_data {
            key_file_path = "${DATA_DIR}/ca-key.pem"
            cert_file_path = "${DATA_DIR}/ca-cert.pem"
        }
    }
}
EOF

# Generate CA if not exists
if [ ! -f "${DATA_DIR}/ca-cert.pem" ]; then
    echo -e "${YELLOW}Generating CA certificate...${NC}"
    openssl req -x509 -newkey rsa:4096 -nodes \
        -keyout "${DATA_DIR}/ca-key.pem" \
        -out "${DATA_DIR}/ca-cert.pem" \
        -days 3650 \
        -subj "/C=US/ST=State/L=City/O=SPIRE/CN=SPIRE Root CA"
fi

# Start SPIRE Server
echo -e "${GREEN}Starting SPIRE Server...${NC}"
echo -e "${YELLOW}Server will be available at:${NC}"
echo -e "${YELLOW}  - TCP gRPC API: localhost:8081 (TLS/mTLS for agents)${NC}"
echo -e "${YELLOW}  - Unix socket: ${SOCKET_PATH} (admin privileges)${NC}"

# Start SPIRE Server in background
${SPIRE_DIR}/spire-server run -config ${CONFIG_DIR}/server.conf &
SPIRE_PID=$!

# Save PID for stop script
echo ${SPIRE_PID} > /tmp/spire-server.pid

echo -e "${GREEN}SPIRE Server started with PID: ${SPIRE_PID}${NC}"
echo -e "${YELLOW}Use './scripts/stop-spire-server.sh' to stop the server${NC}"

# Wait a moment for server to start
sleep 2

# Check if server is still running
if kill -0 ${SPIRE_PID} 2>/dev/null; then
    echo -e "${GREEN}SPIRE Server is running successfully${NC}"
else
    echo -e "${RED}SPIRE Server failed to start${NC}"
    exit 1
fi
