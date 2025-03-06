use crate::db::models::{Group, GroupKey};
use native_db::{db_type, transaction::query::PrimaryScanIterator};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum GroupError {
    #[error("Group must be soft-deleted before hard deletion")]
    NotSoftDeleted,
    #[error("Group not found")]
    GroupNotFound,
    #[error("Database error: {0}")]
    DbError(#[from] db_type::Error),
}

pub struct GroupService<'a> {
    db: Arc<native_db::Database<'a>>,
}

#[allow(dead_code)]
impl<'a> GroupService<'a> {
    pub fn new(db: Arc<native_db::Database<'a>>) -> Self {
        Self { db }
    }

    // Create a new group
    pub fn create(&self, name: String) -> Result<Group, db_type::Error> {
        let group = Group::new(name);
        let rw = self.db.rw_transaction()?;
        rw.insert(group.clone())?;
        rw.commit()?;
        Ok(group)
    }

    // Read group by ID
    pub fn get_by_id(&self, id: u32) -> Result<Option<Group>, db_type::Error> {
        let r = self.db.r_transaction()?;
        r.get().primary(id)
    }

    // Read group by name
    pub fn find_by_name(&self, name: String) -> Result<Vec<Group>, db_type::Error> {
        let r = self.db.r_transaction()?;
        r.scan()
            .secondary(GroupKey::name)?
            .range(name.clone()..=name)?
            .collect()
    }

    // Update group
    pub fn update(&self, group: Group) -> Result<(), GroupError> {
        let rw = self.db.rw_transaction()?;
        let old_group = rw
            .get()
            .primary(group.id)?
            .ok_or(GroupError::GroupNotFound)?;
        rw.update(old_group, group)?;
        rw.commit().map_err(GroupError::DbError)
    }

    // Delete group by ID (soft delete)
    pub fn delete(&self, id: u32) -> Result<(), db_type::Error> {
        let rw = self.db.rw_transaction()?;
        // Get existing group
        let mut group: Group = match rw.get().primary(id)? {
            Some(group) => group,
            None => return Ok(()), // Group doesn't exist, nothing to delete
        };
        let previous_group = group.clone();
        // Soft delete by setting deleted_at
        group.deleted_at = Some(chrono::Utc::now().naive_utc());
        rw.update(previous_group, group)?;
        rw.commit()
    }

    // Hard delete group by ID (permanent removal)
    // Only allowed for groups that have already been soft-deleted
    pub fn hard_delete(&self, id: u32) -> Result<(), GroupError> {
        let rw = self.db.rw_transaction()?;
        // Check if group exists
        let group: Option<Group> = rw.get().primary(id)?;
        match group {
            Some(group) => {
                // Check if the group is soft-deleted
                if group.deleted_at.is_none() {
                    rw.abort()?;
                    return Err(GroupError::NotSoftDeleted);
                }
                // Group is soft-deleted, proceed with hard deletion
                rw.remove(group)?;
                rw.commit()?;
                Ok(())
            }
            None => {
                rw.abort()?;
                Ok(()) // Group doesn't exist, nothing to delete
            }
        }
    }

    // List all active groups
    pub fn list_active(&self) -> Result<Vec<Group>, db_type::Error> {
        let r = self.db.r_transaction()?;
        let primary = r.scan().primary()?;
        let groups: PrimaryScanIterator<Group> = primary.all()?;
        // Filter out deleted groups
        Ok(groups
            .filter_map(Result::ok)
            .filter(|g| g.deleted_at.is_none())
            .collect())
    }

    // Count total number of groups
    pub fn count(&self) -> Result<usize, db_type::Error> {
        let r = self.db.r_transaction()?;
        let primary = r.scan().primary()?;
        let groups: PrimaryScanIterator<Group> = primary.all()?;
        Ok(groups.filter_map(Result::ok).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models;
    use native_db::Builder;

    fn setup_test_db<'a>() -> (Arc<native_db::Database<'a>>, GroupService<'a>) {
        let db = Arc::new(Builder::new().create_in_memory(&models::MODELS).unwrap());
        let service = GroupService::new(db.clone());
        (db, service)
    }

    #[test]
    fn test_create_group() {
        let (_, service) = setup_test_db();
        let group = service.create("Test Group".to_string()).unwrap();
        assert_eq!(group.name, "Test Group");
        assert!(group.deleted_at.is_none());
    }

    #[test]
    fn test_get_group() {
        let (_, service) = setup_test_db();
        let _created_0 = service.create("Test Group 0".to_string()).unwrap();
        let created_1 = service.create("Test Group 1".to_string()).unwrap();
        let _created_2 = service.create("Test Group 2".to_string()).unwrap();
        let by_id = service.get_by_id(created_1.id).unwrap().unwrap();
        assert_eq!(by_id.id, created_1.id);
        let by_name = service.find_by_name("Test Group 1".to_string());
        match by_name {
            Ok(groups) => {
                assert_eq!(groups.len(), 1);
                assert_eq!(groups[0].id, created_1.id);
            }
            Err(e) => panic!("Error finding group by name: {}", e),
        }
    }

    #[test]
    fn test_update_group() {
        let (_, service) = setup_test_db();
        let mut group = service.create("Test Group".to_string()).unwrap();
        let g_id = group.id;
        group.name = "Updated Name".to_string();
        service.update(group).unwrap();
        let updated = service.get_by_id(g_id).unwrap().unwrap();
        assert_eq!(updated.name, "Updated Name");
    }

    #[test]
    fn test_soft_delete() {
        let (_, service) = setup_test_db();
        let group = service.create("Test Group".to_string()).unwrap();
        service.delete(group.id).unwrap();
        let deleted = service.get_by_id(group.id).unwrap().unwrap();
        assert!(deleted.deleted_at.is_some());
        let active_groups = service.list_active().unwrap();
        assert!(active_groups.is_empty());
    }

    #[test]
    fn test_hard_delete() {
        let (_, service) = setup_test_db();
        // Create group
        let group = service.create("To Be Deleted".to_string()).unwrap();
        let group_id = group.id;
        // Verify group exists
        let found_group = service.get_by_id(group_id).unwrap();
        assert!(found_group.is_some(), "Group should exist before deletion");
        // Try to hard delete without soft delete first (should fail)
        let result = service.hard_delete(group_id);
        assert!(
            result.is_err(),
            "Hard delete without soft delete should fail"
        );
        // Soft delete the group
        service.delete(group_id).unwrap();
        // Now hard delete should succeed
        service.hard_delete(group_id).unwrap();
        // Verify group no longer exists
        let deleted_group = service.get_by_id(group_id).unwrap();
        assert!(
            deleted_group.is_none(),
            "Group should not exist after hard deletion"
        );
    }

    #[test]
    fn test_count() {
        let (_, service) = setup_test_db();
        service.create("Test Group 1".to_string()).unwrap();
        {
            let c = service.count().unwrap();
            assert_eq!(c, 1);
        }
        service.create("Test Group 2".to_string()).unwrap();
        {
            let c = service.count().unwrap();
            assert_eq!(c, 2);
        }
    }
}
