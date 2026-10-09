mod common;
use axum::http::StatusCode;
use common::*;
use tower::ServiceExt;

#[sqlx::test]
async fn signup_login_and_protected_route(pool: sqlx::PgPool) {
    let res = test_app(pool.clone())
        .oneshot(json_req(
            "POST",
            "/auth/signup",
            None,
            serde_json::json!({"email": "a@x.com", "password": "password1"}),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let no_token = test_app(pool.clone())
        .oneshot(json_req("GET", "/workouts", None, serde_json::json!({})))
        .await
        .unwrap();
    assert_eq!(no_token.status(), StatusCode::UNAUTHORIZED);

    let login = test_app(pool.clone())
        .oneshot(json_req(
            "POST",
            "/auth/login",
            None,
            serde_json::json!({"email": "a@x.com", "password": "password1"}),
        ))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let token = read_json(login).await["token"].as_str().unwrap().to_string();

    let ok = test_app(pool.clone())
        .oneshot(json_req("GET", "/workouts", Some(&token), serde_json::json!({})))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
}