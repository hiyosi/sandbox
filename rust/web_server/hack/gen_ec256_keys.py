#!/usr/bin/env python3
"""
EC256鍵ペアを生成し、PEM形式とJWK形式で保存するスクリプト
"""

import os
import json
import base64
import datetime
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives import serialization

def main():
    print("EC256鍵ペアを生成中...")

    # EC256鍵ペアを生成
    private_key = ec.generate_private_key(ec.SECP256R1())
    public_key = private_key.public_key()

    # jwkディレクトリがなければ作成
    if not os.path.exists('jwk'):
        os.makedirs('jwk')

    # 秘密鍵をPEM形式で保存
    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()
    )
    with open('jwk/private.pem', 'wb') as f:
        f.write(private_pem)
    print("秘密鍵を jwk/private.pem に保存しました")

    # 公開鍵をPEM形式で保存
    public_pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    with open('jwk/public.pem', 'wb') as f:
        f.write(public_pem)
    print("公開鍵を jwk/public.pem に保存しました")

    # JWK形式に変換して保存
    jwk = create_jwk(public_key)
    with open('jwk/jwk.json', 'w') as f:
        json.dump(jwk, f, indent=2)
    print("JWKを jwk/jwk.json に保存しました")

    print("生成されたJWK:")
    print(json.dumps(jwk, indent=2))

def create_jwk(public_key):
    """公開鍵をJWK形式に変換"""
    numbers = public_key.public_numbers()

    # x座標とy座標をBase64URLエンコード
    x_b64 = base64_url_encode(numbers.x.to_bytes(32, byteorder="big"))
    y_b64 = base64_url_encode(numbers.y.to_bytes(32, byteorder="big"))

    # JWK形式を作成
    return {
        "kty": "EC",
        "crv": "P-256",
        "x": x_b64,
        "y": y_b64,
        "use": "sig",
        "alg": "ES256",
        "kid": f"key-{int(datetime.datetime.now().timestamp())}"
    }

def base64_url_encode(data):
    """データをBase64URLエンコード"""
    encoded = base64.urlsafe_b64encode(data).decode('ascii')
    return encoded.rstrip('=')  # パディングを削除

if __name__ == "__main__":
    main()
