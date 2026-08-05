use tarzi::{
    FetchMode, Format, Result, WebFetcher, config::Config, converter::Converter,
    search::SearchEngine,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== Tarzi Simple Usage Example ===\n");

    // Load configuration and configure Chrome driver
    let mut config = Config::load().unwrap_or_default();
    config.fetcher.web_driver = "chromedriver".to_string();

    // Example 1: Simple HTTP fetching
    println!("1. Fetching content with HTTP mode:");
    let mut fetcher = WebFetcher::from_config(&config);
    let test_url = "https://httpbin.org/html";

    match fetcher
        .fetch(test_url, FetchMode::PlainRequest, Format::Html)
        .await
    {
        Ok(content) => println!("   HTTP request successful: {} characters", content.len()),
        Err(e) => println!("   HTTP request failed: {e}"),
    }

    // Example 2: Converting HTML to different formats
    println!("\n2. Converting HTML to different formats:");
    let html_content = r#"
        <html>
            <head><title>Test Page</title></head>
            <body>
                <h1>Welcome to Tarzi</h1>
                <p>This is a <strong>test</strong> page with <a href="https://example.com">a link</a>.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </body>
        </html>
    "#;

    // Convert to Markdown
    let converter = Converter::new();
    match converter.convert(html_content, Format::Markdown).await {
        Ok(markdown) => {
            println!("   Markdown conversion:");
            println!("   {}", markdown.trim());
        }
        Err(e) => println!("   Markdown conversion failed: {e}"),
    }

    // Convert to JSON
    match converter.convert(html_content, Format::Json).await {
        Ok(json) => {
            println!("\n   JSON conversion:");
            println!("   {}", json.trim());
        }
        Err(e) => println!("   JSON conversion failed: {e}"),
    }

    // Example 3: Search engine configuration
    println!("\n3. Testing search engine configuration:");
    println!("   Default search engine: {}", config.search.engine);
    println!(
        "   Search browser fallback: {} (TARZI_SEARCH_BROWSER)",
        config.search.browser
    );
    println!("   Search limit: {}", config.search.limit);
    println!("   Query pattern: {}", config.search.query_pattern);

    // Create search engine
    let search_engine = SearchEngine::from_config(&config);
    println!(
        "   Search engine initialized (browser={}, engines={:?})",
        search_engine.browser_enabled(),
        search_engine.engines()
    );

    println!("\n=== Simple Example Complete ===");
    Ok(())
}
