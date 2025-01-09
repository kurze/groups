use crate::database::db;
use crate::database::models::{NewUser, User};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

impl db::DB {
    /// Lists all users in the database.
    ///
    /// # Returns
    /// A `Result` containing a vector of `User` on success, or a `diesel::result::Error` on failure.
    pub fn list_users(&mut self) -> Result<Vec<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        users.select(User::as_select()).load::<User>(&mut self.conn)
    }

    /// Retrieves a user by their ID.
    ///
    /// # Parameters
    /// - `user_id`: The ID of the user to retrieve.
    ///
    /// # Returns
    /// A `Result` containing an `Option<User>` on success, or a `diesel::result::Error` on failure.
    pub fn get_user(&mut self, user_id: i32) -> Result<Option<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        users
            .find(user_id)
            .select(User::as_select())
            .first(&mut self.conn)
            .optional()
    }

    /// Creates a new user with the specified username and password.
    ///
    /// # Parameters
    /// - `username`: The username of the user to create.
    /// - `password`: The password of the user to create.
    ///
    /// # Returns
    /// A `Result` containing the newly created `User` on success, or a `diesel::result::Error` on failure.
    pub fn create_user(&mut self, login: &str, pwd: &str) -> Result<User, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        let new_user = NewUser {
            username: login.to_string(),
            password: pwd.to_string(),
            creation_date: chrono::Utc::now().naive_utc(),
            deletion_date: None,
        };
        diesel::insert_into(users)
            .values(&new_user)
            .get_result(&mut self.conn)
    }

    /// Deletes a user by their ID.
    ///
    /// # Parameters
    /// - `user_id`: The ID of the user to delete.
    ///
    /// # Returns
    /// A `Result` containing the number of rows affected on success, or a `diesel::result::Error` on failure.
    pub fn delete_user(&mut self, user_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        diesel::delete(users.find(user_id)).execute(&mut self.conn)
    }

    /// Updates the username and password of a user by their ID.
    ///
    /// # Parameters
    /// - `user_id`: The ID of the user to update.
    /// - `username`: The new username of the user.
    /// - `password`: The new password of the user.
    ///
    /// # Returns
    /// A `Result` containing the number of rows affected on success, or a `diesel::result::Error` on failure.
    pub fn update_user(
        &mut self,
        user_id: i32,
        username: &str,
        password: &str,
    ) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        diesel::update(users.find(user_id))
            .set((username.eq(username), password.eq(password)))
            .execute(&mut self.conn)
    }
}
