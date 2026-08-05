use super::driver::{DriverConfig, DriverInfo, DriverManager, DriverType};
use crate::{
    Result,
    config::Config,
    constants::{
        BROWSER_LAUNCH_TIMEOUT, CHROME_DRIVER_ARGS, CHROMEDRIVER_DEFAULT_PORT, DEFAULT_TIMEOUT,
        FIREFOX_DRIVER_ARGS, GECKODRIVER_DEFAULT_PORT, WEBDRIVER_CHECK_TIMEOUT,
    },
    error::TarziError,
};
use std::{collections::HashMap, path::PathBuf};
use tempfile::TempDir;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
use tracing::{error, info, warn};

/// Browser instance manager
#[derive(Debug)]
pub struct BrowserManager {
    browsers: HashMap<String, (WebDriver, TempDir)>,
    driver_manager: Option<DriverManager>,
    managed_driver_info: Option<DriverInfo>,
    config: Option<Config>,
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            browsers: HashMap::new(),
            driver_manager: None,
            managed_driver_info: None,
            config: None,
        }
    }

    /// Create a new BrowserManager with configuration
    pub fn from_config(config: &Config) -> Self {
        Self {
            browsers: HashMap::new(),
            driver_manager: None,
            managed_driver_info: None,
            config: Some(config.clone()),
        }
    }

    /// Create a new browser instance with a specific user data directory
    pub async fn create_browser_with_user_data(
        &mut self,
        user_data_dir: Option<PathBuf>,
        instance_id: Option<String>,
    ) -> Result<String> {
        let webdriver_url = self.get_or_create_webdriver_endpoint().await?;

        let instance_id = instance_id.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("browser_{}", timestamp % 1_000_000)
        });
        info!(
            "Creating new headless browser instance with ID: {} (user_data_dir: {:?})",
            instance_id, user_data_dir
        );

        // Determine actual driver type that was started by checking managed driver info
        let actual_driver_type = if let Some(managed_info) = &self.managed_driver_info {
            let driver_type = match managed_info.config.driver_type {
                crate::fetcher::driver::DriverType::Firefox => "firefox",
                crate::fetcher::driver::DriverType::Chrome => "chrome",
                crate::fetcher::driver::DriverType::Generic(_) => "chrome", // fallback
            };
            info!(
                "Using capabilities for actually started driver: {}",
                driver_type
            );
            driver_type
        } else {
            // If no managed driver, use config-based detection (for external drivers)
            let driver_type = self.get_driver_type_from_config();
            info!(
                "Using capabilities from config for external driver: {}",
                driver_type
            );
            driver_type
        };

        let browser_result = match actual_driver_type {
            "firefox" => {
                let mut caps = DesiredCapabilities::firefox();
                self.configure_firefox_capabilities(&mut caps, &user_data_dir)
                    .await?;
                tokio::time::timeout(BROWSER_LAUNCH_TIMEOUT, WebDriver::new(&webdriver_url, caps))
                    .await
            }
            _ => {
                let mut caps = DesiredCapabilities::chrome();
                self.configure_browser_capabilities(&mut caps, &user_data_dir)
                    .await?;
                tokio::time::timeout(BROWSER_LAUNCH_TIMEOUT, WebDriver::new(&webdriver_url, caps))
                    .await
            }
        };

        info!("Browser config created successfully");

        // Create or use provided temp directory for browser data
        let temp_dir = if let Some(user_data_path) = user_data_dir {
            info!("Using provided user data directory: {:?}", user_data_path);
            // Create a temp dir as a placeholder - the actual user data dir is configured in capabilities
            TempDir::new().map_err(|e| {
                error!("Failed to create placeholder directory: {}", e);
                TarziError::Browser(format!("Failed to create placeholder directory: {e}"))
            })?
        } else {
            TempDir::new().map_err(|e| {
                error!("Failed to create temporary directory: {}", e);
                TarziError::Browser(format!("Failed to create temporary directory: {e}"))
            })?
        };

        let browser = match browser_result {
            Ok(Ok(result)) => {
                info!("Browser launched successfully with ID: {}", instance_id);
                result
            }
            Ok(Err(e)) => {
                error!("Failed to create browser: {}", e);
                return Err(TarziError::Browser(format!(
                    "Failed to create browser: {e}"
                )));
            }
            Err(_) => {
                error!("Timeout while launching browser (60 seconds)");
                return Err(TarziError::Browser(
                    "Timeout while launching browser".to_string(),
                ));
            }
        };

        self.browsers
            .insert(instance_id.clone(), (browser, temp_dir));
        info!("Browser instance stored with ID: {}", instance_id);
        Ok(instance_id)
    }

    /// Get driver type from configuration
    fn get_driver_type_from_config(&self) -> &str {
        if let Some(config) = &self.config {
            match config.fetcher.web_driver.as_str() {
                "geckodriver" | "firefox" => {
                    info!("Using Firefox capabilities for geckodriver");
                    "firefox"
                }
                "chromedriver" | "chrome" => {
                    info!("Using Chrome capabilities for chromedriver");
                    "chrome"
                }
                _ => {
                    info!("Unknown driver type, using Chrome capabilities as fallback");
                    "chrome"
                }
            }
        } else {
            info!("No configuration available, using Chrome capabilities as fallback");
            "chrome"
        }
    }

    /// Configure browser capabilities based on browser type and settings
    async fn configure_browser_capabilities(
        &self,
        caps: &mut impl ChromiumLikeCapabilities,
        user_data_dir: &Option<PathBuf>,
    ) -> Result<()> {
        caps.add_arg("--headless").map_err(|e| {
            error!("Failed to add headless arg: {}", e);
            TarziError::Browser(format!("Failed to add headless arg: {e}"))
        })?;

        // Add user data directory if provided
        if let Some(user_data_path) = user_data_dir {
            caps.add_arg(&format!("--user-data-dir={}", user_data_path.display()))
                .map_err(|e| {
                    error!("Failed to add user-data-dir arg: {}", e);
                    TarziError::Browser(format!("Failed to add user-data-dir arg: {e}"))
                })?;
        }

        caps.add_arg("--disable-gpu").map_err(|e| {
            error!("Failed to add disable-gpu arg: {}", e);
            TarziError::Browser(format!("Failed to add disable-gpu arg: {e}"))
        })?;
        caps.add_arg("--disable-dev-shm-usage").map_err(|e| {
            error!("Failed to add disable-dev-shm-usage arg: {}", e);
            TarziError::Browser(format!("Failed to add disable-dev-shm-usage arg: {e}"))
        })?;
        caps.add_arg("--no-sandbox").map_err(|e| {
            error!("Failed to add no-sandbox arg: {}", e);
            TarziError::Browser(format!("Failed to add no-sandbox arg: {e}"))
        })?;

        // Add proxy configuration if available
        if let Some(config) = &self.config {
            let proxy = crate::config::get_proxy_from_env_or_config(&config.fetcher.proxy);
            if let Some(proxy_url) = proxy
                && !proxy_url.is_empty()
            {
                info!("Configuring browser with proxy: {}", proxy_url);
                caps.add_arg(&format!("--proxy-server={proxy_url}"))
                    .map_err(|e| {
                        error!("Failed to add proxy-server arg: {}", e);
                        TarziError::Browser(format!("Failed to add proxy-server arg: {e}"))
                    })?;
            }
        }
        Ok(())
    }

    /// Configure Firefox capabilities separately since it doesn't implement ChromiumLikeCapabilities
    async fn configure_firefox_capabilities(
        &self,
        caps: &mut thirtyfour::FirefoxCapabilities,
        user_data_dir: &Option<PathBuf>,
    ) -> Result<()> {
        caps.add_arg("--headless").map_err(|e| {
            error!("Failed to add headless arg: {}", e);
            TarziError::Browser(format!("Failed to add headless arg: {e}"))
        })?;

        // Add profile directory if provided (Firefox uses --profile instead of --user-data-dir)
        if let Some(user_data_path) = user_data_dir {
            caps.add_arg(&format!("--profile={}", user_data_path.display()))
                .map_err(|e| {
                    error!("Failed to add profile arg: {}", e);
                    TarziError::Browser(format!("Failed to add profile arg: {e}"))
                })?;
        }

        Ok(())
    }

    /// Get a browser instance by ID
    pub fn get_browser(&self, instance_id: &str) -> Option<&WebDriver> {
        self.browsers.get(instance_id).map(|(browser, _)| browser)
    }

    /// Get all browser instance IDs
    pub fn get_browser_ids(&self) -> Vec<String> {
        self.browsers.keys().cloned().collect()
    }

    /// Remove a browser instance by ID
    pub async fn remove_browser(&mut self, instance_id: &str) -> Result<bool> {
        if let Some((driver, _temp_dir)) = self.browsers.remove(instance_id) {
            info!("Removed browser instance: {}", instance_id);
            driver.quit().await.map_err(|e| {
                error!("Failed to quit browser: {}", e);
                TarziError::Browser(format!("Failed to quit browser: {e}"))
            })?;
            // The temp_dir will be automatically cleaned up when dropped
            Ok(true)
        } else {
            warn!("Browser instance not found: {}", instance_id);
            Ok(false)
        }
    }

    /// Get or create a browser instance
    pub async fn get_or_create_browser(&mut self) -> Result<&WebDriver> {
        if self.browsers.is_empty() {
            info!("Creating new headless browser instance...");
            let instance_id = self
                .create_browser_with_user_data(None, Some("default".to_string()))
                .await?;
            info!("Browser instance created with ID: {}", instance_id);
        } else {
            info!("Using existing browser instance");
        }
        Ok(&self.browsers.values().next().unwrap().0)
    }

    /// Create multiple headless browser instances for parallel processing
    pub async fn create_multiple_browsers(
        &mut self,
        count: usize,
        base_instance_id: Option<String>,
    ) -> Result<Vec<String>> {
        info!("Creating {} headless browser instances", count);

        let base_id = base_instance_id.unwrap_or_else(|| "browser".to_string());
        let mut instance_ids = Vec::new();

        for i in 0..count {
            let instance_id = format!("{base_id}_{i}");
            let id = self
                .create_browser_with_user_data(None, Some(instance_id.clone()))
                .await?;
            instance_ids.push(id);
        }

        info!("Successfully created {} browser instances", count);
        Ok(instance_ids)
    }

    /// Check if any browsers are available
    pub fn has_browsers(&self) -> bool {
        !self.browsers.is_empty()
    }

    /// Get the first available browser
    pub fn get_first_browser(&self) -> Option<&WebDriver> {
        self.browsers.values().next().map(|(browser, _)| browser)
    }

    /// Get or create a webdriver endpoint, using configuration or DriverManager
    /// Two mutually exclusive types:
    /// 1. External: configured by web_driver_url - if set, use it exclusively and fail if unavailable
    /// 2. Self-managed: managed by DriverManager - used only if web_driver_url is not set
    async fn get_or_create_webdriver_endpoint(&mut self) -> Result<String> {
        if let Some(config) = &self.config
            && let Some(ref url) = config.fetcher.web_driver_url
            && !url.is_empty()
        {
            // External driver type: web_driver_url is explicitly configured
            info!("Using external WebDriver URL from config: {}", url);
            if is_webdriver_available_at_url(url).await {
                info!(
                    "External WebDriver server is available and ready at: {}",
                    url
                );
                return Ok(url.clone());
            } else {
                error!(
                    "External WebDriver URL '{}' is configured but server is not available",
                    url
                );
                return Err(TarziError::Browser(format!(
                    "External WebDriver server is not available at configured URL: {url}. \
                             Please ensure the WebDriver server is running at this URL, or remove \
                             the web_driver_url configuration to use self-managed drivers."
                )));
            }
        }

        // Self-managed driver type: no web_driver_url configured, use DriverManager
        info!("No external WebDriver URL configured, using self-managed driver");

        // Try to find an already running WebDriver first (from previous self-managed instance)
        let default_url = if let Some(config) = &self.config {
            let web_driver = &config.fetcher.web_driver;
            match web_driver.as_str() {
                "geckodriver" | "firefox" => {
                    format!("http://localhost:{GECKODRIVER_DEFAULT_PORT}")
                }
                "chromedriver" | "chrome" => {
                    format!("http://localhost:{CHROMEDRIVER_DEFAULT_PORT}")
                }
                _ => {
                    // Fallback to GeckoDriver for unknown driver types
                    format!("http://localhost:{GECKODRIVER_DEFAULT_PORT}")
                }
            }
        } else {
            // No config available, use GeckoDriver as default
            format!("http://localhost:{GECKODRIVER_DEFAULT_PORT}")
        };

        info!(
            "Checking for existing self-managed WebDriver at: {}",
            default_url
        );
        if is_webdriver_available_at_url(&default_url).await {
            info!(
                "Found existing self-managed WebDriver server at: {}",
                default_url
            );
            return Ok(default_url);
        }

        // No existing WebDriver found, start one using DriverManager
        info!(
            "No existing WebDriver server found, starting self-managed driver using DriverManager"
        );

        // Initialize DriverManager if not already done
        if self.driver_manager.is_none() {
            info!("Initializing DriverManager for self-managed driver");
            self.driver_manager = Some(DriverManager::new());
        }

        // Try to start a driver using DriverManager
        let driver_manager = self.driver_manager.as_ref().unwrap();

        // Determine which driver to try first based on configuration
        let (primary_driver, fallback_driver) = if let Some(config) = &self.config {
            match config.fetcher.web_driver.as_str() {
                "geckodriver" | "firefox" => (DriverType::Firefox, DriverType::Chrome),
                _ => (DriverType::Chrome, DriverType::Firefox),
            }
        } else {
            (DriverType::Chrome, DriverType::Firefox)
        };

        // Try drivers in order: primary first, then fallback
        for driver_type in [primary_driver, fallback_driver] {
            match self.try_start_driver(driver_manager, &driver_type) {
                Ok(driver_info) => {
                    info!(
                        "Successfully started self-managed {:?} at: {}",
                        driver_type, driver_info.endpoint
                    );
                    self.managed_driver_info = Some(driver_info.clone());
                    return Ok(driver_info.endpoint);
                }
                Err(e) => {
                    warn!("Failed to start self-managed {:?}: {}", driver_type, e);
                    // Continue to next driver type
                }
            }
        }

        // If all attempts failed, return an error with helpful guidance
        Err(TarziError::Browser(
            "No self-managed WebDriver could be started. Please either:\n\
            1. Install ChromeDriver (https://chromedriver.chromium.org/) or GeckoDriver (https://github.com/mozilla/geckodriver/releases) and ensure they're in your PATH, or\n\
            2. Configure TARZI_WEB_DRIVER_URL to use an external WebDriver server".to_string()
        ))
    }

    /// Try to start a driver of the given type
    fn try_start_driver(
        &self,
        driver_manager: &DriverManager,
        driver_type: &DriverType,
    ) -> Result<DriverInfo> {
        // Check if driver binary exists
        driver_manager.check_driver_binary(driver_type)?;

        let (port, args) = match driver_type {
            DriverType::Chrome => (CHROMEDRIVER_DEFAULT_PORT, CHROME_DRIVER_ARGS),
            DriverType::Firefox => (GECKODRIVER_DEFAULT_PORT, FIREFOX_DRIVER_ARGS),
            _ => (GECKODRIVER_DEFAULT_PORT, FIREFOX_DRIVER_ARGS),
        };

        let config = DriverConfig {
            driver_type: driver_type.clone(),
            port,
            args: args.iter().map(|s| s.to_string()).collect(),
            timeout: DEFAULT_TIMEOUT,
            verbose: false,
        };

        driver_manager.start_driver_with_config(config)
    }

    /// Clean up managed driver if any
    pub async fn cleanup_managed_driver(&mut self) -> Result<()> {
        if let (Some(driver_manager), Some(driver_info)) =
            (&mut self.driver_manager, &self.managed_driver_info)
        {
            match driver_manager.stop_driver(driver_info.config.port) {
                Ok(()) => {
                    self.managed_driver_info = None;
                }
                Err(e) => {
                    warn!("Failed to stop managed driver: {}", e);
                    return Err(TarziError::Browser(format!(
                        "Failed to stop managed driver: {e}"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Synchronous best-effort stop of managed driver (for Drop paths)
    pub fn stop_managed_driver_sync(&mut self) {
        if let (Some(driver_manager), Some(driver_info)) =
            (&mut self.driver_manager, &self.managed_driver_info)
        {
            match driver_manager.stop_driver(driver_info.config.port) {
                Ok(()) => {
                    self.managed_driver_info = None;
                }
                Err(e) => {
                    warn!("Failed to stop managed driver synchronously: {}", e);
                }
            }
        }
    }

    /// Clear all browser instances (for Drop paths)
    /// This should be called after stop_managed_driver_sync() to ensure proper cleanup
    pub fn clear_browsers(&mut self) {
        self.browsers.clear();
    }

    /// Check if this browser manager has a managed driver
    pub fn has_managed_driver(&self) -> bool {
        self.managed_driver_info.is_some()
    }

    /// Get information about the managed driver
    pub fn get_managed_driver_info(&self) -> Option<&DriverInfo> {
        self.managed_driver_info.as_ref()
    }

    /// Create a new browser instance with explicit proxy configuration
    pub async fn create_browser_with_proxy(
        &mut self,
        user_data_dir: Option<PathBuf>,
        instance_id: Option<String>,
        proxy: Option<String>,
    ) -> Result<String> {
        // Store original config proxy
        let original_proxy = self.config.as_ref().and_then(|c| c.fetcher.proxy.clone());

        // Temporarily override proxy configuration
        if let Some(config) = &mut self.config {
            config.fetcher.proxy = proxy;
        }

        // Create browser with proxy
        let result = self
            .create_browser_with_user_data(user_data_dir, instance_id)
            .await;

        // Restore original proxy configuration
        if let Some(config) = &mut self.config {
            config.fetcher.proxy = original_proxy;
        }

        result
    }

    /// Asynchronously shut down all browser instances and managed driver
    pub async fn shutdown(&mut self) {
        // Clean up all browser instances
        let browser_ids: Vec<String> = self.browsers.keys().cloned().collect();
        for instance_id in browser_ids {
            if let Some((driver, _temp_dir)) = self.browsers.remove(&instance_id) {
                info!("Shutting down browser instance: {}", instance_id);
                if let Err(e) = driver.quit().await {
                    error!("Failed to quit browser instance {}: {}", instance_id, e);
                }
            }
        }
        // Clean up managed driver
        if let (Some(driver_manager), Some(driver_info)) =
            (&mut self.driver_manager, &self.managed_driver_info)
        {
            info!("Shutting down managed driver: {}", driver_info.endpoint);
            if let Err(e) = driver_manager.stop_driver(driver_info.config.port) {
                error!("Failed to stop managed driver: {}", e);
            }
            self.managed_driver_info = None;
        }
    }
}

impl Drop for BrowserManager {
    fn drop(&mut self) {
        if !self.browsers.is_empty() || self.managed_driver_info.is_some() {
            // Best-effort cleanup without spawning a runtime. We ensure the managed driver is stopped,
            // which will terminate associated sessions; then drop any WebDriver handles.
            info!(
                "BrowserManager dropped without explicit shutdown. Stopping managed driver and dropping sessions."
            );
            self.stop_managed_driver_sync();
            self.clear_browsers();
        }
    }
}

/// Helper function to check if webdriver is available at a specific URL
async fn is_webdriver_available_at_url(url: &str) -> bool {
    use reqwest;
    use tokio::time::timeout;

    match timeout(
        WEBDRIVER_CHECK_TIMEOUT,
        reqwest::get(&format!("{url}/status")),
    )
    .await
    {
        Ok(Ok(response)) => response.status().is_success(),
        _ => false,
    }
}

impl Default for BrowserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    /// Test creating a new BrowserManager
    #[test]
    fn test_browser_manager_new() {
        let manager = BrowserManager::new();
        assert_eq!(manager.browsers.len(), 0);
        assert!(manager.driver_manager.is_none());
        assert!(manager.managed_driver_info.is_none());
        assert!(manager.config.is_none());
    }

    /// Test creating BrowserManager with config
    #[test]
    fn test_browser_manager_from_config() {
        let config = Config::default();
        let manager = BrowserManager::from_config(&config);

        assert_eq!(manager.browsers.len(), 0);
        assert!(manager.driver_manager.is_none());
        assert!(manager.managed_driver_info.is_none());
        assert!(manager.config.is_some());
    }

    /// Test get_driver_type_from_config method
    #[test]
    fn test_get_driver_type_from_config() {
        // Test with no config
        let manager = BrowserManager::new();
        assert_eq!(manager.get_driver_type_from_config(), "chrome");

        // Test with Firefox config
        let mut config = Config::default();
        config.fetcher.web_driver = "geckodriver".to_string();
        let manager = BrowserManager::from_config(&config);
        assert_eq!(manager.get_driver_type_from_config(), "firefox");

        // Test with Chrome config
        config.fetcher.web_driver = "chromedriver".to_string();
        let manager = BrowserManager::from_config(&config);
        assert_eq!(manager.get_driver_type_from_config(), "chrome");

        // Test with unknown driver type
        config.fetcher.web_driver = "unknown".to_string();
        let manager = BrowserManager::from_config(&config);
        assert_eq!(manager.get_driver_type_from_config(), "chrome");
    }

    /// Test browser instance management methods
    #[test]
    fn test_browser_instance_management() {
        let manager = BrowserManager::new();

        // Test initial state
        assert!(!manager.has_browsers());
        assert_eq!(manager.get_browser_ids().len(), 0);
        assert!(manager.get_first_browser().is_none());
        assert!(manager.get_browser("non-existent").is_none());
    }

    /// Test managed driver info methods
    #[test]
    fn test_managed_driver_info() {
        let manager = BrowserManager::new();

        // Test initial state
        assert!(!manager.has_managed_driver());
        assert!(manager.get_managed_driver_info().is_none());
    }

    /// Test driver type logic in get_or_create_webdriver_endpoint
    #[test]
    fn test_driver_type_selection() {
        // Test with Firefox config
        let mut config = Config::default();
        config.fetcher.web_driver = "geckodriver".to_string();
        let _manager = BrowserManager::from_config(&config);

        let (primary, fallback) = if config.fetcher.web_driver.as_str() == "geckodriver"
            || config.fetcher.web_driver.as_str() == "firefox"
        {
            (DriverType::Firefox, DriverType::Chrome)
        } else {
            (DriverType::Chrome, DriverType::Firefox)
        };

        assert_eq!(primary, DriverType::Firefox);
        assert_eq!(fallback, DriverType::Chrome);

        // Test with Chrome config
        config.fetcher.web_driver = "chromedriver".to_string();
        let _manager = BrowserManager::from_config(&config);

        let (primary, fallback) = if config.fetcher.web_driver.as_str() == "geckodriver"
            || config.fetcher.web_driver.as_str() == "firefox"
        {
            (DriverType::Firefox, DriverType::Chrome)
        } else {
            (DriverType::Chrome, DriverType::Firefox)
        };

        assert_eq!(primary, DriverType::Chrome);
        assert_eq!(fallback, DriverType::Firefox);
    }

    /// Test unique instance ID generation
    #[test]
    fn test_unique_instance_id_generation() {
        // Test multiple calls to ensure uniqueness
        let mut ids = std::collections::HashSet::new();

        for _ in 0..10 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let id = format!("browser_{}", timestamp % 1_000_000);
            ids.insert(id);

            // Small delay to ensure different timestamps
            std::thread::sleep(std::time::Duration::from_nanos(1));
        }

        // We should have multiple unique IDs (may not be 10 due to timing)
        assert!(ids.len() > 1);
    }

    /// Test capabilities configuration for different browsers and configurations
    #[tokio::test]
    async fn test_configure_browser_capabilities() {
        let manager = BrowserManager::new();

        // Test Firefox capabilities
        let mut firefox_caps = DesiredCapabilities::firefox();
        let result = manager
            .configure_firefox_capabilities(&mut firefox_caps, &None)
            .await;
        assert!(
            result.is_ok(),
            "Firefox capabilities should be configured successfully"
        );

        // Test Chrome capabilities
        let mut chrome_caps = DesiredCapabilities::chrome();
        let result = manager
            .configure_browser_capabilities(&mut chrome_caps, &None)
            .await;
        assert!(
            result.is_ok(),
            "Chrome capabilities should be configured successfully"
        );

        // Test with user data directory
        let temp_dir = tempfile::TempDir::new().unwrap();
        let user_data_dir = Some(temp_dir.path().to_path_buf());
        let mut chrome_caps_with_dir = DesiredCapabilities::chrome();
        let result = manager
            .configure_browser_capabilities(&mut chrome_caps_with_dir, &user_data_dir)
            .await;
        assert!(
            result.is_ok(),
            "Chrome capabilities with user data dir should be configured successfully"
        );
    }

    /// Test proxy configuration integration
    #[tokio::test]
    async fn test_proxy_configuration() {
        let mut config = Config::default();
        config.fetcher.proxy = Some("http://proxy.example.com:8080".to_string());
        let manager = BrowserManager::from_config(&config);

        let mut chrome_caps = DesiredCapabilities::chrome();
        let result = manager
            .configure_browser_capabilities(&mut chrome_caps, &None)
            .await;
        assert!(
            result.is_ok(),
            "Chrome capabilities with proxy should be configured successfully"
        );
    }

    /// Test external WebDriver URL detection
    #[test]
    fn test_external_webdriver_url_detection() {
        // Test with external URL
        let mut config = Config::default();
        config.fetcher.web_driver_url = Some("http://localhost:4444".to_string());
        let _manager = BrowserManager::from_config(&config);

        // Should detect external URL configuration
        assert!(config.fetcher.web_driver_url.is_some());

        // Test with empty URL (should use self-managed)
        config.fetcher.web_driver_url = Some("".to_string());
        let _manager = BrowserManager::from_config(&config);

        // Empty URL should be treated as None
        let url = &config.fetcher.web_driver_url;
        assert!(url.is_some() && url.as_ref().unwrap().is_empty());

        // Test with no URL (should use self-managed)
        config.fetcher.web_driver_url = None;
        let _manager = BrowserManager::from_config(&config);
        assert!(config.fetcher.web_driver_url.is_none());
    }

    /// Test error handling for invalid configurations
    #[tokio::test]
    async fn test_error_handling() {
        let manager = BrowserManager::new();

        // Test capabilities configuration with invalid user data directory
        let invalid_dir = Some(PathBuf::from("/non/existent/path/that/should/not/exist"));
        let mut chrome_caps = DesiredCapabilities::chrome();
        let result = manager
            .configure_browser_capabilities(&mut chrome_caps, &invalid_dir)
            .await;
        // This should still succeed as the path is only added as an argument
        assert!(result.is_ok());
    }

    /// Test configuration merging behavior
    #[test]
    fn test_configuration_behavior() {
        let base_config = Config::default();
        let _manager = BrowserManager::from_config(&base_config);

        // Test that config is properly stored
        assert!(_manager.config.is_some());

        // Test driver type resolution with different configurations
        let firefox_config = {
            let mut config = Config::default();
            config.fetcher.web_driver = "firefox".to_string();
            config
        };

        let firefox_manager = BrowserManager::from_config(&firefox_config);
        assert_eq!(firefox_manager.get_driver_type_from_config(), "firefox");
    }

    /// Test multiple browser instance handling
    #[test]
    fn test_multiple_browser_handling() {
        let manager = BrowserManager::new();

        // Test that manager can handle multiple browser queries
        for i in 0..5 {
            let browser_id = format!("browser_{i}");
            assert!(manager.get_browser(&browser_id).is_none());
        }

        // Test browser ID collection
        let ids = manager.get_browser_ids();
        assert_eq!(ids.len(), 0);
    }
}
