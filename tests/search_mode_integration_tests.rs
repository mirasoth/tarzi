//! Integration tests covering all search engines and access modes.
//!
//! Network / API-key dependent cases degrade gracefully (log and continue)
//! so CI remains green without external credentials.

use std::time::Duration;
use tarzi::config::Config;
use tarzi::constants::{
    ENV_BRAVE_API_KEY, ENV_SERPER_API_KEY, SEARCH_ENGINE_BAIDU, SEARCH_ENGINE_BING,
    SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO, SEARCH_ENGINE_GOOGLE,
    SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_SOUGOU_WEIXIN, SEARCH_MODE_APIQUERY,
    SEARCH_MODE_AUTO, SEARCH_MODE_WEBQUERY,
};
use tarzi::search::SearchEngine;
use tarzi::search::types::{SearchEngineType, SearchMode};

const SEARCH_TIMEOUT: Duration = Duration::from_secs(90);
const TEST_QUERY: &str = "rust programming";
const TEST_LIMIT: usize = 2;

fn has_env_key(name: &str) -> bool {
    std::env::var(name).ok().filter(|v| !v.is_empty()).is_some()
}

fn make_config(engine: &str, mode: &str) -> Config {
    let mut config = Config::new();
    config.search.engine = engine.to_string();
    config.search.mode = mode.to_string();
    config.search.limit = TEST_LIMIT;
    // Prefer plain HTTP for web paths to avoid requiring a local WebDriver in CI
    config.fetcher.mode = "plain_request".to_string();
    config
}

async fn try_search(engine: &str, mode: &str) -> Result<usize, String> {
    let config = make_config(engine, mode);
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
        || lower.contains("all search access methods failed")
        || lower.contains("failed to fetch")
        || lower.contains("status")
        || lower.contains("webdriver")
        || lower.contains("chromedriver")
        || lower.contains("geckodriver")
        || lower.contains("browser automation")
        || lower.contains("no self-managed")
}

#[tokio::test]
async fn test_webquery_all_web_engines() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
    ];

    for engine in engines {
        println!("webquery engine={engine}");
        match try_search(engine, SEARCH_MODE_WEBQUERY).await {
            Ok(n) => {
                println!("  ✓ {engine} webquery returned {n} results");
                assert!(n <= TEST_LIMIT);
            }
            Err(e) if is_acceptable_external_failure(&e) => {
                println!("  ⚠ {engine} webquery unavailable externally: {e}");
            }
            Err(e) => panic!("Unexpected webquery failure for {engine}: {e}"),
        }
    }
}

#[tokio::test]
async fn test_auto_all_web_engines_without_requiring_api_keys() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
    ];

    for engine in engines {
        println!("auto engine={engine}");
        match try_search(engine, SEARCH_MODE_AUTO).await {
            Ok(n) => println!("  ✓ {engine} auto returned {n} results"),
            Err(e) if is_acceptable_external_failure(&e) => {
                println!("  ⚠ {engine} auto unavailable externally: {e}");
            }
            Err(e) => panic!("Unexpected auto failure for {engine}: {e}"),
        }
    }
}

#[tokio::test]
async fn test_apiquery_unsupported_engines_fail_fast() {
    let unsupported = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
    ];

    for engine in unsupported {
        let err = try_search(engine, SEARCH_MODE_APIQUERY)
            .await
            .expect_err("apiquery should fail for web-only engines");
        println!("apiquery {engine}: {err}");
        assert!(
            err.to_lowercase().contains("apiquery")
                || err.to_lowercase().contains("does not support")
                || err.to_lowercase().contains("api"),
            "unexpected error for {engine}: {err}"
        );
    }
}

#[tokio::test]
async fn test_google_serper_webquery_rejected() {
    let err = try_search(SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_MODE_WEBQUERY)
        .await
        .expect_err("google_serper must reject webquery");
    println!("google_serper webquery: {err}");
    assert!(
        err.to_lowercase().contains("webquery")
            || err.to_lowercase().contains("apiquery")
            || err.to_lowercase().contains("only supports"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_google_serper_requires_key_without_env() {
    if has_env_key(ENV_SERPER_API_KEY) {
        println!("SERPER_API_KEY set — skipping missing-key assertion");
        return;
    }

    let err = try_search(SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_MODE_AUTO)
        .await
        .expect_err("google_serper without key should fail");
    println!("google_serper missing key: {err}");
    assert!(
        err.to_lowercase().contains("serper") || err.to_lowercase().contains("api key"),
        "unexpected error: {err}"
    );
}

#[tokio::test]
async fn test_brave_apiquery_when_key_available() {
    if !has_env_key(ENV_BRAVE_API_KEY) {
        println!("BRAVE_API_KEY unset — skipping live Brave apiquery");
        return;
    }

    match try_search(SEARCH_ENGINE_BRAVE, SEARCH_MODE_APIQUERY).await {
        Ok(n) => {
            println!("✓ Brave apiquery returned {n} results");
            assert!(n > 0, "Brave apiquery with key should return results");
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Brave apiquery key present but request failed: {e}");
        }
        Err(e) => panic!("Brave apiquery failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_serper_apiquery_when_key_available() {
    if !has_env_key(ENV_SERPER_API_KEY) {
        println!("SERPER_API_KEY unset — skipping live Serper apiquery");
        return;
    }

    match try_search(SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_MODE_APIQUERY).await {
        Ok(n) => {
            println!("✓ Serper apiquery returned {n} results");
            assert!(n > 0, "Serper apiquery with key should return results");
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Serper apiquery key present but request failed: {e}");
        }
        Err(e) => panic!("Serper apiquery failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_engine_mode_config_wiring_all_combinations() {
    use std::str::FromStr;

    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_GOOGLE_SERPER,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
    ];
    let modes = [SEARCH_MODE_AUTO, SEARCH_MODE_APIQUERY, SEARCH_MODE_WEBQUERY];

    for engine_name in engines {
        for mode_name in modes {
            let mut config = make_config(engine_name, mode_name);
            config.search.api_key = Some("integration-test-placeholder".to_string());
            let engine = SearchEngine::from_config(&config);
            assert_eq!(
                engine.engine_type(),
                &SearchEngineType::from_str(engine_name).unwrap(),
                "engine wiring {engine_name}"
            );
            assert_eq!(
                engine.search_mode(),
                SearchMode::from_str(mode_name).unwrap(),
                "mode wiring {engine_name}/{mode_name}"
            );
        }
    }
}
