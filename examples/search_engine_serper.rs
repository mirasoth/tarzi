//! Google search via Serper API (`google_serper` / `serper`).
//!
//! Requires:
//!   export SERPER_API_KEY=your-key
//!
//! Run:
//!   cargo run --example search_engine_serper

use tarzi::{config::Config, search::SearchEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut config = Config::load().unwrap_or_default();
    config.search.engine = "google_serper".to_string();
    config.search.limit = 5;
    // Prefer SERPER_API_KEY env; optional fallback:
    // config.search.api_key = Some("your-serper-api-key".to_string());

    let mut search_engine = SearchEngine::from_config(&config);
    let query = "agentic AI frameworks";

    match search_engine.search(query, config.search.limit).await {
        Ok(results) => {
            println!("Found {} Serper results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Snippet: {}", result.snippet);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Serper search failed: {e}");
            eprintln!("Set SERPER_API_KEY (or search.api_key) and retry.");
        }
    }

    search_engine.shutdown().await;
    Ok(())
}
