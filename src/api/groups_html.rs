use crate::db::group::GroupService;
use actix_web::{HttpResponse, Responder, get, post, web};
use serde::Deserialize;

// Data structure for form submissions
#[derive(Deserialize)]
pub struct GroupForm {
    pub name: String,
}

// Get all groups as HTML
#[get("/api/groups")]
pub async fn get_groups_html(service: web::Data<GroupService>) -> impl Responder {
    match service.list_active().await {
        Ok(groups) if !groups.is_empty() => {
            let mut html = String::from("<div class=\"group-list-items\">");

            for group in groups {
                html.push_str(&format!(
                    r#"<div class="group-item" id="group-{}" hx-target="this" hx-swap="outerHTML">
                        <h3>{}</h3>
                        <p>Created: {}</p>
                        <div class="group-actions">
                            <button hx-delete="/api/groups/{}">Delete</button>
                        </div>
                    </div>"#,
                    group.id, group.name, group.created_at, group.id
                ));
            }

            html.push_str("</div>");
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Ok(_) => {
            // Empty list
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body("<p>No groups found. Create one below.</p>")
        }
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body(format!("<p>Error loading groups: {}</p>", e)),
    }
}

// Create a new group and return the HTML fragment
#[post("/api/groups")]
pub async fn create_group_html(
    form: web::Form<GroupForm>,
    service: web::Data<GroupService>,
) -> impl Responder {
    // Validate group name is not empty
    if form.name.trim().is_empty() {
        return HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body("<div class=\"error\">Group name cannot be empty</div>");
    }

    match service.create(form.name.clone()).await {
        Ok(group) => {
            // Return just the HTML for the new group
            let html = format!(
                r#"<div class="group-item" id="group-{}" hx-target="this" hx-swap="outerHTML">
                    <h3>{}</h3>
                    <p>Created: {}</p>
                    <div class="group-actions">
                        <button hx-delete="/api/groups/{}">Delete</button>
                    </div>
                </div>"#,
                group.id, group.name, group.created_at, group.id
            );

            HttpResponse::Created()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        Err(e) => HttpResponse::InternalServerError()
            .content_type("text/html; charset=utf-8")
            .body(format!("<div class=\"error\">Database error: {}</div>", e)),
    }
}

// Configure routes for HTML API endpoints
pub fn configure_html_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_groups_html).service(create_group_html);
}
