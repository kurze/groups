use actix_web::{HttpResponse, Responder, get, web};
use std::sync::Mutex;

// App state with counter moved from main.rs
pub struct AppStateWithCounter {
    pub counter: Mutex<i32>,
}

// Hello endpoint handler
#[get("/api/hello")]
pub async fn hello_service(data: web::Data<AppStateWithCounter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    // Return HTML for HTMZ instead of plain text
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<div>Request number: <strong>{}</strong></div>",
            counter
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};

    #[actix_web::test]
    async fn test_hello_service() {
        // Create app state with counter
        let app_state = web::Data::new(AppStateWithCounter {
            counter: Mutex::new(0),
        });

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .service(hello_service),
        )
        .await;

        // First request
        let req = test::TestRequest::get().uri("/api/hello").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert!(String::from_utf8_lossy(&body).contains("Request number: <strong>1</strong>"));

        // Second request - counter should increment
        let req = test::TestRequest::get().uri("/api/hello").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        assert!(String::from_utf8_lossy(&body).contains("Request number: <strong>2</strong>"));

        // Verify counter state directly
        let counter = *app_state.counter.lock().unwrap();
        assert_eq!(counter, 2);
    }
}
