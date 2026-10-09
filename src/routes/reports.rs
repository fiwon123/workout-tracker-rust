use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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