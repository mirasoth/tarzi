//! Comprehensive integration tests for search cascade features:
//! - Access cascade (API → plain HTTP → browser)
//! - `search.browser` / browser toggle
//! - Multi-engine ordered failover
//! - API-only credential probing (env before network)
//! - Config wiring via env and programmatic fields
//!
//! Network / API-key dependent cases degrade gracefully so CI stays green.

use std::sync::Mutex;
use std::time::Duration;
use tarzi::config::Config;
use tarzi::constants::{
    ENV_BRAVE_API_KEY, ENV_GEMINI_API_KEY, ENV_SEARX_HOST, ENV_SERPER_API_KEY,
    ENV_TARZI_SEARCH_BROWSER, ENV_TARZI_SEARCH_ENGINE, ENV_TAVILY_API_KEY, SEARCH_ENGINE_BAIDU,
    SEARCH_ENGINE_BING, SEARCH_ENGINE_BRAVE, SEARCH_ENGINE_DUCKDUCKGO, SEARCH_ENGINE_GOOGLE,
    SEARCH_ENGINE_GOOGLE_SERPER, SEARCH_ENGINE_GOOGLEAI, SEARCH_ENGINE_SEARXNG,
    SEARCH_ENGINE_SERPER_ALIAS, SEARCH_ENGINE_SOUGOU_WEIXIN, SEARCH_ENGINE_TAVILY,
};
use tarzi::search::types::{
    AccessMethod, SearchEngineType, default_engine_list, parse_engine_list,
};
use tarzi::search::{
    SearchEngine, has_api_credentials, resolve_access, resolve_api_key, resolve_base_url,
};

const SEARCH_TIMEOUT: Duration = Duration::from_secs(45);
const TEST_QUERY: &str = "rust programming";
const TEST_LIMIT: usize = 2;

/// Serialize env mutation across integration tests in this file.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn has_env_key(name: &str) -> bool {
    std::env::var(name).ok().filter(|v| !v.is_empty()).is_some()
}

fn with_env_lock<F: FnOnce()>(f: F) {
    let _guard = ENV_LOCK.lock().unwrap();
    f();
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
    try_search_with_config(make_config(engine, browser)).await
}

async fn try_search_with_config(config: Config) -> Result<usize, String> {
    let limit = config.search.limit;
    let mut search = SearchEngine::from_config(&config);
    let outcome = tokio::time::timeout(SEARCH_TIMEOUT, search.search(TEST_QUERY, limit)).await;
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
        || lower.contains("proxy")
}

fn web_engines() -> [&'static str; 6] {
    [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BAIDU,
        SEARCH_ENGINE_SOUGOU_WEIXIN,
    ]
}

fn api_only_engines() -> [&'static str; 4] {
    [
        SEARCH_ENGINE_GOOGLE_SERPER,
        SEARCH_ENGINE_TAVILY,
        SEARCH_ENGINE_GOOGLEAI,
        SEARCH_ENGINE_SEARXNG,
    ]
}

fn api_only_env_keys() -> [&'static str; 4] {
    [
        ENV_SERPER_API_KEY,
        ENV_TAVILY_API_KEY,
        ENV_GEMINI_API_KEY,
        ENV_SEARX_HOST,
    ]
}

// ---------------------------------------------------------------------------
// Config / resolve helpers (no network)
// ---------------------------------------------------------------------------

#[test]
fn test_parse_engine_list_integration() {
    assert_eq!(
        parse_engine_list("brave, duckduckgo, bing").unwrap(),
        vec![
            SearchEngineType::BraveSearch,
            SearchEngineType::DuckDuckGo,
            SearchEngineType::Bing,
        ]
    );
    assert_eq!(
        parse_engine_list("serper,google_serper").unwrap(),
        vec![SearchEngineType::GoogleSerper]
    );
    assert_eq!(parse_engine_list("").unwrap(), default_engine_list());
    assert!(parse_engine_list("brave,not_real").is_err());
}

#[test]
fn test_resolve_access_browser_toggle_all_web_engines() {
    for name in web_engines() {
        let engine: SearchEngineType = name.parse().unwrap();
        let with_browser = resolve_access(engine, false, true).unwrap();
        assert_eq!(
            with_browser,
            vec![AccessMethod::PlainHttp, AccessMethod::Browser],
            "{name} browser=true"
        );
        let no_browser = resolve_access(engine, false, false).unwrap();
        assert_eq!(
            no_browser,
            vec![AccessMethod::PlainHttp],
            "{name} browser=false"
        );
    }
}

#[test]
fn test_resolve_access_brave_with_credentials() {
    let full = resolve_access(SearchEngineType::BraveSearch, true, true).unwrap();
    assert_eq!(
        full,
        vec![
            AccessMethod::Api,
            AccessMethod::PlainHttp,
            AccessMethod::Browser
        ]
    );
    let no_browser = resolve_access(SearchEngineType::BraveSearch, true, false).unwrap();
    assert_eq!(no_browser, vec![AccessMethod::Api, AccessMethod::PlainHttp]);
}

#[test]
fn test_resolve_access_api_only_requires_credentials() {
    for name in api_only_engines() {
        let engine: SearchEngineType = name.parse().unwrap();
        assert!(
            resolve_access(engine, false, true).is_err(),
            "{name} without creds must error"
        );
        let methods = resolve_access(engine, true, true).unwrap();
        assert_eq!(methods, vec![AccessMethod::Api], "{name}");
        // Browser flag irrelevant for API-only
        let methods_nb = resolve_access(engine, true, false).unwrap();
        assert_eq!(methods_nb, vec![AccessMethod::Api], "{name} browser=false");
    }
}

#[test]
fn test_from_config_multi_engine_and_browser() {
    let mut config = Config::new();
    config.search.engine = "brave,duckduckgo,bing".to_string();
    config.search.browser = false;

    let engine = SearchEngine::from_config(&config);
    assert_eq!(engine.engine_type(), &SearchEngineType::BraveSearch);
    assert_eq!(
        engine.engines(),
        &[
            SearchEngineType::BraveSearch,
            SearchEngineType::DuckDuckGo,
            SearchEngineType::Bing
        ]
    );
    assert!(!engine.browser_enabled());
}

#[test]
fn test_from_config_serper_alias_and_all_engines() {
    let engines = [
        SEARCH_ENGINE_BING,
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_GOOGLE,
        SEARCH_ENGINE_GOOGLE_SERPER,
        SEARCH_ENGINE_SERPER_ALIAS,
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
            let expected = engine_name.parse::<SearchEngineType>().unwrap();
            assert_eq!(engine.engine_type(), &expected, "engine={engine_name}");
            assert_eq!(engine.browser_enabled(), browser);
            assert_eq!(engine.engines(), &[expected]);
        }
    }
}

#[test]
fn test_config_load_search_browser_and_engine_list_from_env() {
    with_env_lock(|| {
        let keys = [ENV_TARZI_SEARCH_ENGINE, ENV_TARZI_SEARCH_BROWSER];
        let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
        unsafe {
            for &k in &keys {
                std::env::remove_var(k);
            }
            std::env::set_var(ENV_TARZI_SEARCH_ENGINE, "tavily,brave,duckduckgo");
            std::env::set_var(ENV_TARZI_SEARCH_BROWSER, "false");
        }

        let config = Config::load().unwrap();
        assert_eq!(config.search.engine, "tavily,brave,duckduckgo");
        assert!(!config.search.browser);

        let engine = SearchEngine::from_config(&config);
        assert_eq!(
            engine.engines(),
            &[
                SearchEngineType::Tavily,
                SearchEngineType::BraveSearch,
                SearchEngineType::DuckDuckGo
            ]
        );
        assert!(!engine.browser_enabled());

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
fn test_custom_query_pattern_ignored_for_multi_engine() {
    let mut config = Config::new();
    config.search.engine = "bing,duckduckgo".to_string();
    config.search.query_pattern = "https://example.com/?q={query}".to_string();

    let engine = SearchEngine::from_config(&config);
    // Primary engine pattern, not the custom override
    assert_eq!(
        engine.query_pattern(),
        SearchEngineType::Bing.get_query_pattern()
    );
}

#[test]
fn test_custom_query_pattern_applied_for_single_engine() {
    let mut config = Config::new();
    config.search.engine = SEARCH_ENGINE_BING.to_string();
    config.search.query_pattern = "https://example.com/?q={query}".to_string();

    let engine = SearchEngine::from_config(&config);
    assert_eq!(engine.query_pattern(), "https://example.com/?q={query}");
}

// ---------------------------------------------------------------------------
// Credential probing (no network when keys missing)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_api_only_engines_fail_fast_without_credentials() {
    let keys = api_only_env_keys();
    let originals = {
        let _guard = ENV_LOCK.lock().unwrap();
        let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
        unsafe {
            for &k in &keys {
                std::env::remove_var(k);
            }
        }
        originals
    };

    for engine in api_only_engines() {
        let err = try_search(engine, true)
            .await
            .expect_err(&format!("{engine} without credentials should fail"));
        println!("{engine} no creds: {err}");
        let lower = err.to_lowercase();
        assert!(
            lower.contains("api")
                || lower.contains("key")
                || lower.contains("host")
                || lower.contains("credential")
                || lower.contains("searx")
                || lower.contains("serper")
                || lower.contains("tavily")
                || lower.contains("gemini"),
            "{engine} unexpected error: {err}"
        );
        assert!(
            !lower.contains("timed out after") && !lower.contains("failed to fetch"),
            "{engine} should fail before network: {err}"
        );
    }

    let _guard = ENV_LOCK.lock().unwrap();
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
}

#[tokio::test]
async fn test_multi_engine_skips_api_only_without_key_then_tries_web() {
    let keys = api_only_env_keys();
    let originals = {
        let _guard = ENV_LOCK.lock().unwrap();
        let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
        unsafe {
            for &k in &keys {
                std::env::remove_var(k);
            }
        }
        originals
    };

    match try_search("google_serper,tavily,googleai,searxng,duckduckgo", false).await {
        Ok(n) => {
            println!("✓ failover after API skips returned {n} results");
            assert!(n <= TEST_LIMIT);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ web fallback unavailable externally: {e}");
            let lower = e.to_lowercase();
            assert!(
                lower.contains("serper")
                    || lower.contains("tavily")
                    || lower.contains("all search")
                    || lower.contains("duckduckgo"),
                "expected skip/failure details: {e}"
            );
        }
        Err(e) => panic!("Unexpected multi-engine failure: {e}"),
    }

    let _guard = ENV_LOCK.lock().unwrap();
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
}

#[tokio::test]
async fn test_all_api_only_engines_without_creds_aggregate_error() {
    let keys = api_only_env_keys();
    let originals = {
        let _guard = ENV_LOCK.lock().unwrap();
        let originals: Vec<_> = keys.iter().map(|&k| (k, std::env::var(k).ok())).collect();
        unsafe {
            for &k in &keys {
                std::env::remove_var(k);
            }
        }
        originals
    };

    let err = try_search("google_serper,tavily", true)
        .await
        .expect_err("all API-only without keys must fail");
    println!("aggregate: {err}");
    let lower = err.to_lowercase();
    assert!(lower.contains("all search") || lower.contains("serper") || lower.contains("tavily"));

    let _guard = ENV_LOCK.lock().unwrap();
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
}

#[test]
fn test_resolve_api_key_prefers_env_over_config() {
    with_env_lock(|| {
        let original = std::env::var(ENV_BRAVE_API_KEY).ok();
        unsafe {
            std::env::set_var(ENV_BRAVE_API_KEY, "env-brave-key");
        }
        let key = resolve_api_key(
            SearchEngineType::BraveSearch,
            &Some("config-brave-key".to_string()),
        );
        assert_eq!(key.as_deref(), Some("env-brave-key"));
        unsafe {
            std::env::remove_var(ENV_BRAVE_API_KEY);
            if let Some(v) = original {
                std::env::set_var(ENV_BRAVE_API_KEY, v);
            }
        }
    });
}

#[test]
fn test_has_credentials_and_base_url_searxng() {
    assert!(!has_api_credentials(
        SearchEngineType::SearxNG,
        &None,
        &None
    ));
    assert!(has_api_credentials(
        SearchEngineType::SearxNG,
        &None,
        &Some("http://localhost:8080/search".to_string())
    ));
    with_env_lock(|| {
        let original = std::env::var(ENV_SEARX_HOST).ok();
        unsafe {
            std::env::remove_var(ENV_SEARX_HOST);
        }
        let url = resolve_base_url(
            SearchEngineType::SearxNG,
            &Some("http://localhost:9999".to_string()),
        );
        assert!(url.as_ref().is_some_and(|u| u.contains("localhost:9999")));
        unsafe {
            if let Some(v) = original {
                std::env::set_var(ENV_SEARX_HOST, v);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Live network searches (soft-fail)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_web_engines_browser_disabled() {
    for engine in web_engines() {
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
async fn test_selected_web_engines_browser_enabled() {
    // Prefer engines that often work without WebDriver when plain HTTP fails
    for engine in [
        SEARCH_ENGINE_DUCKDUCKGO,
        SEARCH_ENGINE_BRAVE,
        SEARCH_ENGINE_BING,
    ] {
        println!("search engine={engine} browser=true");
        match try_search(engine, true).await {
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
async fn test_multi_engine_web_failover_chain() {
    match try_search("bing,duckduckgo,brave", false).await {
        Ok(n) => {
            println!("✓ web failover chain returned {n} results");
            assert!(n <= TEST_LIMIT);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ web failover chain unavailable: {e}");
        }
        Err(e) => panic!("Unexpected failure: {e}"),
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
            assert!(n > 0);
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
            assert!(n > 0);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Serper key present but request failed: {e}");
        }
        Err(e) => panic!("Serper failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_tavily_when_key_available() {
    if !has_env_key(ENV_TAVILY_API_KEY) {
        println!("TAVILY_API_KEY unset — skipping live Tavily API");
        return;
    }

    match try_search(SEARCH_ENGINE_TAVILY, true).await {
        Ok(n) => {
            println!("✓ Tavily returned {n} results");
            assert!(n > 0);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Tavily key present but request failed: {e}");
        }
        Err(e) => panic!("Tavily failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_googleai_when_key_available() {
    if !has_env_key(ENV_GEMINI_API_KEY) {
        println!("GEMINI_API_KEY unset — skipping live Google AI API");
        return;
    }

    match try_search(SEARCH_ENGINE_GOOGLEAI, true).await {
        Ok(n) => {
            println!("✓ Google AI returned {n} results");
            assert!(n > 0);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ Google AI key present but request failed: {e}");
        }
        Err(e) => panic!("Google AI failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_searxng_when_host_available() {
    if !has_env_key(ENV_SEARX_HOST) {
        println!("SEARX_HOST unset — skipping live SearxNG");
        return;
    }

    match try_search(SEARCH_ENGINE_SEARXNG, true).await {
        Ok(n) => {
            println!("✓ SearxNG returned {n} results");
            assert!(n > 0);
        }
        Err(e) if is_acceptable_external_failure(&e) => {
            println!("⚠ SearxNG host present but request failed: {e}");
        }
        Err(e) => panic!("SearxNG failed unexpectedly: {e}"),
    }
}

#[tokio::test]
async fn test_programmatic_api_key_for_serper() {
    if has_env_key(ENV_SERPER_API_KEY) {
        println!("SERPER_API_KEY set — skipping programmatic-key-only path");
        return;
    }

    // Without a real key, search should still fail with credentials message
    // (proves programmatic path is wired; live success covered when env key exists)
    let mut config = make_config(SEARCH_ENGINE_GOOGLE_SERPER, true);
    config.search.api_key = Some("invalid-test-key-not-for-live".to_string());

    match try_search_with_config(config).await {
        Ok(_) => println!("✓ programmatic key accepted by remote (unexpected but ok)"),
        Err(e)
            if is_acceptable_external_failure(&e)
                || e.to_lowercase().contains("serper")
                || e.to_lowercase().contains("401")
                || e.to_lowercase().contains("403")
                || e.to_lowercase().contains("unauthorized")
                || e.to_lowercase().contains("invalid")
                || e.to_lowercase().contains("api") =>
        {
            println!("⚠ programmatic key path exercised: {e}");
        }
        Err(e) => panic!("Unexpected failure for programmatic serper key: {e}"),
    }
}

#[tokio::test]
async fn test_search_with_content_duckduckgo() {
    let mut config = make_config(SEARCH_ENGINE_DUCKDUCKGO, false);
    config.search.limit = 1;
    let mut search = SearchEngine::from_config(&config);
    let outcome = tokio::time::timeout(
        SEARCH_TIMEOUT,
        search.search_with_content(
            TEST_QUERY,
            1,
            tarzi::fetcher::FetchMode::PlainRequest,
            tarzi::converter::Format::Markdown,
        ),
    )
    .await;
    search.shutdown().await;

    match outcome {
        Ok(Ok(results)) => {
            println!("✓ search_with_content returned {} pairs", results.len());
            assert!(results.len() <= 1);
        }
        Ok(Err(e)) if is_acceptable_external_failure(&e.to_string()) => {
            println!("⚠ search_with_content unavailable: {e}");
        }
        Ok(Err(e)) => panic!("Unexpected search_with_content failure: {e}"),
        Err(_) => println!("⚠ search_with_content timed out"),
    }
}
