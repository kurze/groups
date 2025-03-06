use actix_web::{HttpResponse, Responder, get, web};
use std::sync::Mutex;

// App state with counter moved from main.rs
pub struct AppStateWithCounter {
    pub counter: Mutex<i32>,
}

// Hello endpoint handler
#[get("/")]
pub async fn hello_service(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    HttpResponse::Ok().body(format!("Request number: {counter}"))
}
