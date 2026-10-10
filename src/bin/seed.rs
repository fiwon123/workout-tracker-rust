use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::EnvFilter;

struct SeedExercise {
    name: &'static str,
    description: &'static str,
    category: &'static str,
    muscle_group: &'static str,
}

const EXERCISES: &[SeedExercise] = &[
    SeedExercise {
        name: "Bench Press",
        description: "Barbell press on a flat bench.",
        category: "strength",
        muscle_group: "chest",
    },
    SeedExercise {
        name: "Incline Dumbbell Press",
        description: "Dumbbell press on an incline bench.",
        category: "strength",
        muscle_group: "chest",
    },
    SeedExercise {
        name: "Push-Up",
        description: "Bodyweight chest press.",
        category: "strength",
        muscle_group: "chest",
    },
    SeedExercise {
        name: "Cable Fly",
        description: "Chest fly using a cable machine.",
        category: "strength",
        muscle_group: "chest",
    },
    SeedExercise {
        name: "Deadlift",
        description: "Barbell lift from the floor.",
        category: "strength",
        muscle_group: "back",
    },
    SeedExercise {
        name: "Pull-Up",
        description: "Bodyweight vertical pull.",
        category: "strength",
        muscle_group: "back",
    },
    SeedExercise {
        name: "Barbell Row",
        description: "Bent-over barbell row.",
        category: "strength",
        muscle_group: "back",
    },
    SeedExercise {
        name: "Lat Pulldown",
        description: "Cable pulldown for the lats.",
        category: "strength",
        muscle_group: "back",
    },
    SeedExercise {
        name: "Back Squat",
        description: "Barbell squat.",
        category: "strength",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Romanian Deadlift",
        description: "Hip-hinge hamstring lift.",
        category: "strength",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Leg Press",
        description: "Machine leg press.",
        category: "strength",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Walking Lunge",
        description: "Alternating forward lunge.",
        category: "strength",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Overhead Press",
        description: "Standing barbell shoulder press.",
        category: "strength",
        muscle_group: "shoulders",
    },
    SeedExercise {
        name: "Lateral Raise",
        description: "Dumbbell side raise.",
        category: "strength",
        muscle_group: "shoulders",
    },
    SeedExercise {
        name: "Face Pull",
        description: "Cable pull toward the face.",
        category: "strength",
        muscle_group: "shoulders",
    },
    SeedExercise {
        name: "Barbell Curl",
        description: "Standing biceps curl.",
        category: "strength",
        muscle_group: "arms",
    },
    SeedExercise {
        name: "Triceps Pushdown",
        description: "Cable triceps extension.",
        category: "strength",
        muscle_group: "arms",
    },
    SeedExercise {
        name: "Running",
        description: "Steady-state or interval running.",
        category: "cardio",
        muscle_group: "full_body",
    },
    SeedExercise {
        name: "Cycling",
        description: "Stationary or outdoor cycling.",
        category: "cardio",
        muscle_group: "full_body",
    },
    SeedExercise {
        name: "Rowing Machine",
        description: "Indoor rowing ergometer.",
        category: "cardio",
        muscle_group: "full_body",
    },
    SeedExercise {
        name: "Jump Rope",
        description: "Skipping rope intervals.",
        category: "cardio",
        muscle_group: "full_body",
    },
    SeedExercise {
        name: "Hamstring Stretch",
        description: "Static hamstring stretch.",
        category: "flexibility",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Hip Flexor Stretch",
        description: "Kneeling hip flexor stretch.",
        category: "flexibility",
        muscle_group: "legs",
    },
    SeedExercise {
        name: "Shoulder Dislocate",
        description: "Band shoulder mobility drill.",
        category: "flexibility",
        muscle_group: "shoulders",
    },
];

async fn seed(db: &PgPool) -> Result<u64, Box<dyn std::error::Error>> {
    let mut inserted = 0u64;
    for e in EXERCISES {
        let result = sqlx::query(
            "INSERT INTO exercises (name, description, category, muscle_group)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (name) DO NOTHING",
        )
        .bind(e.name)
        .bind(e.description)
        .bind(e.category)
        .bind(e.muscle_group)
        .execute(db)
        .await?;
        inserted += result.rows_affected();
    }
    Ok(inserted)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let inserted = seed(&db).await?;
    tracing::info!(
        "seed complete: {inserted} inserted, {} already present",
        EXERCISES.len() as u64 - inserted
    );

    Ok(())
}
