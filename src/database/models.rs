use diesel::prelude::*;

#[derive(Queryable, Selectable, PartialEq, Debug)]
#[diesel(table_name = crate::database::schema::groups)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub creation_date: chrono::NaiveDateTime,
    pub deletion_date: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::groups)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewGroup {
    pub name: String,
    pub creation_date: chrono::NaiveDateTime,
    pub deletion_date: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Selectable, PartialEq, Debug)]
#[diesel(table_name = crate::database::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub creation_date: chrono::NaiveDateTime,
    pub deletion_date: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub creation_date: chrono::NaiveDateTime,
    pub deletion_date: Option<chrono::NaiveDateTime>,
}
