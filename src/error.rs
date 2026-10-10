use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("{0}")]
    Validation(String),
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("email already registered")]
    EmailTaken,
    #[error("not found")]
    NotFound,
    #[error("internal error")]
    Internal,
    #[error("workout already completed")]
    AlreadyCompleted,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiError::EmailTaken => StatusCode::CONFLICT,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::AlreadyCompleted => StatusCode::CONFLICT,
        };
        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}
