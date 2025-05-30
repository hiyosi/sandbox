use std::fs;
use std::path::Path;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use p256::{
    ecdsa::VerifyingKey,
    EncodedPoint,
    FieldBytes,
};

// JWTのクレーム（ペイロード）の構造を定義
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: Option<String>,
    pub exp: usize,
    // 必要に応じて他のクレームを追加
}

// JWK構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct Jwk {
    kty: String,
    crv: String,
    x: String,
    y: String,
    #[serde(rename = "use")] // useが予約語なので置き換え
    usage: Option<String>,
    kid: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("トークンの有効期限が切れています")]
    TokenExpired,
    #[error("署名が無効です")]
    InvalidSignature,
    #[error("ファイル読み込みエラー: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("鍵のフォーマットが無効です")]
    InvalidKeyFormat,
}

pub struct JwtValidator {
    jwk: Jwk,
    validation: Validation,
}

impl JwtValidator {
    pub fn new(jwk_json: &str) -> Result<Self, ValidationError> {
        let jwk: Jwk = serde_json::from_str(jwk_json)
            .map_err(|_| ValidationError::InvalidKeyFormat)?;

        if jwk.kty != "EC" || jwk.crv != "P-256" {
            return Err(ValidationError::InvalidKeyFormat);
        }

        let mut validation = Validation::new(Algorithm::ES256);
        validation.validate_exp = true;
        validation.set_audience(&["web_server"]); // 必要に応じて変更

        Ok(Self { jwk, validation })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ValidationError> {
        let jwk_json = fs::read_to_string(path)
            .map_err(|e| ValidationError::FileReadError(e))?;

        Self::new(&jwk_json)
    }

    pub fn validate(&self, token: &str) -> Result<Claims, ValidationError> {
        let decoding_key = match self.create_decoding_key() {
            Ok(key) => key,
            Err(e) => {
                println!("デコーディングキーの作成エラー: {:?}", e);
                return Err(ValidationError::InvalidKeyFormat);
            }
        };

        let token_data = match decode::<Claims>(token, &decoding_key, &self.validation) {
            Ok(data) => data,
            Err(e) => {
                println!("トークンのデコードエラー: {:?}", e);
                return Err(ValidationError::InvalidSignature);
            }
        };

        // 現在時刻のチェック
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if token_data.claims.exp <= now {
            println!("トークンの有効期限切れ: exp={}, now={}", token_data.claims.exp, now);
            return Err(ValidationError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    fn create_decoding_key(&self) -> Result<DecodingKey, Box<dyn Error>> {
        println!("JWKデータ: x={}, y={}", self.jwk.x, self.jwk.y);

        let x_vec = URL_SAFE_NO_PAD.decode(&self.jwk.x)?;
        let y_vec = URL_SAFE_NO_PAD.decode(&self.jwk.y)?;

        println!("デコード後のバイト長: x={}, y={}", x_vec.len(), y_vec.len());

        let x_bytes: [u8; 32] = x_vec.try_into().map_err(|_| "Invalid x coordinate length")?;
        let y_bytes: [u8; 32] = y_vec.try_into().map_err(|_| "Invalid y coordinate length")?;

        let x = FieldBytes::from(x_bytes);
        let y = FieldBytes::from(y_bytes);

        let point = EncodedPoint::from_affine_coordinates(&x, &y, false);
        let verify_key = VerifyingKey::from_encoded_point(&point)?;
        let key_bytes = verify_key.to_sec1_bytes();

        Ok(DecodingKey::from_ec_der(&key_bytes))
    }
}

// HTTPリクエストからJWTトークンを抽出するユーティリティ関数
pub fn extract_jwt_from_header(request: &str) -> Option<&str> {
    for line in request.lines() {
        if line.starts_with("Authorization: Bearer ") {
            return Some(line.trim_start_matches("Authorization: Bearer ").trim());
        }
    }
    None
}
