use axum::{
    extract::{Extension, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;

use crate::auth::{middleware::require_auth, middleware::AuthUser, password, token};
use crate::AppState;

#[derive(Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}



fn validate(email: &str, password: &str) -> Result<(), ApiError> {
    if !email.contains('@') || email.len() < 5 {
        return Err(ApiError::Validation("invalid email".into()));
    }
    if password.len() < 8 {
        return Err(ApiError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

fn jwt_secret() -> Result<Vec<u8>, ApiError> {
    std::env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .map_err(|_| {
            tracing::error!("JWT_SECRET is not set");
            ApiError::Internal
        })
}

fn jwt_ttl() -> i64 {
    std::env::var("JWT_TTL_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3600)
}

pub async fn signup(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    validate(&creds.email, &creds.password)?;

    let hash = password::hash_password(&creds.password).map_err(|e|{
        tracing::error!("hashing failed: {e}");
        ApiError::Internal
    })?;

    let row = sqlx::query_as::<_, (i32, i32)>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)
              ON CONFLICT (email) DO NOTHING
              RETURNING id, token_version",
    )
    .bind(&creds.email)
    .bind(&hash)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("insert user failed: {e}");
        ApiError::Internal
    })?;

    let (user_id, ver) = row.ok_or(ApiError::EmailTaken)?;

    let token = token::encode_token(&jwt_secret()?, user_id, ver, jwt_ttl())
            .map_err(|e| {
                tracing::error!("token encode failed: {e}");
                ApiError::Internal
            })?;

    Ok((StatusCode::CREATED, Json(AuthResponse {token})))
}

pub async fn login(
    State(state): State<AppState>,
    Json(creds): Json<Credentials>,
) -> Result<Json<AuthResponse>, ApiError> {
    let row = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT id, password_hash, token_version FROM users WHERE email = $1",
    )
    .bind(&creds.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("login query failed: {e}");
        ApiError::Internal
    })?;

    let (user_id, hash, ver) = row.ok_or(ApiError::InvalidCredentials)?;
    if !password::verify_password(&creds.password, &hash) {
        return Err(ApiError::InvalidCredentials);
    }

    let token = token::encode_token(&jwt_secret()?, user_id, ver, jwt_ttl())
        .map_err(|e| {
            tracing::error!("token encode failed: {e}");
            ApiError::Internal
        })?;

    Ok(Json(AuthResponse { token }))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<StatusCode, ApiError> {
    sqlx::query("UPDATE users SET token_version = token_version + 1 WHERE id = $1")
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("logout failed: {e}");
            ApiError::Internal
        })?;

    Ok(StatusCode::NO_CONTENT)
}


pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/signup", post(signup))
        .route("/login", post(login))
        // .layer() here protects ONLY this route
        .route(
            "/logout",
            post(logout).layer(from_fn_with_state(state, require_auth)),
        )
}