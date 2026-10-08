use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;

#[derive(Clone, Copy)]
pub struct AuthUser {
    pub user_id: i32,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v|v.to_str().ok())
        .and_then(|v|v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        tracing::error!("JWT_SECRET is not set");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let claims = crate::auth::token::decode_token(token, secret.as_bytes())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token_version: i32 = sqlx::query_scalar("SELECT token_version FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("token version lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if token_version != claims.ver {
        return Err(StatusCode::UNAUTHORIZED);
    }

    req.extensions_mut().insert(AuthUser {
        user_id: claims.sub,
    });
    

    Ok(next.run(req).await)
}
