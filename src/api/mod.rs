pub mod auth;
pub mod groups_api;
pub mod groups_html;
pub mod hello;

// Re-export API modules for easier imports
pub use groups_api::configure_routes as configure_groups_routes;
pub use groups_html::configure_html_routes;
pub use hello::hello_service;


// Re-export password functions for API layer
pub use crate::password::{hash_password, verify_password};
