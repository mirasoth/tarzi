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
pub mod sogou_weixin;

use crate::search::types::SearchEngineType;

// Re-export parser types
pub use baidu::BaiduParser;
pub use base::BaseParser;
pub use bing::BingParser;
pub use brave::BraveParser;
pub use duckduckgo::DuckDuckGoParser;
pub use google::GoogleParser;
pub use google_serper::GoogleSerperParser;
pub use sogou_weixin::SogouWeixinParser;

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
            SearchEngineType::GoogleSerper => Box::new(GoogleSerperParser::new()),
            SearchEngineType::BraveSearch => Box::new(BraveParser::new()),
            SearchEngineType::Baidu => Box::new(BaiduParser::new()),
            SearchEngineType::SougouWeixin => Box::new(SogouWeixinParser::new()),
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

        // Test that we get the correct parser for each engine type
        let bing_parser = factory.get_parser(&SearchEngineType::Bing);
        assert_eq!(bing_parser.name(), "BingParser");

        let duckduckgo_parser = factory.get_parser(&SearchEngineType::DuckDuckGo);
        assert_eq!(duckduckgo_parser.name(), "DuckDuckGoParser");

        let google_parser = factory.get_parser(&SearchEngineType::Google);
        assert_eq!(google_parser.name(), "GoogleParser");

        let brave_parser = factory.get_parser(&SearchEngineType::BraveSearch);
        assert_eq!(brave_parser.name(), "BraveParser");

        let baidu_parser = factory.get_parser(&SearchEngineType::Baidu);
        assert_eq!(baidu_parser.name(), "BaiduParser");
    }

    #[test]
    fn test_parser_support() {
        let factory = ParserFactory::new();

        // Test that each parser supports its own engine type
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
                "BraveParser",
                factory.get_parser(&SearchEngineType::BraveSearch),
            ),
            ("BaiduParser", factory.get_parser(&SearchEngineType::Baidu)),
        ];

        for (name, parser) in parsers {
            assert!(
                parser.supports(&SearchEngineType::Bing)
                    || parser.supports(&SearchEngineType::DuckDuckGo)
                    || parser.supports(&SearchEngineType::Google)
                    || parser.supports(&SearchEngineType::BraveSearch)
                    || parser.supports(&SearchEngineType::Baidu),
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
