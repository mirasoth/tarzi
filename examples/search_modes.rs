//! Demonstrate search access modes: auto, apiquery, and webquery.
//!
//! Run:
//!   cargo run --example search_modes
//!
//! Optional API keys (used when mode=auto or apiquery with brave / google_serper):
//!   export BRAVE_API_KEY=...
//!   export SERPER_API_KEY=...

use tarzi::{config::Config, search::SearchEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let query = "rust programming language";
    let limit = 3;

    // 1) Default Bing + auto (plain HTTP → browser; no API for Bing)
    println!("=== Bing / mode=auto ===");
    run_search("bing", "auto", None, query, limit).await;

    // 2) DuckDuckGo + webquery (plain HTML then browser; never API)
    println!("\n=== DuckDuckGo / mode=webquery ===");
    run_search("duckduckgo", "webquery", None, query, limit).await;

    // 3) Brave + auto (API if BRAVE_API_KEY set, else web cascade)
    println!("\n=== Brave / mode=auto ===");
    run_search("brave", "auto", None, query, limit).await;

    // 4) Brave + apiquery (requires BRAVE_API_KEY or search.api_key)
    println!("\n=== Brave / mode=apiquery ===");
    run_search("brave", "apiquery", None, query, limit).await;

    // 5) Google Serper + apiquery (API-only engine)
    println!("\n=== google_serper / mode=apiquery ===");
    run_search("google_serper", "apiquery", None, query, limit).await;

    // 6) Google web-only + webquery
    println!("\n=== Google / mode=webquery ===");
    run_search("google", "webquery", None, query, limit).await;

    Ok(())
}

async fn run_search(engine: &str, mode: &str, api_key: Option<&str>, query: &str, limit: usize) {
    let mut config = Config::new();
    config.search.engine = engine.to_string();
    config.search.mode = mode.to_string();
    config.search.limit = limit;
    if let Some(key) = api_key {
        config.search.api_key = Some(key.to_string());
    }

    let mut search_engine = SearchEngine::from_config(&config);
    println!(
        "engine={engine} mode={mode} resolved_mode={:?}",
        search_engine.search_mode()
    );

    match search_engine.search(query, limit).await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("  {}. {} — {}", i + 1, result.title, result.url);
            }
        }
        Err(e) => println!("Search failed (expected if key/network missing): {e}"),
    }

    search_engine.shutdown().await;
}
