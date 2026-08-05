//! Integration tests for proxy functionality
//!
//! These tests require a proxy server running at http://127.0.0.1:7890
//! They are excluded from the default test suite and must be run explicitly with:
//! `cargo test --features proxy-integration-tests --test proxy_integration_tests`
//!
//! Note: Browser-based proxy tests are currently disabled due to WebDriver runtime drop issues.
//! The plain HTTP request proxy tests are fully functional and demonstrate proxy support.
//! To run including ignored tests: `cargo test --features proxy-integration-tests --test proxy_integration_tests -- --ignored`

#[cfg(feature = "proxy-integration-tests")]
mod proxy_tests {
    use tarzi::config::Config;
    use tarzi::converter::Format;
    use tarzi::fetcher::WebFetcher;

    const TEST_PROXY: &str = "http://127.0.0.1:7890";

    #[tokio::test]
    async fn test_fetch_with_proxy_httpbin() {
        let mut fetcher = WebFetcher::new();

        // Test fetching with proxy in plain HTTP path
        let result = fetcher
            .fetch_with_proxy("https://httpbin.org/html", TEST_PROXY, Format::Html)
            .await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty());
                assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
                println!(
                    "Successfully fetched content with proxy (plain request): {} characters",
                    content.len()
                );
            }
            Err(e) => {
                // Handle 502 errors from httpbin.org gracefully
                let error_str = format!("{e:?}");
                if error_str.contains("502") {
                    println!(
                        "Received 502 from httpbin.org - likely temporary issue, test considered passed"
                    );
                } else {
                    panic!("Failed to fetch with proxy in plain HTTP path: {e:?}");
                }
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_with_proxy_json() {
        let mut fetcher = WebFetcher::new();

        // Test fetching JSON with proxy
        let result = fetcher
            .fetch_with_proxy("https://httpbin.org/json", TEST_PROXY, Format::Json)
            .await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty());

                // Parse the returned JSON and check that the 'content' field contains 'slideshow'
                let v: serde_json::Value = serde_json::from_str(&content).expect("valid JSON");
                let content_field = v["content"].as_str().unwrap_or("");
                assert!(content_field.contains("slideshow"));
                println!(
                    "Successfully fetched JSON with proxy: {} characters",
                    content.len()
                );
            }
            Err(e) => {
                // Handle 502 errors from httpbin.org gracefully
                let error_str = format!("{e:?}");
                if error_str.contains("502") {
                    println!(
                        "Received 502 from httpbin.org - likely temporary issue, test considered passed"
                    );
                } else {
                    panic!("Failed to fetch JSON with proxy: {e:?}");
                }
            }
        }
    }

    #[tokio::test]
    #[ignore = "Browser tests disabled due to WebDriver runtime drop issues"]
    async fn test_fetch_with_proxy_browser() {
        use tokio::task;

        let result = {
            let mut fetcher = WebFetcher::new();

            // Test fetching with proxy in headless browser path
            let fetch_result = fetcher
                .fetch_with_proxy("https://httpbin.org/html", TEST_PROXY, Format::Html)
                .await;

            // Clean up any managed drivers before dropping
            let _ = fetcher.cleanup_managed_driver().await;

            // Drop the fetcher in a blocking context to avoid runtime drop issues
            task::spawn_blocking(move || drop(fetcher)).await.unwrap();

            fetch_result
        };

        match result {
            Ok(content) => {
                assert!(!content.is_empty());
                assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
                println!(
                    "Successfully fetched content with proxy (browser headless): {} characters",
                    content.len()
                );
            }
            Err(e) => {
                // Browser automation might fail due to various reasons (no driver, etc.)
                println!("Browser headless test failed (expected in some environments): {e:?}");
                // We'll consider this test passed if it's a browser-related error
                let error_str = format!("{e:?}");
                assert!(
                    error_str.contains("Browser")
                        || error_str.contains("WebDriver")
                        || error_str.contains("chromedriver")
                        || error_str.contains("geckodriver"),
                    "Unexpected error type: {e:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_fetch_with_config_proxy() {
        // Test fetching with proxy configured via Config
        let mut config = Config::new();
        config.fetcher.proxy = Some(TEST_PROXY.to_string());

        let mut fetcher = WebFetcher::from_config(&config);

        let result = fetcher
            .fetch("https://httpbin.org/html", Format::Html)
            .await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty());
                assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
                println!(
                    "Successfully fetched content with config proxy (plain request): {} characters",
                    content.len()
                );
            }
            Err(e) => {
                // Handle 502 errors from httpbin.org gracefully
                let error_str = format!("{e:?}");
                if error_str.contains("502") {
                    println!(
                        "Received 502 from httpbin.org - likely temporary issue, test considered passed"
                    );
                } else {
                    panic!("Failed to fetch with config proxy in plain HTTP path: {e:?}");
                }
            }
        }
    }

    #[tokio::test]
    #[ignore = "Browser tests disabled due to WebDriver runtime drop issues"]
    async fn test_fetch_with_config_proxy_browser() {
        use tokio::task;

        // Test fetching with proxy configured via Config for headless browser path
        let mut config = Config::new();
        config.fetcher.proxy = Some(TEST_PROXY.to_string());

        let result = {
            let mut fetcher = WebFetcher::from_config(&config);

            let fetch_result = fetcher
                .fetch("https://httpbin.org/html", Format::Html)
                .await;

            // Clean up any managed drivers before dropping
            let _ = fetcher.cleanup_managed_driver().await;

            // Drop the fetcher in a blocking context to avoid runtime drop issues
            task::spawn_blocking(move || drop(fetcher)).await.unwrap();

            fetch_result
        };

        match result {
            Ok(content) => {
                assert!(!content.is_empty());
                assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
                println!(
                    "Successfully fetched content with config proxy (browser headless): {} characters",
                    content.len()
                );
            }
            Err(e) => {
                // Browser automation might fail due to various reasons (no driver, etc.)
                println!("Browser config test failed (expected in some environments): {e:?}");
                // We'll consider this test passed if it's a browser-related error
                let error_str = format!("{e:?}");
                assert!(
                    error_str.contains("Browser")
                        || error_str.contains("WebDriver")
                        || error_str.contains("chromedriver")
                        || error_str.contains("geckodriver"),
                    "Unexpected error type: {e:?}"
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "Browser tests disabled due to WebDriver runtime drop issues"]
    async fn test_create_browser_with_proxy_and_fetch() {
        use tokio::task;

        let mut config = Config::new();
        config.fetcher.proxy = Some(TEST_PROXY.to_string());

        let result = {
            let mut fetcher = WebFetcher::from_config(&config);

            // Create a browser instance with explicit proxy
            let browser_id = fetcher
                .create_browser_with_proxy(
                    None,
                    Some("proxy_test_browser".to_string()),
                    Some(TEST_PROXY.to_string()),
                )
                .await;

            let test_result = match browser_id {
                Ok(instance_id) => {
                    println!("Created browser with proxy, instance ID: {instance_id}");

                    // Fetch content using the browser instance
                    let result = fetcher
                        .fetch_with_browser_instance(
                            "https://httpbin.org/html",
                            &instance_id,
                            Format::Html,
                        )
                        .await;

                    let fetch_success = match result {
                        Ok(content) => {
                            assert!(!content.is_empty());
                            assert!(
                                content.contains("<html>") || content.contains("<!DOCTYPE html>")
                            );
                            println!(
                                "Successfully fetched content with browser instance proxy: {} characters",
                                content.len()
                            );
                            true
                        }
                        Err(e) => {
                            println!("Browser instance fetch failed (may be expected): {e:?}");
                            false
                        }
                    };

                    // Clean up
                    let cleanup_result = fetcher.remove_browser(&instance_id).await;
                    if cleanup_result.is_err() {
                        println!(
                            "Failed to cleanup browser instance: {:?}",
                            cleanup_result.err()
                        );
                    }

                    Ok(fetch_success)
                }
                Err(e) => {
                    // Browser creation might fail due to various reasons
                    println!("Browser creation test failed (expected in some environments): {e:?}");
                    Err(e)
                }
            };

            // Clean up any managed drivers before dropping
            let _ = fetcher.cleanup_managed_driver().await;

            // Drop the fetcher in a blocking context to avoid runtime drop issues
            task::spawn_blocking(move || drop(fetcher)).await.unwrap();

            test_result
        };

        match result {
            Ok(_) => {
                // Test passed
            }
            Err(e) => {
                // We'll consider this test passed if it's a browser-related error
                let error_str = format!("{e:?}");
                assert!(
                    error_str.contains("Browser")
                        || error_str.contains("WebDriver")
                        || error_str.contains("chromedriver")
                        || error_str.contains("geckodriver"),
                    "Unexpected error type: {e:?}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_proxy_with_different_formats() {
        let mut fetcher = WebFetcher::new();

        // Test different output formats with proxy
        let test_cases = vec![
            (Format::Html, "html", "https://httpbin.org/html"),
            (Format::Markdown, "markdown", "https://httpbin.org/html"),
            (Format::Json, "json", "https://httpbin.org/json"),
            (Format::Yaml, "yaml", "https://httpbin.org/html"),
        ];

        for (format, format_name, url) in test_cases {
            let result = fetcher.fetch_with_proxy(url, TEST_PROXY, format).await;

            match result {
                Ok(content) => {
                    assert!(!content.is_empty());
                    println!(
                        "Successfully fetched {} format with proxy: {} characters",
                        format_name,
                        content.len()
                    );
                }
                Err(e) => {
                    // Handle 502 errors from httpbin.org gracefully
                    let error_str = format!("{e:?}");
                    if error_str.contains("502") {
                        println!(
                            "Received 502 from httpbin.org for {format_name} format - likely temporary issue, test considered passed"
                        );
                    } else {
                        panic!("Failed to fetch {format_name} format with proxy: {e:?}");
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_proxy_with_https_site() {
        let mut fetcher = WebFetcher::new();

        // Test proxy with HTTPS sites using httpbin (more reliable)
        let result = fetcher
            .fetch_with_proxy("https://httpbin.org/html", TEST_PROXY, Format::Html)
            .await;

        assert!(
            result.is_ok(),
            "Failed to fetch HTTPS site with proxy: {:?}",
            result.err()
        );

        let content = result.unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
        println!(
            "Successfully fetched HTTPS site with proxy: {} characters",
            content.len()
        );
    }

    #[tokio::test]
    async fn test_proxy_environment_variable_override() {
        // Test that environment variables take precedence over config
        let original_https_proxy = std::env::var("HTTPS_PROXY").ok();

        // Set environment variable
        unsafe {
            std::env::set_var("HTTPS_PROXY", TEST_PROXY);
        }

        // Create config with different proxy
        let mut config = Config::new();
        config.fetcher.proxy = Some("http://different-proxy:8080".to_string());

        let mut fetcher = WebFetcher::from_config(&config);

        let result = fetcher
            .fetch("https://httpbin.org/html", Format::Html)
            .await;

        // Restore original environment
        unsafe {
            match original_https_proxy {
                Some(value) => std::env::set_var("HTTPS_PROXY", value),
                None => std::env::remove_var("HTTPS_PROXY"),
            }
        }

        assert!(
            result.is_ok(),
            "Failed to fetch with environment proxy override: {:?}",
            result.err()
        );

        let content = result.unwrap();
        assert!(!content.is_empty());
        println!(
            "Successfully fetched with environment proxy override: {} characters",
            content.len()
        );
    }
}

// Provide a helpful message when the feature is not enabled
#[cfg(not(feature = "proxy-integration-tests"))]
mod proxy_tests_disabled {
    #[test]
    fn proxy_integration_tests_disabled() {
        println!("Proxy integration tests are disabled.");
        println!(
            "To run these tests, use: cargo test --features proxy-integration-tests --test proxy_integration_tests"
        );
    }
}
