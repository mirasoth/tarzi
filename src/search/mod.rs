//! Search module
//!
//! This module provides functionality for searching the web using different search engines:
//! - API query (Brave, Google Serper) when an API key is available
//! - Plain HTTP webquery to public search URLs
//! - Headless browser fallback for JS-heavy SERPs
//! - Extensible parser system for extracting search results from HTML / JSON

pub mod access;
pub mod api;
pub mod engine;
pub mod parser;
pub mod providers;
pub mod types;

// Re-export main types and functions
pub use access::{has_api_credentials, resolve_access, resolve_api_key, resolve_base_url};
pub use engine::SearchEngine;
pub use parser::ParserFactory;
pub use types::{AccessMethod, SearchEngineType, SearchMode, SearchResult};
