//! Official / third-party search API clients (REST over HTTP).

pub mod brave;
pub mod googleai;
pub mod searxng;
pub mod serper;
pub mod tavily;

pub use brave::search_brave_api;
pub use googleai::search_googleai_api;
pub use searxng::search_searxng_api;
pub use serper::search_serper_api;
pub use tavily::search_tavily_api;
