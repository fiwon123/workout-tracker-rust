use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{AppState, error::ApiError};

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct Exercise {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub muscle_group: String,
}

#[derive(Deserialize, IntoParams)]
pub struct ExerciseFilter {
    pub category: Option<String>,
    pub muscle_group: Option<String>,
}

#[utoipa::path(
    get, path = "/exercises", tag = "exercises",
    params(ExerciseFilter),
    responses((status = 200, body = [Exercise]))   // [] = array
)]
pub async fn list_exercises(
    State(state): State<AppState>,
    Query(filter): Query<ExerciseFilter>,
) -> Result<Json<Vec<Exercise>>, ApiError> {
    let exercises = sqlx::query_as::<_, Exercise>(
        "SELECT id, name, description, category, muscle_group
         FROM exercises
         WHERE ($1::text IS NULL OR category = $1)
            AND ($2::text IS NULL OR muscle_group = $2)
         ORDER BY name",
    )
    .bind(filter.category)
    .bind(filter.muscle_group)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("list exercises failed: {e}");
        ApiError::Internal
    })?;

    Ok(Json(exercises))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_exercises))
}
