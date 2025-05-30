# Web Server JWT Authentication

## 前提条件

- Rust (2021 Edition以上)
- cargo-make
- Python 3.x
- 必要なPythonライブラリ:
  - cryptography
  - PyJWT

Pythonライブラリのインストール:

```bash
pip3 install cryptography PyJWT
```

## EC256鍵の生成

### Pythonを使用した鍵生成

```bash
cargo make gen-keys
```

このコマンドは以下のファイルを生成します:
- `jwk/private.pem` - 秘密鍵（PEM形式）
- `jwk/public.pem` - 公開鍵（PEM形式）
- `jwk/jwk.json` - 公開鍵（JWK形式）

## JWT検証用トークンの生成

```bash
cargo make gen-jwt
```

**オプション付きの実行例:**

```bash
./hack/generate_jwt.py --subject "user123" --issuer "my-issuer" --audience "web_server" --expires 3600
```

### 主要なオプション

- `--subject`/`-s`: トークンのsubクレーム (デフォルト: `user123`)
- `--issuer`/`-i`: トークンのissクレーム (デフォルト: `https://example.com`)
- `--audience`/`-a`: トークンのaudクレーム (デフォルト: `https://api.example.com`)
  - **注意**: デフォルトの検証設定では `web_server` のaudience値を期待しています
- `--expires`/`-e`: 有効期限（秒） (デフォルト: `3600`)
- `--output`/`-o`: JWT出力先のファイルパス（指定しない場合は標準出力）

## JWT検証

**注意**: JWTトークンのaudience値（`aud`クレーム）が検証時に期待される値と一致しない場合、`InvalidAudience`エラーが発生します。解決するには以下のいずれかを実施してください:

1. トークン生成時に`--audience "web_server"`を指定する
2. 検証コードの`validation.set_audience(&["web_server"])`を変更する
3. 検証コードを`validation.validate_aud = false`に変更して検証を無効化する

## エラーのデバッグ

JWT検証でエラーが発生した場合、エラーメッセージがコンソールに表示されます。以下は一般的なエラーと解決策です:

- `Error(InvalidAudience)`: トークンのaudクレームが期待値と一致していません。トークン生成時に適切な`--audience`を指定するか、検証の設定を変更してください。
- `Error(ExpiredSignature)`: トークンの有効期限が切れています。新しいトークンを生成してください。
- `Error(InvalidSignature)`: トークンの署名が無効です。正しい鍵で署名されているか確認してください。

## 利用例

1. 鍵の生成:
   ```bash
   cargo make gen-keys
   ```

2. 検証用トークンの生成 (audience値を`web_server`に指定):
   ```bash
   ./hack/generate_jwt.py --audience "web_server"
   ```

3. アプリケーションでトークンを検証:
   ```bash
   # サーバーを起動
   cargo make dev
   ```

4. curlでリクエストを送信:
   ```bash
   # JWTトークンを変数に格納
   TOKEN=$(./hack/generate_jwt.py --audience "web_server" | grep -A 1 "生成されたJWT:" | tail -n 1)
   
   # 保護されたエンドポイントへリクエスト
   curl -X GET http://localhost:3000/protected -H "Authorization: Bearer $TOKEN"
   
   # または直接トークンを指定
   curl -X GET http://localhost:3000/protected -H "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJFUzI1NiIsImtpZCI6ImtleS0xNzE3MTkzMzg5In0.eyJzdWIiOiJ1c2VyMTIzIiwiaXNzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbSIsImF1ZCI6IndlYl9zZXJ2ZXIiLCJpYXQiOjE3MTcxOTMzODksImV4cCI6MTcxNzE5Njk4OX0.xxxxxxxxxxxx"
   ```

5. トークンの検証結果を確認:
   ```
   # 成功時の応答例
   認証成功
   
   # エラー時の応答例（トークンのaudience値が不正な場合）
   無効なトークンです
   
   # エラー時の応答例（トークンの有効期限切れの場合）
   トークンの有効期限が切れています
   
   # エラー時の応答例（Authorization ヘッダーがない場合）
   Authorizationヘッダーが必要です
   ```
