use tarzi::{Format, Result, SearchEngine, WebFetcher, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Tarzi Modular Example ===\n");

    // Load configuration and configure Chrome driver
    let mut config = Config::load().unwrap_or_default();
    config.fetcher.web_driver = "chromedriver".to_string();

    // Example 1: Using the fetcher module (plain HTTP → headless browser cascade)
    println!("1. Fetching content with cascade:");

    let mut fetcher = WebFetcher::from_config(&config);
    let test_url = tarzi::constants::HTTPBIN_HTML_URL;

    match fetcher.fetch(test_url, Format::Html).await {
        Ok(content) => println!("   HTML: {} characters", content.len()),
        Err(e) => println!("   Fetch failed: {e}"),
    }

    match fetcher.fetch(test_url, Format::Markdown).await {
        Ok(content) => println!("   Markdown: {} characters", content.len()),
        Err(e) => println!("   Fetch failed: {e}"),
    }

    println!();

    // Example 2: Using the search module
    println!("2. Searching and fetching content:");

    let mut search_engine = SearchEngine::from_config(&config);
    let query = "agentic AI";

    // Search for results
    match search_engine.search(query, 3).await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Snippet: {}", result.snippet);
            }
        }
        Err(e) => {
            eprintln!("Search failed: {e}");
        }
    }

    println!();

    // Example 3: Search and fetch content for each result
    println!("3. Search and fetch content for each result:");

    match search_engine
        .search_with_content(query, 3, Format::Markdown)
        .await
    {
        Ok(results) => {
            println!("Found {} results with content:", results.len());
            for (i, (result, content)) in results.iter().enumerate() {
                println!("{}. {}", i + 1, result.title);
                println!("   URL: {}", result.url);
                println!("   Content length: {} characters", content.len());
            }
        }
        Err(e) => {
            eprintln!("Search with content failed (WebDriver/network may be unavailable): {e}");
        }
    }

    println!();
    println!("=== Example completed ===");

    Ok(())
}
