use super::access::{has_api_credentials, resolve_access, resolve_api_key, resolve_base_url};
use super::api::{
    search_brave_api, search_googleai_api, search_searxng_api, search_serper_api, search_tavily_api,
};
use super::parser::ParserFactory;
use super::types::{
    AccessMethod, SearchEngineType, SearchResult, default_engine_list, parse_engine_list,
};
use crate::config::Config;
use crate::{
    Result,
    error::TarziError,
    fetcher::{FetchMode, WebFetcher},
};
use std::str::FromStr;

use crate::constants::DEFAULT_QUERY_PATTERN;
use tracing::{info, warn};

pub struct SearchEngine {
    fetcher: WebFetcher,
    /// Ordered failover list; `engine_type` is always `engines[0]`.
    engines: Vec<SearchEngineType>,
    engine_type: SearchEngineType,
    /// Custom query pattern from config; applied only for a single-engine setup.
    query_pattern: String,
    custom_query_pattern: bool,
    user_agent: String,
    parser_factory: ParserFactory,
    fetch_mode: FetchMode,
    browser_enabled: bool,
    /// Programmatic api_key / base_url from config (env still wins per engine at use time).
    config_api_key: Option<String>,
    config_base_url: Option<String>,
}

impl SearchEngine {
    pub fn new() -> Self {
        let engines = default_engine_list();
        let engine_type = engines[0];
        Self {
            fetcher: WebFetcher::new(),
            engines,
            engine_type,
            query_pattern: engine_type.get_query_pattern(),
            custom_query_pattern: false,
            user_agent: crate::constants::DEFAULT_USER_AGENT.to_string(),
            parser_factory: ParserFactory::new(),
            fetch_mode: FetchMode::BrowserHeadless,
            browser_enabled: true,
            config_api_key: None,
            config_base_url: None,
        }
    }

    pub fn engine_type(&self) -> &SearchEngineType {
        &self.engine_type
    }

    pub fn engines(&self) -> &[SearchEngineType] {
        &self.engines
    }

    pub fn query_pattern(&self) -> &str {
        &self.query_pattern
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn browser_enabled(&self) -> bool {
        self.browser_enabled
    }

    pub fn from_config(config: &Config) -> Self {
        let fetcher = crate::fetcher::WebFetcher::from_config(config);

        let engines =
            parse_engine_list(&config.search.engine).unwrap_or_else(|_| default_engine_list());
        let engine_type = engines[0];

        let custom_query_pattern = config.search.query_pattern != DEFAULT_QUERY_PATTERN;
        let query_pattern = if custom_query_pattern && engines.len() == 1 {
            config.search.query_pattern.clone()
        } else {
            if custom_query_pattern && engines.len() > 1 {
                warn!(
                    "Ignoring custom query_pattern for multi-engine list; using per-engine patterns"
                );
            }
            engine_type.get_query_pattern()
        };

        let fetch_mode =
            FetchMode::from_str(&config.fetcher.mode).unwrap_or(FetchMode::BrowserHeadless);

        Self {
            fetcher,
            engines,
            engine_type,
            query_pattern,
            custom_query_pattern,
            user_agent: config.fetcher.user_agent.clone(),
            parser_factory: ParserFactory::new(),
            fetch_mode,
            browser_enabled: config.search.browser,
            config_api_key: config.search.api_key.clone(),
            config_base_url: config.search.base_url.clone(),
        }
    }

    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let mut errors: Vec<String> = Vec::new();

        for engine in self.engines.clone() {
            self.engine_type = engine;
            if !(self.custom_query_pattern && self.engines.len() == 1) {
                self.query_pattern = engine.get_query_pattern();
            }

            let api_key = resolve_api_key(engine, &self.config_api_key);
            let base_url = resolve_base_url(engine, &self.config_base_url);
            let has_credentials = has_api_credentials(engine, &api_key, &base_url);

            // API-only engines: probe credentials before any network call.
            if engine.is_api_only() && !has_credentials {
                let msg = engine.missing_credentials_message();
                warn!("Skipping {:?}: {}", engine, msg);
                errors.push(format!("{engine:?}: {msg}"));
                continue;
            }

            let methods = match resolve_access(engine, has_credentials, self.browser_enabled) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Skipping {:?}: {}", engine, e);
                    errors.push(format!("{engine:?}: {e}"));
                    continue;
                }
            };

            match self
                .search_one_engine(query, limit, engine, &methods, &api_key, &base_url)
                .await
            {
                Ok(results) if !results.is_empty() => {
                    info!("Search succeeded for engine {:?}", engine);
                    return Ok(results);
                }
                Ok(_) => {
                    let msg = format!("{engine:?}: returned no results");
                    warn!("{}", msg);
                    errors.push(msg);
                }
                Err(e) => {
                    warn!("Search failed for {:?}: {}", engine, e);
                    errors.push(format!("{engine:?}: {e}"));
                }
            }
        }

        Err(TarziError::Search(if errors.is_empty() {
            "All search engines failed".to_string()
        } else {
            format!("All search engines failed: {}", errors.join("; "))
        }))
    }

    async fn search_one_engine(
        &mut self,
        query: &str,
        limit: usize,
        engine: SearchEngineType,
        methods: &[AccessMethod],
        api_key: &Option<String>,
        base_url: &Option<String>,
    ) -> Result<Vec<SearchResult>> {
        let mut last_error: Option<TarziError> = None;

        for method in methods {
            match self
                .search_with_method(query, limit, *method, engine, api_key, base_url)
                .await
            {
                Ok(results) if !results.is_empty() => {
                    info!("Search succeeded via {:?} for engine {:?}", method, engine);
                    return Ok(results);
                }
                Ok(_) => {
                    let msg = format!("Search via {method:?} returned no results for {engine:?}");
                    warn!("{}", msg);
                    last_error = Some(TarziError::Search(msg));
                }
                Err(e) => {
                    warn!("Search via {:?} failed for {:?}: {}", method, engine, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TarziError::Search(format!("All search access methods failed for {engine:?}"))
        }))
    }

    async fn search_with_method(
        &mut self,
        query: &str,
        limit: usize,
        method: AccessMethod,
        engine: SearchEngineType,
        api_key: &Option<String>,
        base_url: &Option<String>,
    ) -> Result<Vec<SearchResult>> {
        match method {
            AccessMethod::Api => {
                self.search_via_api(query, limit, engine, api_key, base_url)
                    .await
            }
            AccessMethod::PlainHttp => {
                self.search_via_web(query, limit, FetchMode::PlainRequest, true)
                    .await
            }
            AccessMethod::Browser => {
                let browser_mode = match self.fetch_mode {
                    FetchMode::BrowserHead => FetchMode::BrowserHead,
                    _ => FetchMode::BrowserHeadless,
                };
                self.search_via_web(query, limit, browser_mode, false).await
            }
        }
    }

    async fn search_via_api(
        &self,
        query: &str,
        limit: usize,
        engine: SearchEngineType,
        api_key: &Option<String>,
        base_url: &Option<String>,
    ) -> Result<Vec<SearchResult>> {
        match engine {
            SearchEngineType::SearxNG => {
                let host = base_url.as_deref().ok_or_else(|| {
                    TarziError::Search(SearchEngineType::SearxNG.missing_credentials_message())
                })?;
                search_searxng_api(&self.fetcher, query, limit, host).await
            }
            other => {
                let key = api_key
                    .as_deref()
                    .ok_or_else(|| TarziError::Search(other.missing_credentials_message()))?;
                match other {
                    SearchEngineType::BraveSearch => {
                        search_brave_api(&self.fetcher, query, limit, key).await
                    }
                    SearchEngineType::GoogleSerper => {
                        search_serper_api(&self.fetcher, query, limit, key).await
                    }
                    SearchEngineType::Tavily => {
                        search_tavily_api(&self.fetcher, query, limit, key).await
                    }
                    SearchEngineType::GoogleAi => {
                        search_googleai_api(&self.fetcher, query, limit, key).await
                    }
                    e => Err(TarziError::Search(format!(
                        "Engine {e:?} does not support API access"
                    ))),
                }
            }
        }
    }

    async fn search_via_web(
        &mut self,
        query: &str,
        limit: usize,
        fetch_mode: FetchMode,
        use_plain_pattern: bool,
    ) -> Result<Vec<SearchResult>> {
        if !self.engine_type.supports_web() {
            return Err(TarziError::Search(format!(
                "Engine {:?} does not support web query",
                self.engine_type
            )));
        }

        let pattern = if self.custom_query_pattern && self.engines.len() == 1 {
            self.query_pattern.clone()
        } else if use_plain_pattern {
            self.engine_type.plain_query_pattern()
        } else {
            self.engine_type.browser_query_pattern()
        };

        let search_url = pattern.replace("{query}", &urlencoding::encode(query));
        info!("Web search ({:?}) URL: {}", fetch_mode, search_url);

        let search_page_content = self.fetch_with_retry(&search_url, fetch_mode).await?;
        self.extract_search_results_from_html(&search_page_content, limit)
    }

    async fn fetch_with_retry(&mut self, url: &str, fetch_mode: FetchMode) -> Result<String> {
        const MAX_RETRIES: usize = 3;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

        for attempt in 1..=MAX_RETRIES {
            match self.fetcher.fetch_raw(url, fetch_mode).await {
                Ok(content) => {
                    if attempt > 1 {
                        info!("Successfully fetched content on attempt {}", attempt);
                    }
                    return Ok(content);
                }
                Err(e) => {
                    let error_str = e.to_string();
                    let is_network_error = error_str.contains("nssFailure")
                        || error_str.contains("network")
                        || error_str.contains("timeout")
                        || error_str.contains("connection");

                    if is_network_error && attempt < MAX_RETRIES {
                        warn!(
                            "Network error on attempt {}: {}. Retrying in {} seconds...",
                            attempt,
                            e,
                            RETRY_DELAY.as_secs()
                        );
                        tokio::time::sleep(RETRY_DELAY).await;
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(TarziError::Network("Max retries exceeded".to_string()))
    }

    fn extract_search_results_from_html(
        &self,
        html: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let parser = self.parser_factory.get_parser(&self.engine_type);
        parser.parse(html, limit)
    }

    /// Search and fetch content for each result
    pub async fn search_with_content(
        &mut self,
        query: &str,
        limit: usize,
        fetch_mode: FetchMode,
        format: crate::converter::Format,
    ) -> Result<Vec<(SearchResult, String)>> {
        let effective_fetch_mode = if matches!(fetch_mode, FetchMode::PlainRequest) {
            FetchMode::PlainRequest
        } else {
            FetchMode::BrowserHeadless
        };

        let search_results = self.search(query, limit).await?;

        let mut results_with_content = Vec::new();

        for result in search_results.clone() {
            match self
                .fetcher
                .fetch(&result.url, effective_fetch_mode, format)
                .await
            {
                Ok(content) => {
                    results_with_content.push((result, content));
                }
                Err(e) => {
                    warn!("Failed to fetch content for {}: {}", result.url, e);
                }
            }
        }

        Ok(results_with_content)
    }

    /// Ensure to explicitly shut down browser and driver resources
    pub async fn shutdown(&mut self) {
        self.fetcher.shutdown().await;
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_search_engine_default() {
        let engine = SearchEngine::new();
        assert_eq!(engine.engine_type(), &SearchEngineType::DuckDuckGo);
        assert_eq!(
            engine.query_pattern(),
            SearchEngineType::DuckDuckGo.get_query_pattern()
        );
        assert!(engine.browser_enabled());
        assert_eq!(
            engine.engines(),
            &[
                SearchEngineType::DuckDuckGo,
                SearchEngineType::Bing,
                SearchEngineType::BraveSearch,
            ]
        );
    }

    #[test]
    fn test_search_engine_from_config() {
        let mut config = crate::config::Config::new();
        config.search.engine = SEARCH_ENGINE_GOOGLE.to_string();
        config.search.query_pattern = "custom pattern".to_string();
        config.search.browser = false;

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::Google);
        assert_eq!(engine.query_pattern(), "custom pattern");
        assert!(!engine.browser_enabled());
    }

    #[test]
    fn test_search_engine_multi_engine_list() {
        let mut config = crate::config::Config::new();
        config.search.engine = "brave,duckduckgo,bing".to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::BraveSearch);
        assert_eq!(
            engine.engines(),
            &[
                SearchEngineType::BraveSearch,
                SearchEngineType::DuckDuckGo,
                SearchEngineType::Bing
            ]
        );
    }

    #[test]
    fn test_search_engine_getters() {
        let engine = SearchEngine::new();

        assert_eq!(engine.engine_type(), &SearchEngineType::DuckDuckGo);
        assert!(!engine.query_pattern().is_empty());
        assert!(!engine.user_agent().is_empty());
        assert_eq!(engine.user_agent(), crate::constants::DEFAULT_USER_AGENT);
    }

    #[test]
    fn test_search_engine_config_with_default_pattern() {
        let mut config = crate::config::Config::new();
        config.search.engine = SEARCH_ENGINE_BING.to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::Bing);
        assert_eq!(
            engine.query_pattern(),
            SearchEngineType::Bing.get_query_pattern()
        );
    }

    #[test]
    fn test_search_engine_fallback_to_default() {
        let mut config = crate::config::Config::new();
        config.search.engine = "invalid_engine".to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::DuckDuckGo);
        assert_eq!(
            engine.engines(),
            &[
                SearchEngineType::DuckDuckGo,
                SearchEngineType::Bing,
                SearchEngineType::BraveSearch,
            ]
        );
    }

    #[test]
    fn test_search_engine_from_config_all_engines() {
        let engines = [
            SEARCH_ENGINE_BING,
            SEARCH_ENGINE_DUCKDUCKGO,
            SEARCH_ENGINE_GOOGLE,
            SEARCH_ENGINE_GOOGLE_SERPER,
            SEARCH_ENGINE_SERPER_ALIAS,
            SEARCH_ENGINE_BRAVE,
            SEARCH_ENGINE_BAIDU,
            SEARCH_ENGINE_SOUGOU_WEIXIN,
        ];

        for engine_name in engines {
            let mut config = crate::config::Config::new();
            config.search.engine = engine_name.to_string();
            config.search.api_key = Some("unit-test-key".to_string());
            config.search.browser = true;

            let engine = SearchEngine::from_config(&config);
            let expected_type = SearchEngineType::from_str(engine_name).unwrap();
            assert_eq!(engine.engine_type(), &expected_type, "engine={engine_name}");
            assert!(engine.browser_enabled());
        }
    }

    #[test]
    fn test_search_engine_google_serper_from_config() {
        let mut config = crate::config::Config::new();
        config.search.engine = SEARCH_ENGINE_GOOGLE_SERPER.to_string();
        config.search.api_key = Some("test-key".to_string());

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::GoogleSerper);
    }
}
