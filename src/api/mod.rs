pub mod hello;
pub mod groups_api;

// Re-export API modules for easier imports
pub use hello::hello_service;
pub use groups_api::configure_routes as configure_groups_routes;

// Re-export group service for API layer to use
pub use crate::db::group::GroupService;
