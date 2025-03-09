pub mod groups_api;
pub mod hello;

// Re-export API modules for easier imports
pub use groups_api::configure_routes as configure_groups_routes;
pub use hello::hello_service;

// Re-export group service for API layer to use
pub use crate::db::group::GroupService;
