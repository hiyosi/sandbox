# OpenFGA認可モデル - SPIRE + OpenFGAデモ

## モデル概要

このモデルは、SPIREで認証されたサービス（APIサーバー等）がOpenFGAのAPIを呼び出して、ユーザーのリソースアクセス権限をチェックするデモ用の設計です。

**重要**: SPIREで認証されたサービスは認可の「主体」ではなく、OpenFGAを操作する「クライアント」として機能します。

### タイプ定義

#### 1. user
- 一般的なユーザー（人間）
- 例: `user:alice`, `user:admin`, `user:bob`

#### 2. team
- ユーザーが所属するチーム・組織
- 例: `team:backend-team`, `team:frontend-team`
- 関係:
  - **owner**: チームの所有者
  - **admin**: チームの管理者
  - **member**: チームのメンバー

#### 3. resource
- 保護されたリソース（データ、API、設定等）
- 例: `resource:sensitive-data`, `resource:public-data`
- 関係:
  - **owner**: リソースの所有者
  - **reader**: 読み取り権限
  - **writer**: 書き込み権限
  - **team**: このリソースを管理するチーム
- 計算される権限:
  - **can_read**: 読み取り可能かチェック
  - **can_write**: 書き込み可能かチェック
  - **can_delete**: 削除可能かチェック

## 権限ルール

### 読み取り権限 (can_read)
- 直接的にreaderとして設定されている
- writerまたはownerである（書ける人は読める）
- リソースが属するチームのmemberである

### 書き込み権限 (can_write)
- 直接的にwriterとして設定されている
- ownerである（所有者は書き込める）
- リソースが属するチームのadminである

### 削除権限 (can_delete)
- リソースのownerである
- リソースが属するチームのownerである

## デモデータ

### チーム構成
1. **`team:backend-team`**: バックエンド開発チーム
   - Owner: `user:admin`
   - Admin: `user:alice`
   - Members: `user:bob`, `user:dave`

2. **`team:frontend-team`**: フロントエンド開発チーム
   - Owner: `user:eve`
   - Member: `user:frank`

### リソース構成
1. **`resource:sensitive-data`**: 機密データ
   - Team: `team:backend-team`
   - Owner: `user:admin`

2. **`resource:public-data`**: 公開データ
   - Team: `team:backend-team`
   - Writer: `user:alice`
   - Reader: `user:charlie` (外部ユーザー)

3. **`resource:user-interface-config`**: UI設定
   - Team: `team:frontend-team`
   - Writer: `user:frank`

## アーキテクチャ

```
[ユーザーリクエスト] → [SPIREで認証されたAPIサーバー] → [OpenFGA API呼び出し]
                                                              ↓
                      [権限チェック結果] ← [OpenFGA認可エンジン]
```

SPIREで認証されたAPIサーバーが、ユーザーの代理でOpenFGAに権限チェックを行います。

## 使用例

### 権限チェック例
```bash
# aliceがpublic-dataを読めるか？
# → YES (writerなので自動的にreaderでもある)
check user:alice can_read resource:public-data

# bobがsensitive-dataを読めるか？
# → YES (backend-teamのmemberで、sensitive-dataはbackend-teamが管理)
check user:bob can_read resource:sensitive-data

# charlieがsensitive-dataを読めるか？
# → NO (readerでもなく、backend-teamのmemberでもない)
check user:charlie can_read resource:sensitive-data

# frankがuser-interface-configを編集できるか？
# → YES (直接writerとして設定されている)
check user:frank can_write resource:user-interface-config

# adminがsensitive-dataを削除できるか？
# → YES (リソースの直接的なownerである)
check user:admin can_delete resource:sensitive-data
```

このモデルにより、SPIREで認証されたサービスがOpenFGAで細かい権限制御を行うことができます。
