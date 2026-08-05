use std::time::{Duration, Instant};
use tarzi::config::Config;
use tarzi::converter::Format;
use tarzi::search::SearchEngine;
use tarzi::search::parser::ParserFactory;
use tarzi::search::types::SearchEngineType;

/// Default test timeout for search operations
const SEARCH_TEST_TIMEOUT: Duration = Duration::from_secs(120);
const WEBDRIVER_URL: &str = "http://localhost:9515"; // Chrome WebDriver default port
const WEBDRIVER_DRIVER: &str = "chromedriver";

/// Helper function to check if WebDriver is available at a specific URL
async fn is_webdriver_available_at_url(url: &str) -> bool {
    use reqwest;
    use std::time::Duration;
    use tokio::time::timeout;

    match timeout(
        Duration::from_secs(2),
        reqwest::get(&format!("{url}/status")),
    )
    .await
    {
        Ok(Ok(response)) => response.status().is_success(),
        _ => false,
    }
}

#[tokio::test]
async fn test_parser_functionality() {
    println!("=== Testing Parser Functionality ===");

    let factory = ParserFactory::new();

    let engine_types = vec![
        SearchEngineType::Bing,
        SearchEngineType::DuckDuckGo,
        SearchEngineType::Google,
        SearchEngineType::GoogleSerper,
        SearchEngineType::BraveSearch,
        SearchEngineType::Baidu,
        SearchEngineType::SougouWeixin,
    ];

    for engine_type in engine_types {
        println!("Testing parser for {engine_type:?}");

        let parser = factory.get_parser(&engine_type);
        assert!(!parser.name().is_empty(), "Parser should have a name");
        assert!(
            parser.supports(&engine_type),
            "Parser should support its own engine type"
        );

        // Test with sample HTML content
        let sample_html = r#"
            <html><body>
                <h2><a href="https://example1.com">Test Result 1</a></h2>
                <p>Test snippet 1</p>
                <h2><a href="https://example2.com">Test Result 2</a></h2>
                <p>Test snippet 2</p>
            </body></html>
        "#;

        let results = parser.parse(sample_html, 5);
        match results {
            Ok(results) => {
                println!("  Parsed {} results", results.len());
                for result in results {
                    assert!(
                        !result.title.is_empty() || !result.url.is_empty(),
                        "Result should have title or URL"
                    );
                }
            }
            Err(e) => {
                println!("  Parse error: {e}");
            }
        }
    }
}

#[tokio::test]
async fn test_parser_edge_cases() {
    println!("=== Testing Parser Edge Cases ===");

    let factory = ParserFactory::new();
    let parser = factory.get_parser(&SearchEngineType::DuckDuckGo);

    let edge_cases = vec![
        ("", "empty string"),
        ("<html><body></body></html>", "empty HTML"),
        ("<html><malformed>", "malformed HTML"),
        ("Not HTML at all", "plain text"),
        ("🎉🎊🎈", "emoji only"),
    ];

    for (input, description) in edge_cases {
        println!("Testing edge case: {description}");

        let results = parser.parse(input, 5);
        match results {
            Ok(results) => {
                println!("  Handled gracefully: {} results", results.len());
                // Edge cases should return empty results or handle gracefully
                assert!(results.len() <= 5, "Should respect limit");
            }
            Err(e) => {
                println!("  Rejected with error: {e}");
                // Error is also acceptable for edge cases
            }
        }
    }
}

#[tokio::test]
async fn test_parser_limits() {
    println!("=== Testing Parser Limits ===");

    let factory = ParserFactory::new();
    let parser = factory.get_parser(&SearchEngineType::DuckDuckGo);

    let sample_html = r#"
        <html><body>
            <h2><a href="https://example1.com">Result 1</a></h2>
            <h2><a href="https://example2.com">Result 2</a></h2>
            <h2><a href="https://example3.com">Result 3</a></h2>
            <h2><a href="https://example4.com">Result 4</a></h2>
            <h2><a href="https://example5.com">Result 5</a></h2>
        </body></html>
    "#;

    let limits = vec![0, 1, 3, 5, 10];

    for limit in limits {
        println!("Testing with limit: {limit}");

        let results = parser.parse(sample_html, limit);
        match results {
            Ok(results) => {
                println!("  Returned {} results", results.len());
                if limit > 0 {
                    assert!(results.len() <= limit, "Should respect limit");
                } else {
                    assert_eq!(results.len(), 0, "Limit 0 should return empty results");
                }
            }
            Err(e) => {
                println!("  Parse error: {e}");
            }
        }
    }
}

#[tokio::test]
async fn test_all_engines_parser_factory() {
    println!("=== Testing Parser Factory for All Engines ===");

    let factory = ParserFactory::new();

    let engine_types = vec![
        SearchEngineType::Bing,
        SearchEngineType::DuckDuckGo,
        SearchEngineType::Google,
        SearchEngineType::GoogleSerper,
        SearchEngineType::BraveSearch,
        SearchEngineType::Baidu,
        SearchEngineType::SougouWeixin,
    ];

    for engine_type in engine_types {
        let parser = factory.get_parser(&engine_type);
        println!("Testing parser for {:?}: {}", engine_type, parser.name());

        // Test that parser supports its own engine type
        assert!(
            parser.supports(&engine_type),
            "Parser {} should support {:?}",
            parser.name(),
            engine_type
        );
    }
}

#[tokio::test]
async fn test_search_engine_error_handling() {
    println!("=== Testing Search Engine Error Handling ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let mut engine = SearchEngine::from_config(&config);

    // Test with empty query
    let results = engine.search("", 5).await;
    match results {
        Ok(results) => println!("Empty query returned {} results", results.len()),
        Err(e) => println!("Empty query returned error: {e}"),
    }

    // Test with very long query
    let long_query = "a".repeat(1000);
    let results = engine.search(&long_query, 5).await;
    match results {
        Ok(results) => println!("Long query returned {} results", results.len()),
        Err(e) => println!("Long query returned error: {e}"),
    }
}

#[tokio::test]
async fn test_search_query_edge_cases() {
    println!("=== Testing Search Query Edge Cases ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let mut engine = SearchEngine::from_config(&config);

    let medium_query = "a".repeat(100);
    let long_query = "a".repeat(1000);

    let edge_queries = vec![
        "",                 // Empty query
        "a",                // Single character
        &medium_query,      // Medium length
        &long_query,        // Very long
        "!@#$%^&*()",       // Special characters
        "rust programming", // Normal query
    ];

    for query in edge_queries {
        let results = engine.search(query, 3).await;
        match results {
            Ok(results) => println!("Query '{}': {} results", query, results.len()),
            Err(e) => println!("Query '{query}': error - {e}"),
        }
    }
}

#[tokio::test]
async fn test_search_limit_edge_cases_self_managed() {
    println!("=== Testing Search Limit Edge Cases (Self-Managed) ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let mut engine = SearchEngine::from_config(&config);

    let limits = vec![0, 1, 5, 10];

    for limit in limits {
        let results = engine.search("test query", limit).await;
        match results {
            Ok(results) => {
                println!("Limit {}: returned {} results", limit, results.len());
                if limit > 0 {
                    assert!(results.len() <= limit, "Should respect limit");
                }
            }
            Err(e) => println!("Limit {limit}: error - {e}"),
        }
    }
}

#[tokio::test]
async fn test_concurrent_searches() {
    println!("=== Testing Concurrent Searches ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let queries = vec!["rust programming", "python tutorial", "machine learning"];

    let mut handles = Vec::new();

    for query in &queries {
        let config_clone = config.clone();
        let query_string = query.to_string();
        let handle = tokio::spawn(async move {
            let mut engine = SearchEngine::from_config(&config_clone);
            engine.search(&query_string, 2).await
        });
        handles.push(handle);
    }

    let mut successful_searches = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(results)) => {
                println!("Concurrent search successful: {} results", results.len());
                successful_searches += 1;
            }
            Ok(Err(e)) => println!("Concurrent search failed: {e}"),
            Err(e) => println!("Task join failed: {e}"),
        }
    }

    println!(
        "Concurrent searches: {}/{} successful",
        successful_searches,
        queries.len()
    );
}

#[tokio::test]
async fn test_parser_performance() {
    println!("=== Testing Parser Performance ===");

    let factory = ParserFactory::new();

    let test_cases = vec![
        ("Bing", SearchEngineType::Bing),
        ("DuckDuckGo", SearchEngineType::DuckDuckGo),
        ("Google", SearchEngineType::Google),
        ("GoogleSerper", SearchEngineType::GoogleSerper),
        ("Brave", SearchEngineType::BraveSearch),
        ("Baidu", SearchEngineType::Baidu),
        ("SogouWeixin", SearchEngineType::SougouWeixin),
    ];

    for (name, engine_type) in test_cases {
        println!("\nTesting {name} parser performance...");

        let parser = factory.get_parser(&engine_type);
        let test_html = r#"<html><body>
                <h2><a href="https://example1.com">Test Result 1</a></h2>
                <p>Test snippet 1</p>
                <h2><a href="https://example2.com">Test Result 2</a></h2>
                <p>Test snippet 2</p>
                <h2><a href="https://example3.com">Test Result 3</a></h2>
                <p>Test snippet 3</p>
            </body></html>"#
            .to_string();

        let iterations = 100;
        let mut total_time = Duration::new(0, 0);

        for _ in 0..iterations {
            let start = Instant::now();
            let _results = parser.parse(&test_html, 5);
            let duration = start.elapsed();
            total_time += duration;
        }

        let avg_time = total_time / iterations;
        println!("  Average parse time: {avg_time:?}");
        println!("  Total time for {iterations} iterations: {total_time:?}");
    }
}

#[tokio::test]
async fn test_search_throughput() {
    println!("=== Testing Search Throughput ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let mut engine = SearchEngine::from_config(&config);

    let test_queries = vec![
        "rust programming",
        "python tutorial",
        "machine learning",
        "web development",
        "data science",
    ];

    let mut total_time = Duration::new(0, 0);
    let mut successful_queries = 0;

    for query in &test_queries {
        let start = Instant::now();
        let result = engine.search(query, 3).await;
        let duration = start.elapsed();

        match result {
            Ok(results) => {
                successful_queries += 1;
                total_time += duration;
                println!(
                    "  Query '{}': {} results in {:?}",
                    query,
                    results.len(),
                    duration
                );
            }
            Err(e) => {
                println!("  Query '{query}': Failed in {duration:?} - {e}");
            }
        }
    }

    if successful_queries > 0 {
        let avg_time = total_time / successful_queries;
        println!(
            "\n  Summary: {}/{} queries successful",
            successful_queries,
            test_queries.len()
        );
        println!("  Average response time: {avg_time:?}");
        println!("  Total time: {total_time:?}");
    } else {
        println!("  No successful queries");
    }
}

#[tokio::test]
async fn test_search_latency_percentiles() {
    println!("=== Testing Search Latency Percentiles ===");

    let mut config = Config::new();
    config.search.engine = "duckduckgo".to_string();

    let mut engine = SearchEngine::from_config(&config);
    let num_requests = 5; // Small number for integration test
    let mut latencies = Vec::new();

    println!("Performing {num_requests} search requests to measure latency distribution...");

    for i in 0..num_requests {
        let query = format!("test query {i}");
        let start = Instant::now();
        let result = engine.search(&query, 2).await;
        let duration = start.elapsed();

        match result {
            Ok(results) => {
                latencies.push(duration);
                println!(
                    "Request {}: {} results in {:?}",
                    i + 1,
                    results.len(),
                    duration
                );
            }
            Err(e) => {
                println!("Request {}: failed in {:?} - {}", i + 1, duration, e);
            }
        }
    }

    if !latencies.is_empty() {
        latencies.sort();
        let p50 = latencies[latencies.len() / 2];
        let p90 = latencies[(latencies.len() * 9) / 10];
        let p95 = latencies[(latencies.len() * 19) / 20];

        println!("\nLatency percentiles:");
        println!("  P50 (median): {p50:?}");
        println!("  P90: {p90:?}");
        println!("  P95: {p95:?}");
        println!("  Min: {:?}", latencies[0]);
        println!("  Max: {:?}", latencies[latencies.len() - 1]);
    }
}

/// Helper function to create a config with external WebDriver URL
fn create_external_webdriver_config(search_engine: &str) -> Config {
    let mut config = Config::default();
    config.fetcher.web_driver_url = Some(WEBDRIVER_URL.to_string());
    config.fetcher.web_driver = WEBDRIVER_DRIVER.to_string();
    config.search.engine = search_engine.to_string();
    config
}

/// Test search with external WebDriver - simplified single function implementation
#[tokio::test]
async fn test_search_external_webdriver() {
    // Check if external WebDriver is available
    if !is_webdriver_available_at_url(WEBDRIVER_URL).await {
        println!(
            "⚠ External Chrome WebDriver not available at {WEBDRIVER_URL} — skipping. \
             Start with: chromedriver --port=9515"
        );
        return;
    }

    let config = create_external_webdriver_config("duckduckgo");
    let query = "rust programming language";
    let mut engine = SearchEngine::from_config(&config);

    // Verify the engine type is correct
    assert_eq!(
        engine.engine_type(),
        &SearchEngineType::DuckDuckGo,
        "Expected DuckDuckGo engine type"
    );

    // Perform search with timeout
    let result = tokio::time::timeout(SEARCH_TEST_TIMEOUT, engine.search(query, 5)).await;

    match result {
        Ok(Ok(results)) => {
            assert!(!results.is_empty(), "Search returned no results");
            println!(
                "✓ DuckDuckGo search with external WebDriver successful: {} results",
                results.len()
            );

            // Verify result structure
            for (i, result) in results.iter().enumerate() {
                assert!(!result.title.is_empty(), "Result {i} has empty title");
                assert!(!result.url.is_empty(), "Result {i} has empty URL");
                assert!(
                    result.url.starts_with("http"),
                    "Result {} has invalid URL: {}",
                    i,
                    result.url
                );
                assert_eq!(result.rank, i + 1, "Result {i} has incorrect rank");
            }
        }
        Ok(Err(e)) => panic!("DuckDuckGo search with external WebDriver failed: {e}"),
        Err(_) => panic!(
            "DuckDuckGo search with external WebDriver timed out after {SEARCH_TEST_TIMEOUT:?}"
        ),
    }
}

/// Test search with content fetching using external WebDriver
#[tokio::test]
async fn test_search_with_content_external_webdriver() {
    if !is_webdriver_available_at_url(WEBDRIVER_URL).await {
        println!(
            "⚠ External Chrome WebDriver not available at {WEBDRIVER_URL} — skipping. \
             Start with: chromedriver --port=9515"
        );
        return;
    }

    let config = create_external_webdriver_config("duckduckgo");
    let query = "rust programming tutorial";
    let limit = 3;

    let mut engine = SearchEngine::from_config(&config);

    // Perform search with content fetching
    let result = tokio::time::timeout(
        SEARCH_TEST_TIMEOUT,
        engine.search_with_content(query, limit, Format::Markdown),
    )
    .await;

    match result {
        Ok(Ok(results_with_content)) => {
            println!(
                "✓ Search with content using external WebDriver successful: {} results",
                results_with_content.len()
            );

            // Verify results structure
            for (i, (search_result, content)) in results_with_content.iter().enumerate() {
                assert!(
                    !search_result.title.is_empty(),
                    "Result {i} has empty title"
                );
                assert!(!search_result.url.is_empty(), "Result {i} has empty URL");
                assert!(
                    search_result.url.starts_with("http"),
                    "Result {} has invalid URL: {}",
                    i,
                    search_result.url
                );
                assert_eq!(search_result.rank, i + 1, "Result {i} has incorrect rank");

                // Content should not be empty for successful fetches
                if !content.is_empty() {
                    assert!(
                        content.len() > 10,
                        "Result {} content too short: {} chars",
                        i,
                        content.len()
                    );
                }
            }
        }
        Ok(Err(e)) => {
            panic!("Search with content using external WebDriver failed: {e}");
        }
        Err(_) => {
            panic!(
                "Search with content using external WebDriver timed out after {SEARCH_TEST_TIMEOUT:?}"
            );
        }
    }
}
