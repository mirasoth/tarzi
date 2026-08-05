//! Parser module for extracting search results from different content types
//!
//! This module provides parsers for extracting search results from HTML content
//! and other formats returned by search engines.

pub mod baidu;
pub mod base;
pub mod bing;
pub mod brave;
pub mod duckduckgo;
pub mod google;
pub mod google_serper;
pub mod googleai;
pub mod searxng;
pub mod sogou_weixin;
pub mod tavily;

use crate::search::types::SearchEngineType;

// Re-export parser types
pub use baidu::BaiduParser;
pub use base::BaseParser;
pub use bing::BingParser;
pub use brave::BraveParser;
pub use duckduckgo::DuckDuckGoParser;
pub use google::GoogleParser;
pub use google_serper::GoogleSerperParser;
pub use googleai::GoogleAiParser;
pub use searxng::SearxNGParser;
pub use sogou_weixin::SogouWeixinParser;
pub use tavily::TavilyParser;

/// Factory for creating parsers based on search engine type
pub struct ParserFactory;

impl ParserFactory {
    pub fn new() -> Self {
        Self
    }

    /// Get a parser for the given search engine type
    pub fn get_parser(&self, engine_type: &SearchEngineType) -> Box<dyn BaseParser> {
        match engine_type {
            // Web query parsers (HTML-based)
            SearchEngineType::Bing => Box::new(BingParser::new()),
            SearchEngineType::DuckDuckGo => Box::new(DuckDuckGoParser::new()),
            SearchEngineType::Google => Box::new(GoogleParser::new()),
            SearchEngineType::BraveSearch => Box::new(BraveParser::new()),
            SearchEngineType::Baidu => Box::new(BaiduParser::new()),
            SearchEngineType::SougouWeixin => Box::new(SogouWeixinParser::new()),
            // API JSON parsers
            SearchEngineType::GoogleSerper => Box::new(GoogleSerperParser::new()),
            SearchEngineType::Tavily => Box::new(TavilyParser::new()),
            SearchEngineType::GoogleAi => Box::new(GoogleAiParser::new()),
            SearchEngineType::SearxNG => Box::new(SearxNGParser::new()),
        }
    }
}

impl Default for ParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_factory() {
        let factory = ParserFactory::new();

        let expected = [
            (SearchEngineType::Bing, "BingParser"),
            (SearchEngineType::DuckDuckGo, "DuckDuckGoParser"),
            (SearchEngineType::Google, "GoogleParser"),
            (SearchEngineType::GoogleSerper, "GoogleSerperParser"),
            (SearchEngineType::BraveSearch, "BraveParser"),
            (SearchEngineType::Baidu, "BaiduParser"),
            (SearchEngineType::SougouWeixin, "SogouWeixinParser"),
            (SearchEngineType::Tavily, "TavilyParser"),
            (SearchEngineType::GoogleAi, "GoogleAiParser"),
            (SearchEngineType::SearxNG, "SearxNGParser"),
        ];

        for (engine_type, name) in expected {
            let parser = factory.get_parser(&engine_type);
            assert_eq!(parser.name(), name);
            assert!(
                parser.supports(&engine_type),
                "{name} should support {engine_type:?}"
            );
        }
    }

    #[test]
    fn test_parser_support() {
        let factory = ParserFactory::new();

        let parsers = vec![
            ("BingParser", factory.get_parser(&SearchEngineType::Bing)),
            (
                "DuckDuckGoParser",
                factory.get_parser(&SearchEngineType::DuckDuckGo),
            ),
            (
                "GoogleParser",
                factory.get_parser(&SearchEngineType::Google),
            ),
            (
                "GoogleSerperParser",
                factory.get_parser(&SearchEngineType::GoogleSerper),
            ),
            (
                "BraveParser",
                factory.get_parser(&SearchEngineType::BraveSearch),
            ),
            ("BaiduParser", factory.get_parser(&SearchEngineType::Baidu)),
            (
                "SogouWeixinParser",
                factory.get_parser(&SearchEngineType::SougouWeixin),
            ),
        ];

        for (name, parser) in parsers {
            assert!(
                parser.supports(&SearchEngineType::Bing)
                    || parser.supports(&SearchEngineType::DuckDuckGo)
                    || parser.supports(&SearchEngineType::Google)
                    || parser.supports(&SearchEngineType::GoogleSerper)
                    || parser.supports(&SearchEngineType::BraveSearch)
                    || parser.supports(&SearchEngineType::Baidu)
                    || parser.supports(&SearchEngineType::SougouWeixin)
                    || parser.supports(&SearchEngineType::Tavily)
                    || parser.supports(&SearchEngineType::GoogleAi)
                    || parser.supports(&SearchEngineType::SearxNG),
                "Parser {name} should support at least one engine type"
            );
        }
    }

    #[test]
    fn test_all_parsers_with_different_limits() {
        let factory = ParserFactory::new();
        let html = "<html><body>Test content</body></html>";

        let parsers = vec![
            ("BingParser", factory.get_parser(&SearchEngineType::Bing)),
            (
                "GoogleParser",
                factory.get_parser(&SearchEngineType::Google),
            ),
            (
                "DuckDuckGoParser",
                factory.get_parser(&SearchEngineType::DuckDuckGo),
            ),
            (
                "BraveParser",
                factory.get_parser(&SearchEngineType::BraveSearch),
            ),
            ("BaiduParser", factory.get_parser(&SearchEngineType::Baidu)),
            (
                "SogouWeixinParser",
                factory.get_parser(&SearchEngineType::SougouWeixin),
            ),
        ];

        for (name, parser) in parsers {
            assert_eq!(parser.name(), name);

            // Test with different limits
            for limit in [1, 5, 10] {
                let results = parser.parse(html, limit).unwrap();
                assert!(results.len() <= limit);
                assert!(results.len() <= 10); // All our mock parsers limit to 10

                // Verify ranking is correct
                for (i, result) in results.iter().enumerate() {
                    assert_eq!(result.rank, i + 1);
                }
            }
        }
    }
}
