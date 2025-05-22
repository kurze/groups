use actix_files as fs;
use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use db::user::UserService;
use std::sync::Arc;
use tera::Tera;
mod api;
mod db;

use api::GroupService;
use api::hello::AppStateWithCounter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = db::test_native().unwrap();
    let db_arc: Arc<native_db::Database> = db.into();

    // Set up templating
    let mut tera = match Tera::new("src/templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Template parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec!["html"]);
    let tera_data = web::Data::new(tera);

    // Initialize services
    let counter = web::Data::new(AppStateWithCounter {
        counter: std::sync::Mutex::new(0),
    });
    let user_service = UserService::new(db_arc.clone());
    let group_service = web::Data::new(GroupService::new(db_arc.clone()));

    println!("Number of users: {}", user_service.count().unwrap());
    println!("Server running at http://127.0.0.1:8080/");

    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(group_service.clone())
            .app_data(tera_data.clone())
            // Static files
            .service(fs::Files::new("/static", "src/static").show_files_listing())
            // API Routes
            .service(api::hello_service)
            .configure(api::configure_groups_routes)
            .configure(api::configure_html_routes) // Add the HTML API routes
            // HTML Routes
            .service(web::resource("/").to(index))
            .service(web::resource("/groups").to(groups_page))
            // Default 404 handler
            .default_service(web::route().to(not_found))
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

// Page handlers
async fn index(tmpl: web::Data<Tera>) -> HttpResponse {
    let context = tera::Context::new();
    let rendered = tmpl.render("index.html", &context).unwrap_or_else(|e| {
        eprintln!("Template error: {}", e);
        "Template error".to_string()
    });
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn groups_page(tmpl: web::Data<Tera>) -> HttpResponse {
    let context = tera::Context::new();
    let rendered = tmpl.render("groups.html", &context).unwrap_or_else(|e| {
        eprintln!("Template error: {}", e);
        "Template error".to_string()
    });
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("Page not found")
}
