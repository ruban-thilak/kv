//! Server module - handles networking and connection management

mod handler;
pub mod server;

pub use handler::handle_request;
pub use server::Server;
