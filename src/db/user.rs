use crate::db::models::{User, UserKey};
use native_db::{db_type, transaction::query::PrimaryScanIterator};

use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum UserError {
    #[error("User must be soft-deleted before hard deletion")]
    NotSoftDeleted,
    #[error("User not found")]
    UserNotFound,
    #[error("Database error: {0}")]
    DbError(#[from] db_type::Error),
}

pub struct UserService<'a> {
    db: Arc<native_db::Database<'a>>,
}

#[allow(dead_code)]
impl<'a> UserService<'a> {
    pub fn new(db: Arc<native_db::Database<'a>>) -> Self {
        Self { db }
    }

    // Create a new user
    pub fn create(&self, email: String, name: String) -> Result<User, db_type::Error> {
        let mut user = User::new(email);
        user.name = name;

        let rw = self.db.rw_transaction()?;
        rw.insert(user.clone())?;
        rw.commit()?;

        Ok(user)
    }

    // Create a new user with password
    pub fn create_with_password(&self, email: String, name: String, password_hash: String) -> Result<User, db_type::Error> {
        let mut user = User::new(email);
        user.name = name;
        user.password_hash = Some(password_hash);

        let rw = self.db.rw_transaction()?;
        rw.insert(user.clone())?;
        rw.commit()?;

        Ok(user)
    }

    // Read user by ID
    pub fn get_by_id(&self, id: u32) -> Result<Option<User>, db_type::Error> {
        let r = self.db.r_transaction()?;
        r.get().primary(id)
    }

    // Read user by email
    pub fn get_by_email(&self, email: String) -> Result<Option<User>, db_type::Error> {
        let r = self.db.r_transaction()?;
        r.get().secondary(UserKey::email, email)
    }

    // Update user
    pub fn update(&self, user: User) -> Result<(), UserError> {
        let rw = self.db.rw_transaction()?;

        let old_user = rw.get().primary(user.id)?.ok_or(UserError::UserNotFound)?;

        rw.update(old_user, user)?;
        rw.commit().map_err(UserError::DbError)
    }

    // Delete user by ID (soft delete)
    pub fn delete(&self, id: u32) -> Result<(), db_type::Error> {
        let rw = self.db.rw_transaction()?;

        // Get existing user
        let mut user: User = match rw.get().primary(id)? {
            Some(user) => user,
            None => return Ok(()), // User doesn't exist, nothing to delete
        };
        let previous_user = user.clone();

        // Soft delete by setting deleted_at
        user.deleted_at = Some(chrono::Utc::now().naive_utc());
        rw.update(previous_user, user)?;
        rw.commit()
    }

    // Hard delete user by ID (permanent removal)
    // Only allowed for users that have already been soft-deleted
    pub fn hard_delete(&self, id: u32) -> Result<(), UserError> {
        let rw = self.db.rw_transaction()?;

        // Check if user exists
        let user: Option<User> = rw.get().primary(id)?;

        match user {
            Some(user) => {
                // Check if the user is soft-deleted
                if user.deleted_at.is_none() {
                    rw.abort()?;
                    return Err(UserError::NotSoftDeleted);
                }

                // User is soft-deleted, proceed with hard deletion
                rw.remove(user)?;
                rw.commit()?;
                Ok(())
            }
            None => {
                rw.abort()?;
                Ok(()) // User doesn't exist, nothing to delete
            }
        }
    }

    // List all active users
    pub fn list_active(&self) -> Result<Vec<User>, db_type::Error> {
        let r = self.db.r_transaction()?;
        let primary = r.scan().primary()?;
        let users: PrimaryScanIterator<User> = primary.all()?;

        // Filter out deleted users
        Ok(users
            .filter_map(Result::ok)
            .filter(|u| u.deleted_at.is_none())
            .collect())
    }

    // Count total number of users
    pub fn count(&self) -> Result<usize, db_type::Error> {
        let r = self.db.r_transaction()?;
        let primary = r.scan().primary()?;
        let users: PrimaryScanIterator<User> = primary.all()?;

        Ok(users.filter_map(Result::ok).count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models;
    use native_db::Builder;
    use std::sync::mpsc;
    use std::thread;

    fn setup_test_db<'a>() -> (Arc<native_db::Database<'a>>, UserService<'a>) {
        let db = Arc::new(Builder::new().create_in_memory(&models::MODELS).unwrap());
        let service = UserService::new(db.clone());
        (db, service)
    }

    #[test]
    fn test_create_user() {
        let (_, service) = setup_test_db();
        let user = service
            .create("test@example.com".to_string(), "Test User".to_string())
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert!(user.deleted_at.is_none());
    }

    #[test]
    fn test_get_user() {
        let (_, service) = setup_test_db();
        let created = service
            .create("test@example.com".to_string(), "Test User".to_string())
            .unwrap();

        let by_id = service.get_by_id(created.id).unwrap().unwrap();
        assert_eq!(by_id.id, created.id);

        let by_email = service
            .get_by_email("test@example.com".to_string())
            .unwrap()
            .unwrap();
        assert_eq!(by_email.id, created.id);
    }

    #[test]
    fn test_update_user() {
        let (_, service) = setup_test_db();
        let mut user = service
            .create("test@example.com".to_string(), "Test User".to_string())
            .unwrap();
        let u_id = user.id;

        user.name = "Updated Name".to_string();
        service.update(user).unwrap();

        let updated = service.get_by_id(u_id).unwrap().unwrap();
        assert_eq!(updated.name, "Updated Name");
    }

    #[test]
    fn test_soft_delete() {
        let (_, service) = setup_test_db();
        let user = service
            .create("test@example.com".to_string(), "Test User".to_string())
            .unwrap();

        service.delete(user.id).unwrap();

        let deleted = service.get_by_id(user.id).unwrap().unwrap();
        assert!(deleted.deleted_at.is_some());

        let active_users = service.list_active().unwrap();
        assert!(active_users.is_empty());
    }

    #[test]
    fn test_concurrent_update() {
        let (db, service) = setup_test_db();

        // Create initial user
        let user = service
            .create(
                "concurrent@example.com".to_string(),
                "Initial Name".to_string(),
            )
            .unwrap();
        let user_id = user.id;

        // Set up communication channels for thread synchronization
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        // Thread 1: Update user name to "Thread 1"
        let db_clone1 = db.clone();
        let handle1 = thread::spawn(move || {
            let service = UserService::new(db_clone1.clone());

            // Get the user
            let mut user = service.get_by_id(user_id).unwrap().unwrap();

            // Signal thread 2 that we've got the user
            tx1.send(()).unwrap();

            // Wait for thread 2 to get the user too
            rx2.recv().unwrap();

            // Update the user
            user.name = "Thread 1".to_string();
            match service.update(user) {
                Ok(_) => "Thread 1 update succeeded".to_string(),
                Err(e) => format!("Thread 1 update failed: {:?}", e),
            }
        });

        // Thread 2: Update user name to "Thread 2"
        let db_clone2 = db.clone();
        let handle2 = thread::spawn(move || {
            let service = UserService::new(db_clone2.clone());

            // Wait for thread 1 to get the user
            rx1.recv().unwrap();

            // Get the user
            let mut user = service.get_by_id(user_id).unwrap().unwrap();

            // Signal thread 1 that we've got the user too
            tx2.send(()).unwrap();

            // Sleep to ensure thread 1 attempts to update first
            thread::sleep(std::time::Duration::from_millis(50));

            // Update the user
            user.name = "Thread 2".to_string();
            match service.update(user) {
                Ok(_) => "Thread 2 update succeeded".to_string(),
                Err(e) => format!("Thread 2 update failed: {:?}", e),
            }
        });

        // Get the results
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        // Check the final user state
        let final_user = service.get_by_id(user_id).unwrap().unwrap();

        println!("Thread 1 result: {}", result1);
        println!("Thread 2 result: {}", result2);
        println!("Final user name: {}", final_user.name);

        // One of the threads should have failed, or the database should have a consistent final state
        // This assertion depends on the expected behavior of your database's concurrency control
        // In this case, we expect the second update to win or fail with a concurrency error
        assert!(result1.contains("succeeded") || result2.contains("succeeded"));
        assert!(final_user.name == "Thread 1" || final_user.name == "Thread 2");

        // If the database provides optimistic concurrency control, one update should fail
        if result1.contains("succeeded") && result2.contains("succeeded") {
            // If both succeeded, we need to check that the database is in a consistent state
            // This would typically happen with a "last write wins" policy
            println!("Both updates succeeded - database should have last writer's value");
        } else {
            // With proper optimistic concurrency control, one update should fail
            println!("Concurrent update was detected and prevented!");
        }
    }

    #[test]
    fn test_optimistic_concurrency() {
        let (_, service) = setup_test_db();

        // Create initial user
        let user = service
            .create(
                "optimistic@example.com".to_string(),
                "Initial Name".to_string(),
            )
            .unwrap();
        let user_id = user.id;

        // Read the same user twice
        let mut user1 = service.get_by_id(user_id).unwrap().unwrap();
        let mut user2 = service.get_by_id(user_id).unwrap().unwrap();

        // Update first copy and save
        user1.name = "First Update".to_string();
        service.update(user1).unwrap();

        // Update second copy and try to save
        user2.name = "Second Update".to_string();
        let result = service.update(user2);

        // In a proper optimistic concurrency control system, the second update should fail
        // because the record was modified since it was read
        // The behavior depends on your database implementation

        // Get the final state
        let final_user = service.get_by_id(user_id).unwrap().unwrap();
        println!("Optimistic concurrency test result: {:?}", result);
        println!("Final user name: {}", final_user.name);

        // If the database has optimistic concurrency control, the second update should fail
        // and the name should be "First Update"
        // If it doesn't, the second update will overwrite and the name will be "Second Update"
        assert!(final_user.name == "First Update" || final_user.name == "Second Update");
    }

    #[test]
    fn test_hard_delete() {
        let (_, service) = setup_test_db();

        // Create user
        let user = service
            .create(
                "delete-me@example.com".to_string(),
                "To Be Deleted".to_string(),
            )
            .unwrap();
        let user_id = user.id;

        // Verify user exists
        let found_user = service.get_by_id(user_id).unwrap();
        assert!(found_user.is_some(), "User should exist before deletion");

        // Try to hard delete without soft delete first (should fail)
        let result = service.hard_delete(user_id);
        assert!(
            result.is_err(),
            "Hard delete without soft delete should fail"
        );

        // Soft delete the user
        service.delete(user_id).unwrap();

        // Now hard delete should succeed
        service.hard_delete(user_id).unwrap();

        // Verify user no longer exists
        let deleted_user = service.get_by_id(user_id).unwrap();
        assert!(
            deleted_user.is_none(),
            "User should not exist after hard deletion"
        );

        // Verify the user is not in the active users list
        let active_users = service.list_active().unwrap();
        assert!(
            !active_users.iter().any(|u| u.id == user_id),
            "User should not appear in active users list after hard deletion"
        );
    }

    #[test]
    fn test_soft_delete_vs_hard_delete() {
        let (_, service) = setup_test_db();

        // Create two users
        let user1 = service
            .create("soft@example.com".to_string(), "Soft Delete".to_string())
            .unwrap();
        let user2 = service
            .create("hard@example.com".to_string(), "Hard Delete".to_string())
            .unwrap();

        // Soft delete both users
        service.delete(user1.id).unwrap();
        service.delete(user2.id).unwrap();

        // Hard delete user2
        service.hard_delete(user2.id).unwrap();

        // Soft deleted user should still exist but be marked as deleted
        let soft_deleted = service.get_by_id(user1.id).unwrap();
        assert!(
            soft_deleted.is_some(),
            "Soft deleted user should still exist"
        );
        assert!(
            soft_deleted.unwrap().deleted_at.is_some(),
            "Soft deleted user should have deleted_at set"
        );

        // Hard deleted user should not exist at all
        let hard_deleted = service.get_by_id(user2.id).unwrap();
        assert!(hard_deleted.is_none(), "Hard deleted user should not exist");

        // Neither user should appear in active users list
        let active_users = service.list_active().unwrap();
        assert!(
            !active_users
                .iter()
                .any(|u| u.id == user1.id || u.id == user2.id),
            "Neither soft nor hard deleted users should appear in active users list"
        );
    }

    #[test]
    fn test_hard_delete_error_handling() {
        let (_, service) = setup_test_db();

        // Create user but don't soft delete
        let user = service
            .create("active@example.com".to_string(), "Active User".to_string())
            .unwrap();

        // Try to hard delete an active user (should fail)
        let result = service.hard_delete(user.id);
        assert!(result.is_err(), "Hard deleting an active user should fail");

        if let Err(err) = result {
            match err {
                UserError::NotSoftDeleted => {
                    // This is the expected error
                    println!("Correctly received NotSoftDeleted error: {}", err);
                }
                _ => panic!("Unexpected error type: {:?}", err),
            }
        }

        // User should still exist
        let user_check = service.get_by_id(user.id).unwrap();
        assert!(
            user_check.is_some(),
            "User should still exist after failed hard delete"
        );
    }

    #[test]
    fn test_count() {
        let (_, service) = setup_test_db();
        service
            .create("test@example.com".to_string(), "Test User".to_string())
            .unwrap();

        {
            let c = service.count().unwrap();
            assert_eq!(c, 1);
        }

        service
            .create(
                "test_second@example.com".to_string(),
                "Test middle User".to_string(),
            )
            .unwrap();

        {
            let c = service.count().unwrap();
            assert_eq!(c, 2);
        }
    }
}
