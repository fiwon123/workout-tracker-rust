mod common;
use axum::http::StatusCode;
use common::*;
use tower::ServiceExt;

#[sqlx::test]
async fn summary_reflects_completed_workout(pool: sqlx::PgPool) {
    let app = test_app(pool.clone());

    let ex: i32 = sqlx::query_scalar(
        "INSERT INTO exercises (name, category, muscle_group)
         VALUES ('Test Press', 'strength', 'chest') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let res = app.clone().oneshot(json_req(
        "POST", "/auth/signup", None,
        serde_json::json!({"email": "r@x.com", "password": "password1"}),
    )).await.unwrap();
    let token = read_json(res).await["token"].as_str().unwrap().to_string();

    let create = app.clone().oneshot(json_req("POST", "/workouts", Some(&token), serde_json::json!({
        "name": "Push day",
        "scheduled_at": "2030-01-02T10:00:00Z",
        "comments": "",
        "exercises": [{"exercise_id": ex, "sets": 3, "reps": 10, "weight": 50.0}]
    }))).await.unwrap();
    let id = read_json(create).await["id"].as_i64().unwrap() as i32;

    let comp = app.clone().oneshot(json_req("POST", &format!("/workouts/{id}/complete"), Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(comp.status(), StatusCode::OK);

    let sum = app.oneshot(json_req("GET", "/reports/summary", Some(&token), serde_json::json!({})))
        .await.unwrap();
    assert_eq!(sum.status(), StatusCode::OK);
    let body = read_json(sum).await;
    let totals = &body["totals"];
    assert_eq!(totals["total_workouts"], 1);
    assert_eq!(totals["completed_workouts"], 1);
    assert_eq!(totals["pending_workouts"], 0);
    assert_eq!(totals["total_sets"], 3);
    assert_eq!(totals["total_reps"], 30);
    assert_eq!(totals["total_volume"], 1500.0);
    assert_eq!(body["by_muscle_group"][0]["muscle_group"], "chest");
    assert!(body["weekly_frequency"].as_array().unwrap().len() >= 1);
}