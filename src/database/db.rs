use diesel::{Connection, RunQueryDsl, SqliteConnection};
use dotenvy::dotenv;
use std::env;

pub struct DB {
    pub conn: SqliteConnection,
}

impl Default for DB {
    fn default() -> Self {
        Self::new()
    }
}

impl DB {
    pub fn new() -> DB {
        let mut db = establish_connection();
        if let Err(e) = diesel::sql_query("SELECT 1").execute(&mut db) {
            eprintln!("Error checking database connection: {}", e);
        } else {
            println!("Database connection established successfully.");
        }
        DB { conn: db }
    }

    pub fn new_with_connection(conn: SqliteConnection) -> DB {
        DB { conn }
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

mod groups;
