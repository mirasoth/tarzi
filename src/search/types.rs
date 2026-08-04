use crate::constants::{
    BAIDU_QUERY_PATTERN, BING_QUERY_PATTERN, BRAVE_API_QUERY_PATTERN, BRAVE_QUERY_PATTERN,
    DUCKDUCKGO_PLAIN_QUERY_PATTERN, DUCKDUCKGO_QUERY_PATTERN, GOOGLE_QUERY_PATTERN,
    SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BING, SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO,
    SEARCH_ENGINE_GOOGLE, SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_SERPER_ALIAS,
    SEARCH_ENGINE_SOUGOU_WEIXIN, SEARCH_MODE_APIQUERY, SEARCH_MODE_AUTO, SEARCH_MODE_WEBQUERY,
    SERPER_API_URL, SOUGOU_WEIXIN_QUERY_PATTERN,
};
use crate::error::TarziError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchEngineType {
    Bing,
    DuckDuckGo,
    Google,
    GoogleSerper,
    BraveSearch,
    Baidu,
    SougouWeixin,
}

impl FromStr for SearchEngineType {
    type Err = TarziError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            SEARCH_ENGINE_BING => Ok(SearchEngineType::Bing),
            SEARCH_ENGINE_DUCKDUCKGO => Ok(SearchEngineType::DuckDuckGo),
            SEARCH_ENGINE_GOOGLE => Ok(SearchEngineType::Google),
            SEARCH_ENGINE_GOOGLE_SERPER | SEARCH_ENGINE_SERPER_ALIAS => {
                Ok(SearchEngineType::GoogleSerper)
            }
            SEARCH_ENGINE_BRAVE => Ok(SearchEngineType::BraveSearch),
            SEARCH_ENGINE_BAIDU => Ok(SearchEngineType::Baidu),
            SEARCH_ENGINE_SOUGOU_WEIXIN => Ok(SearchEngineType::SougouWeixin),
            _ => Err(TarziError::InvalidEngine(s.to_string())),
        }
    }
}

impl SearchEngineType {
    /// Browser / default web query pattern (used for headless fallback).
    pub fn get_query_pattern(&self) -> String {
        self.browser_query_pattern()
    }

    pub fn browser_query_pattern(&self) -> String {
        match self {
            SearchEngineType::Bing => BING_QUERY_PATTERN.to_string(),
            SearchEngineType::DuckDuckGo => DUCKDUCKGO_QUERY_PATTERN.to_string(),
            SearchEngineType::Google => GOOGLE_QUERY_PATTERN.to_string(),
            SearchEngineType::GoogleSerper => String::new(),
            SearchEngineType::BraveSearch => BRAVE_QUERY_PATTERN.to_string(),
            SearchEngineType::Baidu => BAIDU_QUERY_PATTERN.to_string(),
            SearchEngineType::SougouWeixin => SOUGOU_WEIXIN_QUERY_PATTERN.to_string(),
        }
    }

    /// Plain HTTP query pattern (may differ from browser for some engines).
    pub fn plain_query_pattern(&self) -> String {
        match self {
            SearchEngineType::DuckDuckGo => DUCKDUCKGO_PLAIN_QUERY_PATTERN.to_string(),
            SearchEngineType::GoogleSerper => String::new(),
            other => other.browser_query_pattern(),
        }
    }

    pub fn supports_api(&self) -> bool {
        matches!(
            self,
            SearchEngineType::BraveSearch | SearchEngineType::GoogleSerper
        )
    }

    pub fn supports_web(&self) -> bool {
        !matches!(self, SearchEngineType::GoogleSerper)
    }

    /// API-only engines require a key and have no web fallback under this engine id.
    pub fn is_api_only(&self) -> bool {
        matches!(self, SearchEngineType::GoogleSerper)
    }

    pub fn api_query_pattern(&self) -> Option<&'static str> {
        match self {
            SearchEngineType::BraveSearch => Some(BRAVE_API_QUERY_PATTERN),
            SearchEngineType::GoogleSerper => Some(SERPER_API_URL),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchMode {
    Auto,
    ApiQuery,
    WebQuery,
}

impl FromStr for SearchMode {
    type Err = TarziError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            SEARCH_MODE_AUTO => Ok(SearchMode::Auto),
            SEARCH_MODE_APIQUERY => Ok(SearchMode::ApiQuery),
            SEARCH_MODE_WEBQUERY => Ok(SearchMode::WebQuery),
            _ => Err(TarziError::Config(format!("Invalid search mode: {s}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMethod {
    Api,
    PlainHttp,
    Browser,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub rank: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{
        BAIDU_QUERY_PATTERN, BING_QUERY_PATTERN, BRAVE_QUERY_PATTERN,
        DUCKDUCKGO_PLAIN_QUERY_PATTERN, DUCKDUCKGO_QUERY_PATTERN, GOOGLE_QUERY_PATTERN,
        SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BING, SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE, SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_SERPER_ALIAS,
        SEARCH_ENGINE_SOUGOU_WEIXIN, SOUGOU_WEIXIN_QUERY_PATTERN,
    };

    #[test]
    fn test_search_engine_type_parsing() {
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_DUCKDUCKGO).unwrap(),
            SearchEngineType::DuckDuckGo
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_GOOGLE).unwrap(),
            SearchEngineType::Google
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_GOOGLE_SERPER).unwrap(),
            SearchEngineType::GoogleSerper
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_SERPER_ALIAS).unwrap(),
            SearchEngineType::GoogleSerper
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_BING).unwrap(),
            SearchEngineType::Bing
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_BRAVE).unwrap(),
            SearchEngineType::BraveSearch
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_BAIDU).unwrap(),
            SearchEngineType::Baidu
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_SOUGOU_WEIXIN).unwrap(),
            SearchEngineType::SougouWeixin
        );

        assert!(SearchEngineType::from_str("invalid").is_err());
        assert!(SearchEngineType::from_str("").is_err());
        assert!(SearchEngineType::from_str("web").is_err());
        assert!(SearchEngineType::from_str("api").is_err());
    }

    #[test]
    fn test_search_mode_parsing() {
        assert_eq!(SearchMode::from_str("auto").unwrap(), SearchMode::Auto);
        assert_eq!(
            SearchMode::from_str("apiquery").unwrap(),
            SearchMode::ApiQuery
        );
        assert_eq!(
            SearchMode::from_str("webquery").unwrap(),
            SearchMode::WebQuery
        );
        assert!(SearchMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_query_patterns() {
        assert_eq!(
            SearchEngineType::DuckDuckGo.get_query_pattern(),
            DUCKDUCKGO_QUERY_PATTERN
        );
        assert_eq!(
            SearchEngineType::DuckDuckGo.plain_query_pattern(),
            DUCKDUCKGO_PLAIN_QUERY_PATTERN
        );
        assert_ne!(
            SearchEngineType::DuckDuckGo.plain_query_pattern(),
            SearchEngineType::DuckDuckGo.browser_query_pattern()
        );
        assert_eq!(
            SearchEngineType::Google.get_query_pattern(),
            GOOGLE_QUERY_PATTERN
        );
        assert_eq!(
            SearchEngineType::Bing.get_query_pattern(),
            BING_QUERY_PATTERN
        );
        assert_eq!(
            SearchEngineType::BraveSearch.get_query_pattern(),
            BRAVE_QUERY_PATTERN
        );
        assert_eq!(
            SearchEngineType::Baidu.get_query_pattern(),
            BAIDU_QUERY_PATTERN
        );
        assert_eq!(
            SearchEngineType::SougouWeixin.get_query_pattern(),
            SOUGOU_WEIXIN_QUERY_PATTERN
        );
        assert!(
            SearchEngineType::GoogleSerper
                .plain_query_pattern()
                .is_empty()
        );
        assert!(SearchEngineType::GoogleSerper.supports_api());
        assert!(!SearchEngineType::GoogleSerper.supports_web());
        assert!(SearchEngineType::GoogleSerper.is_api_only());
        assert!(!SearchEngineType::Google.supports_api());
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            rank: 1,
        };

        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "Test snippet");
        assert_eq!(result.rank, 1);
    }
}
