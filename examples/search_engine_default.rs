use tarzi::{config::Config, search::SearchEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration with proper precedence
    let config = Config::load().unwrap_or_default();

    // Cascade: API (if credentials) → plain HTTP → browser (if search.browser)
    println!(
        "Using engine={} browser={}",
        config.search.engine, config.search.browser
    );

    // Create search engine from config
    let mut search_engine = SearchEngine::from_config(&config);

    // Perform a search
    let query = "agentic AI";
    match search_engine.search(query, config.search.limit).await {
        Ok(results) => {
            println!("\nFound {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Snippet: {}", result.snippet);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Search failed (WebDriver/network may be unavailable): {e}");
        }
    }

    search_engine.shutdown().await;

    Ok(())
}
