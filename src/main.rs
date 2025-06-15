use actix_files as fs;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, HttpResponse, HttpServer, cookie::Key, middleware as actix_middleware, web};
use db::group::GroupService;
use db::user::UserService;
use std::env;
use tera::Tera;
mod api;
mod db;
mod middleware;
mod password;

use api::hello::AppStateWithCounter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Check database health
    match db::health_check(&pool).await {
        Ok(true) => println!("Database connection: OK"),
        Ok(false) => {
            eprintln!("Database health check failed");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            std::process::exit(1);
        }
    }

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
    let user_service = web::Data::new(UserService::new(pool.clone()));
    let group_service = web::Data::new(GroupService::new(pool.clone()));

    // Get configuration from environment
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    println!(
        "Number of users: {}",
        user_service.count().await.unwrap_or(0)
    );
    println!("Server running at http://{}:{}/", host, port);

    // Set up logging
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    // Get session secret key from environment or generate one for development
    let secret_key = match env::var("SESSION_SECRET_KEY") {
        Ok(key) => {
            if key.len() >= 64 {
                Key::from(key.as_bytes())
            } else {
                eprintln!(
                    "Warning: SESSION_SECRET_KEY too short (minimum 64 characters), using generated key"
                );
                Key::generate()
            }
        }
        Err(_) => {
            eprintln!(
                "Warning: SESSION_SECRET_KEY not set, using generated key (not suitable for production)"
            );
            Key::generate()
        }
    };

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(user_service.clone())
            .app_data(group_service.clone())
            .app_data(tera_data.clone())
            // Static files
            .service(fs::Files::new("/static", "src/static").show_files_listing())
            // HTML Routes
            .service(web::resource("/").to(index))
            .service(web::resource("/groups").to(groups_page))
            .service(web::resource("/login").to(api::auth::login_page))
            .service(web::resource("/register").to(api::auth::register_page))
            .service(web::resource("/auth/login").route(web::post().to(api::auth::login)))
            .service(web::resource("/logout").to(api::auth::logout))
            .service(web::resource("/auth/register").route(web::post().to(api::auth::register)))
            // Protected routes
            .service(
                web::resource("/groups/new")
                    .wrap(crate::middleware::auth::RequireAuth)
                    .to(new_group_page),
            )
            // API Routes
            .service(api::hello_service)
            .service(
                web::scope("/api")
                    .configure(api::configure_groups_routes)
                    .configure(api::configure_html_routes),
            )
            // Default 404 handler
            .default_service(web::route().to(not_found))
            .wrap(actix_middleware::Logger::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false) // Set to true in production with HTTPS
                    .build(),
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

// Page handlers
async fn index(tmpl: web::Data<Tera>, session: actix_session::Session) -> HttpResponse {
    let mut context = tera::Context::new();

    // Check if user is logged in
    if let Ok(Some(user_email)) = session.get::<String>("user_email") {
        context.insert("user_email", &user_email);
        if let Ok(Some(user_name)) = session.get::<String>("user_name") {
            context.insert("user_name", &user_name);
        }
        context.insert("is_logged_in", &true);
    } else {
        context.insert("is_logged_in", &false);
    }

    let rendered = tmpl.render("index.html", &context).unwrap_or_else(|e| {
        eprintln!("Template error: {}", e);
        "Template error".to_string()
    });
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn groups_page(tmpl: web::Data<Tera>, session: actix_session::Session) -> HttpResponse {
    let mut context = tera::Context::new();

    // Check if user is logged in
    if let Ok(Some(user_email)) = session.get::<String>("user_email") {
        context.insert("user_email", &user_email);
        if let Ok(Some(user_name)) = session.get::<String>("user_name") {
            context.insert("user_name", &user_name);
        }
        context.insert("is_logged_in", &true);
    } else {
        context.insert("is_logged_in", &false);
    }

    let rendered = tmpl.render("groups.html", &context).unwrap_or_else(|e| {
        eprintln!("Template error: {}", e);
        "Template error".to_string()
    });
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn new_group_page(tmpl: web::Data<Tera>, session: actix_session::Session) -> HttpResponse {
    let mut context = tera::Context::new();

    // Check if user is logged in
    if let Ok(Some(user_email)) = session.get::<String>("user_email") {
        context.insert("user_email", &user_email);
        if let Ok(Some(user_name)) = session.get::<String>("user_name") {
            context.insert("user_name", &user_name);
        }
        context.insert("is_logged_in", &true);
    } else {
        context.insert("is_logged_in", &false);
    }

    let rendered = tmpl
        .render("groups_new.html", &context)
        .unwrap_or_else(|e| {
            eprintln!("Template error: {}", e);
            "Template error".to_string()
        });
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().body("Page not found")
}
