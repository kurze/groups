use actix_web::{App, HttpServer, web};
use db::user::UserService;
use std::sync::Arc;
mod api;
mod db;

use api::GroupService;
use api::hello::AppStateWithCounter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = db::test_native().unwrap();
    let db_arc: Arc<native_db::Database> = db.into();

    // Initialize services
    let counter = web::Data::new(AppStateWithCounter {
        counter: std::sync::Mutex::new(0),
    });
    let user_service = UserService::new(db_arc.clone());
    let group_service = web::Data::new(GroupService::new(db_arc.clone()));

    println!("Number of users: {}", user_service.count().unwrap());

    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .app_data(group_service.clone())
            .service(api::hello_service)
            .configure(api::configure_groups_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
