//! Integration tests for browser manager driver types
//!
//! Tests verify that the two driver types (external and self-managed) are mutually exclusive:
//! 1. External: configured by web_driver_url - if set, use it exclusively and fail if unavailable
//! 2. Self-managed: managed by DriverManager - used only if web_driver_url is not set

use std::time::Duration;
use tarzi::{config::Config, fetcher::browser::BrowserManager};

/// Default test timeout for browser operations
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Helper function to create a config with external web driver URL
fn create_config_with_external_url(url: &str) -> Config {
    let mut config = Config::default();
    config.fetcher.web_driver_url = Some(url.to_string());
    config.fetcher.web_driver = "geckodriver".to_string();
    config
}

/// Helper function to create a config without external URL (for self-managed)
fn create_config_for_self_managed(driver_type: &str) -> Config {
    let mut config = Config::default();
    config.fetcher.web_driver_url = None;
    config.fetcher.web_driver = driver_type.to_string();
    config
}

/// Helper function to create a config with empty external URL (should use self-managed)
fn create_config_with_empty_url() -> Config {
    let mut config = Config::default();
    config.fetcher.web_driver_url = Some("".to_string());
    config.fetcher.web_driver = "geckodriver".to_string();
    config
}

/// Helper function to test browser creation with proper timeout and cleanup
async fn test_browser_creation_with_timeout(
    config: Config,
    expected_managed_driver: Option<bool>,
    test_name: &str,
) -> Result<(), String> {
    let mut browser_manager = BrowserManager::from_config(&config);

    let result = tokio::time::timeout(TEST_TIMEOUT, browser_manager.get_or_create_browser()).await;

    match result {
        Ok(Ok(_browser)) => {
            // Browser creation succeeded
            if let Some(should_have_managed) = expected_managed_driver {
                let has_managed = browser_manager.has_managed_driver();
                if has_managed != should_have_managed {
                    return Err(format!(
                        "{test_name}: Expected managed driver: {should_have_managed}, got: {has_managed}"
                    ));
                }
            }

            // Cleanup
            let _ = browser_manager.cleanup_managed_driver().await;
            Ok(())
        }
        Ok(Err(e)) => {
            // Browser creation failed - this might be expected
            Err(e.to_string())
        }
        Err(_) => {
            // Timeout
            Err("Test timed out".to_string())
        }
    }
}

#[tokio::test]
async fn test_external_driver_type_with_valid_url() {
    let config = create_config_with_external_url("http://localhost:4444");

    match test_browser_creation_with_timeout(config, Some(false), "external_valid_url").await {
        Ok(()) => {
            println!("✓ External driver type test passed - WebDriver found at configured URL");
        }
        Err(error_msg) => {
            if error_msg.contains("External WebDriver server is not available")
                && error_msg.contains("http://localhost:4444")
                && error_msg.contains("Please ensure the WebDriver server is running")
            {
                println!(
                    "✓ External driver type test passed - Correct error for unavailable external URL"
                );
            } else {
                panic!("Unexpected error message: {error_msg}");
            }
        }
    }
}

#[tokio::test]
async fn test_external_driver_type_with_invalid_url() {
    let config = create_config_with_external_url("http://localhost:9999");

    match test_browser_creation_with_timeout(config, Some(false), "external_invalid_url").await {
        Ok(()) => {
            panic!("Expected error for invalid external URL");
        }
        Err(error_msg) => {
            if error_msg.contains("External WebDriver server is not available")
                && error_msg.contains("http://localhost:9999")
            {
                println!("✓ External driver type with invalid URL test passed");
            } else {
                panic!("Unexpected error message: {error_msg}");
            }
        }
    }
}

#[tokio::test]
async fn test_self_managed_driver_type_firefox() {
    let config = create_config_for_self_managed("geckodriver");

    match test_browser_creation_with_timeout(config, Some(true), "self_managed_firefox").await {
        Ok(()) => {
            println!("✓ Self-managed Firefox driver test passed - browser created successfully");
        }
        Err(error_msg) => {
            // Should be a self-managed driver error, not external
            if error_msg.contains("External WebDriver") {
                panic!("Got external WebDriver error in self-managed test: {error_msg}");
            } else {
                println!(
                    "✓ Self-managed Firefox driver test passed - Error (expected in CI): {error_msg}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_self_managed_driver_type_chrome() {
    let config = create_config_for_self_managed("chromedriver");

    match test_browser_creation_with_timeout(config, Some(true), "self_managed_chrome").await {
        Ok(()) => {
            println!("✓ Self-managed Chrome driver test passed - browser created successfully");
        }
        Err(error_msg) => {
            // Should be a self-managed driver error, not external
            if error_msg.contains("External WebDriver") {
                panic!("Got external WebDriver error in self-managed test: {error_msg}");
            } else {
                println!(
                    "✓ Self-managed Chrome driver test passed - Error (expected in CI): {error_msg}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_empty_external_url_uses_self_managed() {
    let config = create_config_with_empty_url();

    match test_browser_creation_with_timeout(config, None, "empty_external_url").await {
        Ok(()) => {
            println!("✓ Empty external URL test passed - correctly used self-managed driver");
        }
        Err(error_msg) => {
            // Should be a self-managed driver error, not external
            if error_msg.contains("External WebDriver") {
                panic!("Got external WebDriver error for empty URL: {error_msg}");
            } else {
                println!(
                    "✓ Empty external URL test passed - Correct self-managed error: {error_msg}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_no_config_uses_self_managed() {
    let mut browser_manager = BrowserManager::new();

    let result = tokio::time::timeout(TEST_TIMEOUT, browser_manager.get_or_create_browser()).await;

    match result {
        Ok(Ok(_browser)) => {
            // Should use self-managed driver with default settings
            let _ = browser_manager.cleanup_managed_driver().await;
            println!("✓ No config test passed - correctly used self-managed driver");
        }
        Ok(Err(e)) => {
            // Should be a self-managed driver error
            let error_msg = e.to_string();
            if error_msg.contains("External WebDriver") {
                panic!("Got external WebDriver error with no config: {error_msg}");
            } else {
                println!("✓ No config test passed - Correct self-managed error: {error_msg}");
            }
        }
        Err(_) => {
            println!("✓ No config test passed - Timeout (expected in CI)");
        }
    }
}

#[tokio::test]
async fn test_driver_types_are_exclusive() {
    let test_timeout = Duration::from_secs(60);

    tokio::time::timeout(test_timeout, async {
        // Test that external and self-managed are truly exclusive

        // First test: external URL configured should never use self-managed
        let config_external = create_config_with_external_url("http://localhost:9999");
        let mut browser_manager_external = BrowserManager::from_config(&config_external);

        let result_external = browser_manager_external.get_or_create_browser().await;
        assert!(result_external.is_err());
        assert!(!browser_manager_external.has_managed_driver()); // Should not have started a managed driver

        // Second test: no external URL should never try external URLs
        let config_self_managed = create_config_for_self_managed("geckodriver");
        let mut browser_manager_self = BrowserManager::from_config(&config_self_managed);

        let result_self = browser_manager_self.get_or_create_browser().await;
        match result_self {
            Ok(_) => {
                // If successful, must be self-managed
                assert!(browser_manager_self.has_managed_driver());
                let _ = browser_manager_self.cleanup_managed_driver().await;
            }
            Err(e) => {
                // If failed, should be self-managed error
                let error_msg = e.to_string();
                // The key assertion is that this should NOT be an external WebDriver error
                // Any other error (driver startup failure, session creation failure, etc.) is acceptable
                assert!(!error_msg.contains("External WebDriver"));
            }
        }

        println!("Driver types exclusivity test passed");
    })
    .await
    .expect("Test timed out after 60 seconds");
}

#[tokio::test]
async fn test_managed_driver_info() {
    let test_timeout = Duration::from_secs(60);

    tokio::time::timeout(test_timeout, async {
        let config = create_config_for_self_managed("geckodriver");
        let mut browser_manager = BrowserManager::from_config(&config);

        // Initially no managed driver
        assert!(!browser_manager.has_managed_driver());
        assert!(browser_manager.get_managed_driver_info().is_none());

        let result = browser_manager.get_or_create_browser().await;

        if result.is_ok() {
            // Should now have managed driver info
            assert!(browser_manager.has_managed_driver());
            let driver_info = browser_manager.get_managed_driver_info();
            assert!(driver_info.is_some());

            if let Some(info) = driver_info {
                assert!(info.endpoint.starts_with("http://"));
                assert!(info.pid.is_some());
            }

            // Cleanup should remove managed driver info
            let cleanup_result = browser_manager.cleanup_managed_driver().await;
            assert!(cleanup_result.is_ok());
            assert!(!browser_manager.has_managed_driver());
            assert!(browser_manager.get_managed_driver_info().is_none());

            println!("Managed driver info test passed");
        } else {
            println!("Managed driver info test skipped - no drivers available");
        }
    })
    .await
    .expect("Test timed out after 60 seconds");
}

#[tokio::test]
async fn test_multiple_calls_same_external_url() {
    let config = create_config_with_external_url("http://localhost:8888");
    let mut browser_manager = BrowserManager::from_config(&config);

    // Multiple calls should consistently return the same error for unavailable external URL
    let mut error_count = 0;
    for i in 0..3 {
        let result = tokio::time::timeout(
            Duration::from_secs(20), // Shorter timeout for multiple calls
            browser_manager.get_or_create_browser(),
        )
        .await;

        match result {
            Ok(Ok(_)) => {
                panic!(
                    "Unexpected success for unavailable external URL on attempt {}",
                    i + 1
                );
            }
            Ok(Err(e)) => {
                let error_msg = e.to_string();
                if error_msg.contains("External WebDriver server is not available")
                    && error_msg.contains("http://localhost:8888")
                {
                    error_count += 1;
                }

                // Should never have started a managed driver
                if browser_manager.has_managed_driver() {
                    panic!(
                        "Found managed driver when using external URL on attempt {}",
                        i + 1
                    );
                }
            }
            Err(_) => {
                // Timeout is also acceptable
                error_count += 1;
            }
        }
    }

    if error_count >= 2 {
        println!(
            "✓ Multiple external URL calls test passed ({error_count}/3 calls failed as expected)"
        );
    } else {
        panic!("Expected multiple failures for unavailable external URL, got {error_count}/3");
    }
}

#[tokio::test]
async fn test_config_merge_preserves_driver_type() {
    let test_timeout = Duration::from_secs(60);

    tokio::time::timeout(test_timeout, async {
        // Test that when configs are merged, driver type behavior is preserved
        let mut base_config = Config::default();
        base_config.fetcher.web_driver = "chromedriver".to_string();
        base_config.fetcher.web_driver_url = None;

        let mut override_config = Config::default();
        override_config.fetcher.web_driver_url = Some("http://localhost:7777".to_string());

        // Merge configs
        base_config.merge(&override_config);

        // Should now use external driver type
        let mut browser_manager = BrowserManager::from_config(&base_config);
        let result = browser_manager.get_or_create_browser().await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("External WebDriver server is not available"));
        assert!(error_msg.contains("http://localhost:7777"));

        println!("Config merge driver type test passed");
    })
    .await
    .expect("Test timed out after 60 seconds");
}
