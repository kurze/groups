use crate::database::db;
use crate::database::models::Group;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};

impl db::DB {
    pub fn list_groups(&mut self) -> Result<Vec<Group>, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups
            .select(Group::as_select())
            .load::<Group>(&mut self.conn)
    }

    pub fn count_groups(&mut self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups.count().get_result(&mut self.conn)
    }

    pub fn get_group(&mut self, group_id: i32) -> Result<Option<Group>, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        groups
            .find(group_id)
            .select(Group::as_select())
            .first(&mut self.conn)
            .optional()
    }

    pub fn create_group(&mut self, group_name: &str) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
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
        use crate::database::schema::groups::dsl::*;
        diesel::delete(groups.find(group_id)).execute(&mut self.conn)
    }

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

    pub fn clear_groups(&mut self) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::groups::dsl::*;
        diesel::delete(groups).execute(&mut self.conn)
    }
}
