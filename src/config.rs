use crate::constants::{
    DEFAULT_QUERY_PATTERN, DEFAULT_SEARCH_LIMIT, DEFAULT_SEARCH_MODE, DEFAULT_TIMEOUT_SECS,
    ENV_TARZI_FETCHER_FORMAT, ENV_TARZI_FETCHER_MODE, ENV_TARZI_FETCHER_TIMEOUT,
    ENV_TARZI_LOG_LEVEL, ENV_TARZI_PROXY, ENV_TARZI_QUERY_PATTERN, ENV_TARZI_SEARCH_ENGINE,
    ENV_TARZI_SEARCH_LIMIT, ENV_TARZI_SEARCH_MODE, ENV_TARZI_TIMEOUT, ENV_TARZI_USER_AGENT,
    ENV_TARZI_WEB_DRIVER, ENV_TARZI_WEB_DRIVER_URL, FETCHER_MODE_BROWSER_HEADLESS, FORMAT_MARKDOWN,
    LOG_LEVEL_INFO, SEARCH_ENGINE_BING,
};
use crate::{Result, error::TarziError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub fetcher: FetcherConfig,
    #[serde(default)]
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetcherConfig {
    #[serde(default = "default_fetch_mode")]
    pub mode: String,
    #[serde(default = "default_fetch_format")]
    pub format: String,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    #[serde(default = "default_fetch_timeout")]
    pub timeout: u64,
    pub proxy: Option<String>,
    #[serde(default = "default_web_driver")]
    pub web_driver: String,
    pub web_driver_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_search_engine")]
    pub engine: String,
    #[serde(default = "default_query_pattern")]
    pub query_pattern: String,
    #[serde(default = "default_result_limit")]
    pub limit: usize,
    /// Search access mode: auto | apiquery | webquery
    #[serde(default = "default_search_mode")]
    pub mode: String,
    /// Optional API key for the configured engine (programmatic only).
    /// Prefer engine-specific env vars: `BRAVE_API_KEY`, `SERPER_API_KEY`.
    pub api_key: Option<String>,
}

/// CLI configuration parameters that can override config values
#[derive(Debug, Clone)]
pub struct CliConfigParams {
    pub fetcher_format: Option<String>,
    pub search_limit: Option<usize>,
    pub search_engine: Option<String>,
}

impl CliConfigParams {
    pub fn new() -> Self {
        Self {
            fetcher_format: None,
            search_limit: None,
            search_engine: None,
        }
    }
}

impl Default for CliConfigParams {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            general: GeneralConfig::default(),
            fetcher: FetcherConfig::default(),
            search: SearchConfig::default(),
        }
    }

    /// Load configuration with proper precedence order:
    /// 1. CLI parameters (highest priority — applied by callers via `apply_cli_params`)
    /// 2. Environment variables (`TARZI_*` and related)
    /// 3. Default values (lowest priority)
    pub fn load() -> Result<Self> {
        let mut config = Config::new();
        config.apply_env()?;
        Ok(config)
    }

    /// Apply `TARZI_*` environment variables onto this config.
    /// Unset variables are left unchanged. Invalid numeric values return an error.
    pub fn apply_env(&mut self) -> Result<()> {
        if let Ok(v) = std::env::var(ENV_TARZI_LOG_LEVEL)
            && !v.is_empty()
        {
            self.general.log_level = v;
        }
        if let Some(v) = parse_env_u64(ENV_TARZI_TIMEOUT)? {
            self.general.timeout = v;
        }

        if let Ok(v) = std::env::var(ENV_TARZI_FETCHER_MODE)
            && !v.is_empty()
        {
            self.fetcher.mode = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_FETCHER_FORMAT)
            && !v.is_empty()
        {
            self.fetcher.format = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_USER_AGENT)
            && !v.is_empty()
        {
            self.fetcher.user_agent = v;
        }
        if let Some(v) = parse_env_u64(ENV_TARZI_FETCHER_TIMEOUT)? {
            self.fetcher.timeout = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_PROXY)
            && !v.is_empty()
        {
            self.fetcher.proxy = Some(v);
        }
        if let Ok(v) = std::env::var(ENV_TARZI_WEB_DRIVER)
            && !v.is_empty()
        {
            self.fetcher.web_driver = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_WEB_DRIVER_URL)
            && !v.is_empty()
        {
            self.fetcher.web_driver_url = Some(v);
        }

        if let Ok(v) = std::env::var(ENV_TARZI_SEARCH_ENGINE)
            && !v.is_empty()
        {
            self.search.engine = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_QUERY_PATTERN)
            && !v.is_empty()
        {
            self.search.query_pattern = v;
        }
        if let Some(v) = parse_env_usize(ENV_TARZI_SEARCH_LIMIT)? {
            self.search.limit = v;
        }
        if let Ok(v) = std::env::var(ENV_TARZI_SEARCH_MODE)
            && !v.is_empty()
        {
            self.search.mode = v;
        }

        Ok(())
    }

    /// Merge another config into this one (other config takes precedence for non-default fields)
    pub fn merge(&mut self, other: &Config) {
        if other.general.log_level != default_log_level() {
            self.general.log_level = other.general.log_level.clone();
        }
        if other.general.timeout != default_timeout() {
            self.general.timeout = other.general.timeout;
        }

        if other.fetcher.mode != default_fetch_mode() {
            self.fetcher.mode = other.fetcher.mode.clone();
        }
        if other.fetcher.format != default_fetch_format() {
            self.fetcher.format = other.fetcher.format.clone();
        }
        if other.fetcher.user_agent != default_user_agent() {
            self.fetcher.user_agent = other.fetcher.user_agent.clone();
        }
        if other.fetcher.timeout != default_fetch_timeout() {
            self.fetcher.timeout = other.fetcher.timeout;
        }
        if other.fetcher.proxy.is_some() {
            self.fetcher.proxy = other.fetcher.proxy.clone();
        }
        if other.fetcher.web_driver != default_web_driver() {
            self.fetcher.web_driver = other.fetcher.web_driver.clone();
        }
        if other.fetcher.web_driver_url.is_some() {
            self.fetcher.web_driver_url = other.fetcher.web_driver_url.clone();
        }

        if other.search.engine != default_search_engine() {
            self.search.engine = other.search.engine.clone();
        }
        if other.search.limit != default_result_limit() {
            self.search.limit = other.search.limit;
        }
        if other.search.query_pattern != default_query_pattern() {
            self.search.query_pattern = other.search.query_pattern.clone();
        }
        if other.search.mode != default_search_mode() {
            self.search.mode = other.search.mode.clone();
        }
        if other.search.api_key.is_some() {
            self.search.api_key = other.search.api_key.clone();
        }
    }

    /// Apply CLI parameters to config (highest priority)
    pub fn apply_cli_params(&mut self, cli_params: &CliConfigParams) {
        if let Some(format) = &cli_params.fetcher_format {
            self.fetcher.format = format.clone();
        }
        if let Some(limit) = cli_params.search_limit {
            self.search.limit = limit;
        }
        if let Some(engine) = &cli_params.search_engine {
            self.search.engine = engine.clone();
        }
    }
}

fn parse_env_u64(name: &str) -> Result<Option<u64>> {
    match std::env::var(name) {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => v.parse::<u64>().map(Some).map_err(|_| {
            TarziError::Config(format!(
                "Invalid {name} value '{v}': expected unsigned integer"
            ))
        }),
        Err(_) => Ok(None),
    }
}

fn parse_env_usize(name: &str) -> Result<Option<usize>> {
    match std::env::var(name) {
        Ok(v) if v.is_empty() => Ok(None),
        Ok(v) => v.parse::<usize>().map(Some).map_err(|_| {
            TarziError::Config(format!(
                "Invalid {name} value '{v}': expected unsigned integer"
            ))
        }),
        Err(_) => Ok(None),
    }
}

// Default implementations
impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            timeout: default_timeout(),
        }
    }
}

impl Default for FetcherConfig {
    fn default() -> Self {
        Self {
            mode: default_fetch_mode(),
            format: default_fetch_format(),
            user_agent: default_user_agent(),
            timeout: default_fetch_timeout(),
            proxy: None,
            web_driver: default_web_driver(),
            web_driver_url: None,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            engine: default_search_engine(),
            query_pattern: default_query_pattern(),
            limit: default_result_limit(),
            mode: default_search_mode(),
            api_key: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// Default value functions
fn default_log_level() -> String {
    LOG_LEVEL_INFO.to_string()
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

fn default_fetch_mode() -> String {
    FETCHER_MODE_BROWSER_HEADLESS.to_string()
}

fn default_fetch_format() -> String {
    FORMAT_MARKDOWN.to_string()
}

fn default_user_agent() -> String {
    crate::constants::DEFAULT_USER_AGENT.to_string()
}

fn default_fetch_timeout() -> u64 {
    30
}

fn default_search_engine() -> String {
    SEARCH_ENGINE_BING.to_string()
}

fn default_query_pattern() -> String {
    DEFAULT_QUERY_PATTERN.to_string()
}

fn default_result_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

fn default_search_mode() -> String {
    DEFAULT_SEARCH_MODE.to_string()
}

fn default_web_driver() -> String {
    "chromedriver".to_string()
}

/// Get proxy configuration with environment variable override
/// Environment variables checked in order: HTTPS_PROXY, HTTP_PROXY, https_proxy, http_proxy
/// Falls back to config.proxy if no environment variables are set
pub fn get_proxy_from_env_or_config(config_proxy: &Option<String>) -> Option<String> {
    let env_vars = ["HTTPS_PROXY", "HTTP_PROXY", "https_proxy", "http_proxy"];

    for env_var in &env_vars {
        if let Ok(proxy) = std::env::var(env_var)
            && !proxy.is_empty()
        {
            return Some(proxy);
        }
    }

    config_proxy.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::*;
    use std::sync::Mutex;

    /// Serialize env mutation across config unit tests
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env_lock<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        f();
    }

    #[test]
    fn test_default_config() {
        let config = Config::new();

        assert_eq!(config.general.log_level, LOG_LEVEL_INFO);
        assert_eq!(config.general.timeout, DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.fetcher.mode, FETCHER_MODE_BROWSER_HEADLESS);
        assert_eq!(config.fetcher.format, FORMAT_MARKDOWN);
        assert_eq!(
            config.fetcher.user_agent,
            crate::constants::DEFAULT_USER_AGENT
        );
        assert_eq!(config.fetcher.timeout, 30);
        assert_eq!(config.search.engine, SEARCH_ENGINE_BING);
        assert_eq!(config.search.query_pattern, DEFAULT_QUERY_PATTERN);
        assert_eq!(config.search.limit, DEFAULT_SEARCH_LIMIT);
        assert_eq!(config.search.mode, DEFAULT_SEARCH_MODE);
        assert!(config.search.api_key.is_none());
    }

    #[test]
    fn test_search_mode_and_api_key_from_toml() {
        let toml_str = r#"
[search]
engine = "brave"
mode = "apiquery"
api_key = "test-brave-key"
limit = 7
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.search.engine, SEARCH_ENGINE_BRAVE);
        assert_eq!(config.search.mode, SEARCH_MODE_APIQUERY);
        assert_eq!(config.search.api_key.as_deref(), Some("test-brave-key"));
        assert_eq!(config.search.limit, 7);
    }

    #[test]
    fn test_search_mode_defaults_to_auto() {
        let toml_str = r#"
[search]
engine = "google_serper"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.search.engine, SEARCH_ENGINE_GOOGLE_SERPER);
        assert_eq!(config.search.mode, DEFAULT_SEARCH_MODE);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::new();
        config.search.limit = DEFAULT_SEARCH_LIMIT;
        config.fetcher.mode = FETCHER_MODE_HEAD.to_string();

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed_config: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed_config.search.limit, DEFAULT_SEARCH_LIMIT);
        assert_eq!(parsed_config.fetcher.mode, FETCHER_MODE_HEAD);
    }

    #[test]
    fn test_config_with_custom_values() {
        let config_str = r#"
[general]
log_level = "debug"
timeout = 60

[fetcher]
mode = "head"
format = "json"
user_agent = "Custom User Agent"
timeout = 45
proxy = "http://example.com:8080"
web_driver = "chrome"
web_driver_url = "http://example.com/driver"

[search]
engine = "google.com"
query_pattern = ".*"
limit = 5
"#;

        let config: Config = toml::from_str(config_str).unwrap();

        assert_eq!(config.general.log_level, "debug");
        assert_eq!(config.general.timeout, 60);
        assert_eq!(config.fetcher.mode, FETCHER_MODE_HEAD);
        assert_eq!(config.fetcher.format, FORMAT_JSON);
        assert_eq!(config.fetcher.user_agent, "Custom User Agent");
        assert_eq!(config.fetcher.timeout, 45);
        assert_eq!(
            config.fetcher.proxy,
            Some("http://example.com:8080".to_string())
        );
        assert_eq!(config.fetcher.web_driver, "chrome");
        assert_eq!(
            config.fetcher.web_driver_url,
            Some("http://example.com/driver".to_string())
        );
        assert_eq!(config.search.engine, "google.com");
        assert_eq!(config.search.query_pattern, ".*");
        assert_eq!(config.search.limit, 5);
    }

    #[test]
    fn test_config_with_only_web_driver_url() {
        let config_str = r#"
[fetcher]
web_driver_url = "http://localhost:9999"
"#;
        let config: Config = toml::from_str(config_str).unwrap();
        assert_eq!(config.fetcher.web_driver, CHROMEDRIVER);
        assert_eq!(
            config.fetcher.web_driver_url,
            Some("http://localhost:9999".to_string())
        );
    }

    #[test]
    fn test_apply_env_overrides() {
        with_env_lock(|| {
            let keys = [
                ENV_TARZI_LOG_LEVEL,
                ENV_TARZI_TIMEOUT,
                ENV_TARZI_FETCHER_MODE,
                ENV_TARZI_FETCHER_FORMAT,
                ENV_TARZI_USER_AGENT,
                ENV_TARZI_FETCHER_TIMEOUT,
                ENV_TARZI_PROXY,
                ENV_TARZI_WEB_DRIVER,
                ENV_TARZI_WEB_DRIVER_URL,
                ENV_TARZI_SEARCH_ENGINE,
                ENV_TARZI_QUERY_PATTERN,
                ENV_TARZI_SEARCH_LIMIT,
                ENV_TARZI_SEARCH_MODE,
            ];
            let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
            unsafe {
                for &k in &keys {
                    std::env::remove_var(k);
                }
            }

            unsafe {
                std::env::set_var(ENV_TARZI_LOG_LEVEL, "debug");
                std::env::set_var(ENV_TARZI_TIMEOUT, "90");
                std::env::set_var(ENV_TARZI_FETCHER_MODE, "plain_request");
                std::env::set_var(ENV_TARZI_FETCHER_FORMAT, "json");
                std::env::set_var(ENV_TARZI_USER_AGENT, "EnvAgent/1.0");
                std::env::set_var(ENV_TARZI_FETCHER_TIMEOUT, "45");
                std::env::set_var(ENV_TARZI_PROXY, "http://env-proxy:8080");
                std::env::set_var(ENV_TARZI_WEB_DRIVER, "geckodriver");
                std::env::set_var(ENV_TARZI_WEB_DRIVER_URL, "http://localhost:4444");
                std::env::set_var(ENV_TARZI_SEARCH_ENGINE, "brave");
                std::env::set_var(ENV_TARZI_QUERY_PATTERN, "https://example.com?q={query}");
                std::env::set_var(ENV_TARZI_SEARCH_LIMIT, "12");
                std::env::set_var(ENV_TARZI_SEARCH_MODE, "apiquery");
            }

            let config = Config::load().unwrap();

            assert_eq!(config.general.log_level, "debug");
            assert_eq!(config.general.timeout, 90);
            assert_eq!(config.fetcher.mode, "plain_request");
            assert_eq!(config.fetcher.format, "json");
            assert_eq!(config.fetcher.user_agent, "EnvAgent/1.0");
            assert_eq!(config.fetcher.timeout, 45);
            assert_eq!(
                config.fetcher.proxy,
                Some("http://env-proxy:8080".to_string())
            );
            assert_eq!(config.fetcher.web_driver, "geckodriver");
            assert_eq!(
                config.fetcher.web_driver_url,
                Some("http://localhost:4444".to_string())
            );
            assert_eq!(config.search.engine, "brave");
            assert_eq!(config.search.query_pattern, "https://example.com?q={query}");
            assert_eq!(config.search.limit, 12);
            assert_eq!(config.search.mode, "apiquery");
            assert!(config.search.api_key.is_none());

            unsafe {
                for &k in &keys {
                    std::env::remove_var(k);
                }
                for (k, v) in originals {
                    if let Some(val) = v {
                        std::env::set_var(k, val);
                    }
                }
            }
        });
    }

    #[test]
    fn test_apply_env_invalid_number() {
        with_env_lock(|| {
            let original = std::env::var(ENV_TARZI_SEARCH_LIMIT).ok();
            unsafe {
                std::env::set_var(ENV_TARZI_SEARCH_LIMIT, "not-a-number");
            }

            let result = Config::load();
            assert!(result.is_err());

            unsafe {
                std::env::remove_var(ENV_TARZI_SEARCH_LIMIT);
                if let Some(val) = original {
                    std::env::set_var(ENV_TARZI_SEARCH_LIMIT, val);
                }
            }
        });
    }

    #[test]
    fn test_load_defaults_without_env() {
        with_env_lock(|| {
            let keys = [
                ENV_TARZI_LOG_LEVEL,
                ENV_TARZI_TIMEOUT,
                ENV_TARZI_FETCHER_MODE,
                ENV_TARZI_FETCHER_FORMAT,
                ENV_TARZI_USER_AGENT,
                ENV_TARZI_FETCHER_TIMEOUT,
                ENV_TARZI_PROXY,
                ENV_TARZI_WEB_DRIVER,
                ENV_TARZI_WEB_DRIVER_URL,
                ENV_TARZI_SEARCH_ENGINE,
                ENV_TARZI_QUERY_PATTERN,
                ENV_TARZI_SEARCH_LIMIT,
                ENV_TARZI_SEARCH_MODE,
            ];
            let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
            unsafe {
                for &k in &keys {
                    std::env::remove_var(k);
                }
            }

            let config = Config::load().unwrap();
            assert_eq!(config.search.engine, SEARCH_ENGINE_BING);
            assert_eq!(config.search.mode, DEFAULT_SEARCH_MODE);
            assert!(config.fetcher.proxy.is_none());
            assert!(config.search.api_key.is_none());

            unsafe {
                for (k, v) in originals {
                    if let Some(val) = v {
                        std::env::set_var(k, val);
                    }
                }
            }
        });
    }

    #[test]
    fn test_get_proxy_from_env_or_config() {
        with_env_lock(|| {
            let original_http_proxy = std::env::var("HTTP_PROXY").ok();
            let original_https_proxy = std::env::var("HTTPS_PROXY").ok();
            let original_http_proxy_lower = std::env::var("http_proxy").ok();
            let original_https_proxy_lower = std::env::var("https_proxy").ok();

            unsafe {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                std::env::remove_var("http_proxy");
                std::env::remove_var("https_proxy");
            }

            let result = get_proxy_from_env_or_config(&None);
            assert_eq!(result, None);

            let config_proxy = Some("http://config-proxy:8080".to_string());
            let result = get_proxy_from_env_or_config(&config_proxy);
            assert_eq!(result, config_proxy);

            unsafe {
                std::env::set_var("HTTP_PROXY", "http://env-proxy:8080");
            }
            let result = get_proxy_from_env_or_config(&config_proxy);
            assert_eq!(result, Some("http://env-proxy:8080".to_string()));

            unsafe {
                std::env::set_var("HTTPS_PROXY", "http://https-proxy:8080");
            }
            let result = get_proxy_from_env_or_config(&config_proxy);
            assert_eq!(result, Some("http://https-proxy:8080".to_string()));

            unsafe {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                std::env::set_var("http_proxy", "http://lowercase-proxy:8080");
            }
            let result = get_proxy_from_env_or_config(&config_proxy);
            assert_eq!(result, Some("http://lowercase-proxy:8080".to_string()));

            unsafe {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                std::env::remove_var("http_proxy");
                std::env::remove_var("https_proxy");

                if let Some(val) = original_http_proxy {
                    std::env::set_var("HTTP_PROXY", val);
                }
                if let Some(val) = original_https_proxy {
                    std::env::set_var("HTTPS_PROXY", val);
                }
                if let Some(val) = original_http_proxy_lower {
                    std::env::set_var("http_proxy", val);
                }
                if let Some(val) = original_https_proxy_lower {
                    std::env::set_var("https_proxy", val);
                }
            }
        });
    }

    #[test]
    fn test_get_proxy_from_env_or_config_empty_env() {
        with_env_lock(|| {
            let original_http_proxy = std::env::var("HTTP_PROXY").ok();
            let original_https_proxy = std::env::var("HTTPS_PROXY").ok();
            let original_http_proxy_lower = std::env::var("http_proxy").ok();
            let original_https_proxy_lower = std::env::var("https_proxy").ok();

            unsafe {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                std::env::remove_var("http_proxy");
                std::env::remove_var("https_proxy");
                std::env::set_var("HTTP_PROXY", "");
            }
            let config_proxy = Some("http://config-proxy:8080".to_string());
            let result = get_proxy_from_env_or_config(&config_proxy);
            assert_eq!(result, config_proxy);

            unsafe {
                std::env::remove_var("HTTP_PROXY");
                std::env::remove_var("HTTPS_PROXY");
                std::env::remove_var("http_proxy");
                std::env::remove_var("https_proxy");

                if let Some(val) = original_http_proxy {
                    std::env::set_var("HTTP_PROXY", val);
                }
                if let Some(val) = original_https_proxy {
                    std::env::set_var("HTTPS_PROXY", val);
                }
                if let Some(val) = original_http_proxy_lower {
                    std::env::set_var("http_proxy", val);
                }
                if let Some(val) = original_https_proxy_lower {
                    std::env::set_var("https_proxy", val);
                }
            }
        });
    }

    #[test]
    fn test_cli_params_override() {
        let mut config = Config::new();

        config.fetcher.mode = FETCHER_MODE_BROWSER_HEADLESS.to_string();
        config.fetcher.format = FORMAT_MARKDOWN.to_string();
        config.search.limit = DEFAULT_SEARCH_LIMIT;
        config.search.engine = SEARCH_ENGINE_BING.to_string();

        let mut cli_params = CliConfigParams::new();
        cli_params.fetcher_format = Some(FORMAT_JSON.to_string());
        cli_params.search_limit = Some(DEFAULT_SEARCH_LIMIT);
        cli_params.search_engine = Some(SEARCH_ENGINE_GOOGLE.to_string());

        config.apply_cli_params(&cli_params);

        assert_eq!(config.fetcher.mode, FETCHER_MODE_BROWSER_HEADLESS);
        assert_eq!(config.fetcher.format, FORMAT_JSON);
        assert_eq!(config.search.limit, DEFAULT_SEARCH_LIMIT);
        assert_eq!(config.search.engine, SEARCH_ENGINE_GOOGLE);
    }

    #[test]
    fn test_cli_overrides_env() {
        with_env_lock(|| {
            let original_engine = std::env::var(ENV_TARZI_SEARCH_ENGINE).ok();
            let original_format = std::env::var(ENV_TARZI_FETCHER_FORMAT).ok();
            unsafe {
                std::env::set_var(ENV_TARZI_SEARCH_ENGINE, "brave");
                std::env::set_var(ENV_TARZI_FETCHER_FORMAT, "html");
            }

            let mut config = Config::load().unwrap();
            assert_eq!(config.search.engine, "brave");
            assert_eq!(config.fetcher.format, "html");

            let mut cli_params = CliConfigParams::new();
            cli_params.search_engine = Some(SEARCH_ENGINE_GOOGLE.to_string());
            cli_params.fetcher_format = Some(FORMAT_JSON.to_string());
            config.apply_cli_params(&cli_params);

            assert_eq!(config.search.engine, SEARCH_ENGINE_GOOGLE);
            assert_eq!(config.fetcher.format, FORMAT_JSON);

            unsafe {
                std::env::remove_var(ENV_TARZI_SEARCH_ENGINE);
                std::env::remove_var(ENV_TARZI_FETCHER_FORMAT);
                if let Some(val) = original_engine {
                    std::env::set_var(ENV_TARZI_SEARCH_ENGINE, val);
                }
                if let Some(val) = original_format {
                    std::env::set_var(ENV_TARZI_FETCHER_FORMAT, val);
                }
            }
        });
    }

    #[test]
    fn test_config_merge() {
        let mut base_config = Config::new();

        base_config.general.log_level = LOG_LEVEL_INFO.to_string();
        base_config.fetcher.mode = FETCHER_MODE_BROWSER_HEADLESS.to_string();
        base_config.search.engine = SEARCH_ENGINE_BING.to_string();

        let override_config = Config {
            general: GeneralConfig {
                log_level: LOG_LEVEL_DEBUG.to_string(),
                timeout: 60,
            },
            fetcher: FetcherConfig {
                mode: FETCHER_MODE_PLAIN_REQUEST.to_string(),
                format: FORMAT_JSON.to_string(),
                user_agent: "Custom Agent".to_string(),
                timeout: 45,
                proxy: Some("http://proxy:8080".to_string()),
                web_driver: CHROMEDRIVER.to_string(),
                web_driver_url: Some("http://localhost:4444".to_string()),
            },
            search: SearchConfig {
                engine: SEARCH_ENGINE_GOOGLE.to_string(),
                query_pattern: "custom pattern".to_string(),
                limit: DEFAULT_SEARCH_LIMIT,
                mode: DEFAULT_SEARCH_MODE.to_string(),
                api_key: None,
            },
        };

        base_config.merge(&override_config);

        assert_eq!(base_config.general.log_level, LOG_LEVEL_DEBUG);
        assert_eq!(base_config.general.timeout, 60);
        assert_eq!(base_config.fetcher.mode, FETCHER_MODE_PLAIN_REQUEST);
        assert_eq!(base_config.fetcher.format, FORMAT_JSON);
        assert_eq!(base_config.fetcher.user_agent, "Custom Agent");
        assert_eq!(base_config.fetcher.timeout, 45);
        assert_eq!(
            base_config.fetcher.proxy,
            Some("http://proxy:8080".to_string())
        );
        assert_eq!(base_config.fetcher.web_driver, CHROMEDRIVER);
        assert_eq!(
            base_config.fetcher.web_driver_url,
            Some("http://localhost:4444".to_string())
        );
        assert_eq!(base_config.search.engine, SEARCH_ENGINE_GOOGLE);
        assert_eq!(base_config.search.query_pattern, "custom pattern");
        assert_eq!(base_config.search.limit, DEFAULT_SEARCH_LIMIT);
    }
}
