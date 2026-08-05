//! Search module
//!
//! This module provides functionality for searching the web using different search engines:
//! - Ordered multi-engine failover (`engine = "brave,duckduckgo,bing"`)
//! - Access cascade per engine: API (if credentials present) → plain HTTP → browser (optional)
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
pub use types::{AccessMethod, SearchEngineType, SearchResult, parse_engine_list};
