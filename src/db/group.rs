use crate::db::connection::{DatabaseError, DbPool};
use crate::db::models::Group;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum GroupError {
    #[error("Group must be soft-deleted before hard deletion")]
    NotSoftDeleted,
    #[error("Group not found")]
    GroupNotFound,
    #[error("Database error: {0}")]
    DbError(#[from] DatabaseError),
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),
}

pub struct GroupService {
    pool: DbPool,
}

#[allow(dead_code)]
impl GroupService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // Create a new group
    pub async fn create(&self, name: String) -> Result<Group, GroupError> {
        let group = sqlx::query_as::<_, Group>("INSERT INTO groups (name) VALUES ($1) RETURNING *")
            .bind(&name)
            .fetch_one(&self.pool)
            .await?;

        Ok(group)
    }

    // Read group by ID
    pub async fn get_by_id(&self, id: i32) -> Result<Option<Group>, GroupError> {
        let group = sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(group)
    }

    // Read group by name
    pub async fn find_by_name(&self, name: String) -> Result<Vec<Group>, GroupError> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE name = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(&name)
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    // Update group
    pub async fn update(&self, group: Group) -> Result<(), GroupError> {
        let affected_rows =
            sqlx::query("UPDATE groups SET name = $1 WHERE id = $2 AND deleted_at IS NULL")
                .bind(&group.name)
                .bind(group.id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected_rows == 0 {
            return Err(GroupError::GroupNotFound);
        }

        Ok(())
    }

    // Delete group by ID (soft delete)
    pub async fn delete(&self, id: i32) -> Result<(), GroupError> {
        sqlx::query("UPDATE groups SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Hard delete group by ID (permanent removal)
    // Only allowed for groups that have already been soft-deleted
    pub async fn hard_delete(&self, id: i32) -> Result<(), GroupError> {
        // First check if group exists and is soft-deleted
        let group = self.get_by_id(id).await?;

        match group {
            Some(group) => {
                // Check if the group is soft-deleted
                if group.deleted_at.is_none() {
                    return Err(GroupError::NotSoftDeleted);
                }

                // Group is soft-deleted, proceed with hard deletion
                sqlx::query("DELETE FROM groups WHERE id = $1")
                    .bind(id)
                    .execute(&self.pool)
                    .await?;

                Ok(())
            }
            None => Ok(()), // Group doesn't exist, nothing to delete
        }
    }

    // List all active groups
    pub async fn list_active(&self) -> Result<Vec<Group>, GroupError> {
        let groups = sqlx::query_as::<_, Group>(
            "SELECT * FROM groups WHERE deleted_at IS NULL ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    // Count total number of groups
    pub async fn count(&self) -> Result<i64, GroupError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM groups")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}

// Tests will be rewritten for PostgreSQL in a separate module
