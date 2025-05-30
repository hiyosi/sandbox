#!/usr/bin/env python3
"""
JWK/PEM鍵を使用して検証用のJWTを生成するスクリプト
標準的なクレーム（sub, iss, aud, exp, iat）を含める
"""

import os
import json
import time
import argparse
from datetime import datetime, timedelta, timezone
import jwt  # PyJWT ライブラリを使用

def main():
    parser = argparse.ArgumentParser(description='EC256鍵を使用してJWTを生成する')
    parser.add_argument('--key', default='jwk/private.pem', help='秘密鍵のパス (デフォルト: jwk/private.pem)')
    parser.add_argument('--alg', default='ES256', help='署名アルゴリズム (デフォルト: ES256)')
    parser.add_argument('--subject', '-s', default='user123', help='サブジェクト (sub)')
    parser.add_argument('--issuer', '-i', default='https://example.com', help='発行者 (iss)')
    parser.add_argument('--audience', '-a', default='web_server', help='対象者 (aud)')
    parser.add_argument('--expires', '-e', type=int, default=3600, help='有効期間（秒） (デフォルト: 3600)')
    parser.add_argument('--output', '-o', help='JWT出力先のファイルパス (指定しない場合は標準出力)')
    args = parser.parse_args()

    # 秘密鍵の読み込み
    try:
        with open(args.key, 'rb') as f:
            private_key = f.read()
        print(f"秘密鍵を {args.key} から読み込みました")
    except FileNotFoundError:
        print(f"エラー: 秘密鍵 {args.key} が見つかりません")
        return

    # 現在時刻(UTC)を取得
    now = datetime.now(timezone.utc)

    # JWT用のペイロード（クレーム）を作成
    payload = {
        'sub': args.subject,      # サブジェクト
        'iss': args.issuer,       # 発行者
        'aud': args.audience,     # 対象者
        'iat': int(now.timestamp()),  # 発行時刻
        'exp': int((now + timedelta(seconds=args.expires)).timestamp())  # 有効期限
    }

    # JWTの生成
    try:
        token = jwt.encode(
            payload=payload,
            key=private_key,
            algorithm=args.alg,
            headers={
                'typ': 'JWT',
                'alg': args.alg,
                'kid': f"key-{int(now.timestamp())}"
            }
        )

        # 結果表示とファイル出力
        print("\n生成されたJWT:")
        print(token)

        if args.output:
            with open(args.output, 'w') as f:
                f.write(token)
            print(f"\nJWTを {args.output} に保存しました")

        # デコードしたペイロードの表示
        decoded = jwt.decode(token, options={"verify_signature": False})
        print("\nデコードされたペイロード:")
        print(json.dumps(decoded, indent=2))

    except Exception as e:
        print(f"エラー: JWTの生成に失敗しました: {e}")

if __name__ == "__main__":
    main()
