//! Constants used throughout the Tarzi application
//!
//! This module contains all magic values and constants to avoid duplication
//! and make maintenance easier.

use std::time::Duration;

// ============================================================================
// Network & WebDriver Constants
// ============================================================================

/// Default ChromeDriver URL
pub const CHROMEDRIVER_DEFAULT_URL: &str = "http://localhost:9515";

/// Default ChromeDriver port
pub const CHROMEDRIVER_DEFAULT_PORT: u16 = 9515;

/// Default GeckoDriver port
pub const GECKODRIVER_DEFAULT_PORT: u16 = 4444;

/// Default HTTP client user agent
pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Default ChromeDriver
pub const CHROMEDRIVER: &str = "chromedriver";

/// Default GeckoDriver
pub const GECKODRIVER: &str = "geckodriver";

// ============================================================================
// Timeout Constants
// ============================================================================

/// Default timeout in seconds for various operations
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default timeout duration for various operations
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(DEFAULT_TIMEOUT_SECS);

/// WebDriver availability check timeout in seconds
pub const WEBDRIVER_CHECK_TIMEOUT_SECS: u64 = 2;

/// WebDriver availability check timeout duration
pub const WEBDRIVER_CHECK_TIMEOUT: Duration = Duration::from_secs(WEBDRIVER_CHECK_TIMEOUT_SECS);

/// Browser launch timeout in seconds
pub const BROWSER_LAUNCH_TIMEOUT_SECS: u64 = 60;

/// Browser launch timeout duration
pub const BROWSER_LAUNCH_TIMEOUT: Duration = Duration::from_secs(BROWSER_LAUNCH_TIMEOUT_SECS);

/// Page load wait time in seconds
pub const PAGE_LOAD_WAIT_SECS: u64 = 2;

/// Page load wait duration
pub const PAGE_LOAD_WAIT: Duration = Duration::from_secs(PAGE_LOAD_WAIT_SECS);

// ============================================================================
// Test URLs
// ============================================================================

/// HTTPBin HTML test endpoint
pub const HTTPBIN_HTML_URL: &str = "https://httpbin.org/html";

/// HTTPBin JSON test endpoint
pub const HTTPBIN_JSON_URL: &str = "https://httpbin.org/json";

/// HTTPBin XML test endpoint
pub const HTTPBIN_XML_URL: &str = "https://httpbin.org/xml";

/// HTTPBin 404 status test endpoint
pub const HTTPBIN_404_URL: &str = "https://httpbin.org/status/404";

/// HTTPBin 500 status test endpoint
pub const HTTPBIN_500_URL: &str = "https://httpbin.org/status/500";

/// HTTPBin large response test endpoint (10KB)
pub const HTTPBIN_LARGE_URL: &str = "https://httpbin.org/bytes/10000";

/// Example domain for testing
pub const EXAMPLE_URL: &str = "https://example.com";

/// Example proxy URL for testing
pub const EXAMPLE_PROXY_URL: &str = "http://example.com:8080";

// ============================================================================
// Browser Arguments
// ============================================================================

/// Chrome/Chromium browser arguments for headless mode
pub const CHROME_HEADLESS_ARGS: &[&str] = &[
    "--headless",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    "--no-sandbox",
];

/// Chrome/Chromium browser arguments for driver
pub const CHROME_DRIVER_ARGS: &[&str] =
    &["--disable-gpu", "--no-sandbox", "--disable-dev-shm-usage"];

/// Firefox browser arguments
pub const FIREFOX_DRIVER_ARGS: &[&str] = &["--log=warn"];

// ============================================================================
// Default Configuration Values
// ============================================================================

/// Default search limit
pub const DEFAULT_SEARCH_LIMIT: usize = 5;

/// Default search engine failover list (plain-HTTP-friendly first)
pub const DEFAULT_SEARCH_ENGINE: &str = "duckduckgo,bing,brave";

// Default log levels
pub const LOG_LEVEL_INFO: &str = "info";
pub const LOG_LEVEL_DEBUG: &str = "debug";

// Default formats
pub const FORMAT_MARKDOWN: &str = "markdown";
pub const FORMAT_JSON: &str = "json";
pub const FORMAT_HTML: &str = "html";

// Default fetcher modes
pub const FETCHER_MODE_BROWSER_HEADLESS: &str = "browser_headless";
pub const FETCHER_MODE_BROWSER_HEAD: &str = "browser_head";
pub const FETCHER_MODE_PLAIN_REQUEST: &str = "plain_request";
pub const FETCHER_MODE_HEAD: &str = "head";
pub const FETCHER_MODE_HEADLESS: &str = "headless";
pub const FETCHER_MODE_PLAIN: &str = "plain";

// Default search engines
pub const SEARCH_ENGINE_DUCKDUCKGO: &str = "duckduckgo";
pub const SEARCH_ENGINE_BING: &str = "bing";
pub const SEARCH_ENGINE_GOOGLE: &str = "google";
pub const SEARCH_ENGINE_GOOGLE_SERPER: &str = "google_serper";
pub const SEARCH_ENGINE_SERPER_ALIAS: &str = "serper";
pub const SEARCH_ENGINE_BRAVE: &str = "brave";
pub const SEARCH_ENGINE_BAIDU: &str = "baidu";
pub const SEARCH_ENGINE_SOUGOU_WEIXIN: &str = "sogou_weixin";
pub const SEARCH_ENGINE_TAVILY: &str = "tavily";
pub const SEARCH_ENGINE_GOOGLEAI: &str = "googleai";
pub const SEARCH_ENGINE_GOOGLE_AI_ALIAS: &str = "google_ai";
pub const SEARCH_ENGINE_SEARXNG: &str = "searxng";

/// Default: allow headless/headed browser as a search access fallback
pub const DEFAULT_SEARCH_BROWSER: bool = true;

// API key / host environment variable names
pub const ENV_BRAVE_API_KEY: &str = "BRAVE_API_KEY";
pub const ENV_SERPER_API_KEY: &str = "SERPER_API_KEY";
pub const ENV_TAVILY_API_KEY: &str = "TAVILY_API_KEY";
pub const ENV_GEMINI_API_KEY: &str = "GEMINI_API_KEY";
pub const ENV_SEARX_HOST: &str = "SEARX_HOST";

// Tarzi-specific configuration environment variable names
pub const ENV_TARZI_LOG_LEVEL: &str = "TARZI_LOG_LEVEL";
pub const ENV_TARZI_TIMEOUT: &str = "TARZI_TIMEOUT";
pub const ENV_TARZI_FETCHER_MODE: &str = "TARZI_FETCHER_MODE";
pub const ENV_TARZI_FETCHER_FORMAT: &str = "TARZI_FETCHER_FORMAT";
pub const ENV_TARZI_USER_AGENT: &str = "TARZI_USER_AGENT";
pub const ENV_TARZI_FETCHER_TIMEOUT: &str = "TARZI_FETCHER_TIMEOUT";
pub const ENV_TARZI_PROXY: &str = "TARZI_PROXY";
pub const ENV_TARZI_WEB_DRIVER: &str = "TARZI_WEB_DRIVER";
pub const ENV_TARZI_WEB_DRIVER_URL: &str = "TARZI_WEB_DRIVER_URL";
pub const ENV_TARZI_SEARCH_ENGINE: &str = "TARZI_SEARCH_ENGINE";
pub const ENV_TARZI_QUERY_PATTERN: &str = "TARZI_QUERY_PATTERN";
pub const ENV_TARZI_SEARCH_LIMIT: &str = "TARZI_SEARCH_LIMIT";
pub const ENV_TARZI_SEARCH_BROWSER: &str = "TARZI_SEARCH_BROWSER";

// ============================================================================
// Search Engine Query Patterns
// ============================================================================

/// Browser / JS SERP query patterns
pub const DUCKDUCKGO_QUERY_PATTERN: &str = "https://duckduckgo.com/?q={query}";
/// Plain HTML DuckDuckGo endpoint (no JS required)
pub const DUCKDUCKGO_PLAIN_QUERY_PATTERN: &str = "https://html.duckduckgo.com/html/?q={query}";
pub const BING_QUERY_PATTERN: &str = "https://www.bing.com/search?q={query}";
pub const GOOGLE_QUERY_PATTERN: &str = "https://www.google.com/search?q={query}";
pub const BRAVE_QUERY_PATTERN: &str = "https://search.brave.com/search?q={query}&source=web";
pub const BAIDU_QUERY_PATTERN: &str = "https://www.baidu.com/s?wd={query}";
pub const SOUGOU_WEIXIN_QUERY_PATTERN: &str =
    "https://weixin.sogou.com/weixin?type=2&s_from=input&&ie=utf8&query={query}";

/// Brave Search API endpoint pattern
pub const BRAVE_API_QUERY_PATTERN: &str =
    "https://api.search.brave.com/res/v1/web/search?q={query}&count={limit}";

/// Serper Google search API endpoint
pub const SERPER_API_URL: &str = "https://google.serper.dev/search";

/// Tavily Search API endpoint
pub const TAVILY_API_URL: &str = "https://api.tavily.com/search";

/// Gemini generateContent endpoint pattern (`{model}` substituted at call time)
pub const GOOGLEAI_API_URL_PATTERN: &str =
    "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent";

/// Default Gemini model for googleai grounded search
pub const DEFAULT_GOOGLEAI_MODEL: &str = "gemini-2.0-flash";

// ============================================================================
// Default Values
// ============================================================================

/// Default query pattern (matches first engine in DEFAULT_SEARCH_ENGINE)
pub const DEFAULT_QUERY_PATTERN: &str = DUCKDUCKGO_QUERY_PATTERN;
