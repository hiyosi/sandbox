# OpenFGA Go Client with SPIRE Integration

このクライアントは、SPIREで認証されたサービスがOpenFGAのAPIを呼び出して権限チェックを行うデモ用のGoアプリケーションです。

## 機能

### 1. SPIRE統合
- **JWT SVID取得**: SPIRE Workload APIからJWT SVIDを取得
- **JWT認証**: JWT SVIDをBearerトークンとしてOpenFGAに送信
- **自動認証**: SPIREエージェントとの自動的な認証処理

### 2. OpenFGA操作
- **権限チェック**: ユーザーのリソースアクセス権限を確認
- **バッチ処理**: 複数の権限を一括でチェック
- **エラーハンドリング**: 適切なエラー処理とログ出力

### 3. 実行モード
- **demo**: モックJWTを使用したデモモード
- **spire**: SPIRE認証を使用した本格モード
- **test**: 包括的なテストシナリオの実行

## ビルドと実行

### ローカル実行
```bash
cd client
go mod tidy
go build -o client .

# デモモード
./client demo

# SPIREモード（SPIRE Agent必要）
./client spire

# テストモード
./client test
```

### テスト実行
```bash
# 単体テスト
go test -v

# カバレッジ付き
go test -v -cover

# 特定のテスト
go test -v -run TestPermissionChecks
```

### Docker実行
```bash
# イメージビルド
docker build -t openfga-client .

# コンテナ実行（SPIRE Agent統合）
docker run --rm \
  -v /tmp/spire-agent:/tmp/spire-agent \
  openfga-client ./client spire
```

## アーキテクチャ

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Request  │───▶│  Go Client      │───▶│   OpenFGA       │
└─────────────────┘    │                 │    │                 │
                       │ ┌─────────────┐ │    │ ┌─────────────┐ │
                       │ │ SPIRE       │ │    │ │ Permission  │ │
                       │ │ JWT SVID    │ │◀───│ │ Engine      │ │
                       │ └─────────────┘ │    │ └─────────────┘ │
                       └─────────────────┘    └─────────────────┘
                                 ▲
                       ┌─────────────────┐
                       │   SPIRE Agent   │
                       │                 │
                       │ ┌─────────────┐ │
                       │ │ Workload    │ │
                       │ │ API         │ │
                       │ └─────────────┘ │
                       └─────────────────┘
```

## APIリファレンス

### OpenFGAClient
```go
type OpenFGAClient struct {
    client  *client.OpenFgaClient
    storeID string
}
```

#### メソッド

##### NewOpenFGAClient
```go
func NewOpenFGAClient(apiURL, storeID string, jwtToken string) (*OpenFGAClient, error)
```
通常のJWTトークンを使用してクライアントを作成

##### NewOpenFGAClientWithSPIRE
```go
func NewOpenFGAClientWithSPIRE(apiURL, storeID string) (*OpenFGAClient, error)
```
SPIRE認証を使用してクライアントを作成

##### CheckPermission
```go
func (c *OpenFGAClient) CheckPermission(ctx context.Context, user, relation, object string) (bool, error)
```
単一の権限をチェック

例:
```go
allowed, err := client.CheckPermission(ctx, "user:alice", "can_read", "resource:public-data")
```

##### BatchCheck
```go
func (c *OpenFGAClient) BatchCheck(ctx context.Context, checks []CheckRequest) ([]bool, error)
```
複数の権限を一括チェック

例:
```go
checks := []CheckRequest{
    {"user:alice", "can_read", "resource:public-data"},
    {"user:bob", "can_write", "resource:sensitive-data"},
}
results, err := client.BatchCheck(ctx, checks)
```

## テストシナリオ

### 1. 基本権限テスト
- 直接的な権限（owner, reader, writer）
- 継承権限（writer→reader, owner→writer）
- チーム権限（team member→resource access）

### 2. チーム権限テスト
- backend-team: alice(admin), bob(member), dave(member)
- frontend-team: eve(owner), frank(member)
- リソースアクセス制御

### 3. 権限マトリックステスト
- 全ユーザー × 全リソース × 全権限の組み合わせ
- 期待値との比較検証

### 4. 統合テスト
- SPIRE認証フロー
- OpenFGA API通信
- エラーハンドリング

## 設定

### 環境変数
```bash
# OpenFGA設定
OPENFGA_API_URL=https://localhost:18443
OPENFGA_STORE_ID=01JBQF9Z8P9QX1X1X1X1X1X1X1

# SPIRE設定
SPIRE_SOCKET_PATH=/tmp/spire-agent/public/api.sock
```

### 必要な権限
- SPIREエージェントソケットへのアクセス
- OpenFGA APIエンドポイントへのネットワークアクセス
- TLS証明書の検証（または無効化設定）

## トラブルシューティング

### よくある問題

1. **SPIRE Agent接続エラー**
   ```
   failed to create JWT source: no such file or directory
   ```
   → SPIREエージェントが起動していることを確認

2. **OpenFGA接続エラー**
   ```
   failed to check permission: connection refused
   ```
   → OpenFGAサーバーが起動していることを確認

3. **TLS証明書エラー**
   ```
   x509: certificate signed by unknown authority
   ```
   → 証明書設定またはInsecureSkipVerifyの設定を確認

### デバッグ方法
```bash
# 詳細ログ付きで実行
GODEBUG=x509debug=1 ./client spire

# SPIREエージェント確認
docker exec spire-agent /opt/spire/bin/spire-agent api fetch jwt -audience openfga

# OpenFGA接続確認
curl -k https://localhost:18443/healthz
```

## セキュリティ考慮事項

1. **本番環境では**:
   - `InsecureSkipVerify: false`に設定
   - 適切なCA証明書を使用
   - タイムアウト値の調整

2. **SPIRE統合**:
   - 適切なSPIFFE IDの検証
   - JWT SVIDの有効期限チェック
   - mTLS証明書の検証

3. **ログ**:
   - JWT SVIDのログ出力に注意
   - 機密情報の適切なマスキング