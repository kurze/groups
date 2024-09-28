use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: i32,
    pub name: String,
    pub creation_date: chrono::NaiveDateTime,
    pub deletion_date: Option<chrono::NaiveDateTime>,
}
