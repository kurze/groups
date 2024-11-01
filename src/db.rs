use std::env;

use crate::models::Group;
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
    SqliteConnection,
};
use dotenvy::dotenv;

pub struct DB {
    conn: SqliteConnection,
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

    pub fn list_groups(&mut self) -> Result<Vec<Group>, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        groups
            .select(Group::as_select())
            .load::<Group>(&mut self.conn)
    }

    pub fn count_groups(&mut self) -> Result<i64, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        groups.count().get_result(&mut self.conn)
    }

    pub fn get_group(&mut self, group_id: i32) -> Result<Option<Group>, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        groups
            .find(group_id)
            .select(Group::as_select())
            .first(&mut self.conn)
            .optional()
    }

    pub fn create_group(&mut self, group_name: &str) -> Result<usize, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        let new_group = Group {
            name: group_name.to_string(),
            creation_date: chrono::Utc::now().naive_utc(),
            deletion_date: None,
            id: 0,
        };
        diesel::insert_into(groups)
            .values(&new_group)
            .execute(&mut self.conn)
    }

    pub fn delete_group(&mut self, group_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        diesel::delete(groups.find(group_id)).execute(&mut self.conn)
    }

    pub fn update_group(
        &mut self,
        group_id: i32,
        group_name: &str,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::groups::dsl::*;
        diesel::update(groups.find(group_id))
            .set(name.eq(group_name))
            .execute(&mut self.conn)
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
