use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(String),
}

pub type DbPool = PgPool;
pub type DbResult<T> = Result<T, DatabaseError>;

pub async fn create_pool(database_url: &str) -> DbResult<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> DbResult<()> {
    // Run manual migrations since we're not using sqlx-cli
    let migration_queries = vec![
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_seed_data.sql"),
    ];

    for (i, query) in migration_queries.iter().enumerate() {
        // Split query by semicolons and execute each statement separately
        let statements: Vec<&str> = query
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for statement in statements {
            if !statement.trim().is_empty() {
                sqlx::query(statement).execute(pool).await.map_err(|e| {
                    DatabaseError::Migration(format!(
                        "Migration {} statement '{}' failed: {}",
                        i + 1,
                        statement,
                        e
                    ))
                })?;
            }
        }
    }

    Ok(())
}

pub async fn health_check(pool: &DbPool) -> DbResult<bool> {
    let row = sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await?;

    let result: i32 = row.get("health_check");
    Ok(result == 1)
}

#[cfg(test)]
pub async fn create_test_pool() -> DbResult<DbPool> {
    let database_url = std::env::var("DATABASE_TEST_URL").unwrap_or_else(|_| {
        "postgresql://groups_user:groups_password@localhost:5433/groups_test".to_string()
    });

    let pool = create_pool(&database_url).await?;

    // Clean up existing data for tests
    sqlx::query("TRUNCATE TABLE users, groups RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await?;

    Ok(pool)
}
