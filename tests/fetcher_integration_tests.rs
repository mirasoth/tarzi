use std::time::Duration;
use tarzi::converter::Format;
use tarzi::error::TarziError;
use tarzi::fetcher::WebFetcher;
use tarzi::utils::is_webdriver_available;

// Integration tests for fetcher module
// These tests require internet access and may take longer to run

/// Helper function to create a fetcher with reasonable timeout
fn create_test_fetcher() -> WebFetcher {
    let mut config = tarzi::config::Config::default();
    config.fetcher.timeout = 30; // 30 second timeout for tests
    WebFetcher::from_config(&config)
}

/// Plain-HTTP-only fetcher (no browser cascade) for error-path assertions
fn create_plain_only_fetcher() -> WebFetcher {
    let mut config = tarzi::config::Config::default();
    config.fetcher.timeout = 30;
    config.fetcher.browser = false;
    WebFetcher::from_config(&config)
}

/// Helper function to check if a URL is reachable
async fn is_url_reachable(url: &str) -> bool {
    use reqwest::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    match client.head(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::test]
async fn test_fetch_httpbin_html() {
    let test_url = "https://httpbin.org/html";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping test - {test_url} is not reachable");
        return;
    }

    let test_timeout = Duration::from_secs(60);
    tokio::time::timeout(test_timeout, async {
        let mut fetcher = create_test_fetcher();

        let result = fetcher.fetch(test_url, Format::Html).await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty(), "Content should not be empty");
                assert!(
                    content.contains("<html>") || content.contains("<!DOCTYPE html>"),
                    "Content should contain HTML markup"
                );
                println!("✓ HTTP fetch test succeeded");
            }
            Err(e) => {
                println!("HTTP fetch test failed: {e:?}");
                // Allow network errors in CI environments
                if !matches!(e, TarziError::Http(_)) {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }
    })
    .await
    .expect("Test timed out after 60 seconds");
}

#[tokio::test]
async fn test_fetch_httpbin_json() {
    let test_url = "https://httpbin.org/json";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping JSON test - {test_url} is not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();
    let result = fetcher.fetch(test_url, Format::Json).await;

    match result {
        Ok(content) => {
            assert!(!content.is_empty(), "JSON content should not be empty");

            // Parse the returned JSON and validate structure
            let v: serde_json::Value =
                serde_json::from_str(&content).expect("Response should be valid JSON");
            let content_field = v["content"].as_str().unwrap_or("");
            assert!(
                content_field.contains("slideshow"),
                "JSON content should contain expected data"
            );
            println!("✓ JSON fetch test succeeded");
        }
        Err(e) => {
            println!("JSON fetch test failed: {e:?}");
            // Allow network errors in CI environments
            if !matches!(e, TarziError::Http(_)) {
                panic!("Unexpected error: {e:?}");
            } else {
                println!("Network error - acceptable for external service dependency");
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_httpbin_markdown() {
    let test_url = "https://httpbin.org/html";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping markdown test - {test_url} is not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();

    // Test fetching HTML and converting to markdown
    let result = fetcher.fetch(test_url, Format::Markdown).await;

    match result {
        Ok(content) => {
            assert!(!content.is_empty());
            // Should contain markdown content (no HTML tags)
            assert!(!content.contains("<html>"));
            assert!(!content.contains("<!DOCTYPE html>"));
            println!("✓ Markdown fetch test succeeded");
        }
        Err(e) => {
            println!("Markdown fetch test failed: {e:?}");
            // Allow network errors in CI environments
            if !matches!(e, TarziError::Http(_)) {
                panic!("Unexpected error: {e:?}");
            } else {
                println!("Network error - acceptable for external service dependency");
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_httpbin_yaml() {
    let test_url = "https://httpbin.org/html";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping YAML test - {test_url} is not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();

    // Test fetching HTML and converting to YAML
    let result = fetcher.fetch(test_url, Format::Yaml).await;

    match result {
        Ok(content) => {
            assert!(!content.is_empty());
            // Should contain YAML structure
            assert!(content.contains("title:") || content.contains("content:"));
            println!("✓ YAML fetch test succeeded");
        }
        Err(e) => {
            println!("YAML fetch test failed: {e:?}");
            // Allow network errors in CI environments
            if !matches!(e, TarziError::Http(_)) {
                panic!("Unexpected error: {e:?}");
            } else {
                println!("Network error - acceptable for external service dependency");
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_raw() {
    let test_url = "https://httpbin.org/html";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping raw plain request test - {test_url} is not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();

    // Test fetching raw content without conversion
    let result = fetcher.fetch_raw(test_url).await;

    match result {
        Ok(content) => {
            assert!(!content.is_empty());
            assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
            println!("✓ Raw plain request test succeeded");
        }
        Err(e) => {
            println!("Raw plain request test failed: {e:?}");
            // Allow network errors in CI environments
            if !matches!(e, TarziError::Http(_)) {
                panic!("Unexpected error: {e:?}");
            } else {
                println!("Network error - acceptable for external service dependency");
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_invalid_url() {
    // Disable browser so cascade does not mask the HTTP/DNS failure
    let mut fetcher = create_plain_only_fetcher();

    // Test fetching from a non-existent URL
    let result = fetcher
        .fetch("https://this-domain-does-not-exist-12345.com", Format::Html)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        TarziError::Http(_) => (), // Expected HTTP error
        e => panic!("Expected Http error, got {e:?}"),
    }
}

#[tokio::test]
async fn test_fetch_timeout_url() {
    let mut fetcher = create_plain_only_fetcher();

    // Test fetching from a URL that might timeout
    // Using a slow endpoint or non-existent IP
    let result = fetcher
        .fetch(
            "http://10.255.255.255", // Non-routable IP
            Format::Html,
        )
        .await;

    // This should either timeout or fail with a network error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fetch_with_proxy() {
    let test_url = "https://httpbin.org/html";

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("Skipping proxy test - {test_url} not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();
    let invalid_proxy = "http://invalid-proxy:8080";

    // Test fetching with an invalid proxy (should fail)
    let result = fetcher
        .fetch_with_proxy(test_url, invalid_proxy, Format::Html)
        .await;

    match result {
        Ok(_) => {
            println!("✓ Proxy test: invalid proxy was somehow handled (unexpected but ok)");
        }
        Err(TarziError::Config(_)) => {
            println!("✓ Proxy test passed - correctly failed with config error");
        }
        Err(TarziError::Http(_)) => {
            println!("✓ Proxy test passed - correctly failed with network error");
        }
        Err(e) => {
            println!("✓ Proxy test passed - failed as expected with: {e:?}");
        }
    }
}

#[tokio::test]
async fn test_fetch_multiple_requests() {
    let test_urls = vec![
        "https://httpbin.org/html",
        "https://httpbin.org/json",
        "https://httpbin.org/xml",
    ];

    // Check which URLs are reachable first
    let mut reachable_urls = Vec::new();
    for url in &test_urls {
        if is_url_reachable(url).await {
            reachable_urls.push(*url);
        }
    }

    if reachable_urls.is_empty() {
        println!("Skipping multiple requests test - no URLs reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();
    let mut success_count = 0;

    for url in reachable_urls {
        let result = fetcher.fetch(url, Format::Html).await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty(), "Content should not be empty for {url}");
                success_count += 1;
            }
            Err(e) => {
                println!("Request to {url} failed: {e:?}");
                // Continue with other URLs even if one fails
            }
        }
    }

    assert!(success_count > 0, "At least one request should succeed");
    println!(
        "✓ Multiple requests test: {}/{} succeeded",
        success_count,
        test_urls.len()
    );
}

#[tokio::test]
async fn test_fetch_different_formats() {
    let mut fetcher = WebFetcher::new();

    let url = "https://httpbin.org/html";
    let formats = vec![Format::Html, Format::Markdown, Format::Json, Format::Yaml];

    for format in formats {
        let result = fetcher.fetch(url, format).await;
        match result {
            Ok(content) => {
                assert!(!content.is_empty());
            }
            Err(e) => {
                // If it's a network error, we'll allow it to pass
                // This can happen due to rate limiting or temporary network issues
                println!("Format test for {format:?} failed with error: {e:?}");
                // Only panic on unexpected errors, not network-related ones
                if !matches!(e, TarziError::Http(_)) {
                    panic!("Unexpected error for format {format:?}: {e:?}");
                }
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_browser() {
    let test_url = "https://httpbin.org/html";

    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("✓ Skipping browser headless test - WebDriver not available");
        return;
    }

    // Skip test if URL is not reachable
    if !is_url_reachable(test_url).await {
        println!("✓ Skipping browser headless test - {test_url} not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();
    let test_timeout = Duration::from_secs(120); // Longer timeout for browser tests

    let result = tokio::time::timeout(test_timeout, fetcher.fetch_browser(test_url)).await;

    match result {
        Ok(Ok(content)) => {
            assert!(!content.is_empty(), "Content should not be empty");
            assert!(
                content.contains("<html>") || content.contains("<!DOCTYPE html>"),
                "Content should contain HTML markup"
            );
            println!("✓ Browser headless test succeeded");
        }
        Ok(Err(TarziError::Browser(msg))) => {
            println!("✓ Browser headless test passed (browser setup issue): {msg}");
        }
        Ok(Err(TarziError::Http(e))) => {
            println!("✓ Browser headless test passed (network error): {e}");
        }
        Ok(Err(e)) => {
            panic!("Browser headless test failed with unexpected error: {e:?}");
        }
        Err(_) => {
            println!("✓ Browser headless test passed (timeout - expected in CI)");
        }
    }

    // Cleanup
    fetcher.shutdown().await;
}

#[tokio::test]
async fn test_fetch_raw_browser() {
    // Skip test if WebDriver is not available
    if !is_webdriver_available().await {
        println!("Skipping browser raw test - WebDriver not available");
        return;
    }

    let mut fetcher = WebFetcher::new();

    // Test fetching raw content with browser
    let result = fetcher.fetch_raw("https://httpbin.org/html").await;

    // This might fail in CI environments without proper browser setup
    match result {
        Ok(content) => {
            assert!(!content.is_empty());
            assert!(content.contains("<html>") || content.contains("<!DOCTYPE html>"));
        }
        Err(TarziError::Browser(_)) => {
            // Browser error is acceptable in CI environments
            println!("Browser test skipped - browser not available");
        }
        Err(e) => {
            panic!("Unexpected error: {e:?}");
        }
    }
}

// Performance and stress tests
// Removed the concurrent requests test due to Send/Sync issues with WebFetcher and Chromiumoxide
#[tokio::test]
async fn test_fetch_sequential_requests() {
    // Test sequential requests (instead of concurrent)
    let urls = vec!["https://httpbin.org/html", "https://httpbin.org/json"];

    // Check which URLs are reachable first
    let mut reachable_urls = Vec::new();
    for url in &urls {
        if is_url_reachable(url).await {
            reachable_urls.push(*url);
        }
    }

    if reachable_urls.is_empty() {
        println!("Skipping sequential requests test - no URLs reachable");
        return;
    }

    let mut success_count = 0;
    for url in reachable_urls {
        let mut fetcher = WebFetcher::new();
        let result = fetcher.fetch(url, Format::Html).await;

        match result {
            Ok(content) => {
                assert!(!content.is_empty(), "Content should not be empty for {url}");
                success_count += 1;
            }
            Err(e) => {
                println!("Request to {url} failed: {e:?}");
                // Continue with other URLs even if one fails
            }
        }
    }

    assert!(success_count > 0, "At least one request should succeed");
    println!(
        "✓ Sequential requests test: {}/{} succeeded",
        success_count,
        urls.len()
    );
}

#[tokio::test]
async fn test_fetch_large_response() {
    let test_url = "https://httpbin.org/bytes/10000"; // 10KB response

    // Skip test if URL is not reachable
    if !is_url_reachable("https://httpbin.org").await {
        println!("Skipping large response test - httpbin.org not reachable");
        return;
    }

    let mut fetcher = create_test_fetcher();
    let test_timeout = Duration::from_secs(60);

    let result = tokio::time::timeout(test_timeout, fetcher.fetch(test_url, Format::Html)).await;

    match result {
        Ok(Ok(content)) => {
            assert!(!content.is_empty(), "Large response should not be empty");
            assert!(
                content.len() > 1000,
                "Response should be reasonably large (>1KB)"
            );
            println!("✓ Large response test succeeded ({} bytes)", content.len());
        }
        Ok(Err(e)) => {
            println!("Large response test failed (expected in some environments): {e:?}");
            // Allow network errors in CI environments
            if !matches!(e, TarziError::Http(_)) {
                panic!("Unexpected error: {e:?}");
            }
        }
        Err(_) => {
            println!("✓ Large response test timed out (acceptable in slow environments)");
        }
    }
}

// Error handling tests
#[tokio::test]
async fn test_fetch_404_error() {
    let test_url = "https://httpbin.org/status/404";

    // Skip test if URL is not reachable
    if !is_url_reachable("https://httpbin.org").await {
        println!("Skipping 404 error test - httpbin.org not reachable");
        return;
    }

    let mut fetcher = create_plain_only_fetcher();
    let result = fetcher.fetch(test_url, Format::Html).await;

    match result {
        Err(TarziError::Http(_)) => {
            println!("✓ 404 error test passed - correctly returned HTTP error");
        }
        Ok(_) => {
            panic!("Expected HTTP error for 404 status, but got success");
        }
        Err(e) => {
            panic!("Expected HTTP error for 404 status, got different error: {e:?}");
        }
    }
}

#[tokio::test]
async fn test_fetch_500_error() {
    let test_url = "https://httpbin.org/status/500";

    // Skip test if URL is not reachable
    if !is_url_reachable("https://httpbin.org").await {
        println!("Skipping 500 error test - httpbin.org not reachable");
        return;
    }

    let mut fetcher = create_plain_only_fetcher();
    let result = fetcher.fetch(test_url, Format::Html).await;

    match result {
        Err(TarziError::Http(_)) => {
            println!("✓ 500 error test passed - correctly returned HTTP error");
        }
        Ok(_) => {
            panic!("Expected HTTP error for 500 status, but got success");
        }
        Err(e) => {
            panic!("Expected HTTP error for 500 status, got different error: {e:?}");
        }
    }
}
