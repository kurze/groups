use crate::db::group::GroupService;
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Data transfer objects
#[derive(Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub name: String,
}

// Get a specific group by ID
#[get("/groups/{id}")]
pub async fn get_group(path: web::Path<i32>, service: web::Data<GroupService>) -> impl Responder {
    let group_id = path.into_inner();

    match service.get_by_id(group_id).await {
        Ok(Some(group)) => {
            if group.deleted_at.is_some() {
                return HttpResponse::NotFound().body("Group not found");
            }

            let response = GroupResponse {
                id: group.id,
                name: group.name,
                created_at: group.created_at,
            };

            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().body("Group not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// Create a new group
#[post("/groups")]
pub async fn create_group(
    group_data: web::Json<CreateGroupRequest>,
    service: web::Data<GroupService>,
) -> impl Responder {
    // Validate group name is not empty
    if group_data.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("Group name cannot be empty");
    }

    match service.create(group_data.name.clone()).await {
        Ok(group) => {
            let response = GroupResponse {
                id: group.id,
                name: group.name,
                created_at: group.created_at,
            };

            HttpResponse::Created().json(response)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// Update an existing group
#[put("/groups/{id}")]
pub async fn update_group(
    path: web::Path<i32>,
    group_data: web::Json<UpdateGroupRequest>,
    service: web::Data<GroupService>,
) -> impl Responder {
    // Validate group name is not empty
    if group_data.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("Group name cannot be empty");
    }

    let group_id = path.into_inner();

    // First check if the group exists and is not deleted
    match service.get_by_id(group_id).await {
        Ok(Some(mut group)) => {
            if group.deleted_at.is_some() {
                return HttpResponse::NotFound().body("Group not found");
            }

            // Update the group
            group.name = group_data.name.clone();

            match service.update(group).await {
                Ok(_) => {
                    // Get the updated group to return
                    match service.get_by_id(group_id).await {
                        Ok(Some(updated_group)) => {
                            let response = GroupResponse {
                                id: updated_group.id,
                                name: updated_group.name,
                                created_at: updated_group.created_at,
                            };
                            HttpResponse::Ok().json(response)
                        }
                        _ => HttpResponse::InternalServerError()
                            .body("Failed to retrieve updated group"),
                    }
                }
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Database error: {}", e))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Group not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// Delete a group (soft delete)
#[delete("/groups/{id}")]
pub async fn delete_group(
    path: web::Path<i32>,
    service: web::Data<GroupService>,
) -> impl Responder {
    let group_id = path.into_inner();

    // First check if the group exists and is not already deleted
    match service.get_by_id(group_id).await {
        Ok(Some(group)) => {
            if group.deleted_at.is_some() {
                return HttpResponse::NotFound().body("Group not found or already deleted");
            }

            // Now delete the group
            match service.delete(group_id).await {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => {
                    HttpResponse::InternalServerError().body(format!("Database error: {}", e))
                }
            }
        }
        Ok(None) => HttpResponse::NotFound().body("Group not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// List all groups
#[get("/groups")]
pub async fn list_groups(service: web::Data<GroupService>) -> impl Responder {
    match service.list_active().await {
        Ok(groups) => {
            let responses: Vec<GroupResponse> = groups
                .into_iter()
                .map(|group| GroupResponse {
                    id: group.id,
                    name: group.name,
                    created_at: group.created_at,
                })
                .collect();

            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

// Search groups by name
#[get("/groups/search")]
pub async fn search_groups(
    query: web::Query<SearchQuery>,
    service: web::Data<GroupService>,
) -> impl Responder {
    if let Some(name) = &query.name {
        match service.find_by_name(name.clone()).await {
            Ok(groups) => {
                if groups.is_empty() {
                    return HttpResponse::Ok().json(Vec::<GroupResponse>::new());
                }

                // Filter out soft-deleted groups
                let active_groups: Vec<GroupResponse> = groups
                    .into_iter()
                    .filter(|g| g.deleted_at.is_none())
                    .map(|group| GroupResponse {
                        id: group.id,
                        name: group.name,
                        created_at: group.created_at,
                    })
                    .collect();

                HttpResponse::Ok().json(active_groups)
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
        }
    } else {
        // If no search parameters provided, return all active groups
        match service.list_active().await {
            Ok(groups) => {
                let responses: Vec<GroupResponse> = groups
                    .into_iter()
                    .map(|group| GroupResponse {
                        id: group.id,
                        name: group.name,
                        created_at: group.created_at,
                    })
                    .collect();

                HttpResponse::Ok().json(responses)
            }
            Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
        }
    }
}

// Configure services
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(search_groups) // Put search_groups first so it takes precedence over get_group for patterns
        .service(get_group)
        .service(list_groups)
        .service(create_group)
        .service(delete_group)
        .service(update_group);
}

// Tests disabled for now - need to be rewritten to use test database containers with PostgreSQL
// #[cfg(test)]
// mod tests {
//     use super::*;
//     TODO: Implement PostgreSQL integration tests using testcontainers
// }
