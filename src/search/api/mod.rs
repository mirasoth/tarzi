//! Official / third-party search API clients (REST over HTTP).

pub mod brave;
pub mod serper;

pub use brave::search_brave_api;
pub use serper::search_serper_api;
