//! SearxNG JSON parser for ParserFactory completeness.

use super::base::BaseParser;
use crate::Result;
use crate::search::api::searxng::parse_searxng_api_response;
use crate::search::types::{SearchEngineType, SearchResult};

pub struct SearxNGParser;

impl SearxNGParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SearxNGParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseParser for SearxNGParser {
    fn name(&self) -> &str {
        "SearxNGParser"
    }

    fn engine_type(&self) -> SearchEngineType {
        SearchEngineType::SearxNG
    }

    fn parse(&self, content: &str, limit: usize) -> Result<Vec<SearchResult>> {
        parse_searxng_api_response(content, limit)
    }
}
