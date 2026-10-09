mod common;
use axum::http::StatusCode;
use common::*;
use tower::ServiceExt;

async fn seed_exercise(pool: &sqlx::PgPool) -> i32 {
    sqlx::query_scalar(
        "INSERT INTO exercises (name, category, muscle_group)
         VALUES ($1, 'strength', 'chest') RETURNING id",
    )
    .bind("Test Press")
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn create_workout(app: &axum::Router, token: &str, exercise_id: i32) -> i32 {
    let res = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/workouts",
            Some(token),
            serde_json::json!({
                "name": "Push day",
                "scheduled_at": "2030-01-02T10:00:00Z",
                "comments": "",
                "exercises": [
                    {"exercise_id": exercise_id, "sets": 3, "reps": 10, "weight": 50.0}
                ]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    read_json(res).await["id"].as_i64().unwrap() as i32
}

#[sqlx::test]
async fn create_list_get_update_delete_flow(pool: sqlx::PgPool) {
    let app = test_app(pool.clone());
    let ex = seed_exercise(&pool).await;

    let res = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "b@x.com", "password": "password1"}),
    )).await.unwrap();
    let token = read_json(res).await["token"].as_str().unwrap().to_string();

    let id = create_workout(&app, &token, ex).await;

    let list = app.clone().oneshot(json_req("GET", "/workouts", Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let items = read_json(list).await;
    assert!(items.as_array().unwrap().iter().any(|w| w["id"] == id));

    let one = app.clone().oneshot(json_req("GET", &format!("/workouts/{id}"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(one.status(), StatusCode::OK);
    assert_eq!(read_json(one).await["exercises"][0]["target_weight"], 50.0);

    let upd = app.clone().oneshot(json_req(
        "PATCH", &format!("/workouts/{id}"), Some(&token),
        serde_json::json!({"comments": "felt good"}),
    )).await.unwrap();
    assert_eq!(upd.status(), StatusCode::OK);
    assert_eq!(read_json(upd).await["comments"], "felt good");

    let del = app.clone().oneshot(json_req("DELETE", &format!("/workouts/{id}"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(del.status(), StatusCode::NO_CONTENT);

    let gone = app.oneshot(json_req("GET", &format!("/workouts/{id}"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn unknown_exercise_id_returns_400(pool: sqlx::PgPool) {
    let app = test_app(pool.clone());
    let res = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "c@x.com", "password": "password1"}),
    )).await.unwrap();
    let token = read_json(res).await["token"].as_str().unwrap().to_string();

    let bad = app.oneshot(json_req("POST", "/workouts", Some(&token), serde_json::json!({
        "name": "Bad",
        "scheduled_at": "2030-01-02T10:00:00Z",
        "comments": "",
        "exercises": [{"exercise_id": 999999, "sets": 3, "reps": 10, "weight": 0.0}]
    }))).await.unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn complete_then_recomplete_returns_409(pool: sqlx::PgPool) {
    let app = test_app(pool.clone());
    let ex = seed_exercise(&pool).await;
    let res = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "d@x.com", "password": "password1"}),
    )).await.unwrap();
    let token = read_json(res).await["token"].as_str().unwrap().to_string();
    let id = create_workout(&app, &token, ex).await;

    let c1 = app.clone().oneshot(json_req("POST", &format!("/workouts/{id}/complete"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(c1.status(), StatusCode::OK);

    let c2 = app.oneshot(json_req("POST", &format!("/workouts/{id}/complete"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(c2.status(), StatusCode::CONFLICT);
}

#[sqlx::test]
async fn user_b_cannot_touch_user_a_workout(pool: sqlx::PgPool) {
    let app = test_app(pool.clone());
    let ex = seed_exercise(&pool).await;

    let ta = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "a@x.com", "password": "password1"}),
    )).await.unwrap();
    let token_a = read_json(ta).await["token"].as_str().unwrap().to_string();

    let tb = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "a2@x.com", "password": "password1"}),
    )).await.unwrap();
    let token_b = read_json(tb).await["token"].as_str().unwrap().to_string();

    let id = create_workout(&app, &token_a, ex).await;

    let get_b = app.clone().oneshot(json_req("GET", &format!("/workouts/{id}"), Some(&token_b), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(get_b.status(), StatusCode::NOT_FOUND);

    let del_b = app.oneshot(json_req("DELETE", &format!("/workouts/{id}"), Some(&token_b), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(del_b.status(), StatusCode::NOT_FOUND);
}