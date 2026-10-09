use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::EnvFilter;

mod auth;
mod routes;
mod error;

#[derive(Clone)]
pub(crate )struct AppState{
    pub(crate) db:PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    sqlx::query("SELECT 1")
                .execute(&state.db)
                .await
                .map_err(|error| {
                    tracing::error!("database health check failed: {error}");
                    StatusCode::SERVICE_UNAVAILABLE
                })?;

    Ok(Json(HealthResponse { status: "ok" }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
                .with_env_filter(
                    EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| EnvFilter::new("info")),
                )
                .init();
    
    let database_url = std::env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await?;

    let state = AppState { db };

    let app = Router::new()
                .route("/health", get(health))
                .nest("/auth", routes::auth::router(state.clone()))
                .with_state(state);

    let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    tracing::info!("API listening on http://{listen_addr}");

    axum::serve(listener,app).await?;

    Ok(())
}
