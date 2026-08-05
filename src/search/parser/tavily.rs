//! Tavily JSON parser for ParserFactory completeness.

use super::base::BaseParser;
use crate::Result;
use crate::search::api::tavily::parse_tavily_api_response;
use crate::search::types::{SearchEngineType, SearchResult};

pub struct TavilyParser;

impl TavilyParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TavilyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseParser for TavilyParser {
    fn name(&self) -> &str {
        "TavilyParser"
    }

    fn engine_type(&self) -> SearchEngineType {
        SearchEngineType::Tavily
    }

    fn parse(&self, content: &str, limit: usize) -> Result<Vec<SearchResult>> {
        parse_tavily_api_response(content, limit)
    }
}
