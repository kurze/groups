use diesel::sqlite::SqliteConnection;
use diesel_migrations;

use std::error::Error;
use std::fs; // Add this line

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness}; // Add SqliteMigrations

const DB_URL: &str = "db/test.sqlite";
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn create_test_db() -> SqliteConnection {
    let _ = fs::remove_file(DB_URL); // Remove any existing test database file
    let mut connection =
        SqliteConnection::establish(DB_URL).expect("Failed to create test database");
    run_migrations(&mut connection).expect("Failed to run migrations");

    connection
}

fn run_migrations(
    connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
