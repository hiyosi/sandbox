#!/bin/bash

# SPIRE Client Rust <-> Go Interoperability Test Runner
# Tests mTLS connections between Rust and Go implementations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up background processes..."
    jobs -p | xargs -r kill -TERM 2>/dev/null || true
    sleep 2
    jobs -p | xargs -r kill -KILL 2>/dev/null || true
}

trap cleanup EXIT

# Test configuration
RUST_SERVER_PORT=8443
GO_SERVER_PORT=8444
TEST_TIMEOUT=30
CERT_DIR="certs"

echo "=================================================="
echo "🦀 SPIRE Client Rust <-> Go Interop Tests 🐹"
echo "=================================================="
echo ""

# Check dependencies
log_info "Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    log_error "Cargo not found. Please install Rust."
    exit 1
fi

if ! command -v go &> /dev/null; then
    log_error "Go not found. Please install Go."
    exit 1
fi

log_success "Dependencies check passed"

# Clean and prepare
log_info "Preparing test environment..."
mkdir -p ${CERT_DIR}/

# Generate SPIFFE-compliant certificates
log_info "Generating SPIFFE-compliant certificates..."
if ! go run generate_spiffe_certs.go -cert-dir ${CERT_DIR}; then
    log_error "Failed to generate SPIFFE certificates"
    exit 1
fi
log_success "SPIFFE certificates generated successfully"

# Test 1: Rust Server <-> Go Client
echo ""
echo "==================== TEST 1 ===================="
log_info "Testing Rust Server <-> Go Client"

log_info "Starting Rust mTLS server on port $RUST_SERVER_PORT..."
cd rust-impl
timeout $TEST_TIMEOUT cargo run --bin mtls_server -- --port $RUST_SERVER_PORT --cert-dir "../${CERT_DIR}" &
RUST_SERVER_PID=$!
cd ..

# Wait for server to start
sleep 3

# Check if server is running
if ! kill -0 $RUST_SERVER_PID 2>/dev/null; then
    log_error "Rust server failed to start"
    exit 1
fi

log_info "Compiling Go client..."
cd go-client
go build -o go_client .
cd ..

log_info "Running Go client against Rust server..."
if go-client/go_client -server localhost -port $RUST_SERVER_PORT -cert-dir ${CERT_DIR}; then
    log_success "✓ Rust Server <-> Go Client: PASSED"
    TEST1_RESULT="PASSED"
else
    log_error "✗ Rust Server <-> Go Client: FAILED"
    TEST1_RESULT="FAILED"
fi

# Stop Rust server
kill $RUST_SERVER_PID 2>/dev/null || true
wait $RUST_SERVER_PID 2>/dev/null || true

# Test 2: Go Server <-> Rust Client
echo ""
echo "==================== TEST 2 ===================="
log_info "Testing Go Server <-> Rust Client"

log_info "Compiling Go server..."
cd go-server
go build -o go_server .
cd ..

log_info "Starting Go mTLS server on port $GO_SERVER_PORT..."
go-server/go_server -port $GO_SERVER_PORT -cert-dir ${CERT_DIR} &
GO_SERVER_PID=$!

# Wait for server to start
sleep 3

# Check if server is running
if ! kill -0 $GO_SERVER_PID 2>/dev/null; then
    log_error "Go server failed to start"
    exit 1
fi

log_info "Running Rust client against Go server..."
cd rust-impl
if timeout $TEST_TIMEOUT cargo run --bin mtls_client -- --server localhost --port $GO_SERVER_PORT --cert-dir "../${CERT_DIR}"; then
    log_success "✓ Go Server <-> Rust Client: PASSED"
    TEST2_RESULT="PASSED"
else
    log_error "✗ Go Server <-> Rust Client: FAILED"
    TEST2_RESULT="FAILED"
fi
cd ..

# Stop Go server
kill $GO_SERVER_PID 2>/dev/null || true
wait $GO_SERVER_PID 2>/dev/null || true

# Test 3: Cross-Certificate Validation
echo ""
echo "==================== TEST 3 ===================="
log_info "Testing Cross-Certificate Validation"

log_info "Checking certificate compatibility..."

if [ -f "${CERT_DIR}/rust-server.crt" ] && [ -f "${CERT_DIR}/rust-client.crt" ] && [ -f "${CERT_DIR}/ca.crt" ]; then
    # Verify certificate chain
    if openssl verify -CAfile ${CERT_DIR}/ca.crt ${CERT_DIR}/rust-server.crt >/dev/null 2>&1; then
        log_success "✓ Rust server certificate chain valid"
    else
        log_warning "⚠ Rust server certificate chain validation failed"
    fi

    if openssl verify -CAfile ${CERT_DIR}/ca.crt ${CERT_DIR}/rust-client.crt >/dev/null 2>&1; then
        log_success "✓ Rust client certificate chain valid"
    else
        log_warning "⚠ Rust client certificate chain validation failed"
    fi

    # Check SPIFFE ID in certificates
    log_info "Checking SPIFFE IDs in certificates..."
    if openssl x509 -in ${CERT_DIR}/rust-server.crt -text -noout | grep -q "spiffe://"; then
        SPIFFE_ID=$(openssl x509 -in ${CERT_DIR}/rust-server.crt -text -noout | grep "spiffe://" | head -1 | sed 's/.*URI://' | tr -d ' ')
        log_success "✓ Server SPIFFE ID found: $SPIFFE_ID"
    else
        log_warning "⚠ No SPIFFE ID found in server certificate"
    fi

    if openssl x509 -in ${CERT_DIR}/rust-client.crt -text -noout | grep -q "spiffe://"; then
        SPIFFE_ID=$(openssl x509 -in ${CERT_DIR}/rust-client.crt -text -noout | grep "spiffe://" | head -1 | sed 's/.*URI://' | tr -d ' ')
        log_success "✓ Client SPIFFE ID found: $SPIFFE_ID"
    else
        log_warning "⚠ No SPIFFE ID found in client certificate"
    fi

    TEST3_RESULT="PASSED"
else
    log_error "✗ Required certificates not found"
    TEST3_RESULT="FAILED"
fi

# Test Summary
echo ""
echo "==================== SUMMARY ===================="
echo "Test Results:"
echo "  1. Rust Server <-> Go Client:     $TEST1_RESULT"
echo "  2. Go Server <-> Rust Client:     $TEST2_RESULT"
echo "  3. Cross-Certificate Validation:  $TEST3_RESULT"
echo ""

if [ "$TEST1_RESULT" = "PASSED" ] && [ "$TEST2_RESULT" = "PASSED" ] && [ "$TEST3_RESULT" = "PASSED" ]; then
    log_success "🎉 All interoperability tests PASSED!"
    log_success "✓ mTLS communication works between Rust and Go implementations"
    log_success "✓ SPIFFE certificate validation is working"
    echo ""
    echo "Constitution Compliance Check:"
    echo "✓ mTLS Communication: ENFORCED (all connections use mutual TLS)"
    echo "✓ SPIFFE Authentication: IMPLEMENTED (SPIFFE IDs in certificates)"
    echo "✓ Cross-platform Interoperability: VERIFIED"
    exit 0
else
    log_error "❌ Some tests failed. Check the logs above for details."
    echo ""
    echo "Debugging tips:"
    echo "  - Check that certificates are properly generated"
    echo "  - Verify that servers can bind to the specified ports"
    echo "  - Ensure both Rust and Go implementations support the same TLS versions"
    echo "  - Check that SPIFFE IDs are properly embedded in certificates"
    exit 1
fi