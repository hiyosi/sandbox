#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== SPIRE + OpenFGA Cleanup ==="
echo "Project directory: $PROJECT_DIR"

cd "$PROJECT_DIR"

# 1. すべてのコンテナを停止・削除
echo "1. Stopping and removing containers..."
docker-compose down

# 2. ボリュームの削除
echo "2. Removing volumes..."
docker volume rm openfga_spire-data openfga_spire-agent-socket openfga_spire-server-socket openfga_openfga-data 2>/dev/null || true

# 3. ネットワークの削除
echo "3. Removing networks..."
docker network rm openfga_spire-net 2>/dev/null || true

# 4. 環境ファイルの削除
echo "4. Removing environment files..."
rm -f .env .openfga-store-id

# 5. 未使用のDockerリソースをクリーンアップ
echo "5. Cleaning up unused Docker resources..."
docker system prune -f

echo ""
echo "=== Cleanup Complete! ==="
echo ""
echo "All containers, volumes, networks, and temporary files have been removed."
echo ""
echo "To start fresh:"
echo "  ./scripts/complete-setup.sh"