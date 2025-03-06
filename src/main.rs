use actix_web::{App, HttpServer, web};
use db::user::UserService;
mod api;
mod db;

use api::hello::AppStateWithCounter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = db::test_native().unwrap();
    let counter = web::Data::new(AppStateWithCounter {
        counter: std::sync::Mutex::new(0),
    });

    let user_service = UserService::new(db.into());
    println!("Number of users: {}", user_service.count().unwrap());

    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(move || {
        App::new()
            .app_data(counter.clone())
            .service(api::hello_service)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
