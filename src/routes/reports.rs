use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Extension, Query, State},
    middleware::from_fn_with_state,
    routing::get,
    Json, Router,
};

use crate::auth::middleware::{require_auth, AuthUser};
use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct ReportFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Totals {
    pub total_workouts: i64,
    pub completed_workouts: i64,
    pub pending_workouts: i64,
    pub total_sets: i64,
    pub total_reps: i64,
    pub total_volume: f64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MuscleGroupStats {
    pub muscle_group: String,
    pub workouts: i64,
    pub sets: i64,
    pub volume: f64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub workouts: i64,
    pub sets: i64,
    pub volume: f64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct WeeklyFrequency {
    pub week_start: DateTime<Utc>,
    pub workouts: i64,
}

#[derive(Serialize)]
pub struct ReportSummary {
    pub totals: Totals,
    pub by_muscle_group: Vec<MuscleGroupStats>,
    pub by_category: Vec<CategoryStats>,
    pub weekly_frequency: Vec<WeeklyFrequency>,
}

fn internal(e: sqlx::Error) -> ApiError {
    tracing::error!("database error: {e}");
    ApiError::Internal
}

const TOTALS_SQL: &str = "
SELECT
  COUNT(DISTINCT w.id)::bigint AS total_workouts,
  COUNT(DISTINCT w.id) FILTER (WHERE w.status='completed')::bigint AS completed_workouts,
  COUNT(DISTINCT w.id) FILTER (WHERE w.status='pending')::bigint AS pending_workouts,
  COALESCE(SUM(we.target_sets),0)::bigint AS total_sets,
  COALESCE(SUM(we.target_reps),0)::bigint AS total_reps,
  COALESCE(SUM(we.target_sets*we.target_reps*we.target_weight),0)::double precision AS total_volume
FROM workouts w
LEFT JOIN workout_exercises we ON we.workout_id = w.id
WHERE w.user_id = $1
  AND ($2::timestamptz IS NULL OR w.scheduled_at >= $2)
  AND ($3::timestamptz IS NULL OR w.scheduled_at <= $3)";

pub async fn summary(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<ReportSummary>, ApiError> {
    let totals = sqlx::query_as::<_, Totals>(TOTALS_SQL)
        .bind(auth.user_id)
        .bind(filter.from)
        .bind(filter.to)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;

    let by_muscle_group = sqlx::query_as::<_, MuscleGroupStats>(
        "SELECT e.muscle_group AS muscle_group,
                COUNT(DISTINCT w.id)::bigint AS workouts,
                COALESCE(SUM(we.target_sets),0)::bigint AS sets,
                COALESCE(SUM(we.target_sets*we.target_reps*we.target_weight),0)::double precision AS volume
         FROM workouts w
         JOIN workout_exercises we ON we.workout_id = w.id
         JOIN exercises e ON e.id = we.exercise_id
         WHERE w.user_id=$1 AND w.status='completed'
           AND ($2::timestamptz IS NULL OR w.completed_at >= $2)
           AND ($3::timestamptz IS NULL OR w.completed_at <= $3)
         GROUP BY e.muscle_group
         ORDER BY volume DESC",
    )
    .bind(auth.user_id)
    .bind(filter.from)
    .bind(filter.to)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let by_category = sqlx::query_as::<_, CategoryStats>(
        "SELECT e.category AS category,
                COUNT(DISTINCT w.id)::bigint AS workouts,
                COALESCE(SUM(we.target_sets),0)::bigint AS sets,
                COALESCE(SUM(we.target_sets*we.target_reps*we.target_weight),0)::double precision AS volume
         FROM workouts w
         JOIN workout_exercises we ON we.workout_id = w.id
         JOIN exercises e ON e.id = we.exercise_id
         WHERE w.user_id=$1 AND w.status='completed'
           AND ($2::timestamptz IS NULL OR w.completed_at >= $2)
           AND ($3::timestamptz IS NULL OR w.completed_at <= $3)
         GROUP BY e.category
         ORDER BY volume DESC",
    )
    .bind(auth.user_id)
    .bind(filter.from)
    .bind(filter.to)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    let weekly_frequency = sqlx::query_as::<_, WeeklyFrequency>(
        "SELECT date_trunc('week', w.completed_at)::timestamptz AS week_start,
                COUNT(*)::bigint AS workouts
         FROM workouts w
         WHERE w.user_id=$1 AND w.status='completed' AND w.completed_at IS NOT NULL
           AND ($2::timestamptz IS NULL OR w.completed_at >= $2)
           AND ($3::timestamptz IS NULL OR w.completed_at <= $3)
         GROUP BY 1
         ORDER BY 1",
    )
    .bind(auth.user_id)
    .bind(filter.from)
    .bind(filter.to)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;

    Ok(Json(ReportSummary {
        totals,
        by_muscle_group,
        by_category,
        weekly_frequency,
    }))
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/summary", get(summary))
        .layer(from_fn_with_state(state, require_auth))
}