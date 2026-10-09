use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::Value;
use workout_tracker_rust::{AppState, app};

pub fn test_app(pool: sqlx::PgPool) -> Router {
    unsafe { std::env::set_var("JWT_SECRET", "test-secret") };
    app(AppState { db: pool })
}

pub fn json_req(method: &str, uri: &str, token: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

pub async fn read_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}