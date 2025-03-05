use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use db::user::UserService;
use std::sync::Mutex;
mod db;

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

#[get("/")]
async fn hello(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Request number: {counter}"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = db::test_native().unwrap();
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let user_service = UserService::new(db.into());
    println!("Number of users: {}", user_service.count().unwrap());

    // std::env::set_var("RUST_LOG", "debug");
    // std::env::set_var("RUST_BACKTRACE", "1");

    HttpServer::new(move || App::new().app_data(counter.clone()).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
