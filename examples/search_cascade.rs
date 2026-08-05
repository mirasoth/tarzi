//! Demonstrate search access cascade, browser toggle, and multi-engine failover.
//!
//! Run:
//!   cargo run --example search_cascade
//!
//! Optional API keys:
//!   export BRAVE_API_KEY=...
//!   export SERPER_API_KEY=...

use tarzi::{config::Config, search::SearchEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let query = "rust programming language";
    let limit = 3;

    println!("=== Bing (browser enabled) ===");
    run_search("bing", true, query, limit).await;

    println!("\n=== DuckDuckGo (browser disabled) ===");
    run_search("duckduckgo", false, query, limit).await;

    println!("\n=== Brave (API if BRAVE_API_KEY set, else web) ===");
    run_search("brave", true, query, limit).await;

    println!("\n=== google_serper (API-only; skipped without SERPER_API_KEY) ===");
    run_search("google_serper", true, query, limit).await;

    println!("\n=== Multi-engine failover: google_serper,duckduckgo,bing ===");
    run_search("google_serper,duckduckgo,bing", false, query, limit).await;

    Ok(())
}

async fn run_search(engine: &str, browser: bool, query: &str, limit: usize) {
    let mut config = Config::load().unwrap_or_default();
    config.search.engine = engine.to_string();
    config.search.browser = browser;
    config.search.limit = limit;
    config.fetcher.browser = false;

    let mut search = SearchEngine::from_config(&config);
    match search.search(query, limit).await {
        Ok(results) => {
            println!(
                "OK engine={engine} browser={browser} engines={:?} results={}",
                search.engines(),
                results.len()
            );
            for (i, r) in results.iter().enumerate() {
                println!("  {}. {} — {}", i + 1, r.title, r.url);
            }
        }
        Err(e) => println!("ERR engine={engine}: {e}"),
    }
    search.shutdown().await;
}
