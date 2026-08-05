use std::str::FromStr;
use tarzi::{
    Result,
    config::Config,
    search::{SearchEngine, types::SearchEngineType},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let mut config = Config::load()?;
    config.search.engine = "sogou_weixin".to_string();
    config.search.limit = 10;
    let engine_type = SearchEngineType::from_str(&config.search.engine).unwrap();
    config.search.query_pattern = engine_type.get_query_pattern();
    config.fetcher.web_driver = "chromedriver".to_string();

    // Create search engine from config
    let mut search_engine = SearchEngine::from_config(&config);

    // Query about Oracle Corporation stock price in Chinese
    let query = "甲骨文股价";
    match search_engine.search(query, config.search.limit).await {
        Ok(results) => {
            if results.is_empty() {
                println!("No results found.");
            } else {
                for (i, result) in results.iter().enumerate() {
                    println!("\n{}. {}", i + 1, result.title);
                    println!("   URL: {}", result.url);
                    if !result.snippet.is_empty() {
                        println!("   Snippet: {}", result.snippet);
                    }
                    println!("   Rank: {}", result.rank);
                }
            }
        }
        Err(e) => {
            eprintln!("Search failed: {e}");
            return Err(e);
        }
    }

    // Ensure clean shutdown of browser and driver resources
    search_engine.shutdown().await;

    Ok(())
}
