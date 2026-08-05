//! Google AI JSON parser for ParserFactory completeness.

use super::base::BaseParser;
use crate::Result;
use crate::search::api::googleai::parse_googleai_api_response;
use crate::search::types::{SearchEngineType, SearchResult};

pub struct GoogleAiParser;

impl GoogleAiParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoogleAiParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseParser for GoogleAiParser {
    fn name(&self) -> &str {
        "GoogleAiParser"
    }

    fn engine_type(&self) -> SearchEngineType {
        SearchEngineType::GoogleAi
    }

    fn parse(&self, content: &str, limit: usize) -> Result<Vec<SearchResult>> {
        parse_googleai_api_response(content, limit)
    }
}
