pub mod auth;
pub mod error;
pub mod routes;
pub mod openapi;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi; 

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|error| {
            tracing::error!("database health check failed: {error}");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    Ok(Json(HealthResponse { status: "ok" }))
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", routes::auth::router(state.clone()))
        .nest("/exercises", routes::exercices::router())
        .nest("/workouts", routes::workouts::router(state.clone()))
        .nest("/reports", routes::reports::router(state.clone()))
        .merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi::ApiDoc::openapi()),
        )
        .with_state(state)
}