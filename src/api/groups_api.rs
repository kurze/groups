use crate::db::group::GroupService;
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use serde::{Deserialize, Serialize};

// Data transfer objects
#[derive(Serialize, Deserialize)]
pub struct GroupResponse {
    pub id: u32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
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
pub async fn get_group(
    path: web::Path<u32>,
    service: web::Data<GroupService<'static>>,
) -> impl Responder {
    let group_id = path.into_inner();

    match service.get_by_id(group_id) {
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
    service: web::Data<GroupService<'static>>,
) -> impl Responder {
    // Validate group name is not empty
    if group_data.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("Group name cannot be empty");
    }

    match service.create(group_data.name.clone()) {
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
    path: web::Path<u32>,
    group_data: web::Json<UpdateGroupRequest>,
    service: web::Data<GroupService<'static>>,
) -> impl Responder {
    // Validate group name is not empty
    if group_data.name.trim().is_empty() {
        return HttpResponse::BadRequest().body("Group name cannot be empty");
    }

    let group_id = path.into_inner();

    // First check if the group exists and is not deleted
    match service.get_by_id(group_id) {
        Ok(Some(mut group)) => {
            if group.deleted_at.is_some() {
                return HttpResponse::NotFound().body("Group not found");
            }

            // Update the group
            group.name = group_data.name.clone();

            match service.update(group) {
                Ok(_) => {
                    // Get the updated group to return
                    match service.get_by_id(group_id) {
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
    path: web::Path<u32>,
    service: web::Data<GroupService<'static>>,
) -> impl Responder {
    let group_id = path.into_inner();

    // First check if the group exists and is not already deleted
    match service.get_by_id(group_id) {
        Ok(Some(group)) => {
            if group.deleted_at.is_some() {
                return HttpResponse::NotFound().body("Group not found or already deleted");
            }

            // Now delete the group
            match service.delete(group_id) {
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
pub async fn list_groups(service: web::Data<GroupService<'static>>) -> impl Responder {
    match service.list_active() {
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
    service: web::Data<GroupService<'static>>,
) -> impl Responder {
    if let Some(name) = &query.name {
        match service.find_by_name(name.clone()) {
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
        match service.list_active() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::Group;
    use actix_web::{App, test, web};
    use native_db::Builder;
    use std::cell::RefCell;
    use std::sync::Arc;

    // Modified MockGroupService to properly use the should_fail field
    #[allow(dead_code)] // Suppress warnings about unused methods
    struct MockGroupService {
        should_fail: RefCell<bool>,
    }

    #[allow(dead_code)] // Suppress warnings about unused methods
    impl MockGroupService {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail: RefCell::new(should_fail),
            }
        }

        fn get_by_id(&self, id: u32) -> Result<Option<Group>, String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            // Return a mock group
            Ok(Some(Group {
                id,
                name: format!("Mock Group {}", id),
                created_at: chrono::Utc::now().naive_utc(),
                deleted_at: None,
            }))
        }

        fn list_active(&self) -> Result<Vec<Group>, String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            Ok(vec![])
        }

        fn find_by_name(&self, _name: String) -> Result<Vec<Group>, String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            Ok(vec![])
        }

        fn create(&self, _name: String) -> Result<Group, String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            Ok(Group {
                id: 42,
                name: "Mock Created Group".to_string(),
                created_at: chrono::Utc::now().naive_utc(),
                deleted_at: None,
            })
        }

        fn update(&self, _group: Group) -> Result<(), String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            Ok(())
        }

        fn delete(&self, _id: u32) -> Result<(), String> {
            if *self.should_fail.borrow() {
                return Err("Mock database error".to_string());
            }

            Ok(())
        }
    }

    #[actix_web::test]
    async fn test_get_group() {
        // Create in-memory database
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        // Create test data
        let group = service.create("Test Group 1".to_string()).unwrap();
        let deleted_group = service.create("To Be Deleted".to_string()).unwrap();
        service.delete(deleted_group.id).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        // Test valid group
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", group.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body: GroupResponse = test::read_body_json(resp).await;
        assert_eq!(body.id, group.id);
        assert_eq!(body.name, "Test Group 1");

        // Test non-existent group
        let req = test::TestRequest::get().uri("/groups/999").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test soft-deleted group
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", deleted_group.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_list_groups() {
        // Create in-memory database
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        // Create test data
        service.create("Test Group 1".to_string()).unwrap();
        service.create("Test Group 2".to_string()).unwrap();
        let deleted_group = service.create("To Be Deleted".to_string()).unwrap();
        service.delete(deleted_group.id).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::get().uri("/groups").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let groups: Vec<GroupResponse> = test::read_body_json(resp).await;

        // Should return 2 active groups (not the soft-deleted one)
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().any(|g| g.name == "Test Group 1"));
        assert!(groups.iter().any(|g| g.name == "Test Group 2"));
        assert!(!groups.iter().any(|g| g.name == "To Be Deleted"));
    }

    #[actix_web::test]
    async fn test_search_groups() {
        // Create in-memory database
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        // Create test data
        service.create("Test Group 1".to_string()).unwrap();
        service.create("Test Group 2".to_string()).unwrap();
        let deleted_group = service.create("To Be Deleted".to_string()).unwrap();
        service.delete(deleted_group.id).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        // Search by exact name
        let req = test::TestRequest::get()
            .uri("/groups/search?name=Test%20Group%201")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let groups: Vec<GroupResponse> = test::read_body_json(resp).await;
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Test Group 1");

        // Search by non-existent name
        let req = test::TestRequest::get()
            .uri("/groups/search?name=NonExistent")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let groups: Vec<GroupResponse> = test::read_body_json(resp).await;
        assert_eq!(groups.len(), 0);

        // Search for soft-deleted group
        let req = test::TestRequest::get()
            .uri("/groups/search?name=To%20Be%20Deleted")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let groups: Vec<GroupResponse> = test::read_body_json(resp).await;
        assert_eq!(groups.len(), 0);

        // Empty search should return all groups (same as list)
        let req = test::TestRequest::get().uri("/groups/search").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let groups: Vec<GroupResponse> = test::read_body_json(resp).await;
        assert_eq!(groups.len(), 2);
    }

    #[actix_web::test]
    async fn test_create_group() {
        // Create in-memory database
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        // Test creating a new group
        let request_body = CreateGroupRequest {
            name: "New Test Group".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/groups")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should return 201 Created
        assert_eq!(resp.status(), 201);

        let created_group: GroupResponse = test::read_body_json(resp).await;
        assert_eq!(created_group.name, "New Test Group");

        // Verify the group exists by retrieving it
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", created_group.id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test with empty name
        let request_body = CreateGroupRequest {
            name: "".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/groups")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400); // Now we expect a 400 Bad Request for empty names

        // Test with whitespace-only name
        let request_body = CreateGroupRequest {
            name: "   ".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/groups")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400); // Should also reject whitespace-only names
    }

    #[actix_web::test]
    async fn test_delete_group() {
        // Create in-memory database
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        // Create test data
        let group = service.create("Test Group to Delete".to_string()).unwrap();
        let group_id = group.id; // Store ID for later use

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        // Verify group exists before deletion
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", group_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Delete the group
        let req = test::TestRequest::delete()
            .uri(&format!("/groups/{}", group_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should return 204 No Content
        assert_eq!(resp.status(), 204);

        // Verify group is no longer accessible via API (marked as deleted)
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", group_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test deleting non-existent group
        let req = test::TestRequest::delete().uri("/groups/999").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test deleting already deleted group
        let req = test::TestRequest::delete()
            .uri(&format!("/groups/{}", group_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_update_group() {
        // Create in-memory database with shared DB
        let db = Arc::new(
            Builder::new()
                .create_in_memory(&crate::db::models::MODELS)
                .unwrap(),
        );
        let service = GroupService::new(db);

        // Create test data
        let group = service.create("Original Name".to_string()).unwrap();
        let group_id = group.id; // Store ID for later use
        let deleted_group = service.create("Deleted Group".to_string()).unwrap();
        let deleted_group_id = deleted_group.id; // Store ID for later use
        service.delete(deleted_group_id).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .configure(configure_routes),
        )
        .await;

        // Test updating an active group
        let update_data = UpdateGroupRequest {
            name: "Updated Name".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/groups/{}", group_id))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should return 200 OK
        assert_eq!(resp.status(), 200);

        // Verify the name was updated
        let updated_group: GroupResponse = test::read_body_json(resp).await;
        assert_eq!(updated_group.id, group_id);
        assert_eq!(updated_group.name, "Updated Name");

        // Also verify via direct get
        let req = test::TestRequest::get()
            .uri(&format!("/groups/{}", group_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let fetched_group: GroupResponse = test::read_body_json(resp).await;
        assert_eq!(fetched_group.name, "Updated Name");

        // Test updating non-existent group
        let req = test::TestRequest::put()
            .uri("/groups/999")
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test updating a deleted group
        let req = test::TestRequest::put()
            .uri(&format!("/groups/{}", deleted_group_id))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        // Test updating with empty name
        let update_data = UpdateGroupRequest {
            name: "".to_string(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/groups/{}", group_id))
            .set_json(&update_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400); // Now returns 400 Bad Request for empty names
    }

    #[actix_web::test]
    async fn test_api_error_handling() {
        // Create a mock service that will fail
        let mock_service = web::Data::new(MockGroupService::new(true));

        let app = test::init_service(
            App::new()
                .app_data(mock_service.clone())
                .configure(configure_routes), // Use the configure_routes function to set up all routes
        )
        .await;

        // Test get_group error handling
        let req = test::TestRequest::get().uri("/groups/1").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        // Test list_groups error handling
        let req = test::TestRequest::get().uri("/groups").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        // Test search_groups error handling
        let req = test::TestRequest::get()
            .uri("/groups/search?name=test")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        // Test create_group error handling
        let request_body = CreateGroupRequest {
            name: "Test Group".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/groups")
            .set_json(&request_body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        // Test update_group error handling
        let update_data = UpdateGroupRequest {
            name: "Updated Name".to_string(),
        };

        let req = test::TestRequest::put()
            .uri("/groups/1")
            .set_json(&update_data)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        // Test delete_group error handling
        let req = test::TestRequest::delete().uri("/groups/1").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }
}
