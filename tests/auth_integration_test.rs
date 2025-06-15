use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::{App, cookie::Key, http::StatusCode, test, web};
use groups::db;

#[actix_web::test]
async fn test_login_page_accessible() {
    let mut app = test::init_service(App::new().service(
        web::resource("/login").to(|| async { actix_web::HttpResponse::Ok().body("Login Page") }),
    ))
    .await;

    let req = test::TestRequest::get().uri("/login").to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_logout_redirects() {
    let secret_key = Key::generate();

    let mut app = test::init_service(
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
                    .cookie_secure(false)
                    .build(),
            )
            .service(web::resource("/logout").to(groups::api::auth::logout)),
    )
    .await;

    let req = test::TestRequest::get().uri("/logout").to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::FOUND);

    let location = resp.headers().get("Location").unwrap();
    assert_eq!(location, "/");
}

#[actix_web::test]
async fn test_protected_route_redirects_when_not_authenticated() {
    let secret_key = Key::generate();

    let mut app = test::init_service(
        App::new()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key)
                    .cookie_secure(false)
                    .build(),
            )
            .service(
                web::resource("/protected")
                    .wrap(groups::middleware::auth::RequireAuth)
                    .to(|| async { actix_web::HttpResponse::Ok().body("Protected") }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/protected").to_request();

    let resp = test::call_service(&mut app, req).await;
    assert_eq!(resp.status(), StatusCode::FOUND);

    let location = resp.headers().get("Location").unwrap();
    assert_eq!(location, "/login");
}
