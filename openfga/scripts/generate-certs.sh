#!/bin/bash
set -e

CERT_DIR="./certs"

# Create directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Check if SPIRE upstream CA certificates already exist
if [[ -f "$CERT_DIR/dummy_upstream_ca.crt" && -f "$CERT_DIR/dummy_upstream_ca.key" ]]; then
    echo "SPIRE upstream CA certificates already exist. Skipping generation."
else
    echo "Generating SPIRE upstream CA certificate..."
    
    # Generate private key
    openssl genrsa -out "$CERT_DIR/dummy_upstream_ca.key" 2048
    
    # Generate self-signed certificate
    openssl req -new -x509 \
        -key "$CERT_DIR/dummy_upstream_ca.key" \
        -out "$CERT_DIR/dummy_upstream_ca.crt" \
        -days 3650 \
        -subj "/C=JP/O=Example Organization/CN=SPIRE Upstream CA"
    
    echo "SPIRE upstream CA certificates generated successfully!"
fi

# Check if server CA certificates already exist
if [[ -f "$CERT_DIR/ca.crt" && -f "$CERT_DIR/ca.key" ]]; then
    echo "Server CA certificates already exist. Skipping generation."
else
    echo "Generating server CA certificate..."
    
    # Generate CA private key
    openssl genrsa -out "$CERT_DIR/ca.key" 4096
    
    # Generate CA certificate
    openssl req -new -x509 \
        -key "$CERT_DIR/ca.key" \
        -out "$CERT_DIR/ca.crt" \
        -days 3650 \
        -subj "/C=JP/O=Example Organization/CN=Server CA"
    
    echo "Server CA certificates generated successfully!"
fi

# Check if OIDC serving certificates already exist
if [[ -f "$CERT_DIR/oidc-server.crt" && -f "$CERT_DIR/oidc-server.key" ]]; then
    echo "OIDC serving certificates already exist. Skipping generation."
else
    echo "Generating OIDC Discovery Provider serving certificate..."
    
    # Generate private key for OIDC server
    openssl genrsa -out "$CERT_DIR/oidc-server.key" 2048
    
    # Create OpenSSL config file for OIDC certificate
    cat > "$CERT_DIR/oidc-server.openssl.conf" << EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = JP
O = Example Organization
CN = oidc-discovery-provider

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = oidc-discovery-provider
DNS.2 = spire-server
DNS.3 = localhost
EOF
    
    # Generate certificate signing request
    openssl req -new \
        -key "$CERT_DIR/oidc-server.key" \
        -out "$CERT_DIR/oidc-server.csr" \
        -config "$CERT_DIR/oidc-server.openssl.conf"
    
    # Generate certificate signed by CA
    openssl x509 -req \
        -in "$CERT_DIR/oidc-server.csr" \
        -CA "$CERT_DIR/ca.crt" \
        -CAkey "$CERT_DIR/ca.key" \
        -CAcreateserial \
        -out "$CERT_DIR/oidc-server.crt" \
        -days 365 \
        -extensions v3_req \
        -extfile "$CERT_DIR/oidc-server.openssl.conf"
    
    # Clean up CSR
    rm "$CERT_DIR/oidc-server.csr"
    
    echo "OIDC serving certificates generated successfully!"
fi

# Check if OpenFGA gRPC certificates already exist
if [[ -f "$CERT_DIR/openfga-server.crt" && -f "$CERT_DIR/openfga-server.key" ]]; then
    echo "OpenFGA gRPC certificates already exist. Skipping generation."
else
    echo "Generating OpenFGA gRPC certificate..."
    
    # Generate private key for OpenFGA gRPC server
    openssl genrsa -out "$CERT_DIR/openfga-server.key" 2048
    
    # Create OpenSSL config file for OpenFGA certificate with IP SAN
    cat > "$CERT_DIR/openfga-server.openssl.conf" << EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = JP
O = Example Organization
CN = openfga

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = openfga
DNS.2 = localhost
IP.1 = 127.0.0.1
IP.2 = 0.0.0.0
EOF
    
    # Generate certificate signing request
    openssl req -new \
        -key "$CERT_DIR/openfga-server.key" \
        -out "$CERT_DIR/openfga-server.csr" \
        -config "$CERT_DIR/openfga-server.openssl.conf"
    
    # Generate certificate signed by CA
    openssl x509 -req \
        -in "$CERT_DIR/openfga-server.csr" \
        -CA "$CERT_DIR/ca.crt" \
        -CAkey "$CERT_DIR/ca.key" \
        -CAcreateserial \
        -out "$CERT_DIR/openfga-server.crt" \
        -days 365 \
        -extensions v3_req \
        -extfile "$CERT_DIR/openfga-server.openssl.conf"
    
    # Clean up CSR
    rm "$CERT_DIR/openfga-server.csr"
    
    echo "OpenFGA gRPC certificates generated successfully!"
fi

# Check if OpenFGA HTTP certificates already exist
if [[ -f "$CERT_DIR/openfga-http.crt" && -f "$CERT_DIR/openfga-http.key" ]]; then
    echo "OpenFGA HTTP certificates already exist. Skipping generation."
else
    echo "Generating OpenFGA HTTP certificate..."
    
    # Generate private key for OpenFGA HTTP server
    openssl genrsa -out "$CERT_DIR/openfga-http.key" 2048
    
    # Generate certificate signing request (reuse same config with IP SAN)
    openssl req -new \
        -key "$CERT_DIR/openfga-http.key" \
        -out "$CERT_DIR/openfga-http.csr" \
        -config "$CERT_DIR/openfga-server.openssl.conf"
    
    # Generate certificate signed by CA
    openssl x509 -req \
        -in "$CERT_DIR/openfga-http.csr" \
        -CA "$CERT_DIR/ca.crt" \
        -CAkey "$CERT_DIR/ca.key" \
        -CAcreateserial \
        -out "$CERT_DIR/openfga-http.crt" \
        -days 365 \
        -extensions v3_req \
        -extfile "$CERT_DIR/openfga-server.openssl.conf"
    
    # Clean up CSR
    rm "$CERT_DIR/openfga-http.csr"
    
    echo "OpenFGA HTTP certificates generated successfully!"
fi

echo "All certificates generated!"
echo "  SPIRE Upstream CA Private key: $CERT_DIR/dummy_upstream_ca.key"
echo "  SPIRE Upstream CA Certificate: $CERT_DIR/dummy_upstream_ca.crt"
echo "  Server CA Private key: $CERT_DIR/ca.key"
echo "  Server CA Certificate: $CERT_DIR/ca.crt"
echo "  OIDC Private key: $CERT_DIR/oidc-server.key"
echo "  OIDC Certificate: $CERT_DIR/oidc-server.crt"
echo "  OpenFGA gRPC Private key: $CERT_DIR/openfga-server.key"
echo "  OpenFGA gRPC Certificate: $CERT_DIR/openfga-server.crt"
echo "  OpenFGA HTTP Private key: $CERT_DIR/openfga-http.key"
echo "  OpenFGA HTTP Certificate: $CERT_DIR/openfga-http.crt"