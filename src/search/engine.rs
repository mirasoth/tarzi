use super::access::{resolve_access, resolve_api_key};
use super::api::{search_brave_api, search_serper_api};
use super::parser::ParserFactory;
use super::types::{AccessMethod, SearchEngineType, SearchMode, SearchResult};
use crate::config::Config;
use crate::{
    Result,
    error::TarziError,
    fetcher::{FetchMode, WebFetcher},
};
use std::str::FromStr;

use crate::constants::{DEFAULT_QUERY_PATTERN, DEFAULT_SEARCH_MODE};
use tracing::{info, warn};

pub struct SearchEngine {
    fetcher: WebFetcher,
    engine_type: SearchEngineType,
    query_pattern: String,
    user_agent: String,
    parser_factory: ParserFactory,
    fetch_mode: FetchMode,
    search_mode: SearchMode,
    api_key: Option<String>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            fetcher: WebFetcher::new(),
            engine_type: SearchEngineType::Bing,
            query_pattern: SearchEngineType::Bing.get_query_pattern(),
            user_agent: crate::constants::DEFAULT_USER_AGENT.to_string(),
            parser_factory: ParserFactory::new(),
            fetch_mode: FetchMode::BrowserHeadless,
            search_mode: SearchMode::Auto,
            api_key: None,
        }
    }

    pub fn engine_type(&self) -> &SearchEngineType {
        &self.engine_type
    }

    pub fn query_pattern(&self) -> &str {
        &self.query_pattern
    }

    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    pub fn search_mode(&self) -> SearchMode {
        self.search_mode
    }

    pub fn from_config(config: &Config) -> Self {
        let fetcher = crate::fetcher::WebFetcher::from_config(config);

        let engine_type =
            SearchEngineType::from_str(&config.search.engine).unwrap_or(SearchEngineType::Bing);

        let query_pattern = if config.search.query_pattern != DEFAULT_QUERY_PATTERN {
            config.search.query_pattern.clone()
        } else {
            engine_type.get_query_pattern()
        };

        let fetch_mode =
            FetchMode::from_str(&config.fetcher.mode).unwrap_or(FetchMode::BrowserHeadless);

        let search_mode = SearchMode::from_str(&config.search.mode).unwrap_or_else(|_| {
            SearchMode::from_str(DEFAULT_SEARCH_MODE).unwrap_or(SearchMode::Auto)
        });

        let api_key = resolve_api_key(engine_type, &config.search.api_key);

        Self {
            fetcher,
            engine_type,
            query_pattern,
            user_agent: config.fetcher.user_agent.clone(),
            parser_factory: ParserFactory::new(),
            fetch_mode,
            search_mode,
            api_key,
        }
    }

    pub async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let has_api_key = self.api_key.is_some();
        let methods = resolve_access(self.engine_type, self.search_mode, has_api_key)?;

        let allow_fallback = matches!(self.search_mode, SearchMode::Auto | SearchMode::WebQuery);
        let mut last_error: Option<TarziError> = None;

        for method in methods {
            match self.search_with_method(query, limit, method).await {
                Ok(results) if !results.is_empty() => {
                    info!(
                        "Search succeeded via {:?} for engine {:?}",
                        method, self.engine_type
                    );
                    return Ok(results);
                }
                Ok(_) => {
                    let msg = format!(
                        "Search via {:?} returned no results for {:?}",
                        method, self.engine_type
                    );
                    warn!("{}", msg);
                    last_error = Some(TarziError::Search(msg));
                    if !allow_fallback {
                        break;
                    }
                }
                Err(e) => {
                    warn!(
                        "Search via {:?} failed for {:?}: {}",
                        method, self.engine_type, e
                    );
                    last_error = Some(e);
                    if !allow_fallback {
                        break;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| TarziError::Search("All search access methods failed".to_string())))
    }

    async fn search_with_method(
        &mut self,
        query: &str,
        limit: usize,
        method: AccessMethod,
    ) -> Result<Vec<SearchResult>> {
        match method {
            AccessMethod::Api => self.search_via_api(query, limit).await,
            AccessMethod::PlainHttp => {
                self.search_via_web(query, limit, FetchMode::PlainRequest, true)
                    .await
            }
            AccessMethod::Browser => {
                // Prefer configured browser mode when it is a browser mode; otherwise headless.
                let browser_mode = match self.fetch_mode {
                    FetchMode::BrowserHead => FetchMode::BrowserHead,
                    _ => FetchMode::BrowserHeadless,
                };
                self.search_via_web(query, limit, browser_mode, false).await
            }
        }
    }

    async fn search_via_api(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let api_key = self
            .api_key
            .as_deref()
            .ok_or_else(|| TarziError::Search("API key is required for apiquery".to_string()))?;

        match self.engine_type {
            SearchEngineType::BraveSearch => {
                search_brave_api(&self.fetcher, query, limit, api_key).await
            }
            SearchEngineType::GoogleSerper => {
                search_serper_api(&self.fetcher, query, limit, api_key).await
            }
            other => Err(TarziError::Search(format!(
                "Engine {other:?} does not support API access"
            ))),
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

        let pattern = if self.query_pattern != DEFAULT_QUERY_PATTERN
            && self.query_pattern != self.engine_type.get_query_pattern()
        {
            // Explicit custom pattern from config
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

    pub async fn search_with_proxy(
        &mut self,
        query: &str,
        limit: usize,
        proxy: &str,
    ) -> Result<Vec<SearchResult>> {
        info!("Starting search with proxy hint: {}", proxy);
        // Proxy is applied via WebFetcher config / HTTPS_PROXY env for plain and API paths.
        // Browser proxy wiring remains limited.
        let _ = crate::config::get_proxy_from_env_or_config(&Some(proxy.to_string()));
        self.search(query, limit).await
    }

    /// Backward compatibility
    pub async fn cleanup(&mut self) -> Result<()> {
        self.fetcher.shutdown().await;
        Ok(())
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

impl Drop for SearchEngine {
    fn drop(&mut self) {
        info!("SearchEngine dropping - cleanup will be handled by WebFetcher");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;

    #[test]
    fn test_search_engine_default() {
        let engine = SearchEngine::new();
        assert_eq!(engine.engine_type(), &SearchEngineType::Bing);
        assert_eq!(
            engine.query_pattern(),
            SearchEngineType::Bing.get_query_pattern()
        );
        assert_eq!(engine.search_mode(), SearchMode::Auto);
    }

    #[test]
    fn test_search_engine_from_config() {
        let mut config = crate::config::Config::new();
        config.search.engine = SEARCH_ENGINE_GOOGLE.to_string();
        config.search.query_pattern = "custom pattern".to_string();
        config.search.mode = SEARCH_MODE_WEBQUERY.to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::Google);
        assert_eq!(engine.query_pattern(), "custom pattern");
        assert_eq!(engine.search_mode(), SearchMode::WebQuery);
    }

    #[test]
    fn test_search_engine_getters() {
        let engine = SearchEngine::new();

        assert_eq!(engine.engine_type(), &SearchEngineType::Bing);
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
    fn test_search_engine_fallback_to_bing() {
        let mut config = crate::config::Config::new();
        config.search.engine = "invalid_engine".to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::Bing);
    }

    #[test]
    fn test_search_engine_google_serper_from_config() {
        let mut config = crate::config::Config::new();
        config.search.engine = SEARCH_ENGINE_GOOGLE_SERPER.to_string();
        config.search.api_key = Some("test-key".to_string());
        config.search.mode = SEARCH_MODE_APIQUERY.to_string();

        let engine = SearchEngine::from_config(&config);
        assert_eq!(engine.engine_type(), &SearchEngineType::GoogleSerper);
        assert_eq!(engine.search_mode(), SearchMode::ApiQuery);
    }
}
