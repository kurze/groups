use crate::database::db;
use crate::database::models::{Group, NewGroup};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

impl db::DB {
    /// Lists all groups in the database.
    ///
    /// # Returns
    /// A `Result` containing a vector of `Group` on success, or a `diesel::result::Error` on failure.
    pub fn list_groups(&mut self) -> Result<Vec<Group>, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups
            .select(Group::as_select())
            .load::<Group>(&mut self.conn)
    }

    /// Counts the total number of groups in the database.
    ///
    /// # Returns
    /// A `Result` containing the count of groups as `i64` on success, or a `diesel::result::Error` on failure.
    pub fn count_groups(&mut self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups.count().get_result(&mut self.conn)
    }

    /// Retrieves a group by its ID.
    ///
    /// # Parameters
    /// - `group_id`: The ID of the group to retrieve.
    ///
    /// # Returns
    /// A `Result` containing an `Option<Group>` on success, or a `diesel::result::Error` on failure.
    pub fn get_group(&mut self, group_id: i32) -> Result<Option<Group>, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups
            .find(group_id)
            .select(Group::as_select())
            .first(&mut self.conn)
            .optional()
    }

    /// Creates a new group with the specified name.
    ///
    /// # Parameters
    /// - `group_name`: The name of the group to create.
    ///
    /// # Returns
    /// A `Result` containing the newly created `Group` on success, or a `diesel::result::Error` on failure.
    pub fn create_group(&mut self, group_name: &str) -> Result<Group, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        let new_group = NewGroup {
            name: group_name.to_string(),
            creation_date: chrono::Utc::now().naive_utc(),
            deletion_date: None,
        };
        diesel::insert_into(groups)
            .values(&new_group)
            .get_result(&mut self.conn)
    }

    /// Deletes a group by its ID.
    ///
    /// # Parameters
    /// - `group_id`: The ID of the group to delete.
    ///
    /// # Returns
    /// A `Result` containing the number of rows affected on success, or a `diesel::result::Error` on failure.
    pub fn delete_group(&mut self, group_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        diesel::delete(groups.find(group_id)).execute(&mut self.conn)
    }

    /// Updates the name of a group by its ID.
    ///
    /// # Parameters
    /// - `group_id`: The ID of the group to update.
    /// - `group_name`: The new name of the group.
    ///
    /// # Returns
    /// A `Result` containing the number of rows affected on success, or a `diesel::result::Error` on failure.
    pub fn update_group(
        &mut self,
        group_id: i32,
        group_name: &str,
    ) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        diesel::update(groups.find(group_id))
            .set(name.eq(group_name))
            .execute(&mut self.conn)
    }

    /// Clears all groups from the database.
    ///
    /// # Returns
    /// A `Result` containing the number of rows affected on success, or a `diesel::result::Error` on failure.
    pub fn clear_groups(&mut self) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        diesel::delete(groups).execute(&mut self.conn)
    }
}
