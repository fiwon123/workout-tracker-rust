use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::middleware::{require_auth, AuthUser};
use crate::error::ApiError;
use crate::AppState;

#[derive(Deserialize)]
pub struct WorkoutExerciseInput {
    pub exercise_id: i32,
    pub sets: i32,
    pub reps: i32,
    #[serde(default)]
    pub weight: f64,
    pub position: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateWorkout {
    pub name: String,
    pub scheduled_at: DateTime<Utc>,
    #[serde(default)]
    pub comments: String,
    pub exercises: Vec<WorkoutExerciseInput>,
}

#[derive(Deserialize)]
pub struct UpdateWorkout {
    pub name: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub comments: Option<String>,
    pub exercises: Option<Vec<WorkoutExerciseInput>>,
}

#[derive(Deserialize)]
pub struct WorkoutFilter {
    pub status: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Workout {
    pub id: i32,
    pub name: String,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub comments: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct WorkoutExercise {
    pub id: i32,
    pub exercise_id: i32,
    pub position: i32,
    pub target_sets: i32,
    pub target_reps: i32,
    pub target_weight: f64,
}

#[derive(Serialize)]
pub struct WorkoutDetail {
    #[serde(flatten)]
    pub workout: Workout,
    pub exercises: Vec<WorkoutExercise>,
}

#[derive(Serialize)]
pub struct CreatedWorkout {
    pub id: i32,
}


fn internal(e: sqlx::Error) -> ApiError {
    tracing::error!("database error: {e}");
    ApiError::Internal
}

async fn fetch_workout(
    db: &sqlx::PgPool,
    workout_id: i32,
    user_id: i32,
) -> Result<WorkoutDetail, ApiError> {
    let workout = sqlx::query_as::<_, Workout>(
            "SELECT id, name, scheduled_at, status, comments, created_at, completed_at
            FROM workouts WHERE id = $1 AND user_id = $2",
        )
        .bind(workout_id)
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(internal)?
        .ok_or(ApiError::NotFound)?;

    let exercises = sqlx::query_as::<_, WorkoutExercise>(
        "SELECT id, exercise_id, position, target_sets, target_reps, target_weight
         FROM workout_exercises WHERE workout_id = $1 ORDER BY position",
    )
    .bind(workout_id)
    .fetch_all(db)
    .await
    .map_err(internal)?;

    Ok(WorkoutDetail { workout, exercises })
}

pub async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(input): Json<CreateWorkout>,
) -> Result<(StatusCode, Json<CreatedWorkout>), ApiError> {
    if input.name.trim().is_empty() {
        return Err(ApiError::Validation("name is required".into()));
    }
    if input.exercises.is_empty() {
        return Err(ApiError::Validation("at least one exercise is required".into()));
    }

    let mut tx = state.db.begin().await.map_err(internal)?;

    let workout_id: i32 = sqlx::query_scalar(
        "INSERT INTO workouts (user_id, name, scheduled_at, comments)
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(auth.user_id)
    .bind(&input.name)
    .bind(input.scheduled_at)
    .bind(&input.comments)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    for (i, ex) in input.exercises.iter().enumerate() {
        if ex.sets <= 0 || ex.reps <= 0 {
            return Err(ApiError::Validation("sets and reps must be positive".into()));
        }
        sqlx::query(
            "INSERT INTO workout_exercises
                (workout_id, exercise_id, position, target_sets, target_reps, target_weight)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(workout_id)
        .bind(ex.exercise_id)
        .bind(ex.position.unwrap_or(i as i32))
        .bind(ex.sets)
        .bind(ex.reps)
        .bind(ex.weight)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    }

    tx.commit().await.map_err(internal)?;

    Ok((StatusCode::CREATED, Json(CreatedWorkout { id: workout_id })))
}

pub async fn list(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Query(filter): Query<WorkoutFilter>,
) -> Result<Json<Vec<Workout>>, ApiError> {
        let workouts = sqlx::query_as::<_, Workout>(
            "SELECT id, name, scheduled_at, status, comments, created_at, completed_at
            FROM workouts
            WHERE user_id = $1 AND ($2::text IS NULL OR status = $2)
            ORDER BY scheduled_at ASC",
        )
        .bind(auth.user_id)
        .bind(filter.status)
        .fetch_all(&state.db)
        .await
        .map_err(internal)?;

    Ok(Json(workouts))
}

pub async fn get_one(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<Json<WorkoutDetail>, ApiError> {
    Ok(Json(fetch_workout(&state.db, id, auth.user_id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(input): Json<UpdateWorkout>,
) -> Result<Json<WorkoutDetail>, ApiError> {
    let mut tx = state.db.begin().await.map_err(internal)?;

    let affected = sqlx::query(
        "UPDATE workouts
         SET name = COALESCE($1, name),
             scheduled_at = COALESCE($2, scheduled_at),
             comments = COALESCE($3, comments)
         WHERE id = $4 AND user_id = $5",
    )
    .bind(&input.name)
    .bind(input.scheduled_at)
    .bind(&input.comments)
    .bind(id)
    .bind(auth.user_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?
    .rows_affected();

    if affected == 0 {
        return Err(ApiError::NotFound); // tx drops here -> automatic rollback
    }

    if let Some(exercises) = &input.exercises {
        sqlx::query("DELETE FROM workout_exercises WHERE workout_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;

        for (i, ex) in exercises.iter().enumerate() {
            sqlx::query(
                "INSERT INTO workout_exercises
                    (workout_id, exercise_id, position, target_sets, target_reps, target_weight)
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(ex.exercise_id)
            .bind(ex.position.unwrap_or(i as i32))
            .bind(ex.sets)
            .bind(ex.reps)
            .bind(ex.weight)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        }
    }

    tx.commit().await.map_err(internal)?;

    Ok(Json(fetch_workout(&state.db, id, auth.user_id).await?))
}

pub async fn complete(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<Json<Workout>, ApiError> {
        let workout = sqlx::query_as::<_, Workout>(
            "UPDATE workouts
            SET status = 'completed', completed_at = now()
            WHERE id = $1 AND user_id = $2
            RETURNING id, name, scheduled_at, status, comments, created_at, completed_at",
        )
        .bind(id)
        .bind(auth.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(workout))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> Result<StatusCode, ApiError> {
    let affected = sqlx::query("DELETE FROM workouts WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .map_err(internal)?
        .rows_affected();

    if affected == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}


pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/{id}", get(get_one).patch(update).delete(delete_one))
        .route("/{id}/complete", post(complete))
        .layer(from_fn_with_state(state, require_auth))
}