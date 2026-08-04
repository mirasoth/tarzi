//! Google Serper JSON parser for ParserFactory completeness.

use super::base::BaseParser;
use crate::Result;
use crate::search::api::serper::parse_serper_api_response;
use crate::search::types::{SearchEngineType, SearchResult};

pub struct GoogleSerperParser;

impl GoogleSerperParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoogleSerperParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseParser for GoogleSerperParser {
    fn name(&self) -> &str {
        "GoogleSerperParser"
    }

    fn engine_type(&self) -> SearchEngineType {
        SearchEngineType::GoogleSerper
    }

    fn parse(&self, content: &str, limit: usize) -> Result<Vec<SearchResult>> {
        parse_serper_api_response(content, limit)
    }
}
