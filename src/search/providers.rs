use super::access::{has_api_credentials, resolve_access, resolve_api_key, resolve_base_url};
use super::api::{
    search_brave_api, search_googleai_api, search_searxng_api, search_serper_api, search_tavily_api,
};
use super::parser::ParserFactory;
use super::types::{AccessMethod, SearchEngineType, SearchMode, SearchResult};
use crate::Result;
use crate::error::TarziError;
use crate::fetcher::{FetchMode, WebFetcher};
use async_trait::async_trait;
use tracing::{info, warn};

/// Provider configuration for search
#[derive(Debug)]
pub struct ProviderConfig {
    pub fetcher: Box<WebFetcher>,
    pub search_mode: SearchMode,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl ProviderConfig {
    pub fn new(fetcher: WebFetcher) -> Self {
        Self {
            fetcher: Box::new(fetcher),
            search_mode: SearchMode::Auto,
            api_key: None,
            base_url: None,
        }
    }
}

/// Unified interface for all search providers
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Associated type for the provider's configuration
    type Config;

    /// Create a new provider instance with the given configuration
    fn new(config: Self::Config) -> Self
    where
        Self: Sized;

    /// Perform a search using the provider
    async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;

    /// Check if the provider is healthy/available
    fn is_healthy(&self) -> bool;

    /// Get the search engine type this provider represents
    fn get_engine_type(&self) -> SearchEngineType;
}

/// Shared cascade used by all providers.
async fn provider_search_cascade(
    fetcher: &mut WebFetcher,
    engine_type: SearchEngineType,
    search_mode: SearchMode,
    api_key: &Option<String>,
    base_url: &Option<String>,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let resolved_key = resolve_api_key(engine_type, api_key);
    let resolved_base = resolve_base_url(engine_type, base_url);
    let has_credentials = has_api_credentials(engine_type, &resolved_key, &resolved_base);
    let methods = resolve_access(engine_type, search_mode, has_credentials)?;
    let allow_fallback = matches!(search_mode, SearchMode::Auto | SearchMode::WebQuery);
    let mut last_error: Option<TarziError> = None;

    for method in methods {
        let outcome = match method {
            AccessMethod::Api => match engine_type {
                SearchEngineType::SearxNG => {
                    let host = resolved_base.as_deref().ok_or_else(|| {
                        TarziError::Search(SearchEngineType::SearxNG.missing_credentials_message())
                    })?;
                    search_searxng_api(fetcher, query, limit, host).await
                }
                other => {
                    let key = resolved_key
                        .as_deref()
                        .ok_or_else(|| TarziError::Search(other.missing_credentials_message()))?;
                    match other {
                        SearchEngineType::BraveSearch => {
                            search_brave_api(fetcher, query, limit, key).await
                        }
                        SearchEngineType::GoogleSerper => {
                            search_serper_api(fetcher, query, limit, key).await
                        }
                        SearchEngineType::Tavily => {
                            search_tavily_api(fetcher, query, limit, key).await
                        }
                        SearchEngineType::GoogleAi => {
                            search_googleai_api(fetcher, query, limit, key).await
                        }
                        e => Err(TarziError::Search(format!(
                            "Engine {e:?} does not support API access"
                        ))),
                    }
                }
            },
            AccessMethod::PlainHttp => {
                web_search(
                    fetcher,
                    engine_type,
                    query,
                    limit,
                    FetchMode::PlainRequest,
                    true,
                )
                .await
            }
            AccessMethod::Browser => {
                web_search(
                    fetcher,
                    engine_type,
                    query,
                    limit,
                    FetchMode::BrowserHeadless,
                    false,
                )
                .await
            }
        };

        match outcome {
            Ok(results) if !results.is_empty() => {
                info!("Provider search succeeded via {:?}", method);
                return Ok(results);
            }
            Ok(_) => {
                let msg = format!("Provider search via {method:?} returned no results");
                warn!("{}", msg);
                last_error = Some(TarziError::Search(msg));
                if !allow_fallback {
                    break;
                }
            }
            Err(e) => {
                warn!("Provider search via {:?} failed: {}", method, e);
                last_error = Some(e);
                if !allow_fallback {
                    break;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        TarziError::Search("All provider search access methods failed".to_string())
    }))
}

async fn web_search(
    fetcher: &mut WebFetcher,
    engine_type: SearchEngineType,
    query: &str,
    limit: usize,
    fetch_mode: FetchMode,
    use_plain_pattern: bool,
) -> Result<Vec<SearchResult>> {
    if !engine_type.supports_web() {
        return Err(TarziError::Search(format!(
            "Engine {engine_type:?} does not support web query"
        )));
    }

    let pattern = if use_plain_pattern {
        engine_type.plain_query_pattern()
    } else {
        engine_type.browser_query_pattern()
    };
    let search_url = pattern.replace("{query}", &urlencoding::encode(query));
    info!("Provider web search: {}", search_url);

    let content = fetcher.fetch_raw(&search_url, fetch_mode).await?;
    let parser = ParserFactory::new().get_parser(&engine_type);
    parser.parse(&content, limit)
}

/// Macro to generate search provider implementations
macro_rules! impl_search_provider {
    ($provider_name:ident, $engine_type:expr) => {
        #[derive(Debug)]
        pub struct $provider_name {
            fetcher: WebFetcher,
            search_mode: SearchMode,
            api_key: Option<String>,
            base_url: Option<String>,
        }

        impl $provider_name {
            pub fn new_web(fetcher: WebFetcher) -> Self {
                Self {
                    fetcher,
                    search_mode: SearchMode::Auto,
                    api_key: None,
                    base_url: None,
                }
            }

            pub fn with_options(
                fetcher: WebFetcher,
                search_mode: SearchMode,
                api_key: Option<String>,
                base_url: Option<String>,
            ) -> Self {
                Self {
                    fetcher,
                    search_mode,
                    api_key,
                    base_url,
                }
            }
        }

        #[async_trait]
        impl SearchProvider for $provider_name {
            type Config = crate::fetcher::WebFetcher;

            fn new(config: Self::Config) -> Self {
                Self::new_web(config)
            }

            async fn search(&mut self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
                provider_search_cascade(
                    &mut self.fetcher,
                    $engine_type,
                    self.search_mode,
                    &self.api_key,
                    &self.base_url,
                    query,
                    limit,
                )
                .await
            }

            fn is_healthy(&self) -> bool {
                true
            }

            fn get_engine_type(&self) -> SearchEngineType {
                $engine_type
            }
        }
    };
}

impl_search_provider!(GoogleSearchProvider, SearchEngineType::Google);
impl_search_provider!(GoogleSerperProvider, SearchEngineType::GoogleSerper);
impl_search_provider!(BingSearchProvider, SearchEngineType::Bing);
impl_search_provider!(DuckDuckGoProvider, SearchEngineType::DuckDuckGo);
impl_search_provider!(BraveSearchProvider, SearchEngineType::BraveSearch);
impl_search_provider!(BaiduSearchProvider, SearchEngineType::Baidu);
impl_search_provider!(SougouWeixinProvider, SearchEngineType::SougouWeixin);
impl_search_provider!(TavilySearchProvider, SearchEngineType::Tavily);
impl_search_provider!(GoogleAiSearchProvider, SearchEngineType::GoogleAi);
impl_search_provider!(SearxNGSearchProvider, SearchEngineType::SearxNG);

/// Provider variant enum for different search engines
#[derive(Debug)]
pub enum ProviderVariant {
    Google(GoogleSearchProvider),
    GoogleSerper(GoogleSerperProvider),
    Bing(BingSearchProvider),
    DuckDuckGo(DuckDuckGoProvider),
    BraveSearch(BraveSearchProvider),
    Baidu(BaiduSearchProvider),
    SougouWeixin(SougouWeixinProvider),
    Tavily(TavilySearchProvider),
    GoogleAi(GoogleAiSearchProvider),
    SearxNG(SearxNGSearchProvider),
}

impl ProviderVariant {
    /// Create a provider variant from engine type and configuration
    pub fn from_engine_type(engine_type: SearchEngineType, config: ProviderConfig) -> Result<Self> {
        let ProviderConfig {
            fetcher,
            search_mode,
            api_key,
            base_url,
        } = config;
        let fetcher = *fetcher;

        Ok(match engine_type {
            SearchEngineType::Google => ProviderVariant::Google(
                GoogleSearchProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::GoogleSerper => ProviderVariant::GoogleSerper(
                GoogleSerperProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::Bing => ProviderVariant::Bing(BingSearchProvider::with_options(
                fetcher,
                search_mode,
                api_key,
                base_url,
            )),
            SearchEngineType::DuckDuckGo => ProviderVariant::DuckDuckGo(
                DuckDuckGoProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::BraveSearch => ProviderVariant::BraveSearch(
                BraveSearchProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::Baidu => ProviderVariant::Baidu(BaiduSearchProvider::with_options(
                fetcher,
                search_mode,
                api_key,
                base_url,
            )),
            SearchEngineType::SougouWeixin => ProviderVariant::SougouWeixin(
                SougouWeixinProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::Tavily => ProviderVariant::Tavily(
                TavilySearchProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::GoogleAi => ProviderVariant::GoogleAi(
                GoogleAiSearchProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
            SearchEngineType::SearxNG => ProviderVariant::SearxNG(
                SearxNGSearchProvider::with_options(fetcher, search_mode, api_key, base_url),
            ),
        })
    }

    /// Get the engine type for this provider variant
    pub fn engine_type(&self) -> SearchEngineType {
        match self {
            ProviderVariant::Google(_) => SearchEngineType::Google,
            ProviderVariant::GoogleSerper(_) => SearchEngineType::GoogleSerper,
            ProviderVariant::Bing(_) => SearchEngineType::Bing,
            ProviderVariant::DuckDuckGo(_) => SearchEngineType::DuckDuckGo,
            ProviderVariant::BraveSearch(_) => SearchEngineType::BraveSearch,
            ProviderVariant::Baidu(_) => SearchEngineType::Baidu,
            ProviderVariant::SougouWeixin(_) => SearchEngineType::SougouWeixin,
            ProviderVariant::Tavily(_) => SearchEngineType::Tavily,
            ProviderVariant::GoogleAi(_) => SearchEngineType::GoogleAi,
            ProviderVariant::SearxNG(_) => SearchEngineType::SearxNG,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetcher::WebFetcher;

    #[test]
    fn test_google_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = GoogleSearchProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::Google);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_google_serper_provider() {
        let fetcher = WebFetcher::new();
        let provider = GoogleSerperProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::GoogleSerper);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_bing_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = BingSearchProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::Bing);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_duckduckgo_provider() {
        let fetcher = WebFetcher::new();
        let provider = DuckDuckGoProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::DuckDuckGo);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_brave_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = BraveSearchProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::BraveSearch);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_baidu_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = BaiduSearchProvider::new_web(fetcher);

        assert_eq!(provider.get_engine_type(), SearchEngineType::Baidu);
        assert!(provider.is_healthy());
    }

    #[test]
    fn test_tavily_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = TavilySearchProvider::new_web(fetcher);
        assert_eq!(provider.get_engine_type(), SearchEngineType::Tavily);
    }

    #[test]
    fn test_googleai_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = GoogleAiSearchProvider::new_web(fetcher);
        assert_eq!(provider.get_engine_type(), SearchEngineType::GoogleAi);
    }

    #[test]
    fn test_searxng_search_provider() {
        let fetcher = WebFetcher::new();
        let provider = SearxNGSearchProvider::new_web(fetcher);
        assert_eq!(provider.get_engine_type(), SearchEngineType::SearxNG);
    }

    #[test]
    fn test_provider_variant_from_engine_type() {
        let google_variant = ProviderVariant::from_engine_type(
            SearchEngineType::Google,
            ProviderConfig::new(WebFetcher::new()),
        )
        .unwrap();
        assert_eq!(google_variant.engine_type(), SearchEngineType::Google);

        let serper_variant = ProviderVariant::from_engine_type(
            SearchEngineType::GoogleSerper,
            ProviderConfig::new(WebFetcher::new()),
        )
        .unwrap();
        assert_eq!(serper_variant.engine_type(), SearchEngineType::GoogleSerper);

        let bing_variant = ProviderVariant::from_engine_type(
            SearchEngineType::Bing,
            ProviderConfig::new(WebFetcher::new()),
        )
        .unwrap();
        assert_eq!(bing_variant.engine_type(), SearchEngineType::Bing);

        let tavily_variant = ProviderVariant::from_engine_type(
            SearchEngineType::Tavily,
            ProviderConfig::new(WebFetcher::new()),
        )
        .unwrap();
        assert_eq!(tavily_variant.engine_type(), SearchEngineType::Tavily);
    }

    #[test]
    fn test_provider_config() {
        let fetcher = WebFetcher::new();
        let config = ProviderConfig::new(fetcher);
        assert_eq!(config.search_mode, SearchMode::Auto);
        assert!(config.api_key.is_none());
        assert!(config.base_url.is_none());
    }
}
