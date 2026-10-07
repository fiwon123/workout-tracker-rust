use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_subscriber::EnvFilter;


async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>>{
    let db: PgPool = PgPoolOptions::new()
                        .max_connections(1)
                        .connect(database_url)
                        .await?;

    sqlx::migrate!().run(&db).await?;
    tracing::info!("migrations applied");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")?;

    run_migrations(&database_url).await
}