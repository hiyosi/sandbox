#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== SPIRE + OpenFGA Complete Setup ==="
echo "Project directory: $PROJECT_DIR"

cd "$PROJECT_DIR"

# 1. 証明書の生成
echo "1. Generating certificates..."
chmod +x scripts/generate-certs.sh
./scripts/generate-certs.sh

# 2. SPIRE Serverの起動
echo "2. Starting SPIRE Server..."
docker-compose up -d spire-server

# 3. SPIRE Serverの準備完了を待機
echo "3. Waiting for SPIRE Server to be ready..."
until docker exec spire-server /opt/spire/bin/spire-server healthcheck >/dev/null 2>&1; do
    echo "   Waiting for SPIRE Server..."
    sleep 3
done
echo "   SPIRE Server is ready!"

# 4. Join Tokenの生成とエントリ登録
echo "4. Generating join token and registering entries..."
JOIN_TOKEN=$(docker exec spire-server \
    /opt/spire/bin/spire-server token generate \
    -spiffeID spiffe://example.org/agent/spire-agent | grep "Token:" | cut -d' ' -f2)

echo "   Join token: $JOIN_TOKEN"

# 5. Client workload entries の登録
echo "5. Registering client workload entries..."
docker exec spire-server \
    /opt/spire/bin/spire-server entry create \
    -parentID spiffe://example.org/agent/spire-agent \
    -spiffeID spiffe://example.org/client \
    -selector unix:uid:1000 \
    >/dev/null 2>&1 || echo "   Entry already exists or failed to create"

docker exec spire-server \
    /opt/spire/bin/spire-server entry create \
    -parentID spiffe://example.org/spire/agent/unix/uid/0 \
    -spiffeID spiffe://example.org/go-client \
    -selector unix:uid:0 \
    >/dev/null 2>&1 || echo "   Entry already exists or failed to create"

# 6. 環境変数ファイルの準備
echo "6. Preparing environment file..."
cat > .env << EOF
JOIN_TOKEN=$JOIN_TOKEN
OPENFGA_STORE_ID=
OPENFGA_API_URL=https://openfga:18443
EOF

# 7. OIDC Discovery Providerの起動
echo "7. Starting OIDC Discovery Provider..."
docker-compose up -d oidc-discovery-provider

# 8. OIDC Discovery Providerの準備完了を待機
echo "8. Waiting for OIDC Discovery Provider to be ready..."
until curl -k -s https://localhost:8443/.well-known/openid-configuration >/dev/null 2>&1; do
    echo "   Waiting for OIDC Discovery Provider..."
    sleep 3
done
echo "   OIDC Discovery Provider is ready!"

# 9. SPIRE Agentの起動
echo "9. Starting SPIRE Agent..."
JOIN_TOKEN=$JOIN_TOKEN docker-compose up -d spire-agent

# 10. SPIRE Agentの準備完了を待機
echo "10. Waiting for SPIRE Agent to be ready..."
until docker exec spire-agent /opt/spire/bin/spire-agent healthcheck >/dev/null 2>&1; do
    echo "    Waiting for SPIRE Agent..."
    sleep 3
done
echo "    SPIRE Agent is ready!"

# 11. OpenFGAの起動
echo "11. Starting OpenFGA..."
docker-compose up -d openfga

# 12. OpenFGAの準備完了を待機
echo "12. Waiting for OpenFGA to be ready..."
until curl -k -s https://localhost:18443/healthz | grep -q "SERVING" >/dev/null 2>&1; do
    echo "    Waiting for OpenFGA..."
    sleep 3
done
echo "    OpenFGA is ready!"

# 12.5. SQLiteデータベースマイグレーションの実行
echo "12.5. Running SQLite database migration..."
docker exec openfga /openfga migrate --datastore-engine sqlite --datastore-uri "file:/opt/openfga/data/openfga.db?_fk=1"
echo "    Database migration completed"

# 13. JWT SVIDを使用してOpenFGAをセットアップ
echo "13. Setting up OpenFGA with JWT authentication..."

# JWT SVID取得の準備ができるまで少し待機
sleep 5

# JWT SVID取得
echo "    Getting JWT SVID..."
JWT_TOKEN=""
for i in {1..10}; do
    JWT_TOKEN=$(docker exec -u 1000 spire-agent /opt/spire/bin/spire-agent api fetch jwt -audience openfga -socketPath /tmp/spire-agent/public/api.sock 2>/dev/null | grep -A1 "token" | tail -1 | xargs) || true
    if [ -n "$JWT_TOKEN" ] && [ "$JWT_TOKEN" != "null" ]; then
        break
    fi
    echo "    Attempt $i: Waiting for JWT SVID..."
    sleep 3
done

if [ -z "$JWT_TOKEN" ] || [ "$JWT_TOKEN" = "null" ]; then
    echo "    Error: Failed to get JWT SVID"
    exit 1
fi

echo "    JWT SVID obtained successfully"

# 14. OpenFGAストアの作成
echo "14. Creating OpenFGA store..."
STORE_RESPONSE=$(curl -k -s -X POST "https://localhost:18443/stores" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{"name": "spire-demo-store"}')

STORE_ID=$(echo "$STORE_RESPONSE" | jq -r '.id')
if [ "$STORE_ID" = "null" ] || [ -z "$STORE_ID" ]; then
    echo "    Error: Failed to create store"
    echo "    Response: $STORE_RESPONSE"
    exit 1
fi

echo "    Store created successfully: $STORE_ID"

# 15. 認証モデルの登録
echo "15. Registering authorization model..."
MODEL_RESPONSE=$(curl -k -s -X POST "https://localhost:18443/stores/${STORE_ID}/authorization-models" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d "$(cat ./openfga-model/store-model.json)")

MODEL_ID=$(echo "$MODEL_RESPONSE" | jq -r '.authorization_model_id')
if [ "$MODEL_ID" = "null" ] || [ -z "$MODEL_ID" ]; then
    echo "    Error: Failed to register authorization model"
    echo "    Response: $MODEL_RESPONSE"
    exit 1
fi

echo "    Authorization model registered: $MODEL_ID"

# 16. 初期タプルの登録
echo "16. Registering initial tuples..."
TUPLES_DATA=$(cat ./openfga-model/initial-tuples.json | jq '.tuple_keys')
WRITE_REQUEST=$(jq -n --argjson tuples "$TUPLES_DATA" '{"writes": {"tuple_keys": $tuples}}')

WRITE_RESPONSE=$(curl -k -s -X POST "https://localhost:18443/stores/${STORE_ID}/write" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d "$WRITE_REQUEST")

if [ "$WRITE_RESPONSE" = "{}" ] || [ -z "$WRITE_RESPONSE" ]; then
    echo "    Initial tuples registered successfully"
else
    echo "    Warning: Unexpected response from tuple registration"
    echo "    Response: $WRITE_RESPONSE"
fi

# 17. 環境変数ファイルの更新
echo "17. Updating environment file..."
cat > .env << EOF
JOIN_TOKEN=$JOIN_TOKEN
OPENFGA_STORE_ID=$STORE_ID
OPENFGA_API_URL=https://openfga:18443
EOF

# 18. Go clientの起動
echo "18. Starting Go client..."
docker-compose up -d go-client

echo ""
echo "=== Setup Complete! ==="
echo ""
echo "Services running:"
echo "  - SPIRE Server: http://localhost:8081"
echo "  - SPIRE Server Health: http://localhost:8080/live"
echo "  - OIDC Discovery Provider: https://localhost:8443"
echo "  - SPIRE Agent Health: http://localhost:8082/live"
echo "  - OpenFGA HTTPS API: https://localhost:18443"
echo "  - OpenFGA gRPC: https://localhost:28443"
echo "  - OpenFGA Health: https://localhost:18443/healthz"
echo ""
echo "Store ID: $STORE_ID"
echo "Model ID: $MODEL_ID"
echo ""
echo "To test the integration:"
echo "  docker-compose run --rm go-client"
echo ""
echo "To view logs:"
echo "  docker-compose logs -f go-client"
echo ""
echo "To cleanup:"
echo "  docker-compose down"
echo "  docker volume prune"