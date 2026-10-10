use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Workout Tracker API",
        version = "0.1.0",
        description = "API for managing workout plans and tracking progress."
    ),
    paths(
        crate::routes::auth::signup,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::exercices::list_exercises,
        crate::routes::workouts::create,
        crate::routes::workouts::list,
        crate::routes::workouts::get_one,
        crate::routes::workouts::update,
        crate::routes::workouts::complete,
        crate::routes::workouts::delete_one,
        crate::routes::reports::summary,
    ),
    components(schemas(
        crate::routes::auth::Credentials,
        crate::routes::auth::AuthResponse,
        crate::routes::exercices::Exercise,
        crate::routes::workouts::WorkoutExerciseInput,
        crate::routes::workouts::CreateWorkout,
        crate::routes::workouts::UpdateWorkout,
        crate::routes::workouts::Workout,
        crate::routes::workouts::WorkoutExercise,
        crate::routes::workouts::WorkoutDetail,
        crate::routes::workouts::CreatedWorkout,
        crate::routes::reports::Totals,
        crate::routes::reports::MuscleGroupStats,
        crate::routes::reports::CategoryStats,
        crate::routes::reports::WeeklyFrequency,
        crate::routes::reports::ReportSummary,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "exercises", description = "Exercise catalog"),
        (name = "workouts", description = "Workout management"),
        (name = "reports", description = "Progress reports"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}
