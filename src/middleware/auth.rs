use actix_session::SessionExt;
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use chrono::{DateTime, Duration, Utc};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

pub struct RequireAuth;

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for RequireAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Extract session from request
            let session = req.get_session();

            // Check if user is logged in
            if let Ok(Some(_user_id)) = session.get::<u32>("user_id") {
                // Check session timeouts
                let now = Utc::now();

                // Check idle timeout (30 minutes)
                if let Ok(Some(last_activity)) = session.get::<DateTime<Utc>>("last_activity") {
                    let idle_duration = now.signed_duration_since(last_activity);
                    if idle_duration > Duration::minutes(30) {
                        // Session expired due to inactivity
                        session.purge();
                        let response = HttpResponse::Found()
                            .append_header(("Location", "/login?reason=timeout"))
                            .finish();
                        return Ok(req.into_response(response));
                    }
                }

                // Check absolute timeout (12 hours)
                if let Ok(Some(created_at)) = session.get::<DateTime<Utc>>("created_at") {
                    let session_duration = now.signed_duration_since(created_at);
                    if session_duration > Duration::hours(12) {
                        // Session expired due to age
                        session.purge();
                        let response = HttpResponse::Found()
                            .append_header(("Location", "/login?reason=expired"))
                            .finish();
                        return Ok(req.into_response(response));
                    }
                }

                // Update last activity timestamp
                if let Err(e) = session.insert("last_activity", now) {
                    eprintln!("Failed to update session activity: {}", e);
                }

                // User is authenticated and session valid, proceed with request
                service.call(req).await
            } else {
                // User is not authenticated, redirect to login
                let response = HttpResponse::Found()
                    .append_header(("Location", "/login"))
                    .finish();

                Ok(req.into_response(response))
            }
        })
    }
}
