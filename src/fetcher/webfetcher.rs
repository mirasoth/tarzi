use crate::{
    Result,
    config::Config,
    constants::{DEFAULT_TIMEOUT, DEFAULT_USER_AGENT, PAGE_LOAD_WAIT},
    converter::{Converter, Format},
    error::TarziError,
};
use reqwest::Client;
use tracing::{error, info, warn};
use url::Url;

use super::{browser::BrowserManager, types::FetchMode};

/// Main web content fetcher
#[derive(Debug)]
pub struct WebFetcher {
    http_client: Client,
    browser_manager: BrowserManager,
    converter: Converter,
}

impl WebFetcher {
    pub fn new() -> Self {
        info!("Initializing WebFetcher");
        let http_client = Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(DEFAULT_USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        info!("HTTP client created successfully for WebFetcher");
        Self {
            http_client,
            browser_manager: BrowserManager::new(),
            converter: Converter::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        info!("Initializing WebFetcher from config");
        let mut client_builder = Client::builder()
            .timeout(std::time::Duration::from_secs(config.fetcher.timeout))
            .user_agent(&config.fetcher.user_agent);

        // Use environment variables for proxy with fallback to config
        let proxy = crate::config::get_proxy_from_env_or_config(&config.fetcher.proxy);
        if let Some(proxy) = proxy
            && !proxy.is_empty()
        {
            if let Ok(proxy_obj) = reqwest::Proxy::http(&proxy) {
                client_builder = client_builder.proxy(proxy_obj);
                info!("Using proxy from environment/config: {}", proxy);
            } else {
                warn!("Invalid proxy configuration: {}", proxy);
            }
        }

        let http_client = client_builder
            .build()
            .expect("Failed to create HTTP client from config");
        Self {
            http_client,
            browser_manager: BrowserManager::from_config(config),
            converter: Converter::new(),
        }
    }

    /// Fetch content from URL and convert to specified format
    pub async fn fetch(&mut self, url: &str, mode: FetchMode, format: Format) -> Result<String> {
        let raw_content = self.fetch_raw(url, mode).await?;
        let converted_content = self.converter.convert(&raw_content, format).await?;
        Ok(converted_content)
    }

    /// Get raw content without conversion (for internal use)
    pub async fn fetch_raw(&mut self, url: &str, mode: FetchMode) -> Result<String> {
        match mode {
            FetchMode::PlainRequest => self.fetch_plain_request(url).await,
            FetchMode::BrowserHead => self.fetch_with_browser(url, false).await,
            FetchMode::BrowserHeadless => self.fetch_with_browser(url, true).await,
        }
    }

    /// Fetch raw content using plain HTTP request (no JS rendering)
    async fn fetch_plain_request(&self, url: &str) -> Result<String> {
        let url = Url::parse(url)?;
        let response = self.http_client.get(url).send().await?;
        let response = response.error_for_status()?;
        let content = response.text().await?;
        Ok(content)
    }

    /// GET with custom headers (used by search API clients).
    pub async fn fetch_get_with_headers(
        &self,
        url: &str,
        headers: &[(&str, &str)],
    ) -> Result<String> {
        let url = Url::parse(url)?;
        let mut request = self.http_client.get(url);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await?;
        let response = response.error_for_status()?;
        Ok(response.text().await?)
    }

    /// POST JSON with custom headers (used by search API clients).
    pub async fn fetch_post_json_with_headers(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &serde_json::Value,
    ) -> Result<String> {
        let url = Url::parse(url)?;
        let mut request = self.http_client.post(url).json(body);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await?;
        let response = response.error_for_status()?;
        Ok(response.text().await?)
    }

    /// Fetch content using browser (with or without headless mode)
    async fn fetch_with_browser(&mut self, url: &str, headless: bool) -> Result<String> {
        info!(
            "Fetching URL with browser (headless: {}): {}",
            headless, url
        );

        // Get or create browser instance
        info!("Getting or creating browser instance...");
        let browser = self.browser_manager.get_or_create_browser(headless).await?;
        info!("Using existing browser instance for fetching");

        // Navigate to the URL
        info!("Navigating to URL: {}", url);
        let navigation_result = tokio::time::timeout(DEFAULT_TIMEOUT, browser.get(url)).await;

        match navigation_result {
            Ok(Ok(_)) => {
                info!("Successfully navigated to page");
            }
            Ok(Err(e)) => {
                error!("Failed to navigate to URL: {}", e);
                // Check if it's a network-related error and provide more specific guidance
                let error_msg = if e.to_string().contains("nssFailure")
                    || e.to_string().contains("network")
                {
                    format!(
                        "Network error while navigating to {url}: {e}. This may be due to network connectivity issues, firewall restrictions, or the site being temporarily unavailable."
                    )
                } else {
                    format!("Failed to navigate to {url}: {e}")
                };
                return Err(TarziError::Browser(error_msg));
            }
            Err(_) => {
                error!("Timeout while navigating to URL (30 seconds)");
                return Err(TarziError::Browser(format!(
                    "Timeout while navigating to {url} (30 seconds). The page may be slow to load or the site may be experiencing issues."
                )));
            }
        }

        // Wait for the page to load (simplified approach)
        info!("Waiting for page to load (2 seconds)...");
        tokio::time::sleep(PAGE_LOAD_WAIT).await;
        info!("Wait completed");

        // Get the page content (prefer dynamic DOM via JS execution, fallback to page source)
        info!("Extracting page content (dynamic DOM if available)...");
        let content = match WebFetcher::get_outer_html_from(browser).await {
            Ok(html) => html,
            Err(e) => {
                warn!(
                    "Falling back to page source due to error getting dynamic DOM: {}",
                    e
                );
                let content_result = tokio::time::timeout(DEFAULT_TIMEOUT, browser.source()).await;
                match content_result {
                    Ok(Ok(content)) => content,
                    Ok(Err(e)) => {
                        error!("Failed to get page content: {}", e);
                        return Err(TarziError::Browser(format!("Failed to get content: {e}")));
                    }
                    Err(_) => {
                        error!("Timeout while extracting page content (30 seconds)");
                        return Err(TarziError::Browser(
                            "Timeout while extracting page content".to_string(),
                        ));
                    }
                }
            }
        };

        info!(
            "Successfully extracted page content ({} characters)",
            content.len()
        );
        Ok(content)
    }

    /// Fetch content using proxy
    pub async fn fetch_with_proxy(
        &mut self,
        url: &str,
        proxy: &str,
        mode: FetchMode,
        format: Format,
    ) -> Result<String> {
        info!("Fetching URL with proxy: {} (proxy: {})", url, proxy);

        let raw_content = match mode {
            FetchMode::PlainRequest => {
                let proxy_client = match reqwest::Proxy::http(proxy) {
                    Ok(proxy_config) => {
                        match Client::builder()
                            .timeout(DEFAULT_TIMEOUT)
                            .user_agent(DEFAULT_USER_AGENT)
                            .proxy(proxy_config)
                            .build()
                        {
                            Ok(client) => client,
                            Err(e) => {
                                warn!(
                                    "Failed to create HTTP client with proxy '{}': {}. Falling back to no proxy.",
                                    proxy, e
                                );
                                return Err(TarziError::Config(format!(
                                    "Failed to create proxy client: {e}"
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Invalid proxy URL '{}': {}. Falling back to no proxy.",
                            proxy, e
                        );
                        return Err(TarziError::Config(format!("Invalid proxy URL: {e}")));
                    }
                };

                let url = Url::parse(url)?;
                let response = proxy_client.get(url).send().await?;
                let response = response.error_for_status()?;
                response.text().await?
            }
            FetchMode::BrowserHead | FetchMode::BrowserHeadless => {
                // For browser modes with proxy, create a new browser instance with proxy configuration
                info!("Creating browser with proxy for fetching: {}", proxy);
                let headless = matches!(mode, FetchMode::BrowserHeadless);
                let instance_id = self
                    .browser_manager
                    .create_browser_with_proxy(
                        None,
                        headless,
                        Some("proxy_browser".to_string()),
                        Some(proxy.to_string()),
                    )
                    .await?;

                // Get the browser instance and fetch content
                let browser = self
                    .browser_manager
                    .get_browser(&instance_id)
                    .ok_or_else(|| {
                        TarziError::Browser("Failed to get proxy browser instance".to_string())
                    })?;

                // Navigate to URL
                let navigation_result =
                    tokio::time::timeout(DEFAULT_TIMEOUT, browser.get(url)).await;
                match navigation_result {
                    Ok(Ok(_)) => info!("Successfully navigated to page with proxy"),
                    Ok(Err(e)) => {
                        error!("Failed to navigate to URL with proxy: {}", e);
                        return Err(TarziError::Browser(format!(
                            "Failed to navigate with proxy: {e}"
                        )));
                    }
                    Err(_) => {
                        error!("Timeout while navigating to URL with proxy");
                        return Err(TarziError::Browser(
                            "Timeout while navigating with proxy".to_string(),
                        ));
                    }
                }

                // Wait for page load
                tokio::time::sleep(PAGE_LOAD_WAIT).await;

                // Get page content (prefer dynamic DOM via JS execution, fallback to page source)
                let content = match WebFetcher::get_outer_html_from(browser).await {
                    Ok(html) => html,
                    Err(e) => {
                        warn!(
                            "Falling back to page source (proxy) due to error getting dynamic DOM: {}",
                            e
                        );
                        let content_result =
                            tokio::time::timeout(DEFAULT_TIMEOUT, browser.source()).await;
                        match content_result {
                            Ok(Ok(content)) => content,
                            Ok(Err(e)) => {
                                error!("Failed to get page content with proxy: {}", e);
                                return Err(TarziError::Browser(format!(
                                    "Failed to get content with proxy: {e}"
                                )));
                            }
                            Err(_) => {
                                error!("Timeout while extracting page content with proxy");
                                return Err(TarziError::Browser(
                                    "Timeout while extracting content with proxy".to_string(),
                                ));
                            }
                        }
                    }
                };

                // Clean up the proxy browser instance
                if let Err(e) = self.browser_manager.remove_browser(&instance_id).await {
                    warn!("Failed to cleanup proxy browser instance: {}", e);
                }

                content
            }
        };

        // Convert to specified format
        let converted_content = self.converter.convert(&raw_content, format).await?;
        Ok(converted_content)
    }

    /// Create a new browser instance with a specific user data directory
    pub async fn create_browser_with_user_data(
        &mut self,
        user_data_dir: Option<std::path::PathBuf>,
        headless: bool,
        instance_id: Option<String>,
    ) -> Result<String> {
        self.browser_manager
            .create_browser_with_user_data(user_data_dir, headless, instance_id)
            .await
    }

    /// Create a new browser instance with explicit proxy configuration
    pub async fn create_browser_with_proxy(
        &mut self,
        user_data_dir: Option<std::path::PathBuf>,
        headless: bool,
        instance_id: Option<String>,
        proxy: Option<String>,
    ) -> Result<String> {
        self.browser_manager
            .create_browser_with_proxy(user_data_dir, headless, instance_id, proxy)
            .await
    }

    /// Get a browser instance by ID
    pub fn get_browser(&self, instance_id: &str) -> Option<&thirtyfour::WebDriver> {
        self.browser_manager.get_browser(instance_id)
    }

    /// Get all browser instance IDs
    pub fn get_browser_ids(&self) -> Vec<String> {
        self.browser_manager.get_browser_ids()
    }

    /// Remove a browser instance by ID
    pub async fn remove_browser(&mut self, instance_id: &str) -> Result<()> {
        self.browser_manager.remove_browser(instance_id).await?;
        Ok(())
    }

    /// Fetch content from a specific browser instance
    pub async fn fetch_with_browser_instance(
        &mut self,
        url: &str,
        instance_id: &str,
        format: Format,
    ) -> Result<String> {
        info!(
            "Fetching URL with browser instance {}: {}",
            instance_id, url
        );

        // Get the browser instance
        let browser = self
            .browser_manager
            .get_browser(instance_id)
            .ok_or_else(|| {
                TarziError::Browser(format!("Browser instance {instance_id} not found"))
            })?;

        info!("Using browser instance {} for fetching", instance_id);

        // Navigate to the URL
        info!(
            "Navigating to URL in browser instance {}: {}",
            instance_id, url
        );
        let navigation_result = tokio::time::timeout(DEFAULT_TIMEOUT, browser.get(url)).await;

        match navigation_result {
            Ok(Ok(_)) => {
                info!(
                    "Successfully navigated to page in browser instance {}",
                    instance_id
                );
            }
            Ok(Err(e)) => {
                error!(
                    "Failed to navigate to URL in browser instance {}: {}",
                    instance_id, e
                );
                return Err(TarziError::Browser(format!("Failed to navigate: {e}")));
            }
            Err(_) => {
                error!(
                    "Timeout while navigating to URL in browser instance {} (30 seconds)",
                    instance_id
                );
                return Err(TarziError::Browser(
                    "Timeout while navigating to URL".to_string(),
                ));
            }
        }

        // Wait for the page to load (simplified approach)
        info!(
            "Waiting for page to load in browser instance {} (2 seconds)...",
            instance_id
        );
        tokio::time::sleep(PAGE_LOAD_WAIT).await;
        info!("Wait completed for browser instance {}", instance_id);

        // Get the page content (prefer dynamic DOM via JS execution, fallback to page source)
        info!(
            "Extracting page content from browser instance {} (dynamic DOM if available)...",
            instance_id
        );
        let content = match WebFetcher::get_outer_html_from(browser).await {
            Ok(html) => html,
            Err(e) => {
                warn!(
                    "Falling back to page source for instance {} due to error getting dynamic DOM: {}",
                    instance_id, e
                );
                let content_result = tokio::time::timeout(DEFAULT_TIMEOUT, browser.source()).await;
                match content_result {
                    Ok(Ok(content)) => content,
                    Ok(Err(e)) => {
                        error!(
                            "Failed to get page content from browser instance {}: {}",
                            instance_id, e
                        );
                        return Err(TarziError::Browser(format!("Failed to get content: {e}")));
                    }
                    Err(_) => {
                        error!(
                            "Timeout while extracting page content from browser instance {} (30 seconds)",
                            instance_id
                        );
                        return Err(TarziError::Browser(
                            "Timeout while extracting page content".to_string(),
                        ));
                    }
                }
            }
        };

        // Convert to specified format
        let converted_content = self.converter.convert(&content, format).await?;
        Ok(converted_content)
    }

    /// Create multiple browser instances for parallel processing
    pub async fn create_multiple_browsers(
        &mut self,
        count: usize,
        headless: bool,
        base_instance_id: Option<String>,
    ) -> Result<Vec<String>> {
        self.browser_manager
            .create_multiple_browsers(count, headless, base_instance_id)
            .await
    }

    /// Clean up managed driver if any
    pub async fn cleanup_managed_driver(&mut self) -> Result<()> {
        self.browser_manager.cleanup_managed_driver().await
    }

    /// Check if this fetcher has a managed driver
    pub fn has_managed_driver(&self) -> bool {
        self.browser_manager.has_managed_driver()
    }

    /// Get information about the managed driver
    pub fn get_managed_driver_info(&self) -> Option<&super::driver::DriverInfo> {
        self.browser_manager.get_managed_driver_info()
    }

    pub async fn shutdown(&mut self) {
        self.browser_manager.shutdown().await;
    }

    // Attempt to get the fully-hydrated DOM as HTML via JavaScript execution.
    // Falls back to WebDriver page source on error.
    async fn get_outer_html_from(browser: &thirtyfour::WebDriver) -> Result<String> {
        // Ensure document is ready
        if let Ok(ret) = browser
            .execute(
                "return document.readyState;",
                Vec::<serde_json::Value>::new(),
            )
            .await
            && let Ok(state) = ret.convert::<String>()
            && state != "complete"
        {
            tokio::time::sleep(PAGE_LOAD_WAIT).await;
        }

        // Briefly wait for anchors to populate (dynamic JS apps)
        let mut attempts = 0u8;
        while attempts < 30 {
            if let Ok(ret) = browser
                .execute(
                    "return document.querySelectorAll('a[href]').length;",
                    Vec::<serde_json::Value>::new(),
                )
                .await
                && let Ok(count) = ret.convert::<i64>()
                && count >= 20
            {
                break;
            }
            attempts += 1;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // Try to trigger lazy loading by scrolling
        for _ in 0..3u8 {
            let _ = browser
                .execute(
                    "window.scrollTo({top: document.body.scrollHeight, behavior: 'instant'});",
                    Vec::<serde_json::Value>::new(),
                )
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let _ = browser
                .execute(
                    "window.scrollTo({top: 0, behavior: 'instant'});",
                    Vec::<serde_json::Value>::new(),
                )
                .await;
            tokio::time::sleep(std::time::Duration::from_millis(800)).await;
        }

        // If Brave SERP, inject extracted results into a JSON script tag for parsing
        let _ = browser
            .execute(
                r#"(function(){
                try {
                    const isBraveHost = location.hostname.endsWith('search.brave.com');
                    if (!isBraveHost) return 0;
                    const results = [];
                    const anchors = Array.from(document.querySelectorAll('a[href]'));
                    const isExternal = (u) => { try { const x=new URL(u, location.origin); return x.hostname!==location.hostname; } catch(_) { return false; } };
                    for (const a of anchors) {
                        const href = a.getAttribute('href');
                        if (!href || !isExternal(href)) continue;
                        const title = (a.textContent||'').trim();
                        if (!title || title.length < 5) continue;
                        const rect = a.getBoundingClientRect();
                        if (!rect || rect.width === 0 || rect.height === 0) continue;
                        results.push({ title, url: a.href, snippet: '' });
                        if (results.length >= 15) break;
                    }
                    let s = document.querySelector('#tarzi-brave-results');
                    if (!s) { s = document.createElement('script'); s.id='tarzi-brave-results'; s.type='application/json'; document.body.appendChild(s); }
                    s.textContent = JSON.stringify({ results });
                    return results.length;
                } catch(e) { return -1; }
            })();"#,
                Vec::<serde_json::Value>::new(),
            )
            .await;

        // Return hydrated DOM HTML
        let ret = browser
            .execute(
                "return document.documentElement.outerHTML;",
                Vec::<serde_json::Value>::new(),
            )
            .await
            .map_err(|e| TarziError::Browser(format!("execute() failed: {e}")))?;
        let html: String = ret
            .convert()
            .map_err(|e| TarziError::Browser(format!("failed to convert script return: {e}")))?;
        Ok(html)
    }
}

impl Default for WebFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WebFetcher {
    fn drop(&mut self) {
        if self.browser_manager.has_browsers() || self.browser_manager.has_managed_driver() {
            // Best-effort cleanup without spawning a runtime. We ensure the managed driver is stopped,
            // which will terminate associated sessions; then drop any WebDriver handles.
            tracing::info!(
                "WebFetcher dropped without explicit shutdown. Stopping managed driver and dropping sessions."
            );
            self.browser_manager.stop_managed_driver_sync();
            // Clear browsers to drop WebDriver handles (browser sessions will be terminated by driver shutdown)
            self.browser_manager.clear_browsers();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    /// Test creating a new WebFetcher
    #[test]
    fn test_webfetcher_new() {
        let fetcher = WebFetcher::new();
        assert!(!fetcher.browser_manager.has_browsers());
        assert!(!fetcher.browser_manager.has_managed_driver());
    }

    /// Test creating WebFetcher from config
    #[test]
    fn test_webfetcher_from_config() {
        let config = Config::default();
        let fetcher = WebFetcher::from_config(&config);
        assert!(!fetcher.browser_manager.has_browsers());
        assert!(!fetcher.browser_manager.has_managed_driver());
    }

    /// Test WebFetcher with proxy configuration
    #[test]
    fn test_webfetcher_with_proxy_config() {
        let mut config = Config::default();
        config.fetcher.proxy = Some("http://proxy.example.com:8080".to_string());
        let fetcher = WebFetcher::from_config(&config);
        // Proxy should be configured in the HTTP client (internal state)
        assert!(!fetcher.browser_manager.has_browsers());
    }

    /// Test WebFetcher with custom timeout
    #[test]
    fn test_webfetcher_with_custom_timeout() {
        let mut config = Config::default();
        config.fetcher.timeout = 60; // 60 seconds
        let fetcher = WebFetcher::from_config(&config);
        assert!(!fetcher.browser_manager.has_browsers());
    }

    /// Test WebFetcher with custom user agent
    #[test]
    fn test_webfetcher_with_custom_user_agent() {
        let mut config = Config::default();
        config.fetcher.user_agent = "Test User Agent 1.0".to_string();
        let fetcher = WebFetcher::from_config(&config);
        assert!(!fetcher.browser_manager.has_browsers());
    }

    /// Test WebFetcher default implementation
    #[test]
    fn test_webfetcher_default() {
        let fetcher = WebFetcher::default();
        assert!(!fetcher.browser_manager.has_browsers());
        assert!(!fetcher.browser_manager.has_managed_driver());
    }

    /// Test browser instance management methods
    #[test]
    fn test_browser_instance_management() {
        let fetcher = WebFetcher::new();

        // Test initial state
        assert!(fetcher.get_browser_ids().is_empty());
        assert!(fetcher.get_browser("non-existent").is_none());
    }

    /// Test managed driver info methods
    #[test]
    fn test_managed_driver_info() {
        let fetcher = WebFetcher::new();

        // Test initial state
        assert!(!fetcher.has_managed_driver());
        assert!(fetcher.get_managed_driver_info().is_none());
    }

    /// Test URL validation for fetch operations
    #[tokio::test]
    async fn test_invalid_url_handling() {
        let mut fetcher = WebFetcher::new();

        // Test with invalid URL
        let result = fetcher
            .fetch_raw("not-a-valid-url", FetchMode::PlainRequest)
            .await;
        assert!(result.is_err());

        if let Err(e) = result {
            // Should be a URL parsing error
            assert!(e.to_string().contains("relative URL without a base"));
        }
    }

    /// Test URL validation with different formats
    #[tokio::test]
    async fn test_url_validation() {
        let mut fetcher = WebFetcher::new();

        // Test various invalid URL formats
        let invalid_urls = vec![
            "",
            "not-a-url",
            "://missing-scheme",
            "http://",
            "ftp://unsupported-scheme.com",
        ];

        for invalid_url in invalid_urls {
            let result = fetcher
                .fetch_raw(invalid_url, FetchMode::PlainRequest)
                .await;
            assert!(
                result.is_err(),
                "Expected error for invalid URL: {invalid_url}"
            );
        }
    }

    /// Test FetchMode enum behavior
    #[test]
    fn test_fetch_mode_enum() {
        // Test that FetchMode variants can be created
        let _plain = FetchMode::PlainRequest;
        let _browser_head = FetchMode::BrowserHead;
        let _browser_headless = FetchMode::BrowserHeadless;
    }

    /// Test multiple browser instance creation planning
    #[tokio::test]
    async fn test_multiple_browser_instance_creation() {
        let mut fetcher = WebFetcher::new();

        // This should not actually create browsers since WebDriver is not available
        // But we can test that the method exists and doesn't panic
        let result = fetcher
            .create_multiple_browsers(2, true, Some("test".to_string()))
            .await;

        // In a testing environment without WebDriver, this should fail gracefully
        // The exact error type depends on whether drivers are available
        match result {
            Ok(_) => {
                // If successful, browsers were created (WebDriver available)
                assert!(fetcher.get_browser_ids().len() <= 2);
            }
            Err(_) => {
                // If failed, that's expected in test environment without WebDriver
                // Note: Some browsers might have been created before the failure,
                // so we just verify the method doesn't panic and handles errors gracefully
                let browser_count = fetcher.get_browser_ids().len();
                println!("Browser count after failed creation: {browser_count}");
                // The test passes as long as the method doesn't panic and handles errors
            }
        }
    }

    /// Test browser instance creation with user data directory
    #[tokio::test]
    async fn test_browser_with_user_data_dir() {
        let mut fetcher = WebFetcher::new();

        let temp_dir = tempfile::TempDir::new().unwrap();
        let user_data_path = temp_dir.path().to_path_buf();

        // This should not actually create a browser since WebDriver is not available
        // But we can test that the method exists and handles the user_data_dir parameter
        let result = fetcher
            .create_browser_with_user_data(
                Some(user_data_path),
                true,
                Some("test_with_data_dir".to_string()),
            )
            .await;

        // In a testing environment without WebDriver, this should fail gracefully
        match result {
            Ok(_) => {
                // If successful, browser was created (WebDriver available)
                assert!(!fetcher.get_browser_ids().is_empty());
            }
            Err(_) => {
                // If failed, that's expected in test environment without WebDriver
                assert!(fetcher.get_browser_ids().is_empty());
            }
        }
    }

    /// Test browser instance creation with proxy
    #[tokio::test]
    async fn test_browser_with_proxy() {
        let mut fetcher = WebFetcher::new();

        // This should not actually create a browser since WebDriver is not available
        let result = fetcher
            .create_browser_with_proxy(
                None,
                true,
                Some("test_proxy".to_string()),
                Some("http://proxy.example.com:8080".to_string()),
            )
            .await;

        // In a testing environment without WebDriver, this should fail gracefully
        match result {
            Ok(_) => {
                // If successful, browser was created (WebDriver available)
                assert!(!fetcher.get_browser_ids().is_empty());
            }
            Err(_) => {
                // If failed, that's expected in test environment without WebDriver
                assert!(fetcher.get_browser_ids().is_empty());
            }
        }
    }

    /// Test configuration merging with WebFetcher
    #[test]
    fn test_config_merging() {
        let mut base_config = Config::default();
        base_config.fetcher.timeout = 30;
        base_config.fetcher.user_agent = "Base Agent".to_string();

        let mut override_config = Config::default();
        override_config.fetcher.timeout = 60;
        override_config.fetcher.proxy = Some("http://proxy.example.com:8080".to_string());

        // Merge configs
        base_config.merge(&override_config);

        // Create fetcher with merged config
        let fetcher = WebFetcher::from_config(&base_config);
        assert!(!fetcher.browser_manager.has_browsers());

        // Merged config should have override values where specified
        assert_eq!(base_config.fetcher.timeout, 60);
        assert_eq!(
            base_config.fetcher.proxy,
            Some("http://proxy.example.com:8080".to_string())
        );
        assert_eq!(base_config.fetcher.user_agent, "Base Agent".to_string()); // Should keep base value
    }

    /// Test shutdown behavior
    #[tokio::test]
    async fn test_shutdown() {
        let mut fetcher = WebFetcher::new();

        // Shutdown should not panic even with no browsers
        fetcher.shutdown().await;

        // After shutdown, state should be clean
        assert!(fetcher.get_browser_ids().is_empty());
        assert!(!fetcher.has_managed_driver());
    }

    /// Test browser removal with non-existent instance
    #[tokio::test]
    async fn test_remove_nonexistent_browser() {
        let mut fetcher = WebFetcher::new();

        // Removing a non-existent browser should succeed (no-op)
        let result = fetcher.remove_browser("non-existent").await;
        assert!(result.is_ok());
    }

    /// Test error handling for invalid proxy configuration
    #[tokio::test]
    async fn test_invalid_proxy_handling() {
        let mut fetcher = WebFetcher::new();

        // Test with various invalid proxy formats that will cause errors
        let test_cases = vec![
            ("://invalid", "Invalid proxy URL"),
            ("http://", "Invalid proxy URL"),
            ("invalid-url", "Invalid proxy URL"),
            (
                "http://unreachable-proxy-host:9999",
                "Network error or timeout",
            ),
        ];

        for (invalid_proxy, description) in test_cases {
            let result = fetcher
                .fetch_with_proxy(
                    "https://httpbin.org/html",
                    invalid_proxy,
                    FetchMode::PlainRequest,
                    Format::Html,
                )
                .await;

            match result {
                Err(TarziError::Config(_)) => {
                    println!("✓ Test passed for {invalid_proxy}: {description}");
                }
                Err(TarziError::Http(_)) => {
                    // HTTP errors (like connection failures) are also acceptable for invalid proxies
                    println!("✓ Test passed for {invalid_proxy}: {description} (HTTP error)");
                }
                Err(_) => {
                    println!("✓ Test passed for {invalid_proxy}: {description} (other error)");
                }
                Ok(_) => {
                    // Only fail if we expect a guaranteed error (like malformed URLs)
                    if invalid_proxy == "://invalid" || invalid_proxy == "http://" {
                        panic!("Expected error for clearly invalid proxy: {invalid_proxy}");
                    } else {
                        println!(
                            "ℹ Test passed for {invalid_proxy}: {description} (unexpected success, but acceptable)"
                        );
                    }
                }
            }
        }
    }

    /// Test WebFetcher Drop implementation warning
    #[test]
    fn test_drop_warning() {
        // Create a WebFetcher and let it drop to test the Drop implementation
        // This should not panic and may log a warning if browsers are present
        let _fetcher = WebFetcher::new();
        // WebFetcher drops here
    }
}
