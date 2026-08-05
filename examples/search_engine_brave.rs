use std::str::FromStr;
use tarzi::{
    config::Config,
    search::{SearchEngine, types::SearchEngineType},
};

/// Brave search with access cascade (API → plain HTTP → browser).
///
/// Prefer: export BRAVE_API_KEY=...
/// Or set config.search.api_key. Set search.browser = false to skip browser fallback.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut config = Config::load().unwrap_or_default();
    config.fetcher.mode = "browser_headless".to_string();
    config.search.engine = "brave".to_string();
    config.search.browser = true;
    config.fetcher.user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();

    let engine_type = SearchEngineType::from_str(&config.search.engine).unwrap();
    config.search.query_pattern = engine_type.get_query_pattern();

    let mut search_engine = SearchEngine::from_config(&config);

    let query = "agentic AI";
    match search_engine.search(query, config.search.limit).await {
        Ok(results) => {
            println!(
                "\nFound {} results (browser={}):",
                results.len(),
                search_engine.browser_enabled()
            );
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Snippet: {}", result.snippet);
                println!();
            }
        }
        Err(e) => eprintln!("Brave search failed: {e}"),
    }

    search_engine.shutdown().await;
    Ok(())
}
