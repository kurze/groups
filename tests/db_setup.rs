use diesel::sqlite::SqliteConnection;
use diesel_migrations;

use std::error::Error;
use std::fs; // Add this line

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness}; // Add SqliteMigrations

pub fn create_test_db() -> SqliteConnection {
    let db_url = "db/test.sqlite";
    let _ = fs::remove_file(db_url); // Remove any existing test database file
    let mut connection =
        SqliteConnection::establish(db_url).expect("Failed to create test database");
    run_migrations(&mut connection).expect("Failed to run migrations");

    connection
}

fn run_migrations(
    connection: &mut impl MigrationHarness<diesel::sqlite::Sqlite>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    // This will run the necessary migrations.
    //
    // See the documentation for `MigrationHarness` for
    // all available methods.
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    connection.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}
