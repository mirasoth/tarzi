use crate::constants::{
    BAIDU_QUERY_PATTERN, BING_QUERY_PATTERN, BRAVE_QUERY_PATTERN, DEFAULT_SEARCH_ENGINE,
    DUCKDUCKGO_PLAIN_QUERY_PATTERN, DUCKDUCKGO_QUERY_PATTERN, GOOGLE_QUERY_PATTERN,
    SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BING, SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO,
    SEARCH_ENGINE_GOOGLE, SEARCH_ENGINE_GOOGLE_AI_ALIAS, SEARCH_ENGINE_GOOGLE_SERPER,
    SEARCH_ENGINE_GOOGLEAI, SEARCH_ENGINE_SEARXNG, SEARCH_ENGINE_SERPER_ALIAS,
    SEARCH_ENGINE_SOUGOU_WEIXIN, SEARCH_ENGINE_TAVILY, SOUGOU_WEIXIN_QUERY_PATTERN,
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
    Tavily,
    GoogleAi,
    SearxNG,
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
            SEARCH_ENGINE_TAVILY => Ok(SearchEngineType::Tavily),
            SEARCH_ENGINE_GOOGLEAI | SEARCH_ENGINE_GOOGLE_AI_ALIAS => {
                Ok(SearchEngineType::GoogleAi)
            }
            SEARCH_ENGINE_SEARXNG => Ok(SearchEngineType::SearxNG),
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
            SearchEngineType::BraveSearch => BRAVE_QUERY_PATTERN.to_string(),
            SearchEngineType::Baidu => BAIDU_QUERY_PATTERN.to_string(),
            SearchEngineType::SougouWeixin => SOUGOU_WEIXIN_QUERY_PATTERN.to_string(),
            SearchEngineType::GoogleSerper
            | SearchEngineType::Tavily
            | SearchEngineType::GoogleAi
            | SearchEngineType::SearxNG => String::new(),
        }
    }

    /// Plain HTTP query pattern (may differ from browser for some engines).
    pub fn plain_query_pattern(&self) -> String {
        match self {
            SearchEngineType::DuckDuckGo => DUCKDUCKGO_PLAIN_QUERY_PATTERN.to_string(),
            SearchEngineType::GoogleSerper
            | SearchEngineType::Tavily
            | SearchEngineType::GoogleAi
            | SearchEngineType::SearxNG => String::new(),
            other => other.browser_query_pattern(),
        }
    }

    pub fn supports_api(&self) -> bool {
        matches!(
            self,
            SearchEngineType::BraveSearch
                | SearchEngineType::GoogleSerper
                | SearchEngineType::Tavily
                | SearchEngineType::GoogleAi
                | SearchEngineType::SearxNG
        )
    }

    pub fn supports_web(&self) -> bool {
        !self.is_api_only()
    }

    /// API-only engines have no HTML SERP / browser fallback under this engine id.
    pub fn is_api_only(&self) -> bool {
        matches!(
            self,
            SearchEngineType::GoogleSerper
                | SearchEngineType::Tavily
                | SearchEngineType::GoogleAi
                | SearchEngineType::SearxNG
        )
    }

    /// Whether this API engine authenticates with an API key (vs host URL).
    pub fn requires_api_key(&self) -> bool {
        matches!(
            self,
            SearchEngineType::BraveSearch
                | SearchEngineType::GoogleSerper
                | SearchEngineType::Tavily
                | SearchEngineType::GoogleAi
        )
    }

    /// Whether this engine needs a base host URL (SearxNG).
    pub fn requires_base_url(&self) -> bool {
        matches!(self, SearchEngineType::SearxNG)
    }

    pub fn missing_credentials_message(&self) -> String {
        match self {
            SearchEngineType::GoogleSerper => "google_serper requires SERPER_API_KEY".to_string(),
            SearchEngineType::Tavily => "tavily requires TAVILY_API_KEY".to_string(),
            SearchEngineType::GoogleAi => "googleai requires GEMINI_API_KEY".to_string(),
            SearchEngineType::SearxNG => "searxng requires SEARX_HOST".to_string(),
            SearchEngineType::BraveSearch => "brave requires BRAVE_API_KEY".to_string(),
            other => format!("{other:?} requires credentials"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessMethod {
    Api,
    PlainHttp,
    Browser,
}

/// Parse an ordered engine list from a comma-separated string.
///
/// Empty tokens are ignored. Duplicates are removed while preserving first-seen order.
/// An empty / whitespace-only string yields the default failover list
/// (`duckduckgo`, `bing`, `brave`).
pub fn parse_engine_list(s: &str) -> Result<Vec<SearchEngineType>, TarziError> {
    let mut engines = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let engine = SearchEngineType::from_str(part)?;
        if !engines.contains(&engine) {
            engines.push(engine);
        }
    }
    if engines.is_empty() {
        Ok(default_engine_list())
    } else {
        Ok(engines)
    }
}

/// Default ordered failover list (`DEFAULT_SEARCH_ENGINE`).
pub fn default_engine_list() -> Vec<SearchEngineType> {
    // Parse the constant directly (never empty) so the list cannot drift from
    // `DEFAULT_SEARCH_ENGINE`.
    let mut engines = Vec::new();
    for part in DEFAULT_SEARCH_ENGINE.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let engine = SearchEngineType::from_str(part)
            .unwrap_or_else(|e| panic!("invalid DEFAULT_SEARCH_ENGINE token '{part}': {e}"));
        if !engines.contains(&engine) {
            engines.push(engine);
        }
    }
    assert!(
        !engines.is_empty(),
        "DEFAULT_SEARCH_ENGINE must yield at least one engine"
    );
    engines
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
        SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO, SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_GOOGLE_AI_ALIAS, SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_GOOGLEAI,
        SEARCH_ENGINE_SEARXNG, SEARCH_ENGINE_SERPER_ALIAS, SEARCH_ENGINE_SOUGOU_WEIXIN,
        SEARCH_ENGINE_TAVILY, SOUGOU_WEIXIN_QUERY_PATTERN,
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
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_TAVILY).unwrap(),
            SearchEngineType::Tavily
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_GOOGLEAI).unwrap(),
            SearchEngineType::GoogleAi
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_GOOGLE_AI_ALIAS).unwrap(),
            SearchEngineType::GoogleAi
        );
        assert_eq!(
            SearchEngineType::from_str(SEARCH_ENGINE_SEARXNG).unwrap(),
            SearchEngineType::SearxNG
        );

        assert!(SearchEngineType::from_str("invalid").is_err());
        assert!(SearchEngineType::from_str("").is_err());
        assert!(SearchEngineType::from_str("web").is_err());
        assert!(SearchEngineType::from_str("api").is_err());
    }

    #[test]
    fn test_parse_engine_list() {
        assert_eq!(
            parse_engine_list("brave,duckduckgo,bing").unwrap(),
            vec![
                SearchEngineType::BraveSearch,
                SearchEngineType::DuckDuckGo,
                SearchEngineType::Bing,
            ]
        );
        assert_eq!(
            parse_engine_list("brave, brave, bing").unwrap(),
            vec![SearchEngineType::BraveSearch, SearchEngineType::Bing]
        );
        assert_eq!(parse_engine_list("").unwrap(), default_engine_list());
        assert_eq!(parse_engine_list("  ,  ").unwrap(), default_engine_list());
        assert!(parse_engine_list("brave,not_an_engine").is_err());
        assert_eq!(
            parse_engine_list("serper").unwrap(),
            vec![SearchEngineType::GoogleSerper]
        );
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
        assert!(SearchEngineType::Tavily.browser_query_pattern().is_empty());
        assert!(
            SearchEngineType::GoogleAi
                .browser_query_pattern()
                .is_empty()
        );
        assert!(SearchEngineType::SearxNG.browser_query_pattern().is_empty());
        assert!(SearchEngineType::GoogleSerper.supports_api());
        assert!(!SearchEngineType::GoogleSerper.supports_web());
        assert!(SearchEngineType::GoogleSerper.is_api_only());
        assert!(!SearchEngineType::Google.supports_api());
        assert!(SearchEngineType::Tavily.is_api_only());
        assert!(SearchEngineType::GoogleAi.is_api_only());
        assert!(SearchEngineType::SearxNG.is_api_only());
        assert!(SearchEngineType::SearxNG.requires_base_url());
        assert!(!SearchEngineType::Tavily.requires_base_url());
    }

    #[test]
    fn test_engine_capabilities_matrix() {
        let cases = [
            (SearchEngineType::Bing, false, true, false),
            (SearchEngineType::DuckDuckGo, false, true, false),
            (SearchEngineType::Google, false, true, false),
            (SearchEngineType::GoogleSerper, true, false, true),
            (SearchEngineType::BraveSearch, true, true, false),
            (SearchEngineType::Baidu, false, true, false),
            (SearchEngineType::SougouWeixin, false, true, false),
            (SearchEngineType::Tavily, true, false, true),
            (SearchEngineType::GoogleAi, true, false, true),
            (SearchEngineType::SearxNG, true, false, true),
        ];

        for (engine, api, web, api_only) in cases {
            assert_eq!(engine.supports_api(), api, "{engine:?} supports_api");
            assert_eq!(engine.supports_web(), web, "{engine:?} supports_web");
            assert_eq!(engine.is_api_only(), api_only, "{engine:?} is_api_only");
            if web {
                assert!(
                    !engine.browser_query_pattern().is_empty(),
                    "{engine:?} browser pattern"
                );
            } else {
                assert!(
                    engine.browser_query_pattern().is_empty(),
                    "{engine:?} should have empty browser pattern"
                );
            }
        }
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
