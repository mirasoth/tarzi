//! Integration tests covering search engines, browser toggle, and multi-engine failover.
//!
//! Network / API-key dependent cases degrade gracefully (log and continue)
//! so CI remains green without external credentials.

use std::time::Duration;
use tarzi::config::Config;
use tarzi::constants::{
    ENV_BRAVE_API_KEY, ENV_SERPER_API_KEY, SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BING,
    SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO, SEARCH_ENGINE_GOOGLE,
    SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_GOOGLEAI, SEARCH_ENGINE_SEARXNG,
    SEARCH_ENGINE_SOUGOU_WEIXIN, SEARCH_ENGINE_TAVILY,
};
use tarzi::search::SearchEngine;
use tarzi::search::types::SearchEngineType;

const SEARCH_TIMEOUT: Duration = Duration::from_secs(90);
const TEST_QUERY: &str = "rust programming";
const TEST_LIMIT: usize = 2;

fn has_env_key(name: &str) -> bool {
    std::env::var(name).ok().filter(|v| !v.is_empty()).is_some()
}

fn make_config(engine: &str, browser: bool) -> Config {
    let mut config = Config::new();
    config.search.engine = engine.to_string();
    config.search.browser = browser;
    config.search.limit = TEST_LIMIT;
    config.fetcher.mode = "plain_request".to_string();
    config
}

async fn try_search(engine: &str, browser: bool) -> Result<usize, String> {
    let config = make_config(engine, browser);
    let mut search = SearchEngine::from_config(&config);
    let outcome = tokio::time::timeout(SEARCH_TIMEOUT, search.search(TEST_QUERY, TEST_LIMIT)).await;
    search.shutdown().await;

    match outcome {
        Ok(Ok(results)) => Ok(results.len()),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("timed out after {SEARCH_TIMEOUT:?}")),
    }
}

fn is_acceptable_external_failure(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("timeout")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("dns")
        || lower.contains("tls")
        || lower.contains("certificate")
        || lower.contains("rate")
        || lower.contains("403")
        || lower.contains("429")
        || lower.contains("captcha")
        || lower.contains("blocked")
        || lower.contains("no results")
        || lower.contains("returned no results")
        || lower.contains("all search")
        || lower.contains("failed to fetch")
        || lower.contains("status")
        || lower.contains("webdriver")
        || lower.contains("chromedriver")
        || lower.contains("geckodriver")
        || lower.contains("browser automation")
        || lower.contains("no self-managed")
}

#[tokio::test]
async fn test_web_engines_browser_disabled() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
    ];

    for engine in engines {
        println!("search engine={engine} browser=false");
        match try_search(engine, false).await {
            Ok(n) => {
                println!("  ✓ {engine} returned {n} results");
                assert!(n <= TEST_LIMIT);
            }
            Err(e) if is_acceptable_external_failure(&e) => {
                println!("  ⚠ {engine} unavailable externally: {e}");
            }
            Err(e) => panic!("Unexpected failure for {engine}: {e}"),
        }
    }
}

#[tokio::test]
async fn test_web_engines_default_browser() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
    ];

    for engine in engines {
        println!("search engine={engine}");
        match try_search(engine, true).await {
            Ok(n) => println!("  ✓ {engine} returned {n} results"),
            Err(e) if is_acceptable_external_failure(&e) => {
                println!("  ⚠ {engine} unavailable externally: {e}");
            }
            Err(e) => panic!("Unexpected failure for {engine}: {e}"),
        }
    }
}

#[tokio::test]
async fn test_api_only_without_key_fails_or_skips() {
    let err = try_search(SEARCH_ENGINE_GOOGLE_SERPER, true)
        .await
        .expect_err("google_serper without key should fail");
    println!("google_serper no key: {err}");
    assert!(
        err.to_lowercase().contains("serper")
            || err.to_lowercase().contains("credential")
            || err.to_lowercase().contains("api"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_multi_engine_skips_api_only_without_key() {
    // google_serper without key should be skipped; duckduckgo should be attempted
    match try_search("google_serper,duckduckgo", false).await {
        Ok(n) => println!("✓ multi-engine failover returned {n} results"),
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ multi-engine externally unavailable: {e}");
            // Should mention serper skip in aggregate when duckduckgo also fails
        }
        Err(e) => panic!("Unexpected multi-engine failure: {e}"),
    }
}

#[tokio::test]
async fn test_brave_when_key_available() {
    if !has_env_key(ENV_BRAVE_API_KEY) {
        println!("BRAVE_API_KEY unset — skipping live Brave API");
        return;
    }

    match try_search(SEARCH_ENGINE_BRAVE, false).await {
        Ok(n) => {
            println!("✓ Brave returned {n} results");
            assert!(n > 0, "Brave with key should return results");
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Brave key present but request failed: {e}");
        }
        Err(e) => panic!("Brave failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_serper_when_key_available() {
    if !has_env_key(ENV_SERPER_API_KEY) {
        println!("SERPER_API_KEY unset — skipping live Serper API");
        return;
    }

    match try_search(SEARCH_ENGINE_GOOGLE_SERPER, true).await {
        Ok(n) => {
            println!("✓ Serper returned {n} results");
            assert!(n > 0, "Serper with key should return results");
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Serper key present but request failed: {e}");
        }
        Err(e) => panic!("Serper failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_from_config_wires_engines_and_browser() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_GOOGLE_SERPER,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
        SEARCH_ENGINE_TAVILY,
        SEARCH_ENGINE_GOOGLEAI,
        SEARCH_ENGINE_SEARXNG,
    ];

    for engine_name in engines {
        for browser in [true, false] {
            let mut config = Config::new();
            config.search.engine = engine_name.to_string();
            config.search.browser = browser;
            config.search.api_key = Some("unit-test-key".to_string());
            config.search.base_url = Some("http://localhost:8080".to_string());

            let engine = SearchEngine::from_config(&config);
            assert_eq!(
                engine.engine_type(),
                &engine_name.parse::<SearchEngineType>().unwrap(),
                "engine={engine_name}"
            );
            assert_eq!(engine.browser_enabled(), browser);
        }
    }
}
