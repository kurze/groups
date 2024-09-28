use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use diesel::{prelude::*};
use dotenvy::dotenv;
use std::sync::Mutex;
use std::{env};

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
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    let mut db = establish_connection();
    if let Err(e) = diesel::sql_query("SELECT 1").execute(&mut db) {
        eprintln!("Error checking database connection: {}", e);
    } else {
        println!("Database connection established successfully.");
    }

    HttpServer::new(move || App::new().app_data(counter.clone()).service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
