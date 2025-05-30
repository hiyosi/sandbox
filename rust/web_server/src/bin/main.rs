use std::net::TcpListener;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Arc;
use clap::Parser;
use std::error::Error;
use web_server::{ThreadPool, JwtValidator, ValidationError, extract_jwt_from_header};

// 起動フラグの定義
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H' , long, default_value = "127.0.0.1")]
    host: String,

    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short, long, default_value = "jwk/jwk.json")]
    jwk_file: String,
}

fn handle_client(mut stream: TcpStream, validator: &JwtValidator) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(size) => {
            let request = String::from_utf8_lossy(&buffer[..size]);

            println!("受信したリクエスト:\n{}", request);

            let response = match extract_jwt_from_header(&request) {
                Some(token) => {
                    println!("抽出されたトークン: {}", token);
                    match validator.validate(token) {
                        Ok(claims) => {
                            println!("検証成功。クレーム: {:?}", claims);
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n認証成功\r\n"
                        },
                        Err(e) => {
                            println!("検証エラー: {:?}", e);
                            match e {
                                ValidationError::TokenExpired =>
                                    "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\n\r\nトークンの有効期限が切れています\r\n",
                                ValidationError::InvalidSignature =>
                                    "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\n\r\n無効な署名です\r\n",
                                _ => "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\n\r\n無効なトークンです\r\n",
                            }
                        }
                    }
                },
                None => {
                    println!("Authorizationヘッダーが見つかりません");
                    "HTTP/1.1 401 Unauthorized\r\nContent-Type: text/plain\r\n\r\nAuthorizationヘッダーが必要です\r\n"
                }
            };

            stream.write_all(response.as_bytes()).expect("レスポンスの送信に失敗しました");
            stream.flush().expect("ストリームのフラッシュに失敗しました");
        },
        Err(e) => println!("リクエストの読み取りエラー: {}", e),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();


    let validator = JwtValidator::from_file(&args.jwk_file)
        .map_err(|e| {
            eprintln!("エラー: {}", e);
            std::process::exit(1);
        })?;

    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr)
        .expect("サーバーの起動に失敗しました");

    println!("Server running on http://{}", addr);

    let pool = ThreadPool::new(10);
    let validator = Arc::new(validator);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let validator = Arc::clone(&validator); // validatorのクローンを作成
                pool.execute(move || {
                    handle_client(stream, &validator);
                });
            }
            Err(e) => {
                eprintln!("接続エラー: {}", e);
                continue;
            }
        }
    }

    Ok(())
}
