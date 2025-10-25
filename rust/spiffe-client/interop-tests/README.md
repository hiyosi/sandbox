# SPIRE Client Rust ↔ Go Interoperability Tests

このディレクトリには、RustとGo SPIFFEクライアント実装間の相互接続試験が含まれています。

## 概要

Go-SPIFFEリポジトリの例を参考に、以下のクロスプラットフォーム試験を実装：

1. **Rust Server ↔ Go Client**: RustのmTLSサーバーとGoクライアント間の通信
2. **Go Server ↔ Rust Client**: GoのmTLSサーバーとRustクライアント間の通信
3. **証明書互換性検証**: SPIFFE ID付きX.509証明書の相互検証

## 憲法準拠確認

以下の非交渉的制約への準拠を検証：

- ✅ **mTLS必須通信**: 全通信で相互TLS認証を強制
- ✅ **SPIFFE準拠認証**: 証明書にSPIFFE IDを埋め込み
- ✅ **クロスプラットフォーム互換性**: Rust ↔ Go間の相互運用性

## ディレクトリ構造

```
interop-tests/
├── rust-impl/
│   ├── mtls_server.rs    # Rust mTLSサーバー
│   └── mtls_client.rs    # Rust mTLSクライアント
├── go-impl/
│   ├── go_server.go      # Go mTLSサーバー
│   └── go_client.go      # Go mTLSクライアント
├── certs/                # 生成された証明書（テスト時）
├── run_tests.sh          # 統合テストランナー
└── README.md             # このファイル
```

## 依存関係

### Rust
- Cargo with nightly toolchain (cargo-script機能用)
- rustls, tokio-rustls, rcgen クレート

### Go
- Go 1.19以上
- 標準ライブラリ (crypto/tls, crypto/x509)

### システム
- OpenSSL (証明書検証用)
- bash (テストスクリプト用)

## 実行方法

### 全自動テスト実行

```bash
# プロジェクトルートから
cd interop-tests
./run_tests.sh
```

### 個別コンポーネント実行

#### Rustサーバー単体起動
```bash
cd interop-tests
cargo +nightly -Zscript rust-impl/mtls_server.rs
```

#### Rustクライアント単体実行
```bash
cd interop-tests
cargo +nightly -Zscript rust-impl/mtls_client.rs -- --server localhost --port 8443
```

#### Goサーバー単体起動
```bash
cd interop-tests/go-impl
go run go_server.go 8444
```

#### Goクライアント単体実行
```bash
cd interop-tests/go-impl
go run go_client.go localhost 8443
```

## テストシナリオ

### Test 1: Rust Server ↔ Go Client

1. Rustサーバーがport 8443でmTLS待受開始
2. 自己署名証明書を生成（SPIFFE ID付き）
3. Goクライアントが接続・相互認証
4. メッセージエコー通信を実行
5. 正常切断を確認

### Test 2: Go Server ↔ Rust Client

1. Goサーバーがport 8444でmTLS待受開始
2. Rust生成証明書を再利用
3. Rustクライアントが接続・相互認証
4. メッセージエコー通信を実行
5. 正常切断を確認

### Test 3: Cross-Certificate Validation

1. 生成された証明書チェーンを検証
2. SPIFFE IDの存在確認
3. OpenSSLでの証明書妥当性検証

## 証明書について

テスト用に以下のSPIFFE IDを使用：

- **Rustサーバー**: `spiffe://example.org/rust-server`
- **Rustクライアント**: `spiffe://example.org/rust-client`
- **CA**: `spiffe://example.org`

実際のSPIRE環境では、SPIRE ServerからWorkload APIを通じて動的に証明書を取得します。

## 期待される出力

```
==================================================
🦀 SPIRE Client Rust <-> Go Interop Tests 🐹
==================================================

[INFO] Checking dependencies...
[SUCCESS] Dependencies check passed

==================== TEST 1 ====================
[INFO] Testing Rust Server <-> Go Client
[SUCCESS] ✓ Rust Server <-> Go Client: PASSED

==================== TEST 2 ====================
[INFO] Testing Go Server <-> Rust Client
[SUCCESS] ✓ Go Server <-> Rust Client: PASSED

==================== TEST 3 ====================
[INFO] Testing Cross-Certificate Validation
[SUCCESS] ✓ Cross-Certificate Validation: PASSED

==================== SUMMARY ====================
Test Results:
  1. Rust Server <-> Go Client:     PASSED
  2. Go Server <-> Rust Client:     PASSED
  3. Cross-Certificate Validation:  PASSED

[SUCCESS] 🎉 All interoperability tests PASSED!
[SUCCESS] ✓ mTLS communication works between Rust and Go implementations
[SUCCESS] ✓ SPIFFE certificate validation is working

Constitution Compliance Check:
✓ mTLS Communication: ENFORCED (all connections use mutual TLS)
✓ SPIFFE Authentication: IMPLEMENTED (SPIFFE IDs in certificates)
✓ Cross-platform Interoperability: VERIFIED
```

## トラブルシューティング

### ポートバインドエラー
- ポート8443, 8444が使用中の場合は該当プロセスを停止
- `lsof -i :8443` でポート使用状況を確認

### 証明書エラー
- `certs/`ディレクトリを削除して再生成
- OpenSSLで証明書内容を確認: `openssl x509 -in certs/rust-server.crt -text -noout`

### TLSハンドシェイクエラー
- 両端でTLSバージョンが一致しているか確認（TLS 1.2以上）
- クライアント認証が有効になっているか確認

## 実際のSPIRE統合

このテストは模擬環境での検証です。実際のSPIRE環境では：

1. **SPIRE Server**: 信頼ドメインのルートCAを管理
2. **SPIRE Agent**: Workload APIでSVIDを配布
3. **Workload**: Workload APIからSVIDを取得して使用

Go-SPIFFEライブラリの実際の使用例は、`go-impl/`ファイル内のコメントを参照してください。

## 関連リンク

- [Go-SPIFFE Examples](https://github.com/spiffe/go-spiffe/tree/main/examples)
- [SPIFFE仕様](https://github.com/spiffe/spiffe)
- [SPIRE](https://github.com/spiffe/spire)