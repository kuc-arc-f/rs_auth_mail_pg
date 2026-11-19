use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::{Duration, Utc};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

#[derive(Deserialize)]
struct RequestOtpPayload {
    email: String,
}

#[derive(Deserialize)]
struct VerifyOtpPayload {
    email: String,
    code: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
    token: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 1. DB接続
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // 2. Gmail SMTP クライアントの構築 (TLS Wrapper: Port 465)
    let gmail_user = std::env::var("GMAIL_USER").expect("GMAIL_USER must be set");
    let gmail_pass = std::env::var("GMAIL_APP_PASSWORD").expect("GMAIL_APP_PASSWORD must be set");

    let creds = Credentials::new(gmail_user.clone(), gmail_pass);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    let state = AppState { db: pool, mailer };

    // 3. ルーティング
    let app = Router::new()
        .route("/auth/request-otp", post(request_otp))
        .route("/auth/verify-otp", post(verify_otp))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// OTP発行エンドポイント
async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<RequestOtpPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 6桁のOTPを生成
    let otp: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let expires_at = Utc::now() + Duration::minutes(10);

    // DBに保存 (既存の古いコードは削除)
    sqlx::query("DELETE FROM otp_codes WHERE email = $1")
        .bind(&payload.email)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO otp_codes (email, code, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(&payload.email)
    .bind(&otp)
    .bind(expires_at)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // メール送信
    let email = Message::builder()
        .from("Your App <your-email@gmail.com>".parse().unwrap())
        .to(payload.email.parse().unwrap())
        .subject("Your Login OTP Code")
        .header(ContentType::TEXT_PLAIN)
        .body(format!("Your OTP code is: {}. It expires in 10 minutes.", otp))
        .unwrap();

    state
        .mailer
        .send(email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send email: {}", e)))?;

    Ok(Json(ApiResponse {
        message: "OTP sent to your email".to_string(),
        token: None,
    }))
}

// OTP検証エンドポイント
async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpPayload>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // DBから有効なOTPを検索
    let record = sqlx::query_as::<_, (String, chrono::DateTime<Utc>)>(
        "SELECT code, expires_at FROM otp_codes WHERE email = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    //println!("#user_id");
    println!("email={:?}", &payload.email);

    let (stored_code, expires_at) = match record {
        Some(r) => r,
        None => return Err((StatusCode::BAD_REQUEST, "No OTP requested".to_string())),
    };

    // 期限切れチェック
    if Utc::now() > expires_at {
        return Err((StatusCode::BAD_REQUEST, "OTP expired".to_string()));
    }

    // コード一致チェック
    if payload.code != stored_code {
        return Err((StatusCode::UNAUTHORIZED, "Invalid OTP".to_string()));
    }

    // 認証成功：OTPを無効化
    sqlx::query("DELETE FROM otp_codes WHERE email = $1")
        .bind(&payload.email)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // ユーザーが存在すれば取得、なければ作成 (パスワードレスログイン)
    let user_id: (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (email) VALUES ($1) ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email RETURNING id"
    )
    .bind(&payload.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    println!("#user_id");
    println!("{:?}", user_id);

    // 実際のアプリケーションではここでJWTを発行します。
    // ここでは簡易的にUUIDをトークンとして返します。
    let token = uuid::Uuid::new_v4().to_string();

    Ok(Json(ApiResponse {
        message: "Login successful".to_string(),
        token: Some(token),
    }))
}